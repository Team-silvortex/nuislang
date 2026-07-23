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
    compile_objc_process_adapter, PreparedProviderProcessAdapter,
};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::SystemTime,
};

#[cfg(target_os = "macos")]
const COREML_RUNNER_SOURCE: &str = include_str!("../provider-runners/coreml_vector_affine.m");

#[cfg(target_os = "macos")]
pub(crate) fn prepare_coreml_worker_invocation() -> Result<PreparedProviderProcessAdapter, String> {
    compile_objc_process_adapter(
        "coreml-worker-adapter",
        COREML_RUNNER_SOURCE,
        "nuis-coreml-model-prediction-provider-runner-v1",
        &["Foundation", "CoreML"],
    )
}

pub(crate) struct CoreMlProviderExecution {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) device: String,
    pub(crate) compute_plan_contract: String,
    pub(crate) compute_plan_status: String,
    pub(crate) compute_plan_layer_count: usize,
    pub(crate) compute_plan_preferred_devices: String,
    pub(crate) compute_plan_supported_devices: String,
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

pub(crate) struct CoreMlProviderInput<'a> {
    pub(crate) source: CoreMlProviderInputSource<'a>,
    pub(crate) feature: &'a str,
    pub(crate) shape: &'a [usize],
}

#[derive(Clone, Copy)]
pub(crate) enum CoreMlProviderInputSource<'a> {
    Carrier(&'a ProviderCarrierInput),
    PreparedChannel(&'a PreparedProviderCarrierChannel),
}

pub(crate) fn execute_model_prediction_inputs(
    model_path: &Path,
    inputs: &[CoreMlProviderInput<'_>],
    output_feature: &str,
    output_shape: &[usize],
) -> Result<CoreMlProviderExecution, String> {
    if inputs.is_empty() || inputs.iter().any(|input| input.shape.is_empty()) {
        return Err("CoreML provider runner requires named input tensors".to_owned());
    }
    execute_model_prediction_platform(model_path, inputs, output_feature, output_shape)
}

#[cfg(target_os = "macos")]
fn execute_model_prediction_platform(
    model_path: &Path,
    inputs: &[CoreMlProviderInput<'_>],
    output_feature: &str,
    output_shape: &[usize],
) -> Result<CoreMlProviderExecution, String> {
    let paths = TempCoreMlRunnerPaths::new();
    fs::write(&paths.source, COREML_RUNNER_SOURCE)
        .map_err(|error| format!("failed to materialize CoreML runner source: {error}"))?;
    let compile = Command::new("clang")
        .args([
            "-fobjc-arc",
            "-fblocks",
            "-framework",
            "Foundation",
            "-framework",
            "CoreML",
        ])
        .arg(&paths.source)
        .arg("-o")
        .arg(&paths.binary)
        .output()
        .map_err(|error| format!("failed to launch CoreML runner compiler: {error}"))?;
    if !compile.status.success() {
        return Err(format!(
            "CoreML runner compilation failed: {}",
            String::from_utf8_lossy(&compile.stderr).trim()
        ));
    }
    let mut command = Command::new(&paths.binary);
    command
        .arg(model_path)
        .arg("--multi")
        .arg(output_feature)
        .arg(format_shape(output_shape));
    let carrier_frames = inputs
        .iter()
        .filter_map(|input| match input.source {
            CoreMlProviderInputSource::Carrier(ProviderCarrierInput::Path(_))
            | CoreMlProviderInputSource::PreparedChannel(_) => None,
            CoreMlProviderInputSource::Carrier(ProviderCarrierInput::OpaqueBytes {
                ref bytes,
                ..
            }) => Some(bytes.as_slice()),
        })
        .collect::<Vec<_>>();
    let channel = if carrier_frames.is_empty() {
        None
    } else {
        let adapter = select_provider_carrier_channel_adapter("auto")
            .ok_or_else(|| "CoreML provider carrier channel is unavailable".to_owned())?;
        Some(prepare_provider_carrier_channel(adapter, &carrier_frames)?)
    };
    let mut frame_index = 0;
    for input in inputs {
        command.arg(input.feature);
        match input.source {
            CoreMlProviderInputSource::Carrier(ProviderCarrierInput::Path(path)) => {
                command.arg(path)
            }
            CoreMlProviderInputSource::Carrier(ProviderCarrierInput::OpaqueBytes { .. }) => {
                let argument = channel
                    .as_ref()
                    .expect("opaque inputs require a prepared carrier channel")
                    .frame_argument(frame_index);
                frame_index += 1;
                command.arg(argument)
            }
            CoreMlProviderInputSource::PreparedChannel(channel) => {
                command.arg(channel.frame_argument(0))
            }
        };
        command.arg(format_shape(input.shape));
    }
    let output_byte_len = output_shape
        .iter()
        .try_fold(4usize, |bytes, dimension| bytes.checked_mul(*dimension))
        .ok_or_else(|| "CoreML provider output byte length overflow".to_owned())?;
    let output_adapter = select_provider_output_carrier_adapter("auto")
        .ok_or_else(|| "CoreML provider output carrier is unavailable".to_owned())?;
    let output_carrier = prepare_provider_output_carrier(output_adapter, output_byte_len)?;
    if let Some(channel) = &channel {
        channel.configure_command(&mut command);
    }
    for input in inputs {
        if let CoreMlProviderInputSource::PreparedChannel(channel) = input.source {
            channel.configure_command(&mut command);
        }
    }
    output_carrier.configure_command(&mut command)?;
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch CoreML provider runner: {error}"))?;
    if let Some(channel) = &channel {
        channel.complete_spawn(&mut child)?;
    }
    for input in inputs {
        if let CoreMlProviderInputSource::PreparedChannel(channel) = input.source {
            channel.complete_spawn(&mut child)?;
        }
    }
    let execution = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for CoreML provider runner: {error}"))?;
    if !execution.status.success() {
        return Err(format!(
            "CoreML provider runner failed: {}",
            String::from_utf8_lossy(&execution.stderr).trim()
        ));
    }
    let output = String::from_utf8_lossy(&execution.stdout);
    let consumption = output_carrier.consume(&output)?;
    let mut parsed = parse_coreml_runner_output_with_payload(&output, consumption.payload)?;
    parsed.output_carrier_adapter_id = output_adapter.adapter_id.to_owned();
    parsed.output_carrier_mode = output_adapter.mode.to_owned();
    parsed.output_residency_kind = output_adapter.residency_kind.to_owned();
    parsed.output_transfer_scope = output_adapter.transfer_scope.to_owned();
    parsed.output_observation_mode = output_adapter.observation_mode.to_owned();
    parsed.output_device_retention_status = output_adapter.device_retention_status.to_owned();
    parsed.transferable_output = consumption.transferable;
    Ok(parsed)
}

#[cfg(not(target_os = "macos"))]
fn execute_model_prediction_platform(
    _model_path: &Path,
    _inputs: &[CoreMlProviderInput<'_>],
    _output_feature: &str,
    _output_shape: &[usize],
) -> Result<CoreMlProviderExecution, String> {
    Err("CoreML provider runner is unavailable on this host".to_owned())
}

pub(crate) fn format_shape(shape: &[usize]) -> String {
    shape
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join("x")
}

#[cfg(test)]
fn parse_coreml_runner_output(output: &str) -> Result<CoreMlProviderExecution, String> {
    parse_coreml_runner_output_with_payload(output, None)
}

fn parse_coreml_runner_output_with_payload(
    output: &str,
    carrier_payload: Option<ProviderOutputPayload>,
) -> Result<CoreMlProviderExecution, String> {
    let field = |name: &str| {
        output
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name}=")))
    };
    if field("protocol") != Some("nuis-coreml-model-prediction-provider-runner-v1") {
        return Err("CoreML provider runner returned an unsupported protocol".to_owned());
    }
    if field("status") != Some("ready") {
        return Err("CoreML provider runner did not report ready".to_owned());
    }
    let device = field("device")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "CoreML provider runner omitted device identity".to_owned())?
        .to_owned();
    let compute_plan_contract = field("compute_plan_contract")
        .filter(|value| *value == "nuis-coreml-compute-plan-evidence-v1")
        .ok_or_else(|| "CoreML provider runner omitted compute-plan contract".to_owned())?
        .to_owned();
    let compute_plan_status = field("compute_plan_status")
        .filter(|value| matches!(*value, "ready" | "unavailable"))
        .ok_or_else(|| "CoreML provider runner returned invalid compute-plan status".to_owned())?
        .to_owned();
    let compute_plan_layer_count = field("compute_plan_layer_count")
        .ok_or_else(|| "CoreML provider runner omitted compute-plan layer count".to_owned())?
        .parse::<usize>()
        .map_err(|error| format!("CoreML compute-plan layer count is invalid: {error}"))?;
    if compute_plan_status == "ready" && compute_plan_layer_count == 0 {
        return Err("CoreML ready compute plan contains no layers".to_owned());
    }
    let compute_plan_preferred_devices =
        required_device_set(field("compute_plan_preferred_devices"))?;
    let compute_plan_supported_devices =
        required_device_set(field("compute_plan_supported_devices"))?;
    let output_payload = match carrier_payload {
        Some(payload) => payload,
        None => ProviderOutputPayload::owned(decode_hex(
            field("output_hex")
                .ok_or_else(|| "CoreML provider runner omitted output bytes".to_owned())?,
        )?),
    };
    let declared_bytes = field("output_bytes")
        .ok_or_else(|| "CoreML provider runner omitted output byte count".to_owned())?
        .parse::<usize>()
        .map_err(|error| format!("CoreML provider runner byte count is invalid: {error}"))?;
    if output_payload.as_bytes().len() != declared_bytes {
        return Err("CoreML provider runner output byte count mismatch".to_owned());
    }
    Ok(CoreMlProviderExecution {
        contract: "nuis-coreml-model-prediction-provider-runner-v1",
        status: "coreml-model-prediction-completed",
        device,
        compute_plan_contract,
        compute_plan_status,
        compute_plan_layer_count,
        compute_plan_preferred_devices,
        compute_plan_supported_devices,
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

pub(crate) fn parse_coreml_worker_output(
    output: &[u8],
    consumption: Option<ProviderOutputCarrierConsumption>,
) -> Result<CoreMlProviderExecution, String> {
    let output = std::str::from_utf8(output)
        .map_err(|_| "CoreML worker adapter output is not UTF-8".to_owned())?;
    let (payload, transferable) = consumption
        .map(|consumption| (consumption.payload, consumption.transferable))
        .unwrap_or_default();
    let mut execution = parse_coreml_runner_output_with_payload(output, payload)?;
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

fn required_device_set(value: Option<&str>) -> Result<String, String> {
    let value =
        value.ok_or_else(|| "CoreML provider runner omitted compute-device set".to_owned())?;
    if value.is_empty()
        || value
            .split(',')
            .any(|device| !matches!(device, "cpu" | "gpu" | "neural-engine" | "unknown" | "none"))
    {
        return Err("CoreML provider runner returned invalid compute-device set".to_owned());
    }
    Ok(value.to_owned())
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("CoreML provider runner output hex has odd length".to_owned());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|error| format!("CoreML provider runner output hex is invalid: {error}"))
        })
        .collect()
}

#[cfg(target_os = "macos")]
struct TempCoreMlRunnerPaths {
    source: PathBuf,
    binary: PathBuf,
}

#[cfg(target_os = "macos")]
impl TempCoreMlRunnerPaths {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let stem = format!("nuis-nsdb-coreml-runner-{}-{nonce}", std::process::id());
        let temp = std::env::temp_dir();
        Self {
            source: temp.join(format!("{stem}.m")),
            binary: temp.join(stem),
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for TempCoreMlRunnerPaths {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.source);
        let _ = fs::remove_file(&self.binary);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_coreml_runner_output;

    #[test]
    fn parses_ready_coreml_runner_output() {
        let execution = parse_coreml_runner_output(
            "protocol=nuis-coreml-model-prediction-provider-runner-v1\nstatus=ready\ndevice=CoreML.framework:MLModel:CPUAndNeuralEngine-requested\ncompute_plan_contract=nuis-coreml-compute-plan-evidence-v1\ncompute_plan_status=ready\ncompute_plan_layer_count=1\ncompute_plan_preferred_devices=neural-engine\ncompute_plan_supported_devices=cpu,neural-engine\noutput_bytes=4\noutput_hex=00004040\n",
        )
        .unwrap();
        assert_eq!(
            execution.contract,
            "nuis-coreml-model-prediction-provider-runner-v1"
        );
        assert_eq!(execution.status, "coreml-model-prediction-completed");
        assert_eq!(execution.compute_plan_status, "ready");
        assert_eq!(execution.compute_plan_layer_count, 1);
        assert_eq!(execution.compute_plan_preferred_devices, "neural-engine");
        assert_eq!(
            execution.compute_plan_supported_devices,
            "cpu,neural-engine"
        );
        assert_eq!(execution.output_payload.as_bytes(), [0, 0, 64, 64]);
    }
}
