use crate::artifact_nsdb_replay_cursor::{
    persist_debugger_cursor_selection, resolve_debugger_cursor_output,
};
use crate::debug_resume_command::{resolve_artifact_output_input, resolve_nsdb_program};
use std::{path::PathBuf, process::Command};

pub(crate) fn handle_debug_request(
    input: PathBuf,
    request_id: String,
    json: bool,
    cursor_output: Option<PathBuf>,
) -> Result<(), String> {
    let (output_dir, _) = resolve_artifact_output_input(&input, "debug-request")?;
    let cursor_output = cursor_output
        .as_deref()
        .map(resolve_debugger_cursor_output)
        .transpose()?;
    let mut command = Command::new(resolve_nsdb_program());
    command
        .arg("replay")
        .arg(&output_dir)
        .arg("--request-id")
        .arg(request_id);
    if let Some(output) = cursor_output.as_deref() {
        command.arg("--save-cursor").arg(output);
    }
    if json {
        command.arg("--json");
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to start Nsdb request debugger: {error}"))?;
    if status.success() {
        if let Some(output) = cursor_output.as_deref() {
            persist_debugger_cursor_selection(&output_dir, output)?;
        }
        Ok(())
    } else {
        Err(format!(
            "Nsdb request debugger failed with status {}",
            status.code().unwrap_or(1)
        ))
    }
}
