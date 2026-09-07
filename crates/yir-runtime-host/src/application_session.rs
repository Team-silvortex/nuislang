use crate::application_failure::FailureState;
use yir_core::{ApplicationFailureKind, ModRegistry, Value, YirModule};
use yir_exec::{ExecutionTrace, FunctionSession};

mod boundary;
use boundary::SessionBoundary;

pub use yir_core::{ApplicationSessionEntries, APPLICATION_SESSION_CONTRACT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationSessionPhase {
    Open,
    Faulted,
    Closed,
}

/// Host transport for a Nuis-owned scalar aggregate, not an application policy.
///
/// Each event calls one helper in the same verified execution context. No main
/// entry replay occurs. The caller must explicitly close; Drop does not run Nuis
/// code. A failed event stops event delivery, but permits one close attempt with
/// the last accepted scalar state. Effects are never rolled back or retried.
/// Resource handles in state, native CPU dispatch and window wiring are not yet
/// part of this boundary. The registry is explicit; no backend fallback is chosen.
pub struct ApplicationSession<'a> {
    execution: FunctionSession<'a>,
    boundary: SessionBoundary<'a>,
    state: Value,
    phase: ApplicationSessionPhase,
    close_error: Option<String>,
    event_error: Option<String>,
    failures: FailureState,
}

impl<'a> ApplicationSession<'a> {
    pub fn open_registered(
        module: &'a YirModule,
        registry: &'a ModRegistry,
        id: &str,
        arguments: Vec<Value>,
    ) -> Result<(Self, ExecutionTrace), String> {
        let registration = yir_core::registered_application_session(module, id)?;
        Self::open(module, registry, registration.entries(), arguments)
    }

    pub fn open(
        module: &'a YirModule,
        registry: &'a ModRegistry,
        entries: ApplicationSessionEntries<'_>,
        arguments: Vec<Value>,
    ) -> Result<(Self, ExecutionTrace), String> {
        Self::open_with_failures(
            module,
            registry,
            entries,
            arguments,
            FailureState::default(),
        )
    }

    pub(crate) fn open_with_failures(
        module: &'a YirModule,
        registry: &'a ModRegistry,
        entries: ApplicationSessionEntries<'_>,
        arguments: Vec<Value>,
        failures: FailureState,
    ) -> Result<(Self, ExecutionTrace), String> {
        let boundary = SessionBoundary::bind(module, entries)?;
        FunctionSession::validate_arguments(&boundary.open.parameters, &arguments)?;
        let mut execution = FunctionSession::new(module, registry)?;
        let opened = execution
            .invoke(&boundary.open.name, arguments)
            .inspect_err(|_| failures.record(ApplicationFailureKind::Callback))?;
        boundary
            .state_arguments(&opened.value)
            .inspect_err(|_| failures.record(ApplicationFailureKind::Callback))?;
        Ok((
            Self {
                execution,
                boundary,
                state: opened.value,
                phase: ApplicationSessionPhase::Open,
                close_error: None,
                event_error: None,
                failures,
            },
            opened.trace,
        ))
    }

    pub fn state(&self) -> &Value {
        &self.state
    }

    pub fn phase(&self) -> ApplicationSessionPhase {
        self.phase
    }

    pub fn failure_kind(&self) -> ApplicationFailureKind {
        self.failures.kind()
    }

    pub(crate) fn record_host_failure(&mut self, kind: ApplicationFailureKind) {
        if self.phase != ApplicationSessionPhase::Closed {
            self.failures.record(kind);
            self.phase = ApplicationSessionPhase::Faulted;
            self.event_error
                .get_or_insert_with(|| "application host requested failed cleanup".to_owned());
        }
    }

    /// Cleanup may succeed after a failed event, but that is not a successful
    /// application lifecycle and must not authorize provider evidence persistence.
    pub fn completion_status(&self) -> Result<(), String> {
        if self.phase != ApplicationSessionPhase::Closed {
            return Err("application session was not explicitly closed".to_owned());
        }
        if let (Some(event), Some(close)) = (&self.event_error, &self.close_error) {
            return Err(format!(
                "application session did not complete successfully: {event}; cleanup failed: {close}"
            ));
        }
        if let Some(error) = self.event_error.as_ref().or(self.close_error.as_ref()) {
            return Err(format!(
                "application session did not complete successfully: {error}"
            ));
        }
        if self.failure_kind().is_failure() {
            return Err("application session retained a producer failure".to_owned());
        }
        Ok(())
    }

    pub(crate) fn preflight(
        module: &YirModule,
        entries: ApplicationSessionEntries<'_>,
        arguments: &[Value],
    ) -> Result<(), String> {
        let boundary = SessionBoundary::bind(module, entries)?;
        FunctionSession::validate_arguments(&boundary.open.parameters, arguments)
    }

    pub fn event(&mut self, arguments: Vec<Value>) -> Result<ExecutionTrace, String> {
        if self.phase != ApplicationSessionPhase::Open {
            return Err(format!(
                "application session cannot deliver an event while {:?}",
                self.phase
            ));
        }
        let arguments =
            self.boundary
                .call_arguments(&self.state, self.boundary.event, arguments)?;
        let result = self.execution.invoke(&self.boundary.event.name, arguments);
        match result.and_then(|invocation| {
            self.boundary.state_arguments(&invocation.value)?;
            Ok(invocation)
        }) {
            Ok(invocation) => {
                self.state = invocation.value;
                Ok(invocation.trace)
            }
            Err(error) => {
                self.phase = ApplicationSessionPhase::Faulted;
                self.failures.record(ApplicationFailureKind::Callback);
                self.event_error = Some(error.clone());
                Err(error)
            }
        }
    }

    /// Close at most once, including a failed close. Repeated success is a no-op;
    /// repeated failure returns the original error without re-executing teardown.
    pub fn close(&mut self, arguments: Vec<Value>) -> Result<Option<ExecutionTrace>, String> {
        if self.phase == ApplicationSessionPhase::Closed {
            return match &self.close_error {
                Some(error) => Err(error.clone()),
                None => Ok(None),
            };
        }
        let arguments =
            self.boundary
                .call_arguments(&self.state, self.boundary.close, arguments)?;
        self.phase = ApplicationSessionPhase::Closed;
        let result = self.execution.invoke(&self.boundary.close.name, arguments);
        match result.and_then(|invocation| {
            self.boundary.state_arguments(&invocation.value)?;
            Ok(invocation)
        }) {
            Ok(invocation) => {
                self.state = invocation.value;
                Ok(Some(invocation.trace))
            }
            Err(error) => {
                self.close_error = Some(error.clone());
                self.failures.record(ApplicationFailureKind::Callback);
                Err(error)
            }
        }
    }
}
