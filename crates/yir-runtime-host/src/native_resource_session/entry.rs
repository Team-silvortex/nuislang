use super::{
    NativeCheckedResourceCallback, NativeMixedResourceCallback, NativeResourceBindings,
    NativeResourceCallback, NativeResourceSession, ResourceScript,
};
use std::{ffi::CStr, os::raw::c_char};
use yir_core::native_resource_session::{
    registered, ResourceSessionRegistration, MAX_BINDING_SOURCE_BYTES,
};

/// Static-link record emitted beside the callbacks. It is not a plugin loader.
#[repr(C)]
pub struct NativeResourceDescriptor<C: Copy> {
    pub version: u64,
    pub source: *const u8,
    pub source_len: u64,
    pub registration: *const u8,
    pub registration_len: u64,
    pub exports: *const u8,
    pub exports_len: u64,
    pub open: Option<C>,
    pub event: Option<C>,
    pub close: Option<C>,
}
pub type NativeResourceDescriptorV1 = NativeResourceDescriptor<NativeResourceCallback>;
pub type NativeMixedResourceDescriptorV1 = NativeResourceDescriptor<NativeMixedResourceCallback>;
pub type NativeCheckedResourceDescriptorV1 =
    NativeResourceDescriptor<NativeCheckedResourceCallback>;

unsafe fn text<'a>(pointer: *const u8, length: u64, max: u64) -> Result<&'a str, String> {
    if pointer.is_null() || length == 0 || length > max || length > isize::MAX as u64 {
        return Err("invalid resource descriptor byte range".to_owned());
    }
    std::str::from_utf8(unsafe { std::slice::from_raw_parts(pointer, length as usize) })
        .map_err(|_| "resource descriptor is not UTF-8".to_owned())
}

