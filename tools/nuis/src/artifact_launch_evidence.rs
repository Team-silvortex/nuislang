use crate::{
    json_bool_field, json_field, json_optional_bool_field, json_optional_string_field,
    json_string_array_field, json_usize_field,
};
use std::path::PathBuf;

pub(crate) struct HostRunnerOutput {
    pub(crate) program: PathBuf,
    pub(crate) status: std::process::ExitStatus,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl HostRunnerOutput {
    pub(crate) fn status_code_text(&self) -> String {
        self.status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_owned())
    }
}

pub(crate) struct HostRunnerJsonSurface {
    invoked: bool,
    status: String,
    program: Option<String>,
    exit_status: Option<String>,
    error: Option<String>,
    pub(crate) ready: Option<bool>,
    would_enter_lifecycle_hook: Option<bool>,
    nsb_readable: Option<bool>,
    nsb_hash_matches: Option<bool>,
    nsb_payload_region_mapped: Option<bool>,
    nsb_payload_scan_kind: Option<String>,
    container_loader_status: Option<String>,
    container_ready: Option<bool>,
    container_loader_entry_kind: Option<String>,
    container_loader_entry_symbol: Option<String>,
    container_loader_entry_section_id: Option<String>,
    pub(crate) container_loader_handoff_ready: Option<bool>,
    container_loader_handoff_status: Option<String>,
}

pub(crate) struct RunArtifactLaunchEvidence {
    protocol: &'static str,
    status: String,
    route: String,
    evidence_status: String,
    debugger_contract: &'static str,
    command: Option<String>,
    host_runner_probe_status: String,
    host_runner_probe_ready: Option<bool>,
    first_payload_status: Option<String>,
    first_payload_ready: Option<bool>,
    first_payload_target: Option<String>,
    first_payload_entry_symbol: Option<String>,
    first_payload_entry_kind: Option<String>,
    first_payload_entry_section_id: Option<String>,
    first_payload_first_blocker: Option<String>,
    backend_artifact_payload_evidence_available: bool,
    backend_artifact_payload_count: usize,
    backend_artifact_payload_present_count: usize,
    backend_artifact_payload_role_status: String,
    backend_artifact_payload_ids: Vec<String>,
    backend_artifact_payload_kinds: Vec<String>,
    backend_artifact_payload_first_missing: Option<String>,
    first_blocker: Option<String>,
    reason: String,
}

impl RunArtifactLaunchEvidence {
    pub(crate) fn from_surfaces(
        prelaunch: &crate::run_artifact::RunArtifactPrelaunchSummary,
        host_runner: &HostRunnerJsonSurface,
    ) -> Self {
        Self::from_surfaces_with_backend_payload_evidence(
            prelaunch,
            host_runner,
            &crate::artifact_doctor::BackendArtifactPayloadEvidence::unavailable(),
        )
    }

