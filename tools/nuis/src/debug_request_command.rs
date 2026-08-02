use crate::debug_resume_command::{resolve_artifact_output_input, resolve_nsdb_program};
use std::{path::PathBuf, process::Command};

pub(crate) fn handle_debug_request(
    input: PathBuf,
    request_id: String,
    json: bool,
) -> Result<(), String> {
    let (output_dir, _) = resolve_artifact_output_input(&input, "debug-request")?;
    let mut command = Command::new(resolve_nsdb_program());
    command
        .arg("replay")
        .arg(output_dir)
        .arg("--request-id")
        .arg(request_id);
    if json {
        command.arg("--json");
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to start Nsdb request debugger: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Nsdb request debugger failed with status {}",
            status.code().unwrap_or(1)
        ))
    }
}
