use crate::{
    json_bool_field, json_field, json_object_array_field, json_optional_bool_field,
    json_optional_string_field, json_string_array_field, json_usize_field,
};

pub(crate) use crate::artifact_host_runner::{HostRunnerJsonSurface, HostRunnerOutput};

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
    payload_execution_trace_protocol: &'static str,
    payload_execution_trace_available: bool,
    payload_execution_trace_records: Vec<PayloadExecutionTraceRecord>,
    backend_artifact_payload_evidence_available: bool,
    backend_artifact_payload_count: usize,
    backend_artifact_payload_present_count: usize,
    backend_artifact_payload_role_status: String,
    backend_artifact_payload_ids: Vec<String>,
    backend_artifact_payload_kinds: Vec<String>,
    backend_artifact_payload_first_missing: Option<String>,
    hetero_execution_closure_protocol: &'static str,
    hetero_execution_closure_status: String,
    hetero_execution_closure_ready: bool,
    hetero_execution_closure_first_blocker: Option<String>,
    hetero_execution_closure_next_action: String,
    first_blocker: Option<String>,
    reason: String,
}

pub(crate) struct PayloadExecutionTraceRecord {
    pub(crate) trace_id: String,
    pub(crate) status: String,
    pub(crate) execution_phase: String,
    pub(crate) target: Option<String>,
    pub(crate) entry_symbol: Option<String>,
    pub(crate) entry_kind: Option<String>,
    pub(crate) entry_section_id: Option<String>,
    pub(crate) first_blocker: Option<String>,
    pub(crate) next_action: String,
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
        let first_payload_target = launch_evidence_first_payload_target(prelaunch);
        let first_payload_entry_symbol = if first_payload_ready.is_some() {
            host_runner.container_loader_entry_symbol.clone()
        } else {
            None
        };
        let first_payload_entry_kind = if first_payload_ready.is_some() {
            host_runner.container_loader_entry_kind.clone()
        } else {
            None
        };
        let first_payload_entry_section_id = if first_payload_ready.is_some() {
            host_runner.container_loader_entry_section_id.clone()
        } else {
            None
        };
        let first_payload_first_blocker = launch_evidence_first_payload_first_blocker(
            prelaunch,
            host_runner,
            first_payload_ready,
        );
        let payload_execution_trace_records = payload_execution_trace_records(
            first_payload_ready,
            first_payload_target.clone(),
            first_payload_entry_symbol.clone(),
            first_payload_entry_kind.clone(),
            first_payload_entry_section_id.clone(),
            first_payload_first_blocker.clone(),
        );
        let hetero_execution_closure =
            hetero_execution_closure_summary(host_runner, backend_evidence);
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
            first_payload_target,
            first_payload_entry_symbol,
            first_payload_entry_kind,
            first_payload_entry_section_id,
            first_payload_first_blocker,
            payload_execution_trace_protocol: "nsdb-yir-payload-execution-trace-v1",
            payload_execution_trace_available: !payload_execution_trace_records.is_empty(),
            payload_execution_trace_records,
            backend_artifact_payload_evidence_available: backend_evidence.available,
            backend_artifact_payload_count: backend_evidence.count,
            backend_artifact_payload_present_count: backend_evidence.present_count,
            backend_artifact_payload_role_status: backend_evidence.role_status.clone(),
            backend_artifact_payload_ids: backend_evidence.ids.clone(),
            backend_artifact_payload_kinds: backend_evidence.kinds.clone(),
            backend_artifact_payload_first_missing: backend_evidence.first_missing.clone(),
            hetero_execution_closure_protocol: "nuis-hetero-execution-closure-v1",
            hetero_execution_closure_status: hetero_execution_closure.status,
            hetero_execution_closure_ready: hetero_execution_closure.ready,
            hetero_execution_closure_first_blocker: hetero_execution_closure.first_blocker,
            hetero_execution_closure_next_action: hetero_execution_closure.next_action,
            first_blocker,
            reason: prelaunch.reason.clone(),
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        self.json_fields_with_prefix("launch_evidence")
    }

    pub(crate) fn payload_execution_trace_protocol(&self) -> &'static str {
        self.payload_execution_trace_protocol
    }

    pub(crate) fn payload_execution_trace_records(&self) -> &[PayloadExecutionTraceRecord] {
        &self.payload_execution_trace_records
    }

    pub(crate) fn hetero_execution_closure_protocol(&self) -> &'static str {
        self.hetero_execution_closure_protocol
    }

    pub(crate) fn hetero_execution_closure_status(&self) -> &str {
        &self.hetero_execution_closure_status
    }

    pub(crate) fn hetero_execution_closure_ready(&self) -> bool {
        self.hetero_execution_closure_ready
    }

    pub(crate) fn hetero_execution_closure_first_blocker(&self) -> Option<&str> {
        self.hetero_execution_closure_first_blocker.as_deref()
    }

    pub(crate) fn hetero_execution_closure_next_action(&self) -> &str {
        &self.hetero_execution_closure_next_action
    }

    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        // Contract anchors: launch_evidence_backend_artifact_payload_ids,
        // launch_evidence_backend_artifact_payload_kinds,
        // launch_evidence_payload_execution_trace_records.
        let payload_trace_ready_count = self
            .payload_execution_trace_records
            .iter()
            .filter(|record| record.status == "ready")
            .count();
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
            json_field(
                &format!("{prefix}_payload_execution_trace_protocol"),
                self.payload_execution_trace_protocol,
            ),
            json_bool_field(
                &format!("{prefix}_payload_execution_trace_available"),
                self.payload_execution_trace_available,
            ),
            json_usize_field(
                &format!("{prefix}_payload_execution_trace_record_count"),
                self.payload_execution_trace_records.len(),
            ),
            json_usize_field(
                &format!("{prefix}_payload_execution_trace_ready_record_count"),
                payload_trace_ready_count,
            ),
            json_object_array_field(
                &format!("{prefix}_payload_execution_trace_records"),
                &self
                    .payload_execution_trace_records
                    .iter()
                    .map(PayloadExecutionTraceRecord::json_object)
                    .collect::<Vec<_>>(),
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
            json_field(
                &format!("{prefix}_hetero_execution_closure_protocol"),
                self.hetero_execution_closure_protocol,
            ),
            json_field(
                &format!("{prefix}_hetero_execution_closure_status"),
                &self.hetero_execution_closure_status,
            ),
            json_bool_field(
                &format!("{prefix}_hetero_execution_closure_ready"),
                self.hetero_execution_closure_ready,
            ),
            json_optional_string_field(
                &format!("{prefix}_hetero_execution_closure_first_blocker"),
                self.hetero_execution_closure_first_blocker.as_deref(),
            ),
            json_field(
                &format!("{prefix}_hetero_execution_closure_next_action"),
                &self.hetero_execution_closure_next_action,
            ),
            json_optional_string_field(
                &format!("{prefix}_first_blocker"),
                self.first_blocker.as_deref(),
            ),
            json_field(&format!("{prefix}_reason"), &self.reason),
        ]
    }
}

