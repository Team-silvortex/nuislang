use crate::{
    provider_carrier_channel_registry::PreparedProviderCarrierChannel,
    provider_carrier_input::ProviderCarrierInput, provider_runner_metal::MetalProviderExecution,
};
use std::path::Path;

#[cfg(target_os = "macos")]
use crate::{
    provider_carrier_channel_registry::{
        prepare_provider_carrier_channel, select_provider_carrier_channel_adapter,
    },
    provider_process_adapter::{ProviderProcessAdapterCache, ResolvedProviderProcessAdapter},
};
#[cfg(target_os = "macos")]
use std::{ffi::OsStr, fs};

const U32_COPY_CONTRACT: &str = "nuis-metal-u32-copy-provider-runner-v1";
const U32_CANONICAL_CONTRACT: &str = "nuis-metal-u32-canonical-provider-runner-v1";

#[cfg(target_os = "macos")]
const METAL_U32_COMPUTE_SOURCE: &str = include_str!("../provider-runners/metal_f32_bias.m");

pub(crate) fn u32_compute_runner_contract(operation: &str) -> &'static str {
    if operation == "copy-u32" {
        U32_COPY_CONTRACT
    } else {
        U32_CANONICAL_CONTRACT
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_u32_copy_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    prepare_u32_worker_invocation(cache, "copy-u32")
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_u32_canonical_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    prepare_u32_worker_invocation(cache, "add-u32")
}

#[cfg(target_os = "macos")]
fn prepare_u32_worker_invocation<'a>(
    cache: &'a mut ProviderProcessAdapterCache,
    operation: &str,
) -> Result<ResolvedProviderProcessAdapter<'a>, String> {
    crate::provider_runner_metal::prepare_metal_worker_invocation(
        cache,
        METAL_U32_COMPUTE_SOURCE,
        u32_compute_runner_contract(operation),
    )
}

#[cfg(test)]
pub(crate) fn execute_u32_copy_input(
    input: &ProviderCarrierInput,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    execute_u32_canonical_input(input, code_asset_path, entry, "copy-u32")
}

pub(crate) fn execute_u32_canonical_input(
    input: &ProviderCarrierInput,
    code_asset_path: &Path,
    entry: &str,
    operation: &str,
) -> Result<MetalProviderExecution, String> {
    match input {
        ProviderCarrierInput::Path(path) => {
            execute_u32_canonical_platform(path, code_asset_path, entry, operation)
        }
        ProviderCarrierInput::OpaqueBytes { bytes, .. } => {
            execute_u32_canonical_bytes_platform(bytes, code_asset_path, entry, operation)
        }
    }
}

pub(crate) fn execute_u32_canonical_prepared_channel(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    code_asset_path: &Path,
    entry: &str,
    operation: &str,
) -> Result<MetalProviderExecution, String> {
    execute_u32_canonical_prepared_channel_platform(
        channel,
        byte_len,
        code_asset_path,
        entry,
        operation,
    )
}

#[cfg(target_os = "macos")]
fn execute_u32_canonical_platform(
    input_path: &Path,
    code_asset_path: &Path,
    entry: &str,
    operation: &str,
) -> Result<MetalProviderExecution, String> {
    let output_byte_len = usize::try_from(
        fs::metadata(input_path)
            .map_err(|error| format!("failed to inspect Metal u32 input: {error}"))?
            .len(),
    )
    .map_err(|_| "Metal u32 input length overflow".to_owned())?;
    crate::provider_runner_metal::execute_metal_platform(
        input_path.as_os_str(),
        &[
            code_asset_path.display().to_string(),
            entry.to_owned(),
            operation.to_owned(),
        ],
        u32_compute_runner_contract(operation),
        METAL_U32_COMPUTE_SOURCE,
        None,
        Some(output_byte_len),
    )
}

#[cfg(target_os = "macos")]
fn execute_u32_canonical_bytes_platform(
    input: &[u8],
    code_asset_path: &Path,
    entry: &str,
    operation: &str,
) -> Result<MetalProviderExecution, String> {
    let channel_adapter = select_provider_carrier_channel_adapter("auto")
        .ok_or_else(|| "Metal provider carrier channel is unavailable".to_owned())?;
    let channel = prepare_provider_carrier_channel(channel_adapter, &[input])?;
    execute_u32_canonical_prepared_channel_platform(
        &channel,
        input.len(),
        code_asset_path,
        entry,
        operation,
    )
}

#[cfg(target_os = "macos")]
fn execute_u32_canonical_prepared_channel_platform(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    code_asset_path: &Path,
    entry: &str,
    operation: &str,
) -> Result<MetalProviderExecution, String> {
    let argument = channel.frame_argument(0);
    crate::provider_runner_metal::execute_metal_platform(
        OsStr::new(&argument),
        &[
            code_asset_path.display().to_string(),
            entry.to_owned(),
            operation.to_owned(),
        ],
        u32_compute_runner_contract(operation),
        METAL_U32_COMPUTE_SOURCE,
        Some(channel),
        Some(byte_len),
    )
}

#[cfg(not(target_os = "macos"))]
fn execute_u32_canonical_platform(
    _input_path: &Path,
    _code_asset_path: &Path,
    _entry: &str,
    _operation: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_u32_canonical_bytes_platform(
    _input: &[u8],
    _code_asset_path: &Path,
    _entry: &str,
    _operation: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_u32_canonical_prepared_channel_platform(
    _channel: &PreparedProviderCarrierChannel,
    _byte_len: usize,
    _code_asset_path: &Path,
    _entry: &str,
    _operation: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}
