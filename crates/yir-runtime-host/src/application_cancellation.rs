use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        mpsc::{Receiver, RecvTimeoutError, TryRecvError},
    },
    time::Duration,
};

use crate::{
    application_scope_admission::{ScopeAbandonment, ScopeAdmission},
    ProviderDrainObservation,
};
use yir_core::ApplicationFailureKind;

mod ffi;
pub use ffi::*;

pub const APPLICATION_CANCELLATION_CONTRACT: &str = "nuis-yir-application-cancellation-v1";

const ACTIVE: u8 = 0;
const CANCELLED: u8 = 1;
const FINALIZING: u8 = 2;
const EXITED: u8 = 3;
const CANCELLED_DRAIN: u8 = 4;
const ABANDONING: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CancellationMode {
    HostOnly,
    ProviderDrain,
}

#[derive(Default)]
pub(crate) struct CancellationControl(AtomicU8);

impl CancellationControl {
    pub(crate) fn request(&self, mode: CancellationMode) -> Result<(), String> {
        let next = match mode {
            CancellationMode::HostOnly => CANCELLED,
            CancellationMode::ProviderDrain => CANCELLED_DRAIN,
        };
        self.0
            .compare_exchange(ACTIVE, next, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|phase| {
                match phase {
                    FINALIZING => {
                        "application cancellation rejected: finalization already admitted"
                    }
                    CANCELLED | CANCELLED_DRAIN => "application cancellation already admitted",
                    ABANDONING => "application cancellation rejected: scope already abandoning",
                    _ => "application cancellation rejected: worker already exited",
                }
                .to_owned()
            })
    }

    pub(crate) fn cancelled(&self) -> bool {
        matches!(self.0.load(Ordering::Acquire), CANCELLED | CANCELLED_DRAIN)
    }

    /// Called only after the worker's entire borrowed execution scope returns.
    pub(crate) fn retire(&self) -> Option<CancellationMode> {
        match self.0.swap(EXITED, Ordering::AcqRel) {
            CANCELLED => Some(CancellationMode::HostOnly),
            CANCELLED_DRAIN => Some(CancellationMode::ProviderDrain),
            _ => None,
        }
    }
}

impl ScopeAdmission for CancellationControl {
    fn checkpoint(&self) -> Result<(), String> {
        if self.cancelled() {
            Err("application cancelled at a host callback boundary".to_owned())
        } else {
            Ok(())
        }
    }

    fn admit_finalization(&self) -> Result<(), String> {
        self.0
            .compare_exchange(ACTIVE, FINALIZING, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| "application finalization admission rejected".to_owned())
    }

    fn admit_abandonment(&self) -> ScopeAbandonment {
        match self
            .0
            .compare_exchange(ACTIVE, ABANDONING, Ordering::AcqRel, Ordering::Acquire)
        {
            Err(CANCELLED_DRAIN) => ScopeAbandonment::Drain,
            _ => ScopeAbandonment::Drop,
        }
    }
}

/// Descriptive acknowledgement that the pump's worker-owned module, execution
/// state, registry and transport have been dropped. Not device/provider resource
/// retirement, application success, or permission to reuse in-flight resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationHostRetirementAck {
    cleanup_completed: bool,
    failure_kind: ApplicationFailureKind,
    provider_drain: ProviderDrainObservation,
}

impl ApplicationHostRetirementAck {
    pub(crate) fn new(cleanup_completed: bool, failure_kind: ApplicationFailureKind) -> Self {
        Self {
            cleanup_completed,
            failure_kind,
            provider_drain: ProviderDrainObservation::NotRequested,
        }
    }

    /// Only true if an explicitly admitted close callback passed trace checks.
    /// Cancellation itself never executes close or manufactures this flag.
    pub fn cleanup_completed(&self) -> bool {
        self.cleanup_completed
    }

    /// First latched execution/provider failure, independent of cancellation.
    /// None does not mean the abandoned application completed successfully.
    pub fn failure_kind(&self) -> ApplicationFailureKind {
        self.failure_kind
    }

    pub fn provider_drain(&self) -> ProviderDrainObservation {
        self.provider_drain
    }

    pub(crate) fn with_provider_drain(mut self, observation: ProviderDrainObservation) -> Self {
        self.provider_drain = observation;
        self
    }

    pub(crate) fn for_cancellation(mut self, mode: CancellationMode) -> Self {
        if mode == CancellationMode::ProviderDrain
            && self.provider_drain == ProviderDrainObservation::NotRequested
        {
            self.provider_drain = ProviderDrainObservation::NotObserved;
        }
        self
    }
}

/// One admitted cancellation, one host-scope acknowledgement. Polling or timing
/// out never retries work; dropping this ticket neither joins nor runs cleanup.
#[derive(Debug)]
pub struct ApplicationCancellation {
    reply: Option<Receiver<ApplicationHostRetirementAck>>,
}

impl ApplicationCancellation {
    pub(crate) fn new(reply: Receiver<ApplicationHostRetirementAck>) -> Self {
        Self { reply: Some(reply) }
    }

    pub fn poll(&mut self) -> Result<Option<ApplicationHostRetirementAck>, String> {
        let Some(reply) = &self.reply else {
            return Ok(None);
        };
        match reply.try_recv() {
            Ok(ack) => {
                self.reply.take();
                Ok(Some(ack))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => self.disconnected(),
        }
    }

    pub fn wait(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<ApplicationHostRetirementAck>, String> {
        let Some(reply) = &self.reply else {
            return Ok(None);
        };
        match reply.recv_timeout(timeout) {
            Ok(ack) => {
                self.reply.take();
                Ok(Some(ack))
            }
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => self.disconnected(),
        }
    }

    fn disconnected<T>(&mut self) -> Result<T, String> {
        self.reply.take();
        Err("application worker disconnected without a host retirement acknowledgement".to_owned())
    }
}

#[cfg(test)]
#[path = "application_cancellation/tests.rs"]
mod tests;
