use std::{ptr, slice};

use yir_core::ModRegistry;
use yir_exec::ExecutionTrace;

#[repr(C)]
pub struct NuisRenderedBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

pub fn render_module_to_ppm_bytes(module_source: &str, scale: usize) -> Result<Vec<u8>, String> {
    let registry = yir_verify::default_registry();
    let trace = execute_module_source_with_registry(module_source, &registry)?;
    render_trace_to_ppm_bytes(&trace, scale)
}

pub fn execute_module_source_with_registry(
    module_source: &str,
    registry: &ModRegistry,
) -> Result<ExecutionTrace, String> {
    let module = yir_syntax::parse_module(module_source)?;
    yir_exec::execute_module_with_registry(&module, registry)
}

pub fn render_trace_to_ppm_bytes(trace: &ExecutionTrace, scale: usize) -> Result<Vec<u8>, String> {
    let frame = trace
        .presented_frames
        .last()
        .ok_or_else(|| "executed YIR graph did not present a frame".to_owned())?;
    let image = yir_host_render::rasterize_frame(frame, scale);
    Ok(image.to_ppm())
}

#[no_mangle]
/// # Safety
///
/// `source_ptr` must point to `source_len` readable bytes, and `out_buffer` must be a valid,
/// writable pointer to a `NuisRenderedBuffer`.
pub unsafe extern "C" fn nuis_render_embedded_yir_ppm(
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
/// # Safety
///
/// `ptr` and `len` must come from a successful `nuis_render_embedded_yir_ppm` call and must not
/// have been freed already.
pub unsafe extern "C" fn nuis_rendered_buffer_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[no_mangle]
/// # Safety
///
/// `out_buffer` must be a valid, writable pointer to a `NuisRenderedBuffer`.
pub unsafe extern "C" fn nuis_rendered_buffer_reset(out_buffer: *mut NuisRenderedBuffer) {
    if out_buffer.is_null() {
        return;
    }
    unsafe {
        (*out_buffer).ptr = ptr::null_mut();
        (*out_buffer).len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{FrameSurface, Value};

    #[test]
    fn renderer_requires_a_structurally_presented_frame() {
        let frame = FrameSurface {
            width: 1,
            height: 1,
            rows: vec!["#".to_owned()],
            rgba8: None,
        };
        let mut trace = ExecutionTrace::default();
        trace
            .values
            .insert("unpresented".to_owned(), Value::Frame(frame.clone()));
        assert!(render_trace_to_ppm_bytes(&trace, 1)
            .unwrap_err()
            .contains("did not present a frame"));

        trace.presented_frames.push(frame);
        let ppm = render_trace_to_ppm_bytes(&trace, 1).unwrap();
        assert!(ppm.starts_with(b"P6\n1 1\n255\n"));
    }
}
