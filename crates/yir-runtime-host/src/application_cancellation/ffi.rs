use std::ptr;

use super::ApplicationCancellation;

/// Independent host and provider observations. Provider status codes: 0 not
/// requested, 1 not observed, 2 replay only, 3 unavailable frontier, 4 failed,
/// 5 drained. Dispatches is -1 unless status is 5. Cleanup never implies success.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NuisApplicationCancellationReceipt {
    pub cleanup_completed: i32,
    pub provider_status: i32,
    pub failure_kind: i64,
    pub provider_failure_kind: i64,
    pub completed_dispatches: i64,
}

impl From<super::ApplicationHostRetirementAck> for NuisApplicationCancellationReceipt {
    fn from(ack: super::ApplicationHostRetirementAck) -> Self {
        let provider = ack.provider_drain();
        Self {
            cleanup_completed: i32::from(ack.cleanup_completed()),
            provider_status: provider.code(),
            failure_kind: ack.failure_kind().code(),
            provider_failure_kind: provider.failure_kind().code(),
            completed_dispatches: provider.completed_dispatches().map_or(-1, |n| n as i64),
        }
    }
}

impl NuisApplicationCancellationReceipt {
    pub fn provider_drain_exit_status(&self) -> i32 {
        if matches!(self.cleanup_completed, 0 | 1)
            && self.failure_kind == 0
            && self.provider_status == 5
            && self.provider_failure_kind == 0
            && (0..=yir_core::provider_runtime_ipc::MAX_DISPATCHES as i64)
                .contains(&self.completed_dispatches)
        {
            super::APPLICATION_CANCELLED_EXIT_CODE
        } else {
            1
        }
    }
}

/// Portable classification of an explicitly requested provider-drain receipt.
/// It does not poll, perform cleanup or mutate the ticket. A platform adapter
/// must not replace this decision with an inference from its own window state.
/// # Safety
/// Receipt must be null or point to a readable, aligned receipt for this call.
#[no_mangle]
pub unsafe extern "C" fn nuis_application_provider_drain_exit_status(
    receipt: *const NuisApplicationCancellationReceipt,
) -> i32 {
    let Some(receipt) = (unsafe { receipt.as_ref() }) else {
        return 1;
    };
    receipt.provider_drain_exit_status()
}

fn fail(error: impl std::fmt::Display) -> i32 {
    eprintln!("nuis application cancellation: {error}");
    -1
}

/// Nonblocking: 0 pending/already consumed, 1 one host-retirement receipt,
/// -1 invalid arguments or lost acknowledgement. Outputs are written only on 1;
/// cleanup is 0/1 and failure is the first ApplicationFailureKind code. Neither
/// field certifies application success or provider/device resource retirement.
/// # Safety
/// Ticket must be an exclusive live cancellation handle. Outputs must be
/// writable and nonaliasing with each other and the ticket. Calls must not overlap.
#[no_mangle]
pub unsafe extern "C" fn nuis_application_cancellation_poll(
    ticket: *mut ApplicationCancellation,
    cleanup: *mut i32,
    failure: *mut i64,
) -> i32 {
    if cleanup.is_null() || failure.is_null() {
        return fail("null retirement output");
    }
    let Some(ticket) = (unsafe { ticket.as_mut() }) else {
        return fail("null cancellation ticket");
    };
    match ticket.poll() {
        Ok(Some(ack)) => {
            unsafe {
                *cleanup = i32::from(ack.cleanup_completed());
                *failure = ack.failure_kind().code();
            }
            1
        }
        Ok(None) => 0,
        Err(error) => fail(error),
    }
}

/// Nonblocking: 0 pending/already consumed, 1 receipt, -1 invalid/lost receipt.
/// This and the host-only poll consume the same one-shot acknowledgement.
/// Invalid arguments and pending polls do not change outputs or consume it.
/// # Safety
/// Ticket must be exclusive and live; output must be writable, aligned and
/// nonaliasing with the ticket. Poll and free calls must not overlap.
#[no_mangle]
pub unsafe extern "C" fn nuis_application_cancellation_poll_with_provider(
    ticket: *mut ApplicationCancellation,
    output: *mut NuisApplicationCancellationReceipt,
) -> i32 {
    if output.is_null() {
        return fail("null retirement output");
    }
    let Some(ticket) = (unsafe { ticket.as_mut() }) else {
        return fail("null cancellation ticket");
    };
    match ticket.poll() {
        Ok(Some(ack)) => {
            unsafe {
                *output = ack.into();
            }
            1
        }
        Ok(None) => 0,
        Err(error) => fail(error),
    }
}

/// Abandon observation without joining, running cleanup or claiming retirement.
/// # Safety
/// Slot must be writable and contain null or an exclusively owned live ticket.
/// Never free a copy of a previously freed handle; null slots are harmless.
#[no_mangle]
pub unsafe extern "C" fn nuis_application_cancellation_free(
    slot: *mut *mut ApplicationCancellation,
) {
    if let Some(slot) = unsafe { slot.as_mut() } {
        let handle = std::mem::replace(slot, ptr::null_mut());
        if !handle.is_null() {
            drop(unsafe { Box::from_raw(handle) });
        }
    }
}

#[cfg(test)]
mod tests;