struct HeteroExecutionClosureSummary {
    status: String,
    ready: bool,
    first_blocker: Option<String>,
    next_action: String,
}

impl PayloadExecutionTraceRecord {
    fn json_object(&self) -> String {
        let fields = vec![
            json_field("trace_id", &self.trace_id),
            json_field("status", &self.status),
            json_field("execution_phase", &self.execution_phase),
            json_optional_string_field("target", self.target.as_deref()),
            json_optional_string_field("entry_symbol", self.entry_symbol.as_deref()),
            json_optional_string_field("entry_kind", self.entry_kind.as_deref()),
            json_optional_string_field("entry_section_id", self.entry_section_id.as_deref()),
            json_optional_string_field("first_blocker", self.first_blocker.as_deref()),
            json_field("next_action", &self.next_action),
        ];
        format!("{{{}}}", fields.join(","))
    }
}

fn hetero_execution_closure_summary(
    host_runner: &HostRunnerJsonSurface,
    backend_evidence: &crate::artifact_doctor::BackendArtifactPayloadEvidence,
) -> HeteroExecutionClosureSummary {
    if !backend_evidence.available || backend_evidence.count == 0 {
        return hetero_execution_closure_blocked(
            "payload-missing",
            "backend-artifact-payload:missing",
            "materialize-backend-artifact-payload",
        );
    }
    if backend_evidence.present_count == 0 {
        return hetero_execution_closure_blocked(
            "payload-pending",
            backend_evidence
                .first_missing
                .as_deref()
                .unwrap_or("backend-artifact-payload:not-present"),
            "repair-backend-artifact-payload-presence",
        );
    }
    if backend_evidence.role_status != "ready" {
        return hetero_execution_closure_blocked(
            "payload-blocked",
            &format!(
                "backend-artifact-payload-role:{}",
                backend_evidence.role_status
            ),
            "repair-backend-artifact-payload-role",
        );
    }
    let Some(host_count) = host_runner.backend_artifact_payload_count else {
        return hetero_execution_closure_blocked(
            "host-runner-pending",
            "host-runner-backend-artifact-payload:not-observed",
            "run-host-runner-payload-probe",
        );
    };
    if host_count < backend_evidence.present_count {
        return hetero_execution_closure_blocked(
            "host-runner-mismatch",
            "host-runner-backend-artifact-payload-count:mismatch",
            "repair-host-runner-backend-artifact-payload-table",
        );
    }
    if host_runner
        .backend_artifact_payload_ready_count
        .unwrap_or(0)
        == 0
    {
        return hetero_execution_closure_blocked(
            "host-runner-pending",
            "host-runner-backend-artifact-payload-ready-count:zero",
            "complete-host-runner-backend-artifact-payload-ready-probe",
        );
    }
    if let (Some(expected), Some(observed)) = (
        backend_evidence.ids.first(),
        host_runner.backend_artifact_payload_first_id.as_ref(),
    ) {
        if expected != observed {
            return hetero_execution_closure_blocked(
                "host-runner-mismatch",
                "host-runner-backend-artifact-payload-id:mismatch",
                "repair-host-runner-backend-artifact-payload-table",
            );
        }
    }
    if let (Some(expected), Some(observed)) = (
        backend_evidence.kinds.first(),
        host_runner.backend_artifact_payload_first_kind.as_ref(),
    ) {
        if expected != observed {
            return hetero_execution_closure_blocked(
                "host-runner-mismatch",
                "host-runner-backend-artifact-payload-kind:mismatch",
                "repair-host-runner-backend-artifact-payload-table",
            );
        }
    }
    HeteroExecutionClosureSummary {
        status: "closed".to_owned(),
        ready: true,
        first_blocker: None,
        next_action: "handoff-hetero-execution-evidence-to-nsdb".to_owned(),
    }
}

