use crate::provider_carrier_input::ProviderCarrierInput;
use std::{ffi::OsStr, path::Path};
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf, process::Command, time::SystemTime};

#[cfg(target_os = "macos")]
const METAL_RUNNER_SOURCE: &str = include_str!("../provider-runners/metal_gray8_invert.m");
#[cfg(target_os = "macos")]
const METAL_F32_BIAS_SOURCE: &str = include_str!("../provider-runners/metal_f32_bias.m");

pub(crate) struct MetalProviderExecution {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) device: String,
    pub(crate) output_bytes: Vec<u8>,
}

pub(crate) fn execute_gray8_invert(
    input_path: &Path,
    max_value: u8,
) -> Result<MetalProviderExecution, String> {
    execute_gray8_invert_platform(input_path, max_value)
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

#[cfg(target_os = "macos")]
fn execute_f32_bias_platform(
    input_path: &Path,
    bias: f32,
) -> Result<MetalProviderExecution, String> {
    execute_metal_scalar_platform(
        input_path.as_os_str(),
        &bias.to_string(),
        "nuis-metal-f32-bias-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
    )
}

#[cfg(target_os = "macos")]
fn execute_f32_bias_bytes_platform(
    input: &[u8],
    bias: f32,
) -> Result<MetalProviderExecution, String> {
    let argument = format!("hex:{}", encode_hex(input));
    execute_metal_scalar_platform(
        argument.as_ref(),
        &bias.to_string(),
        "nuis-metal-f32-bias-provider-runner-v1",
        METAL_F32_BIAS_SOURCE,
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

#[cfg(target_os = "macos")]
fn execute_metal_scalar_platform(
    input_argument: &OsStr,
    scalar: &str,
    contract: &'static str,
    source: &str,
) -> Result<MetalProviderExecution, String> {
    let paths = TempMetalRunnerPaths::new();
    fs::write(&paths.source, source)
        .map_err(|error| format!("failed to materialize Metal runner source: {error}"))?;
    let compile = Command::new("clang")
        .args([
            "-fobjc-arc",
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
    let execution = Command::new(&paths.binary)
        .arg(input_argument)
        .arg(scalar)
        .output()
        .map_err(|error| format!("failed to launch Metal provider runner: {error}"))?;
    if !execution.status.success() {
        return Err(format!(
            "Metal provider runner failed: {}",
            String::from_utf8_lossy(&execution.stderr).trim()
        ));
    }
    parse_metal_runner_output_for(&String::from_utf8_lossy(&execution.stdout), contract)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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

fn parse_metal_runner_output_for(
    output: &str,
    expected_contract: &'static str,
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
    let output_bytes = decode_hex(
        field("output_hex")
            .ok_or_else(|| "Metal provider runner omitted output bytes".to_owned())?,
    )?;
    let declared_bytes = field("output_bytes")
        .ok_or_else(|| "Metal provider runner omitted output byte count".to_owned())?
        .parse::<usize>()
        .map_err(|error| format!("Metal provider runner byte count is invalid: {error}"))?;
    if output_bytes.len() != declared_bytes {
        return Err("Metal provider runner output byte count mismatch".to_owned());
    }
    Ok(MetalProviderExecution {
        contract: expected_contract,
        status: "metal-command-buffer-completed",
        device,
        output_bytes,
    })
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
        assert_eq!(execution.output_bytes, [15, 11, 6, 7]);
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
        assert_eq!(execution.output_bytes, [15, 11, 6, 7]);
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
            .output_bytes
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(values, [11.0, 17.0, 23.0, 29.0]);
    }
}
