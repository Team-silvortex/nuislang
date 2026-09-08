use yir_core::{
    ApplicationCloseReason, ApplicationFailureKind, ApplicationOutcome,
    ApplicationSessionSignature, Value, YirFunctionParameter, YirModule,
};

use crate::{
    ApplicationCancellation, ApplicationEventPump, ApplicationOutcomeDelivery,
    ApplicationProviderSource, ApplicationPumpOperation, ApplicationPumpPhase,
};

mod ffi;
pub use ffi::*;

pub const WINDOW_SESSION_CONTRACT: &str = "nuis-yir-window-session-v3";

/// Window-specific host adapter, not a YIR application or rendering policy.
/// The registered Nuis helpers own configuration, image transitions and clocks.
pub struct WindowSession {
    pump: ApplicationEventPump,
    close_reason: Option<ApplicationCloseReason>,
    cleanup_completed: bool,
    failure_kind: ApplicationFailureKind,
    outcome: Option<ApplicationOutcome>,
    outcome_taken: bool,
    cancellation_admitted: bool,
}

pub struct WindowSessionReply {
    pub operation: ApplicationPumpOperation,
    pub phase: ApplicationPumpPhase,
    pub state: Option<Value>,
    pub close_reason: Option<ApplicationCloseReason>,
    pub cleanup_completed: bool,
    pub failure_kind: ApplicationFailureKind,
    pub outcome: Option<ApplicationOutcome>,
    pub frame: Result<Option<Vec<u8>>, String>,
}

impl WindowSession {
    pub fn spawn(
        source: String,
        provider: ApplicationProviderSource<'_>,
        id: String,
        width: i64,
        height: i64,
    ) -> Result<Self, String> {
        if !(1..=16384).contains(&width) || !(1..=16384).contains(&height) {
            return Err("window dimensions must be in 1..=16384".to_owned());
        }
        Ok(Self {
            pump: ApplicationEventPump::spawn_checked(
                source,
                provider,
                id,
                vec![Value::Int(width), Value::Int(height)],
                crate::application_event_pump::ApplicationPumpChecks {
                    preflight: validate_window_session,
                    trace: validate_window_trace,
                },
            )?,
            close_reason: None,
            cleanup_completed: false,
            failure_kind: ApplicationFailureKind::None,
            outcome: None,
            outcome_taken: false,
            cancellation_admitted: false,
        })
    }

    pub fn phase(&self) -> ApplicationPumpPhase {
        self.pump.phase()
    }

    pub fn pending(&self) -> bool {
        self.pump.pending().is_some()
    }

    pub fn event(&mut self, kind: i64, code: i64) -> Result<(), String> {
        match kind {
            0 if code == 0 => {}
            1 if u32::try_from(code).ok().and_then(char::from_u32).is_some() => {}
            _ => {
                return Err("window event requires redraw(0,0) or key(1,Unicode scalar)".to_owned())
            }
        }
        let result = self.pump.event(vec![Value::Int(kind), Value::Int(code)]);
        self.observe_stopped_error(&result);
        result
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.close_with_reason(ApplicationCloseReason::Requested)
    }

    /// Forward cooperative cancellation without running close or publishing an
    /// application outcome. The independent ticket may outlive this window;
    /// its acknowledgement covers host scope, not provider/device retirement.
    /// Rejection leaves the original request and observed state unchanged.
    pub fn cancel(&mut self) -> Result<ApplicationCancellation, String> {
        let ticket = self.pump.cancel()?;
        self.cancellation_admitted = true;
        Ok(ticket)
    }

    /// Request provider drain through the common session boundary. The window
    /// neither owns the transport nor turns the receipt into a parent outcome.
    pub fn cancel_with_provider_drain(&mut self) -> Result<ApplicationCancellation, String> {
        let ticket = self.pump.cancel_with_provider_drain()?;
        self.cancellation_admitted = true;
        Ok(ticket)
    }

    pub fn close_reason(&self) -> Option<ApplicationCloseReason> {
        self.close_reason
    }

    pub fn cleanup_completed(&self) -> bool {
        self.cleanup_completed
    }

    pub fn failure_kind(&self) -> ApplicationFailureKind {
        self.failure_kind
    }

    /// Read-only terminal snapshot; does not poll, rerun cleanup or revive state.
    pub fn outcome(&self) -> Option<ApplicationOutcome> {
        self.outcome
    }

    /// Issue one explicit parent delivery after terminal observation. Pending
    /// calls do not consume the slot. Readonly snapshots remain available; a
    /// dropped or failed delivery cannot be taken again or restart this child.
    pub fn take_outcome_delivery(&mut self) -> Option<ApplicationOutcomeDelivery> {
        if self.outcome_taken {
            return None;
        }
        let outcome = self.outcome?;
        self.outcome_taken = true;
        Some(ApplicationOutcomeDelivery::new(outcome))
    }

