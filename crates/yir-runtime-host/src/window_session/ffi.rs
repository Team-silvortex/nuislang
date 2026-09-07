use std::{
    ffi::{c_char, CStr},
    path::PathBuf,
    ptr, slice,
};

use super::WindowSession;
use crate::{ApplicationProviderSource, ApplicationPumpPhase, NuisRenderedBuffer};

fn fail(error: impl std::fmt::Display) -> i32 {
    eprintln!("nuis window session: {error}");
    -1
}

/// Copies embedded source/id before starting asynchronous admission. Exactly one
/// explicit provider environment variable is required; there is no reference fallback.
/// # Safety
/// Source must name readable bytes; id must be a NUL-terminated UTF-8 string.
/// Output must be writable and initially null. The resulting handle is exclusively
/// owned by one host thread, with no concurrent calls, and must be freed once.
#[no_mangle]
pub unsafe extern "C" fn nuis_window_session_open(
    source: *const u8,
    length: usize,
    id: *const c_char,
    width: i64,
    height: i64,
    output: *mut *mut WindowSession,
) -> i32 {
    if source.is_null() || id.is_null() || output.is_null() || length > isize::MAX as usize {
        return fail("invalid open pointer or source length");
    }
    if !unsafe { *output }.is_null() {
        return fail("open output must be empty");
    }
    let source = unsafe { slice::from_raw_parts(source, length) };
    let (Ok(source), Ok(id)) = (
        std::str::from_utf8(source),
        unsafe { CStr::from_ptr(id) }.to_str(),
    ) else {
        return fail("source and registration ID must be UTF-8");
    };
    let live = std::env::var_os(crate::PROVIDER_DISPATCH_SOCKET_ENV).map(PathBuf::from);
    let replay = std::env::var_os(crate::PROVIDER_RESULT_STREAM_ENV).map(PathBuf::from);
    let provider = match (&live, &replay) {
        #[cfg(unix)]
        (Some(path), None) => ApplicationProviderSource::Ipc(path),
        (None, Some(path)) => ApplicationProviderSource::Replay(path),
        _ => return fail("exactly one registered IPC or replay source is required"),
    };
    match WindowSession::spawn(source.to_owned(), provider, id.to_owned(), width, height) {
        Ok(session) => {
            unsafe {
                *output = Box::into_raw(Box::new(session));
            }
            0
        }
        Err(error) => fail(error),
    }
}

/// Return 0 for admission, 1 for busy (nothing queued), -1 for rejection.
/// # Safety
/// Session must be an exclusive live handle returned by nuis_window_session_open.
#[no_mangle]
pub unsafe extern "C" fn nuis_window_session_event(
    session: *mut WindowSession,
    kind: i64,
    code: i64,
) -> i32 {
    let Some(session) = (unsafe { session.as_mut() }) else {
        return fail("null session");
    };
    if session.pending() {
        return 1;
    }
    session.event(kind, code).map_or_else(fail, |_| 0)
}

/// Return 0 for admission, 1 for busy, -1 for rejection; never retry an executed close.
/// # Safety
/// Session must be an exclusive live handle returned by nuis_window_session_open.
#[no_mangle]
pub unsafe extern "C" fn nuis_window_session_close(session: *mut WindowSession) -> i32 {
    let Some(session) = (unsafe { session.as_mut() }) else {
        return fail("null session");
    };
    if session.pending() {
        return 1;
    }
    session.close().map_or_else(fail, |_| 0)
}

/// Return 0 for pending, 1 for one reply, -1 for error. Phase codes are opening=0,
/// open=1, faulted=2, closed=3, stopped=4. A returned buffer is owned by the caller
/// and must use nuis_rendered_buffer_free; replies without frames leave it empty.
/// # Safety
/// Session must be exclusive and live. Phase/buffer must be writable, nonaliasing
/// outputs and the buffer must be empty (null pointer and zero length).
#[no_mangle]
pub unsafe extern "C" fn nuis_window_session_poll(
    session: *mut WindowSession,
    buffer: *mut NuisRenderedBuffer,
    phase: *mut i32,
) -> i32 {
    if buffer.is_null() || phase.is_null() {
        return fail("null poll output");
    }
    let Some(session) = (unsafe { session.as_mut() }) else {
        return fail("null session");
    };
    let output = unsafe { &mut *buffer };
    if !output.ptr.is_null() || output.len != 0 {
        return fail("poll buffer must be empty");
    }
    let reply = session.poll();
    unsafe {
        *phase = match session.phase() {
            ApplicationPumpPhase::Opening => 0,
            ApplicationPumpPhase::Open => 1,
            ApplicationPumpPhase::Faulted => 2,
            ApplicationPumpPhase::Closed => 3,
            ApplicationPumpPhase::Stopped => 4,
        };
    }
    match reply {
        Ok(None) => 0,
        Ok(Some(reply)) => match reply.frame {
            Ok(Some(frame)) => {
                let mut bytes = frame.into_boxed_slice();
                output.len = bytes.len();
                output.ptr = bytes.as_mut_ptr();
                std::mem::forget(bytes);
                1
            }
            Ok(None) => 1,
            Err(error) => fail(error),
        },
        Err(error) => fail(error),
    }
}

/// Abandon the handle; does not join a worker or implicitly execute Nuis cleanup.
/// # Safety
/// Slot must be writable and contain null or the exclusive live handle returned
/// by open. Never free a copy of a handle that was already freed.
#[no_mangle]
pub unsafe extern "C" fn nuis_window_session_free(slot: *mut *mut WindowSession) {
    if let Some(slot) = unsafe { slot.as_mut() } {
        let handle = std::mem::replace(slot, ptr::null_mut());
        if !handle.is_null() {
            drop(unsafe { Box::from_raw(handle) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_rejects_invalid_inputs_and_preserves_live_output_buffers() {
        let mut slot = ptr::null_mut();
        unsafe {
            assert_eq!(
                nuis_window_session_open(ptr::null(), 0, c"id".as_ptr(), 1, 1, &mut slot),
                -1
            );
            assert_eq!(
                nuis_window_session_open([0xff].as_ptr(), 1, c"id".as_ptr(), 1, 1, &mut slot),
                -1
            );
            assert!(slot.is_null());
            assert_eq!(nuis_window_session_event(slot, 0, 0), -1);
            assert_eq!(nuis_window_session_close(slot), -1);
            nuis_window_session_free(&mut slot);
        }
        let session = WindowSession::spawn(
            "yir 0.1\n".to_owned(),
            ApplicationProviderSource::Replay(std::path::Path::new("missing-window-replay")),
            "id".to_owned(),
            1,
            1,
        )
        .unwrap();
        slot = Box::into_raw(Box::new(session));
        let mut bytes = [10, 20, 30];
        let mut buffer = NuisRenderedBuffer {
            ptr: bytes.as_mut_ptr(),
            len: bytes.len(),
        };
        let mut phase = 99;
        unsafe {
            assert_eq!(nuis_window_session_poll(slot, &mut buffer, &mut phase), -1);
            assert_eq!(phase, 99);
            assert_eq!(buffer.ptr, bytes.as_mut_ptr());
            assert_eq!(buffer.len, 3);
            assert_eq!(bytes, [10, 20, 30]);
            nuis_window_session_free(&mut slot);
            assert!(slot.is_null());
            nuis_window_session_free(&mut slot);
        }
    }
}