    pub(crate) fn from_surfaces_with_backend_payload_evidence(
        prelaunch: &crate::run_artifact::RunArtifactPrelaunchSummary,
        host_runner: &HostRunnerJsonSurface,
        backend_evidence: &crate::artifact_doctor::BackendArtifactPayloadEvidence,
    ) -> Self {
        let first_blocker = launch_evidence_first_blocker(prelaunch, host_runner);
        let first_payload_ready = launch_evidence_first_payload_ready(prelaunch, host_runner);
        Self {
            protocol: "nuis-run-artifact-launch-evidence-v1",
            status: if first_blocker.is_none() {
                "ready".to_owned()
            } else {
                "blocked".to_owned()
            },
            route: prelaunch.kind.clone(),
            evidence_status: prelaunch.evidence_status.clone(),
            debugger_contract: "nsdb-yir-launch-evidence-v1",
            command: prelaunch.command.clone(),
            host_runner_probe_status: host_runner.status.clone(),
            host_runner_probe_ready: host_runner.ready,
            first_payload_status: launch_evidence_first_payload_status(first_payload_ready),
            first_payload_ready,
            first_payload_target: launch_evidence_first_payload_target(prelaunch),
            first_payload_entry_symbol: if first_payload_ready.is_some() {
                host_runner.container_loader_entry_symbol.clone()
            } else {
                None
            },
            first_payload_entry_kind: if first_payload_ready.is_some() {
                host_runner.container_loader_entry_kind.clone()
            } else {
                None
            },
            first_payload_entry_section_id: if first_payload_ready.is_some() {
                host_runner.container_loader_entry_section_id.clone()
            } else {
                None
            },
            first_payload_first_blocker: launch_evidence_first_payload_first_blocker(
                prelaunch,
                host_runner,
                first_payload_ready,
            ),
            backend_artifact_payload_evidence_available: backend_evidence.available,
            backend_artifact_payload_count: backend_evidence.count,
            backend_artifact_payload_present_count: backend_evidence.present_count,
            backend_artifact_payload_role_status: backend_evidence.role_status.clone(),
            backend_artifact_payload_ids: backend_evidence.ids.clone(),
            backend_artifact_payload_kinds: backend_evidence.kinds.clone(),
            backend_artifact_payload_first_missing: backend_evidence.first_missing.clone(),
            first_blocker,
            reason: prelaunch.reason.clone(),
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        self.json_fields_with_prefix("launch_evidence")
    }

    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        // Contract anchors: launch_evidence_backend_artifact_payload_ids,
        // launch_evidence_backend_artifact_payload_kinds.
        vec![
            json_field(&format!("{prefix}_protocol"), self.protocol),
            json_field(&format!("{prefix}_status"), &self.status),
            json_field(&format!("{prefix}_route"), &self.route),
            json_field(&format!("{prefix}_status_code"), &self.evidence_status),
            json_field(
                &format!("{prefix}_debugger_contract"),
                self.debugger_contract,
            ),
            json_optional_string_field(&format!("{prefix}_command"), self.command.as_deref()),
            json_field(
                &format!("{prefix}_host_runner_probe_status"),
                &self.host_runner_probe_status,
            ),
            json_optional_bool_field(
                &format!("{prefix}_host_runner_probe_ready"),
                self.host_runner_probe_ready,
            ),
            json_optional_string_field(
                &format!("{prefix}_first_payload_status"),
                self.first_payload_status.as_deref(),
            ),
            json_optional_bool_field(
                &format!("{prefix}_first_payload_ready"),
                self.first_payload_ready,
            ),
            json_optional_string_field(
                &format!("{prefix}_first_payload_target"),
                self.first_payload_target.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_payload_entry_symbol"),
                self.first_payload_entry_symbol.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_payload_entry_kind"),
                self.first_payload_entry_kind.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_payload_entry_section_id"),
                self.first_payload_entry_section_id.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_payload_first_blocker"),
                self.first_payload_first_blocker.as_deref(),
            ),
            json_bool_field(
                &format!("{prefix}_backend_artifact_payload_evidence_available"),
                self.backend_artifact_payload_evidence_available,
            ),
            json_usize_field(
                &format!("{prefix}_backend_artifact_payload_count"),
                self.backend_artifact_payload_count,
            ),
            json_usize_field(
                &format!("{prefix}_backend_artifact_payload_present_count"),
                self.backend_artifact_payload_present_count,
            ),
            json_field(
                &format!("{prefix}_backend_artifact_payload_role_status"),
                &self.backend_artifact_payload_role_status,
            ),
            json_string_array_field(
                &format!("{prefix}_backend_artifact_payload_ids"),
                &self.backend_artifact_payload_ids,
            ),
            json_string_array_field(
                &format!("{prefix}_backend_artifact_payload_kinds"),
                &self.backend_artifact_payload_kinds,
            ),
            json_optional_string_field(
                &format!("{prefix}_backend_artifact_payload_first_missing"),
                self.backend_artifact_payload_first_missing.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_blocker"),
                self.first_blocker.as_deref(),
            ),
            json_field(&format!("{prefix}_reason"), &self.reason),
        ]
    }
}

impl HostRunnerJsonSurface {
    pub(crate) fn not_invoked(status: &str) -> Self {
        Self {
            invoked: false,
            status: status.to_owned(),
            program: None,
            exit_status: None,
            error: None,
            ready: None,
            would_enter_lifecycle_hook: None,
            nsb_readable: None,
            nsb_hash_matches: None,
            nsb_payload_region_mapped: None,
            nsb_payload_scan_kind: None,
            container_loader_status: None,
            container_ready: None,
            container_loader_entry_kind: None,
            container_loader_entry_symbol: None,
            container_loader_entry_section_id: None,
            container_loader_handoff_ready: None,
            container_loader_handoff_status: None,
        }
    }

    pub(crate) fn from_error(program: PathBuf, status: &str, error: String) -> Self {
        Self {
            invoked: true,
            status: status.to_owned(),
            program: Some(program.display().to_string()),
            exit_status: None,
            error: Some(error),
            ready: None,
            would_enter_lifecycle_hook: None,
            nsb_readable: None,
            nsb_hash_matches: None,
            nsb_payload_region_mapped: None,
            nsb_payload_scan_kind: None,
            container_loader_status: None,
            container_ready: None,
            container_loader_entry_kind: None,
            container_loader_entry_symbol: None,
            container_loader_entry_section_id: None,
            container_loader_handoff_ready: None,
            container_loader_handoff_status: None,
        }
    }

