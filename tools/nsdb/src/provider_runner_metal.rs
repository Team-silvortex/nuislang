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
    ProviderOutputPayload, PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT,
    PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE, PROVIDER_OUTPUT_RESIDENCY_CONTRACT,
};
use std::{ffi::OsStr, path::Path};
#[cfg(target_os = "macos")]
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::SystemTime,
};

#[cfg(target_os = "macos")]
const METAL_RUNNER_SOURCE: &str = include_str!("../provider-runners/metal_gray8_invert.m");
#[cfg(target_os = "macos")]
const METAL_F32_BIAS_SOURCE: &str = include_str!("../provider-runners/metal_f32_bias.m");

#[cfg(target_os = "macos")]
pub(crate) struct PreparedMetalWorkerInvocation {
    paths: TempMetalRunnerPaths,
    pub(crate) contract: &'static str,
    pub(crate) executable_hash: String,
    pub(crate) scalar_argument: String,
}

#[cfg(target_os = "macos")]
impl PreparedMetalWorkerInvocation {
    pub(crate) fn executable_path(&self) -> &Path {
        &self.paths.binary
    }
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

#[cfg(target_os = "macos")]
pub(crate) fn prepare_gray8_worker_invocation(
    max_value: u8,
) -> Result<PreparedMetalWorkerInvocation, String> {
    let paths = compile_metal_runner(METAL_RUNNER_SOURCE)?;
    let executable = fs::read(&paths.binary)
        .map_err(|error| format!("failed to hash Metal worker adapter: {error}"))?;
    Ok(PreparedMetalWorkerInvocation {
        paths,
        contract: "nuis-metal-gray8-provider-runner-v1",
        executable_hash: crate::provider_sample_artifact::fnv1a64_hex(&executable),
        scalar_argument: max_value.to_string(),
    })
}

pub(crate) fn execute_f32_bias(
    input_path: &Path,
    bias: f32,
) -> Result<MetalProviderExecution, String> {
    execute_f32_bias_platform(input_path, bias)
}

pub(crate) fn execute_f32_bias_input(
    input: &ProviderCarrierInput,
    bias: f32,
) -> Result<MetalProviderExecution, String> {
    match input {
        ProviderCarrierInput::Path(path) => execute_f32_bias(path, bias),
        ProviderCarrierInput::OpaqueBytes { bytes, .. } => {
            execute_f32_bias_bytes_platform(bytes, bias)
        }
    }
}

pub(crate) fn execute_f32_bias_prepared_channel(
    channel: &PreparedProviderCarrierChannel,
    byte_len: usize,
    bias: f32,
) -> Result<MetalProviderExecution, String> {
    execute_f32_bias_prepared_channel_platform(channel, byte_len, bias)
}

#[cfg(target_os = "macos")]
fn execute_f32_bias_platform(
    input_path: &Path,
    bias: f32,
) -> Result<MetalProviderExecution, String> {
    let output_byte_len = usize::try_from(
        fs::metadata(input_path)
            .map_err(|error| format!("failed to inspect Metal f32 input: {error}"))?
            .len(),
    )
    .map_err(|_| "Metal f32 input length overflow".to_owned())?;
    execute_metal_scalar_platform(
        input_path.as_os_str(),
        &bias.to_string(),
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
) -> Result<MetalProviderExecution, String> {
    let channel_adapter = select_provider_carrier_channel_adapter("auto")
        .ok_or_else(|| "Metal provider carrier channel is unavailable".to_owned())?;
    let channel = prepare_provider_carrier_channel(channel_adapter, &[input])?;
    let argument = channel.frame_argument(0);
    execute_metal_scalar_platform(
        OsStr::new(&argument),
        &bias.to_string(),
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
) -> Result<MetalProviderExecution, String> {
    let argument = channel.frame_argument(0);
    execute_metal_scalar_platform(
        OsStr::new(&argument),
        &bias.to_string(),
        "nuis-metal-f32-bias-provider-runner-v1",
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
    execute_metal_scalar_platform(
        input_path.as_os_str(),
        &max_value.to_string(),
        "nuis-metal-gray8-provider-runner-v1",
        METAL_RUNNER_SOURCE,
        None,
        None,
    )
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_bias_platform(
    _input_path: &Path,
    _bias: f32,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_bias_bytes_platform(
    _input: &[u8],
    _bias: f32,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn execute_f32_bias_prepared_channel_platform(
    _channel: &PreparedProviderCarrierChannel,
    _byte_len: usize,
    _bias: f32,
) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

#[cfg(target_os = "macos")]
fn execute_metal_scalar_platform(
    input_argument: &OsStr,
    scalar: &str,
    contract: &'static str,
    source: &str,
    carrier_channel: Option<&PreparedProviderCarrierChannel>,
    output_byte_len: Option<usize>,
) -> Result<MetalProviderExecution, String> {
    let paths = compile_metal_runner(source)?;
    let mut command = Command::new(&paths.binary);
    command.arg(input_argument).arg(scalar);
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

#[cfg(test)]
fn parse_metal_runner_output(output: &str) -> Result<MetalProviderExecution, String> {
    parse_metal_runner_output_for(output, "nuis-metal-gray8-provider-runner-v1")
}

#[cfg(test)]
fn parse_metal_runner_output_for(
    output: &str,
    expected_contract: &'static str,
) -> Result<MetalProviderExecution, String> {
    parse_metal_runner_output_with_payload(output, expected_contract, None)
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
) -> Result<MetalProviderExecution, String> {
    let output = std::str::from_utf8(output)
        .map_err(|_| "Metal worker adapter output is not UTF-8".to_owned())?;
    parse_metal_runner_output_with_payload(output, expected_contract, None)
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
mod tests {
    use super::{execute_f32_bias_input, execute_gray8_invert, parse_metal_runner_output};
    use crate::provider_carrier_input::ProviderCarrierInput;

    #[test]
    fn parses_ready_metal_runner_output() {
        let execution = parse_metal_runner_output(
            "protocol=nuis-metal-gray8-provider-runner-v1\nstatus=ready\ndevice=Apple M2\noutput_bytes=4\noutput_hex=0f0b0607\n",
        )
        .unwrap();

        assert_eq!(execution.contract, "nuis-metal-gray8-provider-runner-v1");
        assert_eq!(execution.status, "metal-command-buffer-completed");
        assert_eq!(execution.device, "Apple M2");
        assert_eq!(execution.output_payload.as_bytes(), [15, 11, 6, 7]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn executes_gray8_invert_on_the_system_metal_device() {
        let input = std::env::temp_dir().join(format!(
            "nuis-metal-gray8-input-{}-{}.bin",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        std::fs::write(&input, [0, 4, 9, 8]).unwrap();
        let execution = execute_gray8_invert(&input, 15).expect("system Metal provider execution");
        let _ = std::fs::remove_file(input);

        assert_eq!(execution.contract, "nuis-metal-gray8-provider-runner-v1");
        assert_eq!(execution.status, "metal-command-buffer-completed");
        assert!(!execution.device.is_empty());
        assert_eq!(execution.output_payload.as_bytes(), [15, 11, 6, 7]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn executes_f32_bias_from_opaque_carrier_bytes() {
        let input = ProviderCarrierInput::OpaqueBytes {
            handle: "memory:metal-test".to_owned(),
            bytes: [10.0f32, 16.0, 22.0, 28.0]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect(),
        };
        let execution = execute_f32_bias_input(&input, 1.0).expect("opaque Metal input");
        let values = execution
            .output_payload
            .as_bytes()
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(values, [11.0, 17.0, 23.0, 29.0]);
    }
}
