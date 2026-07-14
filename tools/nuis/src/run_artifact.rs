use crate::workflow::nsld_final_executable_tail_summary;
use std::{
    fs,
    path::{Path, PathBuf},
};

const NSLD_HOST_ENTRYPOINT_STUB_PROTOCOL: &str = "nuis-nsld-host-entrypoint-v1";

pub(crate) struct RunArtifactPrelaunchSummary {
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) command: Option<String>,
    pub(crate) entrypoint_path: Option<String>,
    pub(crate) reason: String,
}

impl RunArtifactPrelaunchSummary {
    pub(crate) fn nsld_runtime_handoff_ready(&self) -> bool {
        self.kind == "nsld-host-entrypoint"
            && self.status == "ready"
            && self.command.is_some()
            && self.entrypoint_path.is_some()
    }
}

pub(crate) fn run_artifact_prelaunch_summary(
    output_dir: Option<&Path>,
    resolved_binary: Option<&Path>,
) -> RunArtifactPrelaunchSummary {
    if let Some(output_dir) = output_dir {
        let nsld_tail = nsld_final_executable_tail_summary(output_dir);
        if nsld_tail.entrypoint_materialization_ready == Some(true)
            && nsld_tail.entrypoint_materialization_present == Some(true)
            && nsld_tail.entrypoint_materialization_hash.is_some()
            && nsld_tail
                .entrypoint_materialization_runner_command
                .is_some()
        {
            if let Some(entrypoint_path) = nsld_tail.entrypoint_materialization_path.as_deref() {
                return nsld_host_entrypoint_prelaunch_summary(
                    output_dir,
                    entrypoint_path,
                    nsld_tail.entrypoint_materialization_runner_command,
                );
            }
        }
    }
    if let Some(binary) = resolved_binary {
        return RunArtifactPrelaunchSummary {
            kind: "host-binary".to_owned(),
            status: "ready".to_owned(),
            command: Some(binary.display().to_string()),
            entrypoint_path: None,
            reason: "legacy host binary path is resolved and can be executed directly".to_owned(),
        };
    }
    RunArtifactPrelaunchSummary {
        kind: "none".to_owned(),
        status: "blocked".to_owned(),
        command: None,
        entrypoint_path: None,
        reason: "no runnable host entrypoint or legacy host binary could be resolved".to_owned(),
    }
}

fn nsld_host_entrypoint_prelaunch_summary(
    output_dir: &Path,
    entrypoint_path: &str,
    command: Option<String>,
) -> RunArtifactPrelaunchSummary {
    let resolved_entrypoint_path = resolve_output_relative_path(output_dir, entrypoint_path);
    if resolved_entrypoint_path.is_file() {
        if !nsld_host_entrypoint_stub_protocol_valid(&resolved_entrypoint_path) {
            return RunArtifactPrelaunchSummary {
                kind: "nsld-host-entrypoint".to_owned(),
                status: "blocked".to_owned(),
                command,
                entrypoint_path: Some(resolved_entrypoint_path.display().to_string()),
                reason: format!(
                    "nsld final executable pipeline reports an entrypoint, but the host entrypoint stub does not declare `{NSLD_HOST_ENTRYPOINT_STUB_PROTOCOL}`"
                ),
            };
        }
        return RunArtifactPrelaunchSummary {
            kind: "nsld-host-entrypoint".to_owned(),
            status: "ready".to_owned(),
            command,
            entrypoint_path: Some(resolved_entrypoint_path.display().to_string()),
            reason: "nsld final executable pipeline materialized a verified host entrypoint stub"
                .to_owned(),
        };
    }
    RunArtifactPrelaunchSummary {
        kind: "nsld-host-entrypoint".to_owned(),
        status: "blocked".to_owned(),
        command,
        entrypoint_path: Some(resolved_entrypoint_path.display().to_string()),
        reason: "nsld final executable pipeline reports an entrypoint, but the host entrypoint stub is missing on disk".to_owned(),
    }
}

fn nsld_host_entrypoint_stub_protocol_valid(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|source| {
        source.contains(&format!(
            "NUIS_HOST_ENTRYPOINT_STUB_PROTOCOL='{NSLD_HOST_ENTRYPOINT_STUB_PROTOCOL}'"
        )) && source.contains("export NUIS_HOST_ENTRYPOINT_STUB_PROTOCOL")
    })
}

fn resolve_output_relative_path(output_dir: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        output_dir.join(path)
    }
}
