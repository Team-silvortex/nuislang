use super::final_executable_container_loader::FinalExecutableContainerLoaderEvidence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalExecutableOutputHandoff {
    pub(crate) contract: String,
    pub(crate) ready: bool,
    pub(crate) status: String,
    pub(crate) target: String,
    pub(crate) evidence_status: String,
    pub(crate) first_blocker: Option<String>,
    pub(crate) decision_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalExecutableFirstPayloadExecution {
    pub(crate) status: String,
    pub(crate) ready: bool,
    pub(crate) target: String,
    pub(crate) entry_symbol: Option<String>,
    pub(crate) entry_kind: Option<String>,
    pub(crate) entry_section_id: Option<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn final_executable_output_materialization_status(
    boundary_status: &str,
    host_native_output: bool,
    output_image_header_valid: bool,
    matches_expected_image: bool,
) -> &'static str {
    if boundary_status != "ready" {
        return "blocked";
    }
    if host_native_output {
        return "host-native-ready";
    }
    if output_image_header_valid && matches_expected_image {
        return "self-contained-image-ready";
    }
    "invalid"
}

pub(crate) fn final_executable_output_recommended_next_action(
    boundary_status: &str,
    host_native_output: bool,
    entrypoint_materialization_evidence_status: &str,
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "handoff-to-runner",
        "ready" if first_payload_execution.ready => "handoff-to-container-loader",
        "ready" if first_payload_execution.target == "container-loader" => {
            "inspect-container-loader-handoff"
        }
        "ready" if entrypoint_materialization_evidence_status == "launcher-dry-run-ready" => {
            "run-artifact-or-handoff-to-runtime"
        }
        "ready" if entrypoint_materialization_evidence_status == "launcher-manifest-ready" => {
            "emit-final-executable-launcher-dry-run"
        }
        "ready" => "emit-final-executable-launcher-manifest",
        "missing" => "emit-final-executable-pipeline",
        "not-nsld-owned" => "run-nsld-drive-or-inspect-output-boundary",
        "unreadable" => "inspect-final-output-permissions",
        "invalid" => "inspect-final-output-diagnostics",
        _ => "inspect-final-output-boundary",
    }
}

pub(crate) fn final_executable_output_entrypoint_materialization_evidence_status(
    boundary_status: &str,
    host_native_output: bool,
    launcher_manifest_ready: Option<bool>,
    launcher_dry_run_ready: Option<bool>,
    launcher_dry_run_would_enter_lifecycle_hook: Option<bool>,
) -> &'static str {
    if boundary_status != "ready" {
        return "blocked";
    }
    if host_native_output {
        return "host-runner-ready";
    }
    if launcher_dry_run_ready == Some(true)
        && launcher_dry_run_would_enter_lifecycle_hook == Some(true)
    {
        return "launcher-dry-run-ready";
    }
    if launcher_manifest_ready == Some(true) {
        return "launcher-manifest-ready";
    }
    "launcher-evidence-missing"
}

pub(crate) fn final_executable_output_execution_handoff_contract() -> &'static str {
    "nsld-final-output-handoff-v1"
}