    pub(crate) fn from_output(output: &HostRunnerOutput) -> Self {
        let status = if output.status.success() {
            json_bool_value(&output.stdout, "ready")
                .map(|ready| if ready { "ready" } else { "blocked" })
                .unwrap_or("reported")
        } else {
            "failed"
        };
        Self {
            invoked: true,
            status: status.to_owned(),
            program: Some(output.program.display().to_string()),
            exit_status: Some(output.status_code_text()),
            error: if output.status.success() {
                None
            } else {
                Some(format!(
                    "stdout:\n{}\nstderr:\n{}",
                    output.stdout, output.stderr
                ))
            },
            ready: json_bool_value(&output.stdout, "ready"),
            would_enter_lifecycle_hook: json_bool_value(
                &output.stdout,
                "would_enter_lifecycle_hook",
            ),
            nsb_readable: json_bool_value(&output.stdout, "nsb_readable"),
            nsb_hash_matches: json_bool_value(&output.stdout, "nsb_hash_matches"),
            nsb_payload_region_mapped: json_bool_value(&output.stdout, "nsb_payload_region_mapped"),
            nsb_payload_scan_kind: json_string_value(&output.stdout, "nsb_payload_scan_kind"),
            container_loader_status: json_string_value(&output.stdout, "container_loader_status"),
            container_ready: json_bool_value(&output.stdout, "container_ready"),
            container_loader_entry_kind: json_string_value(
                &output.stdout,
                "container_loader_entry_kind",
            ),
            container_loader_entry_symbol: json_string_value(
                &output.stdout,
                "container_loader_entry_symbol",
            ),
            container_loader_entry_section_id: json_string_value(
                &output.stdout,
                "container_loader_entry_section_id",
            ),
            container_loader_handoff_ready: json_bool_value(
                &output.stdout,
                "container_loader_handoff_ready",
            ),
            container_loader_handoff_status: json_string_value(
                &output.stdout,
                "container_loader_handoff_status",
            ),
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_bool_field("host_runner_invoked", self.invoked),
            json_field("host_runner_status", &self.status),
            json_optional_string_field("host_runner_program", self.program.as_deref()),
            json_optional_string_field("host_runner_exit_status", self.exit_status.as_deref()),
            json_optional_string_field("host_runner_error", self.error.as_deref()),
            json_optional_bool_field("host_runner_ready", self.ready),
            json_optional_bool_field(
                "host_runner_would_enter_lifecycle_hook",
                self.would_enter_lifecycle_hook,
            ),
            json_optional_bool_field("host_runner_nsb_readable", self.nsb_readable),
            json_optional_bool_field("host_runner_nsb_hash_matches", self.nsb_hash_matches),
            json_optional_bool_field(
                "host_runner_nsb_payload_region_mapped",
                self.nsb_payload_region_mapped,
            ),
            json_optional_string_field(
                "host_runner_nsb_payload_scan_kind",
                self.nsb_payload_scan_kind.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_container_loader_status",
                self.container_loader_status.as_deref(),
            ),
            json_optional_bool_field("host_runner_container_ready", self.container_ready),
            json_optional_string_field(
                "host_runner_container_loader_entry_kind",
                self.container_loader_entry_kind.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_container_loader_entry_symbol",
                self.container_loader_entry_symbol.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_container_loader_entry_section_id",
                self.container_loader_entry_section_id.as_deref(),
            ),
            json_optional_bool_field(
                "host_runner_container_loader_handoff_ready",
                self.container_loader_handoff_ready,
            ),
            json_optional_string_field(
                "host_runner_container_loader_handoff_status",
                self.container_loader_handoff_status.as_deref(),
            ),
        ]
    }
}

fn launch_evidence_first_blocker(
    prelaunch: &crate::run_artifact::RunArtifactPrelaunchSummary,
    host_runner: &HostRunnerJsonSurface,
) -> Option<String> {
    if prelaunch.status != "ready" {
        return Some(format!("prelaunch:{}", prelaunch.evidence_status));
    }
    if prelaunch.nsld_runtime_handoff_ready() {
        if host_runner.status != "ready" {
            return Some(format!("host-runner-probe:{}", host_runner.status));
        }
        if host_runner.ready != Some(true) {
            return Some("host-runner-probe:not-ready".to_owned());
        }
        if host_runner.container_loader_handoff_ready != Some(true) {
            return Some("container-loader-handoff:not-ready".to_owned());
        }
    }
    None
}

