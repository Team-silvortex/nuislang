use crate::provider_bundle_registry::{
    ProviderBundleRegistration, PROVIDER_BUNDLE_REGISTRY_CONTRACT,
};
use crate::provider_carrier_channel_registry::PreparedProviderCarrierChannel;
#[cfg(target_os = "macos")]
use crate::provider_carrier_channel_registry::{
    prepare_provider_carrier_channel, select_provider_carrier_channel_adapter,
};
use crate::provider_carrier_input::ProviderCarrierInput;
#[cfg(target_os = "macos")]
use crate::provider_output_carrier_registry::{
    prepare_provider_output_carrier, select_provider_output_carrier_adapter,
};
use crate::provider_output_carrier_registry::{
    ProviderOutputCarrierConsumption, ProviderOutputPayload,
    PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT, PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE,
    PROVIDER_OUTPUT_RESIDENCY_CONTRACT,
};
#[cfg(target_os = "macos")]
use crate::provider_process_adapter::{
    ProviderProcessAdapterCache, ResolvedProviderProcessAdapter,
};
use crate::provider_runner_registry::{
    framework_probe_status, ProviderRunnerAdapter, ProviderRunnerProfile,
    PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::SystemTime,
};

#[cfg(target_os = "macos")]
const METAL_GRAY8_UNARY_SOURCE: &str = include_str!("../provider-runners/metal_gray8_unary.m");
#[cfg(target_os = "macos")]
const METAL_F32_BIAS_SOURCE: &str = include_str!("../provider-runners/metal_f32_bias.m");

pub(crate) const PROVIDER_BUNDLE: ProviderBundleRegistration = ProviderBundleRegistration {
    registry_contract: PROVIDER_BUNDLE_REGISTRY_CONTRACT,
    bundle_id: "metal.apple-silicon-gpu.bundle.v1",
    runner_profile: RUNNER_PROFILE,
    #[cfg(unix)]
    execution_adapter: crate::provider_execution_metal::REGISTRATION,
};

pub(crate) const RUNNER_PROFILE: ProviderRunnerProfile = ProviderRunnerProfile {
    registry_contract: PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
    provider_family: "metal:apple-silicon-gpu",
    probe_status: metal_probe_status,
    available_probe_status: "real-device-candidate-available",
    available_adapter: ProviderRunnerAdapter {
        adapter_id: "metal.apple-silicon-gpu.real-device",
        capability_status: "registered-real-device",
        real_device_capable: true,
        kind: "metal-real-device-runner",
        execution_mode: "real-device-provider-runner",
    },
    fallback_adapter: ProviderRunnerAdapter {
        adapter_id: "metal.apple-silicon-gpu.host-simulated",
        capability_status: "registered-host-simulated",
        real_device_capable: false,
        kind: "metal-host-simulated-runner",
        execution_mode: "host-simulated-provider-runner",
    },
};

fn metal_probe_status() -> &'static str {
    framework_probe_status("Metal.framework")
}

pub(crate) struct MetalProviderExecution {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) device: String,
    pub(crate) output_carrier_registry_contract: String,
    pub(crate) output_carrier_registry_source: String,
    pub(crate) output_carrier_adapter_id: String,
    pub(crate) output_carrier_mode: String,
    pub(crate) output_residency_contract: String,
    pub(crate) output_residency_kind: String,
    pub(crate) output_transfer_scope: String,
    pub(crate) output_observation_mode: String,
    pub(crate) output_device_retention_status: String,
    pub(crate) output_payload: ProviderOutputPayload,
    pub(crate) transferable_output: Option<PreparedProviderCarrierChannel>,
}

pub(crate) fn execute_gray8_invert(
    input_path: &Path,
    max_value: u8,
) -> Result<MetalProviderExecution, String> {
    execute_gray8_invert_platform(input_path, max_value)
}

