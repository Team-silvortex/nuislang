use super::{NativeScalarCallback, NativeSessionBindings};
use crate::ApplicationSession;
use std::{ffi::CStr, os::raw::c_char};
use yir_core::native_scalar_session::MAX_BINDING_SOURCE_BYTES;

#[path = "input.rs"]
mod input;

/// Frontdoors and the packaged host use the same typed, bounded script grammar.
/// This admits arguments only; it never opens a session or resolves a provider.
pub fn validate_native_application_script(
    module: &yir_core::YirModule,
    id: &str,
    layout: &str,
    arguments: &[&str],
) -> Result<(), String> {
    yir_verify::verify_module(module)?;
    let layout = yir_core::native_scalar_session::ScalarStateLayout::parse(layout)?;
    let kinds = layout.bind(module, id)?;
    input::parse(arguments, id, &kinds, layout.fields().len()).map(|_| ())
}

/// Internal static-link descriptor. Lengths and callback slot counts are u64 on
/// every target, rather than platform size_t. Version 1 has exactly three roles.
/// The producer emits this beside its native callbacks, not in a mutable sidecar.
#[repr(C)]
pub struct NativeSessionDescriptorV1 {
    pub version: u64,
    pub source: *const u8,
    pub source_len: u64,
    pub id: *const u8,
    pub id_len: u64,
    pub layout: *const u8,
    pub layout_len: u64,
    pub open: Option<NativeScalarCallback>,
    pub event: Option<NativeScalarCallback>,
    pub close: Option<NativeScalarCallback>,
}

unsafe fn utf8<'a>(pointer: *const u8, length: u64, max: u64) -> Result<&'a str, String> {
    if pointer.is_null() || length == 0 || length > max || length > isize::MAX as u64 {
        return Err("invalid native application descriptor byte range".to_owned());
    }
    std::str::from_utf8(unsafe { std::slice::from_raw_parts(pointer, length as usize) })
        .map_err(|_| "native application descriptor is not UTF-8".to_owned())
}

unsafe fn run(
    source: *const u8,
    length: u64,
    descriptor: *const NativeSessionDescriptorV1,
    argc: i32,
    argv: *const *const c_char,
) -> Result<(), String> {
    if descriptor.is_null() || argv.is_null() || !(1..=input::MAX_ARGC as i32).contains(&argc) {
        return Err("invalid native application entry arguments".to_owned());
    }
    let descriptor = unsafe { &*descriptor };
    if descriptor.version != 1 {
        return Err("unsupported native application descriptor version".to_owned());
    }
    let expected = unsafe { utf8(source, length, MAX_BINDING_SOURCE_BYTES)? };
    let compiled = unsafe {
        utf8(
            descriptor.source,
            descriptor.source_len,
            MAX_BINDING_SOURCE_BYTES,
        )?
    };
    let id = unsafe { utf8(descriptor.id, descriptor.id_len, 256)? };
    let layout = unsafe { utf8(descriptor.layout, descriptor.layout_len, 64 * 1024)? };
    // Parse, do not execute. Full graph equality detects stale/mixed objects even
    // when registration names, arity and state layout happen to be unchanged.
    let module = yir_syntax::parse_explicit_module(expected)?;
    let compiled_module = yir_syntax::parse_explicit_module(compiled)?;
    if module != compiled_module {
        return Err("native application session artifact identity mismatch".to_owned());
    }
    let callbacks = [
        descriptor.open.ok_or("native open export is missing")?,
        descriptor.event.ok_or("native event export is missing")?,
        descriptor.close.ok_or("native close export is missing")?,
    ];
    let binding =
        unsafe { NativeSessionBindings::from_static(&compiled_module, id, layout, callbacks)? };
    let pointers = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
    let arguments = pointers[1..]
        .iter()
        .map(|&pointer| {
            if pointer.is_null() {
                return Err("null native application script argument");
            }
            unsafe { CStr::from_ptr(pointer) }
                .to_str()
                .map_err(|_| "native application script argument is not UTF-8")
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Admit the entire script before open, including every later event and close.
    let script = input::parse(
        &arguments,
        id,
        &binding.arguments,
        binding.layout.fields().len(),
    )?;
    let (mut session, _) =
        ApplicationSession::open_native_registered(&module, id, binding, script.open)?;
    eprintln!("nuis: native_session_state=open:{}", session.state());
    for event in script.events {
        if session.event(event).is_err() {
            break;
        }
        eprintln!("nuis: native_session_state=event:{}", session.state());
    }
    if session.close(script.close).is_ok() {
        eprintln!("nuis: native_session_state=close:{}", session.state());
    }
    session.completion_status()?;
    eprintln!("nuis: native_session_completed=1");
    Ok(())
}

/// Explicit native packaged-process entry. There is no provider selection,
/// interpreter fallback, cancellation or native-fuel claim in this first profile.
/// # Safety
/// Both YIR byte ranges, descriptor and its strings must be valid for this call.
/// The descriptor callbacks must satisfy NativeSessionBindings::from_static and
/// actually belong to the bound static object. argv must contain argc valid
/// NUL-terminated UTF-8 strings. No pointer is retained after return. Native traps
/// are process failures; catch_unwind below only contains Rust host panics.
#[no_mangle]
pub unsafe extern "C" fn nuis_native_application_script_main(
    source: *const u8,
    length: u64,
    descriptor: *const NativeSessionDescriptorV1,
    argc: i32,
    argv: *const *const c_char,
) -> i32 {
    match std::panic::catch_unwind(|| unsafe { run(source, length, descriptor, argc, argv) }) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("nuis native application: {error}");
            1
        }
        Err(_) => 1,
    }
}
