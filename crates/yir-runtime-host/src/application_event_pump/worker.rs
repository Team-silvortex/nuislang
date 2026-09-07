use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, SyncSender},
};

use yir_core::Value;

use super::{
    ApplicationPumpChecks, ApplicationPumpOperation, ApplicationPumpPhase, ApplicationPumpReply,
    Command,
};
use crate::{
    provider_application_session::with_registered_provider_application_session_checked,
    ApplicationProviderSource, ApplicationSessionPhase,
};

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
    requests: Receiver<Command>,
    replies: SyncSender<ApplicationPumpReply>,
    checks: ApplicationPumpChecks,
) {
    let mut operation = ApplicationPumpOperation::Open;
    let mut final_state = None;
    let result = with_registered_provider_application_session_checked(
        &source,
        provider.borrowed(),
        &id,
        arguments,
        checks.preflight,
        |session, opened| {
            let send = |operation, trace, session: &crate::ApplicationSession<'_>| {
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
                        trace,
                    })
                    .map_err(|_| "application event pump reply receiver disconnected".to_owned())
            };
            (checks.trace)(operation, &opened)?;
            send(operation, Ok(opened), session)?;
            while let Ok(command) = requests.recv() {
                operation = command.operation;
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
                        final_state = Some(session.state().clone());
                        return Err(error);
                    }
                }
                if session.phase() == ApplicationSessionPhase::Closed {
                    final_state = Some(session.state().clone());
                    // The scoped API gates this result on completion_status AND
                    // provider Finish/Closed (or complete replay consumption).
                    return trace;
                }
                send(operation, trace, session)?;
            }
            Err("application event pump disconnected without explicit close".to_owned())
        },
    );
    let phase = if result.is_ok() {
        ApplicationPumpPhase::Closed
    } else {
        ApplicationPumpPhase::Stopped
    };
    let _ = replies.send(ApplicationPumpReply {
        operation,
        phase,
        state: final_state,
        trace: result,
    });
}
