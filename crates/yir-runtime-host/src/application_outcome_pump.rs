use std::{
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
    thread,
};

use yir_core::{ApplicationSessionSignature, ModRegistry, Value, YirModule};
use yir_exec::FunctionSession;

use crate::{ApplicationOutcomeDelivery, ApplicationSession};

mod ffi;
mod host_registry;
pub use ffi::*;

pub const APPLICATION_OUTCOME_PUMP_CONTRACT: &str = "nuis-yir-application-outcome-pump-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationOutcomePumpPhase {
    Opening,
    Open,
    Finishing,
    /// The parent completed, not proof of child success or resource retirement.
    Closed,
    Stopped,
}

#[derive(Debug)]
pub struct ApplicationOutcomePumpReply {
    pub phase: ApplicationOutcomePumpPhase,
    pub state: Option<Value>,
    pub delivered: bool,
    pub cleanup_completed: bool,
    pub result: Result<(), String>,
}

/// A separate, explicitly registered parent context. It opens before accepting
/// one owned child outcome and an explicit request to close. Only scalars cross
/// this channel; the parent owns its registry, fuel and effects. No provider
/// registry or completion witness is inherited from the child.
///
/// Poll and Drop never join or execute Nuis code. Abandoning the handle wakes an
/// idle worker but does not cancel admitted effects or implicitly call close.
pub struct ApplicationOutcomePump {
    commands: Option<SyncSender<ApplicationOutcomeDelivery>>,
    replies: Receiver<ApplicationOutcomePumpReply>,
    phase: ApplicationOutcomePumpPhase,
}

impl ApplicationOutcomePump {
    pub fn spawn(
        source: String,
        id: String,
        registry: ModRegistry,
        max_steps: usize,
    ) -> Result<Self, String> {
        Self::spawn_with_registry(source, id, move |_| Ok(registry), max_steps, false)
    }

    fn spawn_with_registry(
        source: String,
        id: String,
        registry: impl FnOnce(&YirModule) -> Result<ModRegistry, String> + Send + 'static,
        max_steps: usize,
        rooted: bool,
    ) -> Result<Self, String> {
        if max_steps == 0 {
            return Err("parent outcome pump requires a positive step budget".to_owned());
        }
        let (commands, requests) = mpsc::sync_channel::<ApplicationOutcomeDelivery>(1);
        let (responses, replies) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("nuis-application-outcome-pump".to_owned())
            .spawn(move || {
                let mut state = None;
                let mut delivered = false;
                let mut cleanup_completed = false;
                let result = (|| {
                    let module = yir_syntax::parse_module(&source)?;
                    validate_outcome_parent(&module, &id)?;
                    let registry = registry(&module)?;
                    let open = if rooted {
                        ApplicationSession::open_registered_rooted_budgeted
                    } else {
                        ApplicationSession::open_registered_budgeted
                    };
                    let (mut parent, _) = open(&module, &registry, &id, vec![], max_steps)?;
                    state = Some(parent.state().clone());
                    responses
                        .send(ApplicationOutcomePumpReply {
                            phase: ApplicationOutcomePumpPhase::Open,
                            state: state.clone(),
                            delivered: false,
                            cleanup_completed: false,
                            result: Ok(()),
                        })
                        .map_err(|_| "parent outcome receiver disconnected".to_owned())?;
                    let delivery = requests
                        .recv()
                        .map_err(|_| "parent abandoned without explicit finish".to_owned())?;
                    let event = delivery.deliver(&mut parent, max_steps);
                    delivered = event.is_ok();
                    // This close is authorized by finish_with_outcome, not Drop
                    // or an implicit retry after a failed event.
                    let close = parent.close_budgeted(vec![], max_steps);
                    cleanup_completed = matches!(close, Ok(Some(_)));
                    state = Some(parent.state().clone());
                    parent
                        .completion_status()
                        .and(event.map(|_| ()))
                        .and(close.map(|_| ()))
                })();
                let _ = responses.send(ApplicationOutcomePumpReply {
                    phase: if result.is_ok() {
                        ApplicationOutcomePumpPhase::Closed
                    } else {
                        ApplicationOutcomePumpPhase::Stopped
                    },
                    state,
                    delivered,
                    cleanup_completed,
                    result,
                });
            })
            .map_err(|error| format!("parent outcome worker could not start: {error}"))?;
        Ok(Self {
            commands: Some(commands),
            replies,
            phase: ApplicationOutcomePumpPhase::Opening,
        })
    }

    pub fn phase(&self) -> ApplicationOutcomePumpPhase {
        self.phase
    }

    /// Consumes the attempt even on rejection. The explicit command authorizes
    /// one event followed by one close, including cleanup after event failure.
    pub fn finish_with_outcome(
        &mut self,
        delivery: ApplicationOutcomeDelivery,
    ) -> Result<(), String> {
        if self.phase != ApplicationOutcomePumpPhase::Open {
            return Err("parent must be open before outcome delivery".to_owned());
        }
        self.phase = ApplicationOutcomePumpPhase::Finishing;
        let result = self.commands.take().unwrap().try_send(delivery);
        if result.is_err() {
            self.phase = ApplicationOutcomePumpPhase::Stopped;
            return Err("parent outcome worker disconnected before delivery".to_owned());
        }
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Option<ApplicationOutcomePumpReply>, String> {
        if !matches!(
            self.phase,
            ApplicationOutcomePumpPhase::Opening | ApplicationOutcomePumpPhase::Finishing
        ) {
            return Ok(None);
        }
        match self.replies.try_recv() {
            Ok(reply) => {
                self.phase = reply.phase;
                if self.phase == ApplicationOutcomePumpPhase::Stopped {
                    self.commands.take();
                }
                Ok(Some(reply))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                self.phase = ApplicationOutcomePumpPhase::Stopped;
                self.commands.take();
                Err("parent outcome worker disconnected before replying".to_owned())
            }
        }
    }
}

/// A host profile of the existing application-session registration, not new AST
/// syntax: open(), event(state, status, cleanup, failure), close(state).
pub fn validate_outcome_parent(module: &YirModule, id: &str) -> Result<(), String> {
    let registration = yir_core::registered_application_session(module, id)?;
    let signature = ApplicationSessionSignature::bind(module, registration.entries())?;
    FunctionSession::validate_arguments(&signature.open.parameters, &[])?;
    let state_count = signature.state_parameters.len();
    FunctionSession::validate_arguments(
        &signature.event.parameters[state_count..],
        &[Value::Int(1), Value::Int(1), Value::Int(0)],
    )?;
    FunctionSession::validate_arguments(&signature.close.parameters[state_count..], &[])
}

#[cfg(test)]
mod tests;
