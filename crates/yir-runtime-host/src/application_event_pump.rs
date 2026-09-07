use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError, TrySendError},
    thread,
    time::Duration,
};

use yir_core::{ApplicationFailureKind, Value};
use yir_exec::ExecutionTrace;

use crate::ApplicationProviderSource;

mod outcome;
mod worker;

pub const APPLICATION_EVENT_PUMP_CONTRACT: &str = "nuis-yir-application-event-pump-v1";

pub(crate) struct ApplicationPumpChecks {
    pub preflight: fn(&yir_core::YirModule, &str) -> Result<(), String>,
    pub trace: fn(ApplicationPumpOperation, &ExecutionTrace) -> Result<(), String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationPumpOperation {
    Open,
    Event,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationPumpPhase {
    Opening,
    Open,
    Faulted,
    Closed,
    Stopped,
}

/// One delivery, not a retained event history. State is the last accepted Nuis
/// aggregate, or absent if opening failed. A failed trace never contains frames
/// from a failed callback. Only Closed certifies both lifecycle and provider finish.
#[derive(Debug)]
pub struct ApplicationPumpReply {
    pub operation: ApplicationPumpOperation,
    pub phase: ApplicationPumpPhase,
    pub state: Option<Value>,
    /// A validated close callback returned; provider/lifecycle completion can
    /// still fail. This flag never authorizes success or retrying cleanup.
    pub cleanup_completed: bool,
    pub failure_kind: ApplicationFailureKind,
    pub trace: Result<ExecutionTrace, String>,
}

struct Command {
    operation: ApplicationPumpOperation,
    arguments: Vec<Value>,
    failed_close: Option<ApplicationFailureKind>,
}

/// Owned host handle with nonblocking admission and polling. Module, registry,
/// execution state and provider transport stay in one worker's borrowed scope;
/// no self-referential allocation or application policy crosses the channel.
///
/// Exactly one request may be outstanding, including an unconsumed reply. Drop
/// disconnects rather than joining or executing implicit Nuis close. It cannot
/// interrupt an already admitted callback or undo an explicitly requested close.
/// Socket deadlines remain those of the provider protocol, not GUI deadlines.
pub struct ApplicationEventPump {
    commands: Option<SyncSender<Command>>,
    replies: Option<Receiver<ApplicationPumpReply>>,
    pending: Option<ApplicationPumpOperation>,
    phase: ApplicationPumpPhase,
}

impl ApplicationEventPump {
    /// Start admission asynchronously. Poll/wait for the Open reply before
    /// sending events. All bindings come from the embedded registration ID.
    pub fn spawn(
        source: String,
        provider: ApplicationProviderSource<'_>,
        id: String,
        arguments: Vec<Value>,
    ) -> Result<Self, String> {
        Self::spawn_checked(
            source,
            provider,
            id,
            arguments,
            ApplicationPumpChecks {
                preflight: |_, _| Ok(()),
                trace: |_, _| Ok(()),
            },
        )
    }

    pub(crate) fn spawn_checked(
        source: String,
        provider: ApplicationProviderSource<'_>,
        id: String,
        arguments: Vec<Value>,
        checks: ApplicationPumpChecks,
    ) -> Result<Self, String> {
        let provider = worker::OwnedProvider::from(provider);
        let (commands, requests) = mpsc::sync_channel(1);
        let (responses, replies) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("nuis-application-event-pump".to_owned())
            .spawn(move || {
                worker::run(source, provider, id, arguments, requests, responses, checks)
            })
            .map_err(|error| format!("application event pump could not start: {error}"))?;
        Ok(Self {
            commands: Some(commands),
            replies: Some(replies),
            pending: Some(ApplicationPumpOperation::Open),
            phase: ApplicationPumpPhase::Opening,
        })
    }

    pub fn phase(&self) -> ApplicationPumpPhase {
        self.phase
    }

    pub fn pending(&self) -> Option<ApplicationPumpOperation> {
        self.pending
    }

    /// Admission is not execution success. Consume the matching reply before
    /// issuing another event or close; busy requests are never queued or retried.
    pub fn event(&mut self, arguments: Vec<Value>) -> Result<(), String> {
        self.submit(ApplicationPumpOperation::Event, arguments, None)
    }

    pub fn close(&mut self, arguments: Vec<Value>) -> Result<(), String> {
        self.submit(ApplicationPumpOperation::Close, arguments, None)
    }

    pub(crate) fn close_after_failure(
        &mut self,
        arguments: Vec<Value>,
        kind: ApplicationFailureKind,
    ) -> Result<(), String> {
        if !kind.is_failure() {
            return Err("failed cleanup requires a failure kind".to_owned());
        }
        self.submit(ApplicationPumpOperation::Close, arguments, Some(kind))
    }

    pub fn poll(&mut self) -> Result<Option<ApplicationPumpReply>, String> {
        if self.pending.is_none() {
            return Ok(None);
        }
        match self.replies.as_ref().unwrap().try_recv() {
            Ok(reply) => self.accept(reply).map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => self.disconnected(),
        }
    }

    /// Bounded host wait. A timeout leaves the original request pending; it is
    /// not cancellation, failure, permission to retry, or a new provider budget.
    pub fn wait(&mut self, timeout: Duration) -> Result<Option<ApplicationPumpReply>, String> {
        if self.pending.is_none() {
            return Ok(None);
        }
        match self.replies.as_ref().unwrap().recv_timeout(timeout) {
            Ok(reply) => self.accept(reply).map(Some),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => self.disconnected(),
        }
    }

    /// Abandon transport ownership without blocking the host or running Nuis
    /// cleanup. Idle workers wake on disconnect. In-flight effects may finish;
    /// in particular an already admitted close may still finish the provider.
    pub fn abort(&mut self) {
        self.commands.take();
        self.replies.take();
        self.pending = None;
        self.phase = ApplicationPumpPhase::Stopped;
    }

    fn submit(
        &mut self,
        operation: ApplicationPumpOperation,
        arguments: Vec<Value>,
        failed_close: Option<ApplicationFailureKind>,
    ) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("application event pump is busy; consume the pending reply".to_owned());
        }
        if self.phase != ApplicationPumpPhase::Open
            && !(operation == ApplicationPumpOperation::Close
                && self.phase == ApplicationPumpPhase::Faulted)
        {
            return Err(format!(
                "application event pump cannot submit {operation:?} while {:?}",
                self.phase
            ));
        }
        let command = Command {
            operation,
            arguments,
            failed_close,
        };
        match self.commands.as_ref().unwrap().try_send(command) {
            Ok(()) => {
                self.pending = Some(operation);
                Ok(())
            }
            Err(TrySendError::Full(_)) => Err("application event pump channel is full".to_owned()),
            Err(TrySendError::Disconnected(_)) => self.disconnected(),
        }
    }

    fn accept(&mut self, reply: ApplicationPumpReply) -> Result<ApplicationPumpReply, String> {
        if self.pending != Some(reply.operation) {
            self.abort();
            return Err("application event pump received an out-of-order reply".to_owned());
        }
        if let Err(error) = reply.outcome() {
            self.abort();
            return Err(error);
        }
        self.pending = None;
        self.phase = reply.phase;
        if matches!(
            self.phase,
            ApplicationPumpPhase::Closed | ApplicationPumpPhase::Stopped
        ) {
            self.commands.take();
            self.replies.take();
        }
        Ok(reply)
    }

    fn disconnected<T>(&mut self) -> Result<T, String> {
        self.abort();
        Err("application event pump worker disconnected before replying".to_owned())
    }
}
