#[cfg(all(test, target_os = "macos"))]
use std::{fs, path::PathBuf, time::SystemTime};

#[cfg(target_os = "macos")]
use crate::provider_process_adapter::{
    ProviderProcessAdapterCache, ResolvedProviderProcessAdapter,
};
use crate::provider_runner_metal::MetalProviderExecution;
use std::path::Path;

#[cfg(target_os = "macos")]
const METAL_RGBA8_RENDER_SOURCE: &str = include_str!("../provider-runners/metal_rgba8_render.m");
pub(crate) const METAL_RGBA8_RENDER_CONTRACT: &str = "nuis-metal-rgba8-render-provider-runner-v3";

#[cfg(target_os = "macos")]
pub(crate) fn prepare_rgba8_render_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    crate::provider_runner_metal::prepare_metal_worker_invocation(
        cache,
        METAL_RGBA8_RENDER_SOURCE,
        METAL_RGBA8_RENDER_CONTRACT,
    )
}

#[cfg(test)]
pub(crate) fn execute_rgba8_render(
    msl_source: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    width: usize,
    height: usize,
) -> Result<MetalProviderExecution, String> {
    execute_rgba8_render_platform(msl_source, vertex_entry, fragment_entry, width, height)
}

#[cfg(all(test, target_os = "macos"))]
fn execute_rgba8_render_platform(
    msl_source: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    width: usize,
    height: usize,
) -> Result<MetalProviderExecution, String> {
    if width == 0 || height == 0 {
        return Err("Metal RGBA8 render dimensions must be positive".to_owned());
    }
    let source = TempMslSource::materialize(msl_source)?;
    execute_rgba8_render_asset_platform(
        &source.path,
        vertex_entry,
        fragment_entry,
        width,
        height,
        (4, 1),
        "none",
    )
}

pub(crate) fn execute_rgba8_render_asset(
    msl_path: &Path,
    vertex_entry: &str,
    fragment_entry: &str,
    width: usize,
    height: usize,
    counts: (usize, usize),
    uniform_upload: &str,
) -> Result<MetalProviderExecution, String> {
    execute_rgba8_render_asset_platform(
        msl_path,
        vertex_entry,
        fragment_entry,
        width,
        height,
        counts,
        uniform_upload,
    )
}

#[cfg(target_os = "macos")]
fn execute_rgba8_render_asset_platform(
    msl_path: &Path,
    vertex_entry: &str,
    fragment_entry: &str,
    width: usize,
    height: usize,
    counts: (usize, usize),
    uniform_upload: &str,
) -> Result<MetalProviderExecution, String> {
    let (vertex_count, instance_count) = counts;
    if width == 0 || height == 0 {
        return Err("Metal RGBA8 render dimensions must be positive".to_owned());
    }
    if !(1..=4).contains(&vertex_count) || !(1..=256).contains(&instance_count) {
        return Err("Metal unbound draw count exceeds admitted budget".to_owned());
    }
    let output_byte_len = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "Metal RGBA8 render byte count overflow".to_owned())?;
    crate::provider_runner_metal::execute_metal_platform(
        msl_path.as_os_str(),
        &[
            vertex_entry.to_owned(),
            fragment_entry.to_owned(),
            width.to_string(),
            height.to_string(),
            vertex_count.to_string(),
            instance_count.to_string(),
            uniform_upload.to_owned(),
        ],
        METAL_RGBA8_RENDER_CONTRACT,
        METAL_RGBA8_RENDER_SOURCE,
        None,
        Some(output_byte_len),
    )
}

#[cfg(not(target_os = "macos"))]
fn execute_rgba8_render_asset_platform(
    _msl_path: &Path,
    _vertex_entry: &str,
    _fragment_entry: &str,
    _width: usize,
    _height: usize,
    _counts: (usize, usize),
    _uniform_upload: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(all(test, not(target_os = "macos")))]
fn execute_rgba8_render_platform(
    _msl_source: &str,
    _vertex_entry: &str,
    _fragment_entry: &str,
    _width: usize,
    _height: usize,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(all(test, target_os = "macos"))]
struct TempMslSource {
    path: PathBuf,
}

#[cfg(all(test, target_os = "macos"))]
impl TempMslSource {
    fn materialize(source: &str) -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nuis-nsdb-metal-render-{}-{nonce}.metal",
            std::process::id()
        ));
        fs::write(&path, source)
            .map_err(|error| format!("failed to materialize Metal render source: {error}"))?;
        Ok(Self { path })
    }
}

#[cfg(all(test, target_os = "macos"))]
impl Drop for TempMslSource {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