fn hetero_execution_closure_blocked(
    status: &str,
    first_blocker: &str,
    next_action: &str,
) -> HeteroExecutionClosureSummary {
    HeteroExecutionClosureSummary {
        status: status.to_owned(),
        ready: false,
        first_blocker: Some(first_blocker.to_owned()),
        next_action: next_action.to_owned(),
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

fn payload_execution_trace_records(
    first_payload_ready: Option<bool>,
    target: Option<String>,
    entry_symbol: Option<String>,
    entry_kind: Option<String>,
    entry_section_id: Option<String>,
    first_blocker: Option<String>,
) -> Vec<PayloadExecutionTraceRecord> {
    let Some(ready) = first_payload_ready else {
        return Vec::new();
    };
    let target_value = target.as_deref().unwrap_or("unknown-target");
    let symbol_value = entry_symbol.as_deref().unwrap_or("unknown-symbol");
    let status = if ready { "ready" } else { "blocked" };
    let next_action = if ready {
        "handoff-payload-trace-to-nsdb"
    } else {
        "resolve-payload-execution-blocker"
    };

    vec![PayloadExecutionTraceRecord {
        trace_id: format!("payload-trace:{target_value}:{symbol_value}"),
        status: status.to_owned(),
        execution_phase: "container-loader-handoff".to_owned(),
        target,
        entry_symbol,
        entry_kind,
        entry_section_id,
        first_blocker,
        next_action: next_action.to_owned(),
    }]
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
    let payload_trace_ready_count = evidence
        .payload_execution_trace_records
        .iter()
        .filter(|record| record.status == "ready")
        .count();
    println!(
        "  launch_evidence_payload_execution_trace_protocol: {}",
        evidence.payload_execution_trace_protocol
    );
    println!(
        "  launch_evidence_payload_execution_trace_available: {}",
        evidence.payload_execution_trace_available
    );
    println!(
        "  launch_evidence_payload_execution_trace_record_count: {}",
        evidence.payload_execution_trace_records.len()
    );
    println!(
        "  launch_evidence_payload_execution_trace_ready_record_count: {}",
        payload_trace_ready_count
    );
    for record in &evidence.payload_execution_trace_records {
        println!(
            "  launch_evidence_payload_execution_trace_record: {} {} {}",
            record.trace_id, record.execution_phase, record.status
        );
    }
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
        "  launch_evidence_hetero_execution_closure_protocol: {}",
        evidence.hetero_execution_closure_protocol
    );
    println!(
        "  launch_evidence_hetero_execution_closure_status: {}",
        evidence.hetero_execution_closure_status
    );
    println!(
        "  launch_evidence_hetero_execution_closure_ready: {}",
        evidence.hetero_execution_closure_ready
    );
    println!(
        "  launch_evidence_hetero_execution_closure_first_blocker: {}",
        evidence
            .hetero_execution_closure_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  launch_evidence_hetero_execution_closure_next_action: {}",
        evidence.hetero_execution_closure_next_action
    );
    println!(
        "  launch_evidence_first_blocker: {}",
        evidence.first_blocker.as_deref().unwrap_or("<none>")
    );
    println!("  launch_evidence_reason: {}", evidence.reason);
}

pub(crate) fn optional_bool_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "<none>",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launch_evidence_emits_payload_execution_trace_record_for_container_handoff() {
        let prelaunch = crate::run_artifact::RunArtifactPrelaunchSummary {
            kind: "nsld-host-entrypoint".to_owned(),
            status: "ready".to_owned(),
            evidence_status: "entrypoint-ready".to_owned(),
            command: Some("nuis-host-runner app.nsb".to_owned()),
            runner_command_present: true,
            entrypoint_path: Some("nuis.nsld.final-executable-launcher.toml".to_owned()),
            entrypoint_present: true,
            entrypoint_protocol: Some("nuis-nsld-launcher-v1".to_owned()),
            entrypoint_protocol_valid: Some(true),
            reason: "ready".to_owned(),
        };
        let mut host_runner = HostRunnerJsonSurface {
            invoked: true,
            status: "ready".to_owned(),
            program: Some("nuis-host-runner".to_owned()),
            exit_status: Some("0".to_owned()),
            error: None,
            ready: Some(true),
            would_enter_lifecycle_hook: Some(true),
            nsb_readable: Some(true),
            nsb_hash_matches: Some(true),
            nsb_payload_region_mapped: Some(true),
            nsb_payload_scan_kind: Some("nsld-container-toml".to_owned()),
            container_loader_status: Some("parsed".to_owned()),
            container_ready: Some(true),
            container_loader_entry_kind: Some("lifecycle-bootstrap".to_owned()),
            container_loader_entry_symbol: Some("main".to_owned()),
            container_loader_entry_section_id: Some("sec0000.compiled-artifact".to_owned()),
            container_loader_handoff_ready: Some(true),
            container_loader_handoff_status: Some("ready".to_owned()),
            backend_artifact_payload_count: Some(0),
            backend_artifact_payload_parsed_count: Some(0),
            backend_artifact_payload_ready_count: Some(0),
            backend_artifact_payload_first_id: None,
            backend_artifact_payload_first_kind: None,
            backend_artifact_payload_first_role_status: None,
            backend_artifact_payload_table_hash: None,
        };

        let evidence = RunArtifactLaunchEvidence::from_surfaces(&prelaunch, &host_runner);
        let json = evidence.json_fields().join(",");

        assert!(json.contains(
            "\"launch_evidence_payload_execution_trace_protocol\":\"nsdb-yir-payload-execution-trace-v1\""
        ));
        assert!(json.contains("\"launch_evidence_payload_execution_trace_available\":true"));
        assert!(json.contains("\"launch_evidence_payload_execution_trace_record_count\":1"));
        assert!(json.contains("\"launch_evidence_payload_execution_trace_ready_record_count\":1"));
        assert!(json.contains("\"launch_evidence_payload_execution_trace_records\":[{"));
        assert!(json.contains("\"trace_id\":\"payload-trace:container-loader:main\""));
        assert!(json.contains("\"execution_phase\":\"container-loader-handoff\""));
        assert!(json.contains("\"target\":\"container-loader\""));
        assert!(json.contains("\"entry_symbol\":\"main\""));
        assert!(json.contains("\"entry_section_id\":\"sec0000.compiled-artifact\""));
        assert!(json.contains("\"next_action\":\"handoff-payload-trace-to-nsdb\""));
        assert!(json
            .contains("\"launch_evidence_hetero_execution_closure_status\":\"payload-missing\""));

        host_runner.backend_artifact_payload_count = Some(1);
        host_runner.backend_artifact_payload_parsed_count = Some(1);
        host_runner.backend_artifact_payload_ready_count = Some(1);
        host_runner.backend_artifact_payload_first_id =
            Some("payload0005.backend-artifact".to_owned());
        host_runner.backend_artifact_payload_first_kind =
            Some("nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu".to_owned());
        host_runner.backend_artifact_payload_first_role_status = Some("ready".to_owned());
        let backend_evidence = crate::artifact_doctor::BackendArtifactPayloadEvidence {
            available: true,
            path: None,
            count: 1,
            present_count: 1,
            role_status: "ready".to_owned(),
            ids: vec!["payload0005.backend-artifact".to_owned()],
            kinds: vec!["nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu".to_owned()],
            first_missing: None,
        };
        let closed_json = RunArtifactLaunchEvidence::from_surfaces_with_backend_payload_evidence(
            &prelaunch,
            &host_runner,
            &backend_evidence,
        )
        .json_fields()
        .join(",");
        assert!(
            closed_json.contains("\"launch_evidence_hetero_execution_closure_status\":\"closed\"")
        );
        assert!(closed_json.contains("\"launch_evidence_hetero_execution_closure_ready\":true"));
        assert!(closed_json.contains(
            "\"launch_evidence_hetero_execution_closure_next_action\":\"handoff-hetero-execution-evidence-to-nsdb\""
        ));
    }
}
