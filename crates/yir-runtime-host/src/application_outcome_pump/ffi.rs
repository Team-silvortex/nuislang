use std::{
    ffi::{c_char, CStr},
    ptr, slice,
};

use super::{
    host_registry::cpu_parent_registry, ApplicationOutcomePump, ApplicationOutcomePumpPhase,
};
use crate::WindowSession;

fn fail(error: impl std::fmt::Display) -> i32 {
    eprintln!("nuis parent outcome: {error}");
    -1
}

/// Explicit CPU-only parent adapter. No provider environment is read or inherited.
/// Source/id are copied; init/open/event/close each have 100,000 executor steps.
/// # Safety
/// Source must name readable UTF-8 bytes and id a NUL-terminated UTF-8 string.
/// Output must be writable and initially null. One host thread exclusively owns
/// the resulting handle; calls must not overlap and it must be freed once.
#[no_mangle]
pub unsafe extern "C" fn nuis_outcome_parent_open(
    source: *const u8,
    length: usize,
    id: *const c_char,
    output: *mut *mut ApplicationOutcomePump,
) -> i32 {
    if source.is_null() || id.is_null() || output.is_null() || length > isize::MAX as usize {
        return fail("invalid parent open pointer or length");
    }
    if !unsafe { *output }.is_null() {
        return fail("parent output must be empty");
    }
    let source = unsafe { slice::from_raw_parts(source, length) };
    let (Ok(source), Ok(id)) = (
        std::str::from_utf8(source),
        unsafe { CStr::from_ptr(id) }.to_str(),
    ) else {
        return fail("parent source and ID must be UTF-8");
    };
    match ApplicationOutcomePump::spawn_with_registry(
        source.to_owned(),
        id.to_owned(),
        cpu_parent_registry,
        100_000,
        true,
    ) {
        Ok(parent) => {
            unsafe {
                *output = Box::into_raw(Box::new(parent));
            }
            0
        }
        Err(error) => fail(error),
    }
}

/// Issue the child's one owned outcome attempt, then explicitly close the parent
/// after its event (also on event failure). Return 0 for admission, -1 otherwise.
/// Phase/availability rejection happens before consuming the child slot. Once
/// admitted, a failed or abandoned delivery is not retriable.
/// # Safety
/// Parent and child must be distinct, exclusive live handles on their host thread.
#[no_mangle]
pub unsafe extern "C" fn nuis_outcome_parent_finish(
    parent: *mut ApplicationOutcomePump,
    child: *mut WindowSession,
) -> i32 {
    let (Some(parent), Some(child)) = (unsafe { parent.as_mut() }, unsafe { child.as_mut() })
    else {
        return fail("null parent or child");
    };
    if parent.phase() != ApplicationOutcomePumpPhase::Open {
        return fail("parent is not open");
    }
    let Some(delivery) = child.take_outcome_delivery() else {
        return fail("child outcome unavailable or already taken");
    };
    parent
        .finish_with_outcome(delivery)
        .map_or_else(fail, |_| 0)
}

/// Return 0 pending/no reply, 1 accepted reply, -1 error. Phases: opening=0,
/// open=1, finishing=2, closed=3, stopped=4. Delivered/cleanup are independent
/// callback results, not child success or resource-retirement authority.
/// # Safety
/// Parent must be exclusive and live; outputs writable and pairwise nonaliasing.
#[no_mangle]
pub unsafe extern "C" fn nuis_outcome_parent_poll(
    parent: *mut ApplicationOutcomePump,
    phase: *mut i32,
    delivered: *mut i32,
    cleanup: *mut i32,
) -> i32 {
    if phase.is_null() || delivered.is_null() || cleanup.is_null() {
        return fail("null parent poll output");
    }
    let Some(parent) = (unsafe { parent.as_mut() }) else {
        return fail("null parent");
    };
    let reply = parent.poll();
    unsafe {
        *phase = match parent.phase() {
            ApplicationOutcomePumpPhase::Opening => 0,
            ApplicationOutcomePumpPhase::Open => 1,
            ApplicationOutcomePumpPhase::Finishing => 2,
            ApplicationOutcomePumpPhase::Closed => 3,
            ApplicationOutcomePumpPhase::Stopped => 4,
        };
        *delivered = 0;
        *cleanup = 0;
    }
    match reply {
        Ok(None) => 0,
        Ok(Some(reply)) => {
            unsafe {
                *delivered = i32::from(reply.delivered);
                *cleanup = i32::from(reply.cleanup_completed);
            }
            reply.result.map_or_else(fail, |_| 1)
        }
        Err(error) => fail(error),
    }
}

/// Abandon without joining, implicit cleanup or cancellation of admitted effects.
/// # Safety
/// Slot must be writable and contain null or the exclusive live handle from open.
#[no_mangle]
pub unsafe extern "C" fn nuis_outcome_parent_free(slot: *mut *mut ApplicationOutcomePump) {
    if let Some(slot) = unsafe { slot.as_mut() } {
        let handle = std::mem::replace(slot, ptr::null_mut());
        if !handle.is_null() {
            drop(unsafe { Box::from_raw(handle) });
        }
    }
}
