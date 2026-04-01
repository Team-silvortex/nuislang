use std::{ptr, slice};

use yir_core::Value;

#[repr(C)]
pub struct NuisRenderedBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

pub fn render_module_to_ppm_bytes(module_source: &str, scale: usize) -> Result<Vec<u8>, String> {
    let module = yir_syntax::parse_module(module_source)?;
    yir_verify::verify_module(&module)?;
    let trace = yir_exec::execute_module(&module)?;
    let frame = trace
        .values
        .values()
        .find_map(|value| match value {
            Value::Frame(frame) => Some(frame),
            _ => None,
        })
        .ok_or_else(|| "no frame value found in executed YIR graph".to_owned())?;
    let image = yir_host_render::rasterize_frame(frame, scale);
    Ok(image.to_ppm())
}

#[no_mangle]
pub extern "C" fn nuis_render_embedded_yir_ppm(
    source_ptr: *const u8,
    source_len: usize,
    scale: usize,
    out_buffer: *mut NuisRenderedBuffer,
) -> i32 {
    if source_ptr.is_null() || out_buffer.is_null() {
        return 1;
    }

    let source_bytes = unsafe { slice::from_raw_parts(source_ptr, source_len) };
    let Ok(source) = std::str::from_utf8(source_bytes) else {
        return 2;
    };
    let Ok(ppm) = render_module_to_ppm_bytes(source, scale) else {
        return 3;
    };

    let mut bytes = ppm.into_boxed_slice();
    let len = bytes.len();
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);

    unsafe {
        (*out_buffer).ptr = ptr;
        (*out_buffer).len = len;
    }
    0
}

#[no_mangle]
pub extern "C" fn nuis_rendered_buffer_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[no_mangle]
pub extern "C" fn nuis_rendered_buffer_reset(out_buffer: *mut NuisRenderedBuffer) {
    if out_buffer.is_null() {
        return;
    }
    unsafe {
        (*out_buffer).ptr = ptr::null_mut();
        (*out_buffer).len = 0;
    }
}