unsafe fn run<C: Copy>(
    source: *const u8,
    length: u64,
    descriptor: *const NativeResourceDescriptor<C>,
    argc: i32,
    argv: *const *const c_char,
    version: u64,
    bind: for<'a> unsafe fn(
        &'a yir_core::YirModule,
        &ResourceSessionRegistration,
        [C; 3],
    ) -> Result<NativeResourceBindings<'a>, String>,
) -> Result<(), String> {
    if descriptor.is_null()
        || !(descriptor as usize)
            .is_multiple_of(std::mem::align_of::<NativeResourceDescriptor<C>>())
        || argv.is_null()
        || !(1..=super::script::MAX_ARGC as i32).contains(&argc)
    {
        return Err("invalid native resource entry arguments".to_owned());
    }
    let descriptor = unsafe { &*descriptor };
    if descriptor.version != version {
        return Err("unsupported resource descriptor version".to_owned());
    }
    let expected = unsafe { text(source, length, MAX_BINDING_SOURCE_BYTES)? };
    let compiled = unsafe {
        text(
            descriptor.source,
            descriptor.source_len,
            MAX_BINDING_SOURCE_BYTES,
        )?
    };
    let declaration = unsafe { text(descriptor.registration, descriptor.registration_len, 2048)? };
    let exports = unsafe { text(descriptor.exports, descriptor.exports_len, 4096)? };
    let module = yir_syntax::parse_explicit_module(expected)?;
    let compiled_module = yir_syntax::parse_explicit_module(compiled)?;
    if module != compiled_module {
        return Err("native resource artifact graph identity mismatch".to_owned());
    }
    let declaration_registration = ResourceSessionRegistration::from_fields(
        &declaration.split_whitespace().collect::<Vec<_>>(),
    )?;
    let registration = registered(&module, &declaration_registration.id)?;
    if registration != &declaration_registration
        || registration.canonical() != declaration
        || registration.symbols().join("\n") != exports
    {
        return Err("native resource registration/export identity mismatch".to_owned());
    }
    let callbacks = [
        descriptor.open.ok_or("missing resource open export")?,
        descriptor.event.ok_or("missing resource event export")?,
        descriptor.close.ok_or("missing resource close export")?,
    ];
    let pointers = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
    let args = pointers[1..]
        .iter()
        .map(|&pointer| {
            if pointer.is_null() {
                return Err("null resource script argument");
            }
            unsafe { CStr::from_ptr(pointer) }
                .to_str()
                .map_err(|_| "resource script argument is not UTF-8")
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Admit the complete script before executing even open.
    yir_verify::verify_module(&module)?;
    let script = ResourceScript::parse(&args, registration)?;
    if let Some(frame) = &script.frame {
        frame.preflight_state(
            &script.initial,
            registration.mixed_layout(&module)?.as_ref(),
        )?;
    }
    let binding = unsafe { bind(&compiled_module, registration, callbacks)? };
    let (mut session, completion) =
        NativeResourceSession::open(&module, registration, binding, script.initial, script.open)?;
    eprintln!(
        "nuis: native_resource_state=open:{}:{:?}",
        completion.sequence,
        session.state()
    );
    log_scalars(&session, "open", completion.sequence);
    for argument in script.events {
        match session.event(argument) {
            Ok(completion) => {
                eprintln!(
                    "nuis: native_resource_state=event:{}:{:?}",
                    completion.sequence,
                    session.state()
                );
                log_scalars(&session, "event", completion.sequence);
            }
            Err(_) => break,
        }
    }
    if let Some(completion) = session.close(script.close)? {
        eprintln!(
            "nuis: native_resource_state=close:{}:{:?}",
            completion.sequence,
            session.state()
        );
        log_scalars(&session, "close", completion.sequence);
    }
    session.completion_status()?;
    if let Some(frame) = &script.frame {
        frame.write_closed(&session)?;
    }
    eprintln!("nuis: native_resource_completed=1");
    Ok(())
}

/// Native-only packaged entry. No dynamic loading or reference fallback.
/// # Safety
/// Descriptor and UTF-8 ranges must be readable for this call. Callback pointers
/// must belong to its trusted static object and satisfy from_static's contract.
/// argv contains argc valid NUL-terminated strings. No pointer escapes. Native
/// traps terminate the process; catch_unwind only contains Rust host panics.
#[no_mangle]
pub unsafe extern "C" fn nuis_native_resource_script_main(
    source: *const u8,
    length: u64,
    descriptor: *const NativeResourceDescriptorV1,
    argc: i32,
    argv: *const *const c_char,
) -> i32 {
    finish(std::panic::catch_unwind(|| unsafe {
        run(source, length, descriptor, argc, argv, 1, bind_bytes)
    }))
}

fn log_scalars(session: &NativeResourceSession<'_>, role: &str, sequence: u64) {
    if session.mixed_layout().is_some() {
        eprintln!(
            "nuis: native_resource_scalars={role}:{sequence}:{:?}",
            session.scalar_state()
        );
    }
}

/// Static entry for the distinct mixed-resource callback ABI.
/// # Safety
/// Same valid ranges and static provenance requirements as the resource entry,
/// but the descriptor must contain mixed-ABI callbacks and version 2.
#[no_mangle]
pub unsafe extern "C" fn nuis_native_mixed_resource_script_main(
    source: *const u8,
    length: u64,
    descriptor: *const NativeMixedResourceDescriptorV1,
    argc: i32,
    argv: *const *const c_char,
) -> i32 {
    finish(std::panic::catch_unwind(|| unsafe {
        run(source, length, descriptor, argc, argv, 2, bind_mixed)
    }))
}

unsafe fn bind_bytes<'a>(
    module: &'a yir_core::YirModule,
    registration: &ResourceSessionRegistration,
    callbacks: [NativeResourceCallback; 3],
) -> Result<NativeResourceBindings<'a>, String> {
    unsafe { NativeResourceBindings::from_static(module, registration, callbacks) }
}

/// Static entry for Result<State, i64>, with a distinct error output channel.
/// # Safety
/// Same readable ranges and static provenance requirements as the other entries;
/// the descriptor must use checked callbacks and discriminator 3.
#[no_mangle]
pub unsafe extern "C" fn nuis_native_checked_resource_script_main(
    source: *const u8,
    length: u64,
    descriptor: *const NativeCheckedResourceDescriptorV1,
    argc: i32,
    argv: *const *const c_char,
) -> i32 {
    finish(std::panic::catch_unwind(|| unsafe {
        run(source, length, descriptor, argc, argv, 3, bind_checked)
    }))
}

unsafe fn bind_checked<'a>(
    module: &'a yir_core::YirModule,
    registration: &ResourceSessionRegistration,
    callbacks: [NativeCheckedResourceCallback; 3],
) -> Result<NativeResourceBindings<'a>, String> {
    unsafe { NativeResourceBindings::from_checked_static(module, registration, callbacks) }
}
unsafe fn bind_mixed<'a>(
    module: &'a yir_core::YirModule,
    registration: &ResourceSessionRegistration,
    callbacks: [NativeMixedResourceCallback; 3],
) -> Result<NativeResourceBindings<'a>, String> {
    unsafe { NativeResourceBindings::from_mixed_static(module, registration, callbacks) }
}

fn finish(result: std::thread::Result<Result<(), String>>) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("nuis native resource: {error}");
            1
        }
        Err(_) => 1,
    }
}
