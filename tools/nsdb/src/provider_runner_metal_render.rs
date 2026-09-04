#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf, time::SystemTime};

use crate::provider_runner_metal::MetalProviderExecution;

#[cfg(target_os = "macos")]
const METAL_RGBA8_RENDER_SOURCE: &str = include_str!("../provider-runners/metal_rgba8_render.m");
const METAL_RGBA8_RENDER_CONTRACT: &str = "nuis-metal-rgba8-render-provider-runner-v1";

pub(crate) fn execute_rgba8_render(
    msl_source: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    width: usize,
    height: usize,
) -> Result<MetalProviderExecution, String> {
    execute_rgba8_render_platform(msl_source, vertex_entry, fragment_entry, width, height)
}

#[cfg(target_os = "macos")]
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
    let output_byte_len = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "Metal RGBA8 render byte count overflow".to_owned())?;
    let source = TempMslSource::materialize(msl_source)?;
    crate::provider_runner_metal::execute_metal_platform(
        source.path.as_os_str(),
        &[
            vertex_entry.to_owned(),
            fragment_entry.to_owned(),
            width.to_string(),
            height.to_string(),
        ],
        METAL_RGBA8_RENDER_CONTRACT,
        METAL_RGBA8_RENDER_SOURCE,
        None,
        Some(output_byte_len),
    )
}

#[cfg(not(target_os = "macos"))]
fn execute_rgba8_render_platform(
    _msl_source: &str,
    _vertex_entry: &str,
    _fragment_entry: &str,
    _width: usize,
    _height: usize,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(target_os = "macos")]
struct TempMslSource {
    path: PathBuf,
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
impl Drop for TempMslSource {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
