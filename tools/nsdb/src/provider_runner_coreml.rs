use std::path::Path;
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf, process::Command, time::SystemTime};

#[cfg(target_os = "macos")]
const COREML_RUNNER_SOURCE: &str = include_str!("../provider-runners/coreml_vector_affine.m");

pub(crate) struct CoreMlProviderExecution {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) device: String,
    pub(crate) compute_plan_contract: String,
    pub(crate) compute_plan_status: String,
    pub(crate) compute_plan_layer_count: usize,
    pub(crate) compute_plan_preferred_devices: String,
    pub(crate) compute_plan_supported_devices: String,
    pub(crate) output_bytes: Vec<u8>,
}

pub(crate) fn execute_model_prediction(
    model_path: &Path,
    input_path: &Path,
    input_feature: &str,
    output_feature: &str,
    shape: &[usize],
) -> Result<CoreMlProviderExecution, String> {
    execute_model_prediction_platform(model_path, input_path, input_feature, output_feature, shape)
}

#[cfg(target_os = "macos")]
fn execute_model_prediction_platform(
    model_path: &Path,
    input_path: &Path,
    input_feature: &str,
    output_feature: &str,
    shape: &[usize],
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
    let execution = Command::new(&paths.binary)
        .arg(model_path)
        .arg(input_path)
        .arg(input_feature)
        .arg(output_feature)
        .arg(
            shape
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join("x"),
        )
        .output()
        .map_err(|error| format!("failed to launch CoreML provider runner: {error}"))?;
    if !execution.status.success() {
        return Err(format!(
            "CoreML provider runner failed: {}",
            String::from_utf8_lossy(&execution.stderr).trim()
        ));
    }
    parse_coreml_runner_output(&String::from_utf8_lossy(&execution.stdout))
}

#[cfg(not(target_os = "macos"))]
fn execute_model_prediction_platform(
    _model_path: &Path,
    _input_path: &Path,
    _input_feature: &str,
    _output_feature: &str,
    _shape: &[usize],
) -> Result<CoreMlProviderExecution, String> {
    Err("CoreML provider runner is unavailable on this host".to_owned())
}

fn parse_coreml_runner_output(output: &str) -> Result<CoreMlProviderExecution, String> {
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
    let output_bytes = decode_hex(
        field("output_hex")
            .ok_or_else(|| "CoreML provider runner omitted output bytes".to_owned())?,
    )?;
    let declared_bytes = field("output_bytes")
        .ok_or_else(|| "CoreML provider runner omitted output byte count".to_owned())?
        .parse::<usize>()
        .map_err(|error| format!("CoreML provider runner byte count is invalid: {error}"))?;
    if output_bytes.len() != declared_bytes {
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
        output_bytes,
    })
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
        assert_eq!(execution.output_bytes, [0, 0, 64, 64]);
    }
}
