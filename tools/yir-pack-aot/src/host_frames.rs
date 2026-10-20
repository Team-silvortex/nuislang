use crate::host_runtime_library::ensure_runtime_host_staticlib_built;
use crate::host_source::bytes_to_c_array;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use yir_core::Value;
use yir_core::YirModule;
use yir_exec::execute_module;
use yir_host_render::rasterize_frame;

pub(super) fn maybe_emit_prerendered_frame(
    module: &YirModule,
    output_dir: &Path,
    stem: &str,
    frame_scale: usize,
) -> Result<Option<FrameBundle>, String> {
    let has_non_cpu = module
        .resources
        .iter()
        .any(|resource| !resource.kind.is_family("cpu"));
    if !has_non_cpu {
        return Ok(None);
    }

    let trace = execute_module(module)?;
    let frame = trace
        .values
        .values()
        .filter_map(|value| match value {
            Value::Frame(frame) => Some(frame),
            _ => None,
        })
        .next_back();

    let Some(frame) = frame else {
        return Ok(None);
    };

    let image = rasterize_frame(frame, frame_scale);
    let assets_dir = output_dir.join("assets");
    fs::create_dir_all(&assets_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", assets_dir.display()))?;
    let ppm_path = assets_dir.join(format!("{stem}.ppm"));
    let ppm_bytes = image.to_ppm();
    fs::write(&ppm_path, &ppm_bytes)
        .map_err(|error| format!("failed to write `{}`: {error}", ppm_path.display()))?;
    Ok(Some(FrameBundle {
        asset_path: ppm_path,
        embedded_ppm_bytes: bytes_to_c_array(&ppm_bytes),
    }))
}

pub(super) fn cleanup_stale_fallback_frame(output_dir: &Path, stem: &str) -> Result<(), String> {
    let stale_path = output_dir.join("assets").join(format!("{stem}.ppm"));
    if stale_path.exists() {
        fs::remove_file(&stale_path)
            .map_err(|error| format!("failed to remove `{}`: {error}", stale_path.display()))?;
    }
    Ok(())
}

pub(super) struct FrameBundle {
    pub(super) asset_path: PathBuf,
    pub(super) embedded_ppm_bytes: String,
}

pub(super) struct RuntimeFrameSupport {
    pub(super) staticlib_path: PathBuf,
    pub(super) embedded_module_bytes: String,
    pub(super) frame_scale: usize,
}

pub(super) fn maybe_prepare_embedded_runtime_support(
    module: &YirModule,
    source: &str,
    frame_scale: usize,
) -> Result<Option<RuntimeFrameSupport>, String> {
    let has_tick = module
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "tick_i64");
    if !has_tick && module.application_sessions.is_empty() {
        return Ok(None);
    }

    let staticlib_path = ensure_runtime_host_staticlib_built()?;
    Ok(Some(RuntimeFrameSupport {
        staticlib_path,
        embedded_module_bytes: bytes_to_c_array(source.as_bytes()),
        frame_scale,
    }))
}
