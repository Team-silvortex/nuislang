mod provider_result_stream;

use std::{path::Path, ptr, slice};

pub use provider_result_stream::{PROVIDER_RESULT_STREAM_CONTRACT, PROVIDER_RESULT_STREAM_ENV};

use yir_core::ModRegistry;
use yir_exec::ExecutionTrace;

#[repr(C)]
pub struct NuisRenderedBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

pub fn render_module_to_ppm_bytes(module_source: &str, scale: usize) -> Result<Vec<u8>, String> {
    let trace = match std::env::var_os(PROVIDER_RESULT_STREAM_ENV) {
        Some(path) => execute_module_source_with_provider_result_stream(module_source, path)?,
        None => {
            let registry = yir_verify::default_registry();
            execute_module_source_with_registry(module_source, &registry)?
        }
    };
    render_trace_to_ppm_bytes(&trace, scale)
}

pub fn render_module_to_ppm_bytes_with_provider_result_stream(
    module_source: &str,
    scale: usize,
    manifest_path: impl AsRef<Path>,
) -> Result<Vec<u8>, String> {
    let trace = execute_module_source_with_provider_result_stream(module_source, manifest_path)?;
    render_trace_to_ppm_bytes(&trace, scale)
}

pub fn execute_module_source_with_provider_result_stream(
    module_source: &str,
    manifest_path: impl AsRef<Path>,
) -> Result<ExecutionTrace, String> {
    provider_result_stream::execute_with_provider_result_stream(
        module_source,
        manifest_path.as_ref(),
    )
}

pub fn execute_module_source_with_registry(
    module_source: &str,
    registry: &ModRegistry,
) -> Result<ExecutionTrace, String> {
    let module = yir_syntax::parse_module(module_source)?;
    yir_exec::execute_module_with_registry(&module, registry)
}

pub fn count_module_node_executions(
    module_source: &str,
    module_name: &str,
    instruction: &str,
    node_name: &str,
    resource: &str,
) -> Result<usize, String> {
    let module = yir_syntax::parse_module(module_source)?;
    let bound = module.nodes.iter().any(|node| {
        node.name == node_name
            && node.resource == resource
            && node.op.module == module_name
            && node.op.instruction == instruction
    });
    if !bound {
        return Err(format!(
            "provider runtime result target `{module_name}.{instruction}` node `{node_name}` resource `{resource}` is absent from YIR"
        ));
    }
    let registry = yir_verify::default_registry();
    let trace = yir_exec::execute_module_with_registry(&module, &registry)?;
    let expected = format!("{module_name}.{instruction} @{resource} -> {node_name}");
    let count = trace
        .lane_steps
        .values()
        .flatten()
        .filter(|step| *step == &expected)
        .count();
    (count > 0).then_some(count).ok_or_else(|| {
        format!("provider runtime result target `{node_name}` was not executed by YIR")
    })
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
