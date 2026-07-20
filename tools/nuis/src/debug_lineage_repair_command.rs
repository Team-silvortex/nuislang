use crate::{
    artifact_nsdb_replay_cursor_lineage::read_debugger_cursor_lineage,
    debug_resume_command::{resolve_artifact_output_input, resolve_nsdb_program},
};
use std::{path::PathBuf, process::Command};

pub(crate) fn handle_debug_lineage_repair(input: PathBuf, json: bool) -> Result<(), String> {
    let (output_dir, _) = resolve_artifact_output_input(&input, "debug-lineage-repair")?;
    let lineage = read_debugger_cursor_lineage(&output_dir);
    if lineage.status == "lineage-unavailable" {
        return Err(format!(
            "debug-lineage-repair rejected `{}`: lineage-unavailable at `{}`",
            input.display(),
            lineage.path
        ));
    }

    let mut command = Command::new(resolve_nsdb_program());
    command.arg("cursor-lineage-repair").arg(&output_dir);
    if json {
        command.arg("--json");
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to start Nsdb cursor lineage repair: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Nsdb cursor lineage repair failed with status {}",
            status.code().unwrap_or(1)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::handle_debug_lineage_repair;
    use std::fs;

    #[test]
    fn rejects_missing_lineage_before_nsdb_dispatch() {
        let root = std::env::temp_dir().join(format!(
            "nuis-debug-lineage-repair-missing-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create lineage repair test directory");
        fs::write(root.join("nuis.build.manifest.toml"), "manifest = true\n")
            .expect("write manifest");

        let error = handle_debug_lineage_repair(root.clone(), true).unwrap_err();
        assert!(error.contains("lineage-unavailable"));
        fs::remove_dir_all(root).expect("remove lineage repair test directory");
    }
}
