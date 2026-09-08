use std::{cell::Cell, path::Path};

use yir_core::Value;
use yir_exec::ExecutionTrace;

use crate::application_failure::FailureState;
use crate::application_scope_admission::{ScopeAbandonment, ScopeAdmission};
use crate::provider_result_stream::{
    drain_provider_source, finish_provider_source_with_failures, provider_registry_with_failures,
    replay_source,
};
use crate::{ApplicationSession, ApplicationSessionEntries, ProviderDrainObservation};

pub const PROVIDER_APPLICATION_SESSION_CONTRACT: &str = "nuis-yir-provider-application-session-v1";

pub(crate) struct SessionControl<'a, A: ScopeAdmission = ()> {
    pub failures: FailureState,
    pub admission: &'a A,
    pub drain: Option<&'a Cell<ProviderDrainObservation>>,
}

impl Default for SessionControl<'_, ()> {
    fn default() -> Self {
        Self {
            failures: FailureState::default(),
            admission: &(),
            drain: None,
        }
    }
}

impl<A: ScopeAdmission> SessionControl<'_, A> {
    fn checkpoint(&self) -> Result<(), String> {
        self.admission.checkpoint()
    }

    fn admit_finalization(&self) -> Result<(), String> {
        self.admission.admit_finalization()
    }
}

/// Explicit admission source. There is no implicit environment lookup or
/// reference-device fallback in the persistent application frontdoor.
pub enum ApplicationProviderSource<'a> {
    #[cfg(unix)]
    Ipc(&'a Path),
    Replay(&'a Path),
}

/// Keep module, registry, provider transport and Nuis state alive across host
/// event deliveries. The driver must explicitly close the application.
///
/// Only a successful driver AND application lifecycle may finish the provider
/// transport. Any failure drops it without a success acknowledgement, including
/// a driver that swallows a failed event or close. Provider close errors propagate
/// instead of releasing a successful return value to the caller.
///
/// This is synchronous and bounded by the existing transport budgets. It neither
/// launches a window nor claims native CPU or arbitrary multi-provider support.
pub fn with_provider_application_session<T>(
    source: &str,
    provider: ApplicationProviderSource<'_>,
    entries: ApplicationSessionEntries<'_>,
    arguments: Vec<Value>,
    drive: impl FnOnce(&mut ApplicationSession<'_>, ExecutionTrace) -> Result<T, String>,
) -> Result<T, String> {
    let module = yir_syntax::parse_module(source)?;
    with_session(
        source,
        &module,
        provider,
        entries,
        arguments,
        SessionControl::default(),
        drive,
    )
}

/// Select only a static registration embedded in the admitted YIR. Unknown or
/// inconsistent bindings fail before provider connection or application effects.
pub fn with_registered_provider_application_session<T>(
    source: &str,
    provider: ApplicationProviderSource<'_>,
    id: &str,
    arguments: Vec<Value>,
    drive: impl FnOnce(&mut ApplicationSession<'_>, ExecutionTrace) -> Result<T, String>,
) -> Result<T, String> {
    with_registered_provider_application_session_checked(
        source,
        provider,
        id,
        arguments,
        |_, _| Ok(()),
        SessionControl::default(),
        drive,
    )
}

pub(crate) fn with_registered_provider_application_session_checked<T, A: ScopeAdmission>(
    source: &str,
    provider: ApplicationProviderSource<'_>,
    id: &str,
    arguments: Vec<Value>,
    preflight: fn(&yir_core::YirModule, &str) -> Result<(), String>,
    control: SessionControl<'_, A>,
    drive: impl FnOnce(&mut ApplicationSession<'_>, ExecutionTrace) -> Result<T, String>,
) -> Result<T, String> {
    let module = yir_syntax::parse_module(source)?;
    let registration = yir_core::registered_application_session(&module, id)?;
    preflight(&module, id)?;
    with_session(
        source,
        &module,
        provider,
        registration.entries(),
        arguments,
        control,
        drive,
    )
}

fn with_session<T, A: ScopeAdmission>(
    source: &str,
    module: &yir_core::YirModule,
    provider: ApplicationProviderSource<'_>,
    entries: ApplicationSessionEntries<'_>,
    arguments: Vec<Value>,
    control: SessionControl<'_, A>,
    drive: impl FnOnce(&mut ApplicationSession<'_>, ExecutionTrace) -> Result<T, String>,
) -> Result<T, String> {
    control.checkpoint()?;
    ApplicationSession::preflight(module, entries, &arguments)?;
    let provider = match provider {
        #[cfg(unix)]
        ApplicationProviderSource::Ipc(path) => {
            crate::provider_result_stream::ProviderResultSource::Live(
                crate::provider_runtime_ipc::connect_provider_runtime(source, module, path)?,
            )
        }
        ApplicationProviderSource::Replay(path) => replay_source(source, path)?,
    };
    let (registry, provider) = provider_registry_with_failures(provider, control.failures.clone());
    let result = (|| {
        control.checkpoint()?;
        let (mut application, opened) = ApplicationSession::open_with_failures(
            module,
            &registry,
            entries,
            arguments,
            control.failures.clone(),
        )?;
        let result = drive(&mut application, opened)?;
        application.completion_status()?;
        drop(application);
        control.admit_finalization()?;
        finish_provider_source_with_failures(&provider, &control.failures)?;
        Ok(result)
    })();
    // No application can dispatch after this point. The admission policy seals
    // the abandonment race; transport health independently decides drain safety.
    drop(registry);
    if result.is_err() && control.admission.admit_abandonment() == ScopeAbandonment::Drain {
        let observation = drain_provider_source(&provider, &control.failures);
        if let Some(slot) = control.drain {
            slot.set(observation);
        }
    }
    result
}

#[cfg(all(test, unix))]
#[path = "provider_application_session/admission_tests.rs"]
mod admission_tests;
