use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, SyncSender},
    sync::Arc,
};

use crate::application_failure::FailureState;
use crate::application_scope_admission::ScopeAdmission;
use crate::{application_cancellation::CancellationControl, ApplicationHostRetirementAck};
use yir_core::{ApplicationFailureKind, Value};

use super::{
    ApplicationPumpChecks, ApplicationPumpOperation, ApplicationPumpPhase, ApplicationPumpReply,
    Command,
};
use crate::{
    provider_application_session::{
        with_registered_provider_application_session_checked, SessionControl,
    },
    ApplicationProviderSource, ApplicationSessionPhase,
};

pub(super) struct Channel {
    pub requests: Receiver<Command>,
    pub replies: SyncSender<ApplicationPumpReply>,
    pub control: Arc<CancellationControl>,
}

pub(super) enum OwnedProvider {
    #[cfg(unix)]
    Ipc(PathBuf),
    Replay(PathBuf),
}

impl From<ApplicationProviderSource<'_>> for OwnedProvider {
    fn from(provider: ApplicationProviderSource<'_>) -> Self {
        match provider {
            #[cfg(unix)]
            ApplicationProviderSource::Ipc(path) => Self::Ipc(path.to_owned()),
            ApplicationProviderSource::Replay(path) => Self::Replay(path.to_owned()),
        }
    }
}

impl OwnedProvider {
    fn borrowed(&self) -> ApplicationProviderSource<'_> {
        match self {
            #[cfg(unix)]
            Self::Ipc(path) => ApplicationProviderSource::Ipc(path),
            Self::Replay(path) => ApplicationProviderSource::Replay(path),
        }
    }
}

pub(super) fn run(
    source: String,
    provider: OwnedProvider,
    id: String,
    arguments: Vec<Value>,
    channel: Channel,
    checks: ApplicationPumpChecks,
) -> ApplicationHostRetirementAck {
    let Channel {
        requests,
        replies,
        control,
    } = channel;
    let mut operation = ApplicationPumpOperation::Open;
    let mut final_state = None;
    let mut cleanup_completed = false;
    let failures = FailureState::default();
    let result = with_registered_provider_application_session_checked(
        &source,
        provider.borrowed(),
        &id,
        arguments,
        checks.preflight,
        SessionControl {
            failures: failures.clone(),
            admission: control.as_ref(),
        },
        |session, opened| {
            let send = |operation, trace, session: &crate::ApplicationSession<'_>| {
                control.checkpoint()?;
                let phase = match session.phase() {
                    ApplicationSessionPhase::Open => ApplicationPumpPhase::Open,
                    ApplicationSessionPhase::Faulted => ApplicationPumpPhase::Faulted,
                    ApplicationSessionPhase::Closed => {
                        unreachable!("close replies wait for finish")
                    }
                };
                replies
                    .send(ApplicationPumpReply {
                        operation,
                        phase,
                        state: Some(session.state().clone()),
                        cleanup_completed: false,
                        failure_kind: session.failure_kind(),
                        trace,
                    })
                    .map_err(|_| "application event pump reply receiver disconnected".to_owned())
            };
            (checks.trace)(operation, &opened)
                .inspect_err(|_| failures.record(ApplicationFailureKind::Host))?;
            send(operation, Ok(opened), session)?;
            while let Ok(command) = requests.recv() {
                control.checkpoint()?;
                operation = command.operation;
                if let Some(kind) = command.failed_close {
                    session.record_host_failure(kind);
                }
                let trace = match operation {
                    ApplicationPumpOperation::Event => session.event(command.arguments),
                    ApplicationPumpOperation::Close => {
                        session.close(command.arguments).and_then(|trace| {
                            trace.ok_or_else(|| "application event pump repeated close".to_owned())
                        })
                    }
                    ApplicationPumpOperation::Open => unreachable!("open is not a host command"),
                };
                if let Ok(trace) = &trace {
                    if let Err(error) = (checks.trace)(operation, trace) {
                        let prior = if operation == ApplicationPumpOperation::Close {
                            session.completion_status().err()
                        } else {
                            None
                        };
                        failures.record(ApplicationFailureKind::Host);
                        final_state = Some(session.state().clone());
                        return Err(match prior {
                            Some(prior) => format!("{prior}; cleanup validation failed: {error}"),
                            None => error,
                        });
                    }
                }
                if session.phase() == ApplicationSessionPhase::Closed {
                    final_state = Some(session.state().clone());
                    cleanup_completed = trace.is_ok();
                    control.checkpoint()?;
                    // The scoped API gates this result on completion_status AND
                    // provider Finish/Closed (or complete replay consumption).
                    // Preserve the original event error as well as a failed
                    // cleanup, before a driver error exits the scoped API.
                    return session.completion_status().and(trace);
                }
                send(operation, trace, session)?;
            }
            Err("application event pump disconnected without explicit close".to_owned())
        },
    );
    if control.cancelled() {
        return ApplicationHostRetirementAck::new(cleanup_completed, failures.kind());
    }
    let phase = if result.is_ok() {
        ApplicationPumpPhase::Closed
    } else {
        failures.record(ApplicationFailureKind::Unclassified);
        ApplicationPumpPhase::Stopped
    };
    let _ = replies.send(ApplicationPumpReply {
        operation,
        phase,
        state: final_state,
        cleanup_completed,
        failure_kind: failures.kind(),
        trace: result,
    });
    ApplicationHostRetirementAck::new(cleanup_completed, failures.kind())
}