fn launch_evidence_first_payload_ready(
    prelaunch: &crate::run_artifact::RunArtifactPrelaunchSummary,
    host_runner: &HostRunnerJsonSurface,
) -> Option<bool> {
    if prelaunch.kind == "nsld-host-entrypoint" {
        Some(host_runner.container_loader_handoff_ready == Some(true))
    } else {
        None
    }
}

fn launch_evidence_first_payload_status(first_payload_ready: Option<bool>) -> Option<String> {
    first_payload_ready.map(|ready| {
        if ready {
            "ready".to_owned()
        } else {
            "blocked".to_owned()
        }
    })
}

fn launch_evidence_first_payload_target(
    prelaunch: &crate::run_artifact::RunArtifactPrelaunchSummary,
) -> Option<String> {
    if prelaunch.kind == "nsld-host-entrypoint" {
        Some("container-loader".to_owned())
    } else {
        None
    }
}

fn launch_evidence_first_payload_first_blocker(
    prelaunch: &crate::run_artifact::RunArtifactPrelaunchSummary,
    host_runner: &HostRunnerJsonSurface,
    first_payload_ready: Option<bool>,
) -> Option<String> {
    if prelaunch.kind != "nsld-host-entrypoint" || first_payload_ready != Some(false) {
        return None;
    }
    let status = host_runner
        .container_loader_handoff_status
        .as_deref()
        .unwrap_or("unavailable");
    Some(format!("container-loader-handoff:{status}"))
}

pub(crate) fn print_launch_evidence_text(evidence: &RunArtifactLaunchEvidence) {
    println!("  launch_evidence_protocol: {}", evidence.protocol);
    println!("  launch_evidence_status: {}", evidence.status);
    println!("  launch_evidence_route: {}", evidence.route);
    println!(
        "  launch_evidence_status_code: {}",
        evidence.evidence_status
    );
    println!(
        "  launch_evidence_debugger_contract: {}",
        evidence.debugger_contract
    );
    println!(
        "  launch_evidence_command: {}",
        evidence.command.as_deref().unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_host_runner_probe_status: {}",
        evidence.host_runner_probe_status
    );
    println!(
        "  launch_evidence_host_runner_probe_ready: {}",
        optional_bool_text(evidence.host_runner_probe_ready)
    );
    println!(
        "  launch_evidence_first_payload_status: {}",
        evidence.first_payload_status.as_deref().unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_first_payload_ready: {}",
        optional_bool_text(evidence.first_payload_ready)
    );
    println!(
        "  launch_evidence_first_payload_target: {}",
        evidence.first_payload_target.as_deref().unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_first_payload_entry_symbol: {}",
        evidence
            .first_payload_entry_symbol
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_first_payload_entry_kind: {}",
        evidence
            .first_payload_entry_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_first_payload_entry_section_id: {}",
        evidence
            .first_payload_entry_section_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_first_payload_first_blocker: {}",
        evidence
            .first_payload_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_backend_artifact_payload_evidence_available: {}",
        evidence.backend_artifact_payload_evidence_available
    );
    println!(
        "  launch_evidence_backend_artifact_payload_count: {}",
        evidence.backend_artifact_payload_count
    );
    println!(
        "  launch_evidence_backend_artifact_payload_present_count: {}",
        evidence.backend_artifact_payload_present_count
    );
    println!(
        "  launch_evidence_backend_artifact_payload_role_status: {}",
        evidence.backend_artifact_payload_role_status
    );
    for payload_id in &evidence.backend_artifact_payload_ids {
        println!("  launch_evidence_backend_artifact_payload_id: {payload_id}");
    }
    for payload_kind in &evidence.backend_artifact_payload_kinds {
        println!("  launch_evidence_backend_artifact_payload_kind: {payload_kind}");
    }
    println!(
        "  launch_evidence_backend_artifact_payload_first_missing: {}",
        evidence
            .backend_artifact_payload_first_missing
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_first_blocker: {}",
        evidence.first_blocker.as_deref().unwrap_or("<none>")
    );
    println!("  launch_evidence_reason: {}", evidence.reason);
}

fn json_bool_value(source: &str, key: &str) -> Option<bool> {
    let true_needle = format!("\"{key}\":true");
    if source.contains(&true_needle) {
        return Some(true);
    }
    let false_needle = format!("\"{key}\":false");
    if source.contains(&false_needle) {
        return Some(false);
    }
    None
}

fn json_string_value(source: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = source.find(&needle)? + needle.len();
    let tail = &source[start..];
    let end = tail.find('"')?;
    Some(tail[..end].to_owned())
}

pub(crate) fn optional_bool_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "<none>",
    }
}