    /// Failed events dominate an ordinary host close request. Host failures are
    /// latched in the worker before cleanup, so they cannot authorize Finish.
    pub fn close_with_reason(&mut self, reason: ApplicationCloseReason) -> Result<(), String> {
        let reason = if self.pump.phase() == ApplicationPumpPhase::Faulted {
            ApplicationCloseReason::EventFailed
        } else {
            reason
        };
        let kind = if self.failure_kind.is_failure() {
            self.failure_kind
        } else {
            match reason {
                ApplicationCloseReason::Requested => ApplicationFailureKind::None,
                ApplicationCloseReason::EventFailed => ApplicationFailureKind::Callback,
                ApplicationCloseReason::HostFailed => ApplicationFailureKind::Host,
            }
        };
        let arguments = vec![Value::Int(reason.code()), Value::Int(kind.code())];
        let result = if reason.is_failure() {
            self.pump.close_after_failure(arguments, kind)
        } else {
            self.pump.close(arguments)
        };
        self.observe_stopped_error(&result);
        result?;
        self.close_reason = Some(reason);
        self.failure_kind = kind;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Option<WindowSessionReply>, String> {
        let result = self.pump.poll();
        self.observe_stopped_error(&result);
        let Some(reply) = result? else {
            return Ok(None);
        };
        // The pump has validated the reply before accepting it.
        let outcome = reply.outcome()?;
        self.cleanup_completed = reply.cleanup_completed;
        if !self.failure_kind.is_failure() {
            self.failure_kind = reply.failure_kind;
        }
        let frame = reply.trace.and_then(|trace| {
            if trace.presented_frames.is_empty() {
                Ok(None)
            } else {
                crate::render_trace_to_ppm_bytes(&trace, 1).map(Some)
            }
        });
        // A host presentation-contract failure must never admit more work or a
        // successful close. Callback failures already carry the pump's fault phase.
        if frame.is_err()
            && matches!(
                reply.phase,
                ApplicationPumpPhase::Open | ApplicationPumpPhase::Closed
            )
        {
            self.pump.abort();
            if !self.failure_kind.is_failure() {
                self.failure_kind = ApplicationFailureKind::Host;
            }
        }
        self.outcome = if self.phase() == ApplicationPumpPhase::Stopped {
            Some(ApplicationOutcome::failed(
                self.cleanup_completed,
                self.failure_kind,
            ))
        } else {
            outcome
        };
        Ok(Some(WindowSessionReply {
            operation: reply.operation,
            phase: self.phase(),
            state: reply.state,
            close_reason: self.close_reason,
            cleanup_completed: self.cleanup_completed,
            failure_kind: self.failure_kind,
            outcome: self.outcome,
            frame,
        }))
    }

    fn observe_stopped_error<T>(&mut self, result: &Result<T, String>) {
        if result.is_err()
            && self.phase() == ApplicationPumpPhase::Stopped
            && self.outcome.is_none()
            && !self.cancellation_admitted
        {
            if !self.failure_kind.is_failure() {
                self.failure_kind = ApplicationFailureKind::Unclassified;
            }
            self.outcome = Some(ApplicationOutcome::failed(
                self.cleanup_completed,
                self.failure_kind,
            ));
        }
    }
}

fn validate_window_trace(
    operation: ApplicationPumpOperation,
    trace: &yir_exec::ExecutionTrace,
) -> Result<(), String> {
    if trace.presented_frames.len() > 1 {
        return Err("window callback presented more than one frame".to_owned());
    }
    if operation == ApplicationPumpOperation::Close && !trace.presented_frames.is_empty() {
        return Err("window close callback must not present a frame".to_owned());
    }
    for frame in &trace.presented_frames {
        let rgba8 = frame
            .rgba8
            .as_ref()
            .ok_or("window frame requires RGBA8 pixels, not a glyph surface")?;
        let bytes = frame
            .width
            .checked_mul(frame.height)
            .and_then(|size| size.checked_mul(4));
        if frame.width == 0
            || frame.height == 0
            || bytes.is_none_or(|bytes| bytes > 16777216)
            || Some(rgba8.len()) != bytes
        {
            return Err("window frame exceeds the valid 16MiB RGBA8 extent".to_owned());
        }
    }
    Ok(())
}

pub fn validate_window_session(module: &YirModule, id: &str) -> Result<(), String> {
    let registration = yir_core::registered_application_session(module, id)?;
    let signature = ApplicationSessionSignature::bind(module, registration.entries())?;
    let state_count = signature.state_parameters.len();
    for (parameters, names) in [
        (
            signature.open.parameters.as_slice(),
            &["width", "height"][..],
        ),
        (
            &signature.event.parameters[state_count..],
            &["kind", "code"][..],
        ),
        (
            &signature.close.parameters[state_count..],
            &["reason", "failure"][..],
        ),
    ] {
        validate_parameters(parameters, names)?;
    }
    Ok(())
}

fn validate_parameters(parameters: &[YirFunctionParameter], names: &[&str]) -> Result<(), String> {
    if parameters.len() != names.len()
        || parameters
            .iter()
            .zip(names)
            .any(|(parameter, name)| parameter.name != *name || parameter.ty != "i64")
    {
        return Err(format!(
            "{WINDOW_SESSION_CONTRACT} requires scalar i64 parameters {names:?}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::FrameSurface;
    use yir_exec::ExecutionTrace;

    #[test]
    fn window_trace_requires_exact_bounded_rgba8_without_glyph_fallback() {
        let valid = FrameSurface::from_rgba8(1, 1, vec![1, 2, 3, 255]).unwrap();
        let mut trace = ExecutionTrace {
            presented_frames: vec![valid],
            ..Default::default()
        };
        validate_window_trace(ApplicationPumpOperation::Event, &trace).unwrap();
        for (width, height, rgba8) in [
            (1, 1, None),
            (1, 1, Some(vec![1, 2, 3])),
            (0, 1, Some(vec![])),
            (1, 0, Some(vec![])),
            (usize::MAX, 2, Some(vec![])),
            (4096, 4096, Some(vec![])),
        ] {
            trace.presented_frames[0] = FrameSurface {
                width,
                height,
                rows: vec!["@".to_owned()],
                rgba8,
            };
            assert!(validate_window_trace(ApplicationPumpOperation::Event, &trace).is_err());
        }
    }
}
