use std::time::{Duration, Instant};

use yir_core::{ApplicationOutcome, Value};

use crate::{
    ApplicationEventPump, ApplicationHostRetirementAck, ApplicationProviderSource,
    ApplicationPumpPhase, ApplicationPumpReply,
};

mod entry;
pub use entry::nuis_application_script_main;

pub const APPLICATION_SCRIPT_CONTRACT: &str = "nuis-yir-application-scalar-script-v1";
pub const MAX_SCRIPT_EVENTS: usize = 64;
pub const MAX_SCRIPT_ARGUMENTS: usize = 16;
pub const MAX_SCRIPT_DURATION: Duration = Duration::from_secs(180);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationScriptTermination {
    Close(Vec<i64>),
    Cancel { drain_provider: bool },
}

/// A bounded host ingress profile, not application or device policy. Arguments
/// are positional i64 values; registration, state and callback validation remain
/// owned by the common application session contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationScript {
    pub id: String,
    pub open: Vec<i64>,
    pub events: Vec<Vec<i64>>,
    pub termination: ApplicationScriptTermination,
}

#[derive(Debug)]
pub enum ApplicationScriptOutcome {
    Finished(ApplicationOutcome),
    Cancelled(ApplicationHostRetirementAck),
}

impl ApplicationScript {
    /// Validate every call before admitting any application effects.
    pub fn validate_source(&self, source: &str) -> Result<(), String> {
        self.validate()?;
        let module = yir_syntax::parse_module(source)?;
        let registration = yir_core::registered_application_session(&module, &self.id)?;
        let signature =
            yir_core::ApplicationSessionSignature::bind(&module, registration.entries())?;
        let check = |parameters: &[yir_core::YirFunctionParameter], arguments: &[i64]| {
            yir_exec::FunctionSession::validate_arguments(parameters, &values(arguments.to_vec()))
        };
        check(&signature.open.parameters, &self.open)?;
        let state_count = signature.state_parameters.len();
        for event in &self.events {
            check(&signature.event.parameters[state_count..], event)?;
        }
        if let ApplicationScriptTermination::Close(arguments) = &self.termination {
            check(&signature.close.parameters[state_count..], arguments)?;
        }
        Ok(())
    }

    /// Share the packaged entry grammar with launchers instead of translating
    /// application arguments through a window-specific ingress profile.
    pub fn from_arguments(arguments: &[&str]) -> Result<Self, String> {
        entry::parse(arguments)
    }

    pub fn to_arguments(&self) -> Result<Vec<String>, String> {
        self.validate()?;
        let scalars = |values: &[i64]| {
            values
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        let mut arguments = vec![
            "--application-session".to_owned(),
            self.id.clone(),
            "--open-args".to_owned(),
            scalars(&self.open),
        ];
        for event in &self.events {
            arguments.extend(["--event-args".to_owned(), scalars(event)]);
        }
        match &self.termination {
            ApplicationScriptTermination::Close(values) => {
                arguments.extend(["--close-args".to_owned(), scalars(values)]);
            }
            ApplicationScriptTermination::Cancel { drain_provider } => {
                arguments.push("--cancel-after-events".to_owned());
                if *drain_provider {
                    arguments.push("--drain-provider".to_owned());
                }
            }
        }
        Ok(arguments)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || self.id.len() > 128 || self.id.contains('\0') {
            return Err("application script requires a registration ID of 1..128 bytes".to_owned());
        }
        if self.events.len() > MAX_SCRIPT_EVENTS {
            return Err("application script exceeds its event limit".to_owned());
        }
        let close = match &self.termination {
            ApplicationScriptTermination::Close(args) => Some(args),
            _ => None,
        };
        if std::iter::once(&self.open)
            .chain(&self.events)
            .chain(close)
            .any(|args| args.len() > MAX_SCRIPT_ARGUMENTS)
        {
            return Err("application script exceeds its argument limit".to_owned());
        }
        Ok(())
    }
}

/// Drive one registered session without a window adapter. Replies are observed
/// one at a time, not retained as history. The deadline covers the whole script,
/// including retirement; a timeout never retries, closes or joins implicitly.
/// Failed callbacks abort the script before its requested terminal operation.
/// Cancellation is admitted only after all preceding replies were validated.
pub fn run_application_script(
    source: String,
    provider: ApplicationProviderSource<'_>,
    script: ApplicationScript,
    timeout: Duration,
    mut observe: impl FnMut(&ApplicationPumpReply),
) -> Result<ApplicationScriptOutcome, String> {
    script.validate()?;
    if timeout.is_zero() || timeout > MAX_SCRIPT_DURATION {
        return Err("application script duration must be in (0, 180s]".to_owned());
    }
    let deadline = Instant::now() + timeout;
    script.validate_source(&source)?;
    remaining(deadline)?;
    let mut pump = ApplicationEventPump::spawn(source, provider, script.id, values(script.open))?;
    receive(&mut pump, deadline, &mut observe)?;
    for arguments in script.events {
        remaining(deadline)?;
        pump.event(values(arguments))?;
        receive(&mut pump, deadline, &mut observe)?;
    }
    remaining(deadline)?;
    match script.termination {
        ApplicationScriptTermination::Close(arguments) => {
            pump.close(values(arguments))?;
            let closed = receive(&mut pump, deadline, &mut observe)?;
            if closed.phase != ApplicationPumpPhase::Closed {
                return Err("application script close did not finish the session".to_owned());
            }
            let outcome = closed
                .outcome()?
                .ok_or("application script close has no terminal outcome")?;
            Ok(ApplicationScriptOutcome::Finished(outcome))
        }
        ApplicationScriptTermination::Cancel { drain_provider } => {
            let mut ticket = if drain_provider {
                pump.cancel_with_provider_drain()?
            } else {
                pump.cancel()?
            };
            drop(pump);
            let ack = ticket
                .wait(remaining(deadline)?)?
                .ok_or("application script retirement deadline exceeded")?;
            Ok(ApplicationScriptOutcome::Cancelled(ack))
        }
    }
}

fn values(arguments: Vec<i64>) -> Vec<Value> {
    arguments.into_iter().map(Value::Int).collect()
}

fn remaining(deadline: Instant) -> Result<Duration, String> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| "application script deadline exceeded".to_owned())
}

fn receive(
    pump: &mut ApplicationEventPump,
    deadline: Instant,
    observe: &mut impl FnMut(&ApplicationPumpReply),
) -> Result<ApplicationPumpReply, String> {
    let reply = pump
        .wait(remaining(deadline)?)?
        .ok_or("application script callback deadline exceeded")?;
    observe(&reply);
    reply.trace.as_ref().map_err(Clone::clone)?;
    if reply.failure_kind.is_failure() {
        return Err(format!(
            "application script callback failed with kind {}",
            reply.failure_kind.code()
        ));
    }
    remaining(deadline)?;
    Ok(reply)
}