pub(crate) fn execute_gray8_threshold(
    input_path: &Path,
    threshold: u8,
    max_value: u8,
) -> Result<MetalProviderExecution, String> {
    execute_gray8_threshold_platform(input_path, threshold, max_value)
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_gray8_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    prepare_metal_worker_invocation(
        cache,
        METAL_GRAY8_UNARY_SOURCE,
        "nuis-metal-gray8-provider-runner-v1",
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_gray8_threshold_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    prepare_metal_worker_invocation(
        cache,
        METAL_GRAY8_UNARY_SOURCE,
        "nuis-metal-gray8-threshold-provider-runner-v1",
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_f32_bias_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    prepare_metal_worker_invocation(
        cache,
        METAL_F32_BIAS_SOURCE,
        "nuis-metal-f32-bias-provider-runner-v1",
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_f32_argmax_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    prepare_metal_worker_invocation(
        cache,
        METAL_F32_BIAS_SOURCE,
        "nuis-metal-f32-argmax-provider-runner-v1",
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_u32_copy_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    prepare_metal_worker_invocation(
        cache,
        METAL_F32_BIAS_SOURCE,
        "nuis-metal-u32-copy-provider-runner-v1",
    )
}

#[cfg(target_os = "macos")]
fn prepare_metal_worker_invocation<'a>(
    cache: &'a mut ProviderProcessAdapterCache,
    source: &str,
    contract: &'static str,
) -> Result<ResolvedProviderProcessAdapter<'a>, String> {
    cache.resolve_objc(
        "metal-worker-adapter",
        source,
        contract,
        &["Foundation", "Metal"],
    )
}

pub(crate) fn execute_f32_bias(
    input_path: &Path,
    bias: f32,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    execute_f32_bias_platform(input_path, bias, code_asset_path, entry)
}

pub(crate) fn execute_f32_bias_input(
    input: &ProviderCarrierInput,
    bias: f32,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    match input {
        ProviderCarrierInput::Path(path) => execute_f32_bias(path, bias, code_asset_path, entry),
        ProviderCarrierInput::OpaqueBytes { bytes, .. } => {
            execute_f32_bias_bytes_platform(bytes, bias, code_asset_path, entry)
        }
    }
}
pub(crate) fn execute_f32_bias_prepared_channel(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    bias: f32,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    execute_f32_bias_prepared_channel_platform(channel, byte_len, bias, code_asset_path, entry)
}
pub(crate) fn execute_f32_argmax_input(
    input: &ProviderCarrierInput,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    match input {
        ProviderCarrierInput::Path(path) => {
            execute_f32_argmax_platform(path, code_asset_path, entry)
        }
        ProviderCarrierInput::OpaqueBytes { bytes, .. } => {
            execute_f32_argmax_bytes_platform(bytes, code_asset_path, entry)
        }
    }
}
pub(crate) fn execute_f32_argmax_prepared_channel(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    execute_f32_argmax_prepared_channel_platform(channel, byte_len, code_asset_path, entry)
}

pub(crate) fn execute_u32_copy_input(
    input: &ProviderCarrierInput,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    match input {
        ProviderCarrierInput::Path(path) => execute_u32_copy_platform(path, code_asset_path, entry),
        ProviderCarrierInput::OpaqueBytes { bytes, .. } => {
            execute_u32_copy_bytes_platform(bytes, code_asset_path, entry)
        }
    }
}

pub(crate) fn execute_u32_copy_prepared_channel(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    execute_u32_copy_prepared_channel_platform(channel, byte_len, code_asset_path, entry)
}

#[cfg(target_os = "macos")]
fn execute_f32_bias_platform(
    input_path: &Path,
    bias: f32,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let output_byte_len = usize::try_from(
        fs::metadata(input_path)
            .map_err(|error| format!("failed to inspect Metal f32 input: {error}"))?
            .len(),
    )
    .map_err(|_| "Metal f32 input length overflow".to_owned())?;
    execute_metal_platform(
        input_path.as_os_str(),
        &[
            code_asset_path.display().to_string(),
            entry.to_owned(),
            bias.to_string(),
        ],
        "nuis-metal-f32-bias-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
        None,
        Some(output_byte_len),
    )
}

#[cfg(target_os = "macos")]
fn execute_f32_bias_bytes_platform(
    input: &[u8],
    bias: f32,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let channel_adapter = select_provider_carrier_channel_adapter("auto")
        .ok_or_else(|| "Metal provider carrier channel is unavailable".to_owned())?;
    let channel = prepare_provider_carrier_channel(channel_adapter, &[input])?;
    let argument = channel.frame_argument(0);
    execute_metal_platform(
        OsStr::new(&argument),
        &[
            code_asset_path.display().to_string(),
            entry.to_owned(),
            bias.to_string(),
        ],
        "nuis-metal-f32-bias-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
        Some(&channel),
        Some(input.len()),
    )
}

#[cfg(target_os = "macos")]
fn execute_f32_bias_prepared_channel_platform(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    bias: f32,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let argument = channel.frame_argument(0);
    execute_metal_platform(
        OsStr::new(&argument),
        &[
            code_asset_path.display().to_string(),
            entry.to_owned(),
            bias.to_string(),
        ],
        "nuis-metal-f32-bias-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
        Some(channel),
        Some(byte_len),
    )
}

#[cfg(target_os = "macos")]
fn execute_f32_argmax_platform(
    input_path: &Path,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    execute_metal_platform(
        input_path.as_os_str(),
        &[code_asset_path.display().to_string(), entry.to_owned()],
        "nuis-metal-f32-argmax-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
        None,
        Some(std::mem::size_of::<u32>()),
    )
}

#[cfg(target_os = "macos")]
fn execute_f32_argmax_bytes_platform(
    input: &[u8],
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let channel_adapter = select_provider_carrier_channel_adapter("auto")
        .ok_or_else(|| "Metal provider carrier channel is unavailable".to_owned())?;
    let channel = prepare_provider_carrier_channel(channel_adapter, &[input])?;
    execute_f32_argmax_prepared_channel_platform(&channel, input.len(), code_asset_path, entry)
}

#[cfg(target_os = "macos")]
fn execute_f32_argmax_prepared_channel_platform(
    channel: &PreparedProviderCarrierChannel,
    _byte_len: usize,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let argument = channel.frame_argument(0);
    execute_metal_platform(
        OsStr::new(&argument),
        &[code_asset_path.display().to_string(), entry.to_owned()],
        "nuis-metal-f32-argmax-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
        Some(channel),
        Some(std::mem::size_of::<u32>()),
    )
}

#[cfg(target_os = "macos")]
fn execute_u32_copy_platform(
    input_path: &Path,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let output_byte_len = usize::try_from(
        fs::metadata(input_path)
            .map_err(|error| format!("failed to inspect Metal u32 input: {error}"))?
            .len(),
    )
    .map_err(|_| "Metal u32 input length overflow".to_owned())?;
    execute_metal_platform(
        input_path.as_os_str(),
        &[
            code_asset_path.display().to_string(),
            entry.to_owned(),
            "copy-u32".to_owned(),
        ],
        "nuis-metal-u32-copy-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
        None,
        Some(output_byte_len),
    )
}

#[cfg(target_os = "macos")]
fn execute_u32_copy_bytes_platform(
    input: &[u8],
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let channel_adapter = select_provider_carrier_channel_adapter("auto")
        .ok_or_else(|| "Metal provider carrier channel is unavailable".to_owned())?;
    let channel = prepare_provider_carrier_channel(channel_adapter, &[input])?;
    execute_u32_copy_prepared_channel_platform(&channel, input.len(), code_asset_path, entry)
}

#[cfg(target_os = "macos")]
fn execute_u32_copy_prepared_channel_platform(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    code_asset_path: &Path,
    entry: &str,
) -> Result<MetalProviderExecution, String> {
    let argument = channel.frame_argument(0);
    execute_metal_platform(
        OsStr::new(&argument),
        &[
            code_asset_path.display().to_string(),
            entry.to_owned(),
            "copy-u32".to_owned(),
        ],
        "nuis-metal-u32-copy-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
        Some(channel),
        Some(byte_len),
    )
}

#[cfg(target_os = "macos")]
fn execute_gray8_invert_platform(
    input_path: &Path,
    max_value: u8,
) -> Result<MetalProviderExecution, String> {
    execute_metal_platform(
        input_path.as_os_str(),
        &[
            "invert".to_owned(),
            max_value.to_string(),
            max_value.to_string(),
        ],
        "nuis-metal-gray8-provider-runner-v1",
        METAL_GRAY8_UNARY_SOURCE,
        None,
        None,
    )
}

#[cfg(target_os = "macos")]
fn execute_gray8_threshold_platform(
    input_path: &Path,
    threshold: u8,
    max_value: u8,
) -> Result<MetalProviderExecution, String> {
    execute_metal_platform(
        input_path.as_os_str(),
        &[
            "threshold".to_owned(),
            threshold.to_string(),
            max_value.to_string(),
        ],
        "nuis-metal-gray8-threshold-provider-runner-v1",
        METAL_GRAY8_UNARY_SOURCE,
        None,
        None,
    )
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_bias_platform(
    _input_path: &Path,
    _bias: f32,
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_bias_bytes_platform(
    _input: &[u8],
    _bias: f32,
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_bias_prepared_channel_platform(
    _channel: &PreparedProviderCarrierChannel,
    _byte_len: usize,
    _bias: f32,
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_argmax_platform(
    _input_path: &Path,
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_argmax_bytes_platform(
    _input: &[u8],
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_argmax_prepared_channel_platform(
    _channel: &PreparedProviderCarrierChannel,
    _byte_len: usize,
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_u32_copy_platform(
    _input_path: &Path,
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_u32_copy_bytes_platform(
    _input: &[u8],
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_u32_copy_prepared_channel_platform(
    _channel: &PreparedProviderCarrierChannel,
    _byte_len: usize,
    _code_asset_path: &Path,
    _entry: &str,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(target_os = "macos")]
fn execute_metal_platform(
    input_argument: &OsStr,
    arguments: &[String],
    contract: &'static str,
    source: &str,
    carrier_channel: Option<&PreparedProviderCarrierChannel>,
    output_byte_len: Option<usize>,
) -> Result<MetalProviderExecution, String> {
    let paths = compile_metal_runner(source)?;
    let mut command = Command::new(&paths.binary);
    command.arg(input_argument).args(arguments);
    let output_adapter = output_byte_len
        .map(|_| {
            select_provider_output_carrier_adapter("auto")
                .ok_or_else(|| "Metal provider output carrier is unavailable".to_owned())
        })
        .transpose()?;
    let output_carrier = output_adapter
        .map(|adapter| {
            prepare_provider_output_carrier(
                adapter,
                output_byte_len.expect("output adapter requires byte length"),
            )
        })
        .transpose()?;
    if let Some(channel) = carrier_channel {
        channel.configure_command(&mut command);
    }
    if let Some(output_carrier) = &output_carrier {
        output_carrier.configure_command(&mut command)?;
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch Metal provider runner: {error}"))?;
    if let Some(channel) = carrier_channel {
        channel.complete_spawn(&mut child)?;
    }
    let execution = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for Metal provider runner: {error}"))?;
    if !execution.status.success() {
        return Err(format!(
            "Metal provider runner failed: {}",
            String::from_utf8_lossy(&execution.stderr).trim()
        ));
    }
    let output = String::from_utf8_lossy(&execution.stdout);
    let consumption = output_carrier
        .map(|carrier| carrier.consume(&output))
        .transpose()?;
    let (carrier_payload, transferable_output) = consumption
        .map(|consumption| (consumption.payload, consumption.transferable))
        .unwrap_or((None, None));
    let mut parsed = parse_metal_runner_output_with_payload(&output, contract, carrier_payload)?;
    if let Some(adapter) = output_adapter {
        parsed.output_carrier_adapter_id = adapter.adapter_id.to_owned();
        parsed.output_carrier_mode = adapter.mode.to_owned();
        parsed.output_residency_kind = adapter.residency_kind.to_owned();
        parsed.output_transfer_scope = adapter.transfer_scope.to_owned();
        parsed.output_observation_mode = adapter.observation_mode.to_owned();
        parsed.output_device_retention_status = adapter.device_retention_status.to_owned();
    }
    parsed.transferable_output = transferable_output;
    Ok(parsed)
}

#[cfg(target_os = "macos")]
fn compile_metal_runner(source: &str) -> Result<TempMetalRunnerPaths, String> {
    let paths = TempMetalRunnerPaths::new();
    fs::write(&paths.source, source)
        .map_err(|error| format!("failed to materialize Metal runner source: {error}"))?;
    let compile = Command::new("clang")
        .args([
            "-fobjc-arc",
            "-fblocks",
            "-framework",
            "Foundation",
            "-framework",
            "Metal",
        ])
        .arg(&paths.source)
        .arg("-o")
        .arg(&paths.binary)
        .output()
        .map_err(|error| format!("failed to launch Metal runner compiler: {error}"))?;
    if !compile.status.success() {
        return Err(format!(
            "Metal runner compilation failed: {}",
            String::from_utf8_lossy(&compile.stderr).trim()
        ));
    }
    Ok(paths)
}

#[cfg(not(target_os = "macos"))]
fn execute_gray8_invert_platform(
    _input_path: &Path,
    _max_value: u8,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_gray8_threshold_platform(
    _input_path: &Path,
    _threshold: u8,
    _max_value: u8,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(test)]
fn parse_metal_runner_output(output: &str) -> Result<MetalProviderExecution, String> {
    parse_metal_runner_output_with_payload(output, "nuis-metal-gray8-provider-runner-v1", None)
}

fn parse_metal_runner_output_with_payload(
    output: &str,
    expected_contract: &'static str,
    carrier_payload: Option<ProviderOutputPayload>,
) -> Result<MetalProviderExecution, String> {
    let field = |name: &str| {
        output
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name}=")))
    };
    if field("protocol") != Some(expected_contract) {
        return Err("Metal provider runner returned an unsupported protocol".to_owned());
    }
    if field("status") != Some("ready") {
        return Err("Metal provider runner did not report ready".to_owned());
    }
    let device = field("device")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Metal provider runner omitted device identity".to_owned())?
        .to_owned();
    let output_payload = match carrier_payload {
        Some(payload) => payload,
        None => ProviderOutputPayload::owned(decode_hex(
            field("output_hex")
                .ok_or_else(|| "Metal provider runner omitted output bytes".to_owned())?,
        )?),
    };
    let declared_bytes = field("output_bytes")
        .ok_or_else(|| "Metal provider runner omitted output byte count".to_owned())?
        .parse::<usize>()
        .map_err(|error| format!("Metal provider runner byte count is invalid: {error}"))?;
    if output_payload.as_bytes().len() != declared_bytes {
        return Err("Metal provider runner output byte count mismatch".to_owned());
    }
    Ok(MetalProviderExecution {
        contract: expected_contract,
        status: "metal-command-buffer-completed",
        device,
        output_carrier_registry_contract: PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT.to_owned(),
        output_carrier_registry_source: PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE.to_owned(),
        output_carrier_adapter_id: "hex.stdout.output.v1".to_owned(),
        output_carrier_mode: "hex-stdout-output".to_owned(),
        output_residency_contract: PROVIDER_OUTPUT_RESIDENCY_CONTRACT.to_owned(),
        output_residency_kind: "host-owned-bytes".to_owned(),
        output_transfer_scope: "observation-only".to_owned(),
        output_observation_mode: "stdout-eager".to_owned(),
        output_device_retention_status: "unsupported".to_owned(),
        output_payload,
        transferable_output: None,
    })
}

pub(crate) fn parse_metal_worker_output(
    output: &[u8],
    expected_contract: &'static str,
    consumption: Option<ProviderOutputCarrierConsumption>,
) -> Result<MetalProviderExecution, String> {
    let output = std::str::from_utf8(output)
        .map_err(|_| "Metal worker adapter output is not UTF-8".to_owned())?;
    let (payload, transferable) = consumption
        .map(|consumption| (consumption.payload, consumption.transferable))
        .unwrap_or_default();
    let mut execution = parse_metal_runner_output_with_payload(output, expected_contract, payload)?;
    if transferable.is_some() {
        execution.output_carrier_adapter_id = "inherited.fd.output.v1".to_owned();
        execution.output_carrier_mode = "inherited-fd-output".to_owned();
        execution.output_residency_kind = "host-visible-file".to_owned();
        execution.output_transfer_scope = "cross-process-static".to_owned();
        execution.output_observation_mode = "mapped-on-demand".to_owned();
        execution.transferable_output = transferable;
    }
    Ok(execution)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("Metal provider runner output hex has odd length".to_owned());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|error| format!("Metal provider runner output hex is invalid: {error}"))
        })
        .collect()
}

#[cfg(target_os = "macos")]
struct TempMetalRunnerPaths {
    source: PathBuf,
    binary: PathBuf,
}

#[cfg(target_os = "macos")]
impl TempMetalRunnerPaths {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let stem = format!("nuis-nsdb-metal-runner-{}-{nonce}", std::process::id());
        let temp = std::env::temp_dir();
        Self {
            source: temp.join(format!("{stem}.m")),
            binary: temp.join(stem),
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for TempMetalRunnerPaths {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.source);
        let _ = fs::remove_file(&self.binary);
    }
}

#[cfg(test)]
#[path = "provider_runner_metal_tests.rs"]
mod provider_runner_metal_tests;
