#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf, process::Command, time::SystemTime};

#[cfg(target_os = "macos")]
const METAL_RUNNER_SOURCE: &str = include_str!("../provider-runners/metal_u32_add.m");

pub(crate) struct MetalProviderExecution {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) device: String,
    pub(crate) output: u32,
}

pub(crate) fn execute_u32_add(input: u32, delta: u32) -> Result<MetalProviderExecution, String> {
    execute_u32_add_platform(input, delta)
}

#[cfg(target_os = "macos")]
fn execute_u32_add_platform(input: u32, delta: u32) -> Result<MetalProviderExecution, String> {
    let paths = TempMetalRunnerPaths::new();
    fs::write(&paths.source, METAL_RUNNER_SOURCE)
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
        .args([input.to_string(), delta.to_string()])
        .output()
        .map_err(|error| format!("failed to launch Metal provider runner: {error}"))?;
    if !execution.status.success() {
        return Err(format!(
            "Metal provider runner failed: {}",
            String::from_utf8_lossy(&execution.stderr).trim()
        ));
    }
    parse_metal_runner_output(&String::from_utf8_lossy(&execution.stdout))
}

#[cfg(not(target_os = "macos"))]
fn execute_u32_add_platform(_input: u32, _delta: u32) -> Result<MetalProviderExecution, String> {
    Err("Metal provider runner is unavailable on this host".to_owned())
}

fn parse_metal_runner_output(output: &str) -> Result<MetalProviderExecution, String> {
    let field = |name: &str| {
        output
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name}=")))
    };
    if field("protocol") != Some("nuis-metal-provider-runner-v1") {
        return Err("Metal provider runner returned an unsupported protocol".to_owned());
    }
    if field("status") != Some("ready") {
        return Err("Metal provider runner did not report ready".to_owned());
    }
    let device = field("device")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Metal provider runner omitted device identity".to_owned())?
        .to_owned();
    let output = field("output")
        .ok_or_else(|| "Metal provider runner omitted output".to_owned())?
        .parse::<u32>()
        .map_err(|error| format!("Metal provider runner output is invalid: {error}"))?;
    Ok(MetalProviderExecution {
        contract: "nuis-metal-provider-runner-v1",
        status: "metal-command-buffer-completed",
        device,
        output,
    })
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
    use super::{execute_u32_add, parse_metal_runner_output};

    #[test]
    fn parses_ready_metal_runner_output() {
        let execution = parse_metal_runner_output(
            "protocol=nuis-metal-provider-runner-v1\nstatus=ready\ndevice=Apple M2\noutput=24\n",
        )
        .unwrap();

        assert_eq!(execution.contract, "nuis-metal-provider-runner-v1");
        assert_eq!(execution.status, "metal-command-buffer-completed");
        assert_eq!(execution.device, "Apple M2");
        assert_eq!(execution.output, 24);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn executes_u32_add_on_the_system_metal_device() {
        let execution = execute_u32_add(20, 4).expect("system Metal provider execution");

        assert_eq!(execution.contract, "nuis-metal-provider-runner-v1");
        assert_eq!(execution.status, "metal-command-buffer-completed");
        assert!(!execution.device.is_empty());
        assert_eq!(execution.output, 24);
    }
}