pub(crate) fn final_executable_output_execution_handoff(
    boundary_status: &str,
    host_native_output: bool,
    blockers: &[String],
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> FinalExecutableOutputHandoff {
    let ready = final_executable_output_execution_handoff_ready(
        boundary_status,
        host_native_output,
        first_payload_execution,
    );
    FinalExecutableOutputHandoff {
        contract: final_executable_output_execution_handoff_contract().to_owned(),
        ready,
        status: final_executable_output_execution_handoff_status(
            boundary_status,
            host_native_output,
            first_payload_execution,
        )
        .to_owned(),
        target: final_executable_output_execution_handoff_target(
            boundary_status,
            host_native_output,
            first_payload_execution,
        )
        .to_owned(),
        evidence_status: final_executable_output_execution_handoff_evidence_status(
            boundary_status,
            host_native_output,
            first_payload_execution,
        )
        .to_owned(),
        first_blocker: final_executable_output_execution_handoff_first_blocker(
            ready,
            blockers,
            first_payload_execution,
        ),
        decision_code: final_executable_output_execution_handoff_decision_code(
            boundary_status,
            host_native_output,
            first_payload_execution,
        )
        .to_owned(),
    }
}

pub(crate) fn final_executable_output_execution_handoff_ready(
    boundary_status: &str,
    host_native_output: bool,
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> bool {
    boundary_status == "ready" && (host_native_output || first_payload_execution.ready)
}

pub(crate) fn final_executable_output_execution_handoff_first_blocker(
    execution_handoff_ready: bool,
    blockers: &[String],
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> Option<String> {
    if execution_handoff_ready {
        None
    } else if first_payload_execution.target == "container-loader" {
        first_payload_execution
            .first_blocker
            .clone()
            .or_else(|| final_executable_output_contract_blocker(blockers))
    } else {
        final_executable_output_contract_blocker(blockers)
    }
}

fn final_executable_output_contract_blocker(blockers: &[String]) -> Option<String> {
    blockers
        .iter()
        .find(|blocker| blocker.starts_with("nustar-dispatch:"))
        .or_else(|| {
            blockers
                .iter()
                .find(|blocker| blocker.starts_with("nustar-backend-artifact:"))
        })
        .or_else(|| {
            blockers
                .iter()
                .find(|blocker| blocker.starts_with("final-executable-output:"))
        })
        .or_else(|| blockers.first())
        .cloned()
}

pub(crate) fn final_executable_output_execution_handoff_decision_code(
    boundary_status: &str,
    host_native_output: bool,
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "handoff-host-runner",
        "ready" if first_payload_execution.ready => "handoff-container-loader-first-payload",
        "ready" => "inspect-container-loader-handoff",
        "missing" => "emit-final-executable",
        "not-nsld-owned" | "unreadable" => "inspect-output-boundary",
        "invalid" => "inspect-output-diagnostics",
        _ => "inspect-output-boundary",
    }
}

pub(crate) fn final_executable_output_execution_handoff_status(
    boundary_status: &str,
    host_native_output: bool,
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "runner-ready",
        "ready" if first_payload_execution.ready => "container-loader-handoff-ready",
        "ready" => "container-loader-handoff-blocked",
        _ => "blocked",
    }
}

pub(crate) fn final_executable_output_execution_handoff_target(
    boundary_status: &str,
    host_native_output: bool,
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "host-runner",
        "ready" if first_payload_execution.target == "container-loader" => "container-loader",
        "ready" => "none",
        _ => "none",
    }
}

pub(crate) fn final_executable_output_execution_handoff_evidence_status(
    boundary_status: &str,
    host_native_output: bool,
    first_payload_execution: &FinalExecutableFirstPayloadExecution,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "host-invoke-plan-ready",
        "ready" if first_payload_execution.ready => "container-loader-handoff-ready",
        "ready" => "container-loader-handoff-blocked",
        _ => "blocked",
    }
}

pub(crate) fn final_executable_first_payload_execution(
    boundary_status: &str,
    host_native_output: bool,
    container_loader_evidence: &FinalExecutableContainerLoaderEvidence,
) -> FinalExecutableFirstPayloadExecution {
    if boundary_status != "ready" {
        return FinalExecutableFirstPayloadExecution {
            status: "blocked".to_owned(),
            ready: false,
            target: "none".to_owned(),
            entry_symbol: None,
            entry_kind: None,
            entry_section_id: None,
            first_blocker: Some(format!("final-output-boundary:{boundary_status}")),
        };
    }
    if host_native_output {
        return FinalExecutableFirstPayloadExecution {
            status: "host-native-runner".to_owned(),
            ready: true,
            target: "host-runner".to_owned(),
            entry_symbol: None,
            entry_kind: None,
            entry_section_id: None,
            first_blocker: None,
        };
    }

    let first_blocker = container_loader_evidence
        .handoff_first_blocker
        .clone()
        .or_else(|| Some("container-loader:handoff-not-ready".to_owned()));
    FinalExecutableFirstPayloadExecution {
        status: if container_loader_evidence.handoff_ready {
            "ready"
        } else {
            "blocked"
        }
        .to_owned(),
        ready: container_loader_evidence.handoff_ready,
        target: "container-loader".to_owned(),
        entry_symbol: container_loader_evidence.entry_symbol.clone(),
        entry_kind: container_loader_evidence.entry_kind.clone(),
        entry_section_id: container_loader_evidence.entry_section_id.clone(),
        first_blocker: if container_loader_evidence.handoff_ready {
            None
        } else {
            first_blocker
        },
    }
}

pub(crate) fn final_executable_output_boundary_status(
    runnable_candidate: bool,
    path_present: bool,
    nsld_owned_output: bool,
    present: bool,
    blockers: &[String],
) -> &'static str {
    if runnable_candidate && blockers.is_empty() {
        return "ready";
    }
    if !path_present {
        return "missing";
    }
    if !nsld_owned_output {
        return "not-nsld-owned";
    }
    if !present {
        return "unreadable";
    }
    "invalid"
}
