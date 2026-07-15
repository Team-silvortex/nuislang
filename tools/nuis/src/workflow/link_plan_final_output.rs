use super::link_plan::{parse_bool_field, parse_string_field, parse_usize_field};
use std::{fs, path::Path};

pub(crate) struct NsldFinalExecutableOutputBoundarySummary {
    pub(crate) ready: bool,
    pub(crate) boundary_status: String,
    pub(crate) materialization_status: String,
    pub(crate) execution_handoff_contract: String,
    pub(crate) execution_handoff_ready: bool,
    pub(crate) execution_handoff_status: String,
    pub(crate) execution_handoff_target: String,
    pub(crate) execution_handoff_evidence_status: String,
    pub(crate) execution_handoff_first_blocker: Option<String>,
    pub(crate) execution_handoff_decision_code: String,
    pub(crate) entrypoint_materialization_evidence_status: String,
    pub(crate) launcher_manifest_present: bool,
    pub(crate) launcher_manifest_ready: Option<bool>,
    pub(crate) launcher_manifest_blocker_count: Option<usize>,
    pub(crate) launcher_dry_run_present: bool,
    pub(crate) launcher_dry_run_ready: Option<bool>,
    pub(crate) launcher_dry_run_would_enter_lifecycle_hook: Option<bool>,
    pub(crate) launcher_dry_run_blocker_count: Option<usize>,
    pub(crate) payload_execution_trace_protocol: String,
    pub(crate) payload_execution_trace_available: bool,
    pub(crate) payload_execution_trace_record_count: usize,
    pub(crate) payload_execution_trace_ready_record_count: usize,
    pub(crate) recommended_next_action: String,
    pub(crate) path_present: bool,
    pub(crate) nsld_owned: Option<bool>,
    pub(crate) object_valid: bool,
    pub(crate) object_path: String,
    pub(crate) object_expected_size_bytes: Option<usize>,
    pub(crate) object_actual_size_bytes: Option<usize>,
    pub(crate) object_expected_hash: Option<String>,
    pub(crate) object_actual_hash: Option<String>,
    pub(crate) object_issues: Vec<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn nsld_final_executable_output_boundary_summary(
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableOutputBoundarySummary {
    let output_path = Path::new(&plan.final_stage.output_path);
    let path_present = output_path.exists();
    let blocked_path = Path::new(&plan.output_dir).join("nuis.nsld.final-executable.blocked.toml");
    let blocked_source = fs::read_to_string(blocked_path).ok();
    let emitted = blocked_source
        .as_deref()
        .and_then(|source| parse_bool_field(source, "emitted"));
    let final_output_present = blocked_source
        .as_deref()
        .and_then(|source| parse_bool_field(source, "final_output_present"));
    let final_output_runnable_candidate = blocked_source
        .as_deref()
        .and_then(|source| parse_bool_field(source, "final_output_runnable_candidate"));
    let launcher_manifest_path =
        Path::new(&plan.output_dir).join("nuis.nsld.final-executable-launcher.toml");
    let launcher_manifest_source = fs::read_to_string(&launcher_manifest_path).ok();
    let launcher_manifest_present = launcher_manifest_source.is_some();
    let launcher_manifest_ready = launcher_manifest_source
        .as_deref()
        .and_then(|source| parse_bool_field(source, "ready"));
    let launcher_manifest_blocker_count = launcher_manifest_source
        .as_deref()
        .and_then(|source| parse_usize_field(source, "blocker_count"));
    let launcher_dry_run_path =
        Path::new(&plan.output_dir).join("nuis.nsld.final-executable-launcher-dry-run.toml");
    let launcher_dry_run_source = fs::read_to_string(&launcher_dry_run_path).ok();
    let launcher_dry_run_present = launcher_dry_run_source.is_some();
    let launcher_dry_run_ready = launcher_dry_run_source
        .as_deref()
        .and_then(|source| parse_bool_field(source, "dry_run_ready"));
    let launcher_dry_run_would_enter_lifecycle_hook = launcher_dry_run_source
        .as_deref()
        .and_then(|source| parse_bool_field(source, "would_enter_lifecycle_hook"));
    let launcher_dry_run_blocker_count = launcher_dry_run_source
        .as_deref()
        .and_then(|source| parse_usize_field(source, "blocker_count"));
    let nsld_owned = emitted.map(|emitted| emitted && path_present);
    let mut blockers = Vec::new();
    if !path_present {
        blockers.push("final-executable-output:missing".to_owned());
    } else if nsld_owned.is_none() {
        blockers.push("final-executable-output:ownership-unknown".to_owned());
    } else if nsld_owned == Some(false) {
        blockers.push("final-executable-output:not-nsld-owned".to_owned());
    }
    let first_blocker = blockers.first().cloned();
    let ready = nsld_owned == Some(true)
        && blockers.is_empty()
        && final_output_runnable_candidate.unwrap_or(true);
    let boundary_status = nsld_final_executable_output_boundary_status(
        ready,
        path_present,
        nsld_owned,
        final_output_present,
        final_output_runnable_candidate,
        &blockers,
    )
    .to_owned();
    let host_native_output = plan.final_stage.link_mode == "host-toolchain-finalize";
    let materialization_status = nsld_final_executable_output_materialization_status(
        boundary_status.as_str(),
        host_native_output,
    )
    .to_owned();
    let execution_handoff = nsld_final_executable_output_execution_handoff(
        boundary_status.as_str(),
        host_native_output,
        &blockers,
    );
    let entrypoint_materialization_evidence_status =
        nsld_final_executable_output_entrypoint_materialization_evidence_status(
            boundary_status.as_str(),
            host_native_output,
            launcher_manifest_ready,
            launcher_dry_run_ready,
            launcher_dry_run_would_enter_lifecycle_hook,
        )
        .to_owned();
    let recommended_next_action = nsld_final_executable_output_recommended_next_action(
        boundary_status.as_str(),
        host_native_output,
        entrypoint_materialization_evidence_status.as_str(),
    )
    .to_owned();
    let payload_execution_trace = nsld_final_executable_output_payload_execution_trace(
        blocked_source.as_deref(),
        ready,
        host_native_output,
    );
    let object_evidence = nsld_final_executable_output_object_evidence(plan);

    NsldFinalExecutableOutputBoundarySummary {
        ready,
        boundary_status,
        materialization_status,
        execution_handoff_contract: execution_handoff.contract,
        execution_handoff_ready: execution_handoff.ready,
        execution_handoff_status: execution_handoff.status,
        execution_handoff_target: execution_handoff.target,
        execution_handoff_evidence_status: execution_handoff.evidence_status,
        execution_handoff_first_blocker: execution_handoff.first_blocker,
        execution_handoff_decision_code: execution_handoff.decision_code,
        entrypoint_materialization_evidence_status,
        launcher_manifest_present,
        launcher_manifest_ready,
        launcher_manifest_blocker_count,
        launcher_dry_run_present,
        launcher_dry_run_ready,
        launcher_dry_run_would_enter_lifecycle_hook,
        launcher_dry_run_blocker_count,
        payload_execution_trace_protocol: payload_execution_trace.protocol,
        payload_execution_trace_available: payload_execution_trace.available,
        payload_execution_trace_record_count: payload_execution_trace.record_count,
        payload_execution_trace_ready_record_count: payload_execution_trace.ready_record_count,
        recommended_next_action,
        path_present,
        nsld_owned,
        object_valid: object_evidence.valid,
        object_path: object_evidence.object_path,
        object_expected_size_bytes: object_evidence.expected_size_bytes,
        object_actual_size_bytes: object_evidence.actual_size_bytes,
        object_expected_hash: object_evidence.expected_hash,
        object_actual_hash: object_evidence.actual_hash,
        object_issues: object_evidence.issues,
        blockers,
        first_blocker,
    }
}

struct NsldFinalExecutableOutputPayloadExecutionTrace {
    protocol: String,
    available: bool,
    record_count: usize,
    ready_record_count: usize,
}

fn nsld_final_executable_output_payload_execution_trace(
    blocked_source: Option<&str>,
    ready: bool,
    host_native_output: bool,
) -> NsldFinalExecutableOutputPayloadExecutionTrace {
    let protocol = blocked_source
        .and_then(|source| parse_string_field(source, "payload_execution_trace_protocol"))
        .unwrap_or_else(|| "nsdb-yir-payload-execution-trace-v1".to_owned());
    let fallback_available = ready && !host_native_output;
    let available = blocked_source
        .and_then(|source| parse_bool_field(source, "payload_execution_trace_available"))
        .unwrap_or(fallback_available);
    let record_count = blocked_source
        .and_then(|source| parse_usize_field(source, "payload_execution_trace_record_count"))
        .unwrap_or(usize::from(available));
    let ready_record_count = blocked_source
        .and_then(|source| parse_usize_field(source, "payload_execution_trace_ready_record_count"))
        .unwrap_or(usize::from(available && ready));

    NsldFinalExecutableOutputPayloadExecutionTrace {
        protocol,
        available,
        record_count,
        ready_record_count,
    }
}

struct NsldFinalExecutableOutputObjectEvidence {
    valid: bool,
    object_path: String,
    expected_size_bytes: Option<usize>,
    actual_size_bytes: Option<usize>,
    expected_hash: Option<String>,
    actual_hash: Option<String>,
    issues: Vec<String>,
}

fn nsld_final_executable_output_object_evidence(
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableOutputObjectEvidence {
    let object_path = Path::new(&plan.output_dir).join(nsld_object_output_file_name(
        plan.cpu_target.object_format.as_str(),
    ));
    let image_dry_run_path = Path::new(&plan.output_dir).join("nuis.nsld.object-image-dry-run.bin");
    let object_bytes = fs::read(&object_path);
    let image_bytes = fs::read(&image_dry_run_path);
    let expected_size_bytes = image_bytes.as_ref().ok().map(Vec::len);
    let actual_size_bytes = object_bytes.as_ref().ok().map(Vec::len);
    let expected_hash = image_bytes.as_ref().ok().map(|bytes| fnv1a64_hex(bytes));
    let actual_hash = object_bytes.as_ref().ok().map(|bytes| fnv1a64_hex(bytes));
    let mut issues = Vec::new();
    if let Err(error) = &object_bytes {
        issues.push(format!(
            "missing_or_unreadable_object_output `{}`: {error}",
            object_path.display()
        ));
    }
    if let Err(error) = &image_bytes {
        issues.push(format!(
            "missing_or_unreadable_object_image_dry_run_bytes `{}`: {error}",
            image_dry_run_path.display()
        ));
    }
    if let (Some(expected), Some(actual)) = (expected_size_bytes, actual_size_bytes) {
        if expected != actual {
            issues.push(format!(
                "object_output_size mismatch: expected {expected}, found {actual}"
            ));
        }
    }
    if let (Some(expected), Some(actual)) = (expected_hash.as_deref(), actual_hash.as_deref()) {
        if expected != actual {
            issues.push(format!(
                "object_output_hash mismatch: expected {expected}, found {actual}"
            ));
        }
    }

    NsldFinalExecutableOutputObjectEvidence {
        valid: issues.is_empty(),
        object_path: object_path.display().to_string(),
        expected_size_bytes,
        actual_size_bytes,
        expected_hash,
        actual_hash,
        issues,
    }
}

fn nsld_object_output_file_name(object_format: &str) -> String {
    let format = object_format.trim();
    let format = if format.is_empty() { "object" } else { format };
    let safe_format = format
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("nuis.nsld.{safe_format}")
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("0x{hash:016x}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldFinalExecutableOutputHandoff {
    contract: String,
    ready: bool,
    status: String,
    target: String,
    evidence_status: String,
    first_blocker: Option<String>,
    decision_code: String,
}

fn nsld_final_executable_output_materialization_status(
    boundary_status: &str,
    host_native_output: bool,
) -> &'static str {
    if boundary_status != "ready" {
        return "blocked";
    }
    if host_native_output {
        return "host-native-ready";
    }
    "self-contained-image-ready"
}

fn nsld_final_executable_output_recommended_next_action(
    boundary_status: &str,
    host_native_output: bool,
    entrypoint_materialization_evidence_status: &str,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "handoff-to-runner",
        "ready" if entrypoint_materialization_evidence_status == "launcher-dry-run-ready" => {
            "run-artifact-or-handoff-to-runtime"
        }
        "ready" if entrypoint_materialization_evidence_status == "launcher-manifest-ready" => {
            "emit-final-executable-launcher-dry-run"
        }
        "ready" => "emit-final-executable-launcher-manifest",
        "missing" => "emit-final-executable-pipeline",
        "not-nsld-owned" | "ownership-unknown" => "run-nsld-drive-or-inspect-output-boundary",
        "unreadable" => "inspect-final-output-permissions",
        "invalid" | "blocked" => "inspect-final-output-diagnostics",
        _ => "inspect-final-output-boundary",
    }
}

fn nsld_final_executable_output_entrypoint_materialization_evidence_status(
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

fn nsld_final_executable_output_execution_handoff_contract() -> &'static str {
    "nsld-final-output-handoff-v1"
}

fn nsld_final_executable_output_execution_handoff(
    boundary_status: &str,
    host_native_output: bool,
    blockers: &[String],
) -> NsldFinalExecutableOutputHandoff {
    let ready = nsld_final_executable_output_execution_handoff_ready(boundary_status);
    NsldFinalExecutableOutputHandoff {
        contract: nsld_final_executable_output_execution_handoff_contract().to_owned(),
        ready,
        status: nsld_final_executable_output_execution_handoff_status(
            boundary_status,
            host_native_output,
        )
        .to_owned(),
        target: nsld_final_executable_output_execution_handoff_target(
            boundary_status,
            host_native_output,
        )
        .to_owned(),
        evidence_status: nsld_final_executable_output_execution_handoff_evidence_status(
            boundary_status,
            host_native_output,
        )
        .to_owned(),
        first_blocker: nsld_final_executable_output_execution_handoff_first_blocker(
            ready, blockers,
        ),
        decision_code: nsld_final_executable_output_execution_handoff_decision_code(
            boundary_status,
            host_native_output,
        )
        .to_owned(),
    }
}

fn nsld_final_executable_output_execution_handoff_ready(boundary_status: &str) -> bool {
    boundary_status == "ready"
}

fn nsld_final_executable_output_execution_handoff_first_blocker(
    execution_handoff_ready: bool,
    blockers: &[String],
) -> Option<String> {
    if execution_handoff_ready {
        None
    } else {
        blockers
            .iter()
            .find(|blocker| blocker.starts_with("final-executable-output:"))
            .or_else(|| blockers.first())
            .cloned()
    }
}

fn nsld_final_executable_output_execution_handoff_decision_code(
    boundary_status: &str,
    host_native_output: bool,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "handoff-host-runner",
        "ready" => "handoff-entrypoint-materializer",
        "missing" => "emit-final-executable",
        "not-nsld-owned" | "ownership-unknown" | "unreadable" => "inspect-output-boundary",
        "invalid" | "blocked" => "inspect-output-diagnostics",
        _ => "inspect-output-boundary",
    }
}

fn nsld_final_executable_output_execution_handoff_status(
    boundary_status: &str,
    host_native_output: bool,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "runner-ready",
        "ready" => "entrypoint-materializer-required",
        _ => "blocked",
    }
}

fn nsld_final_executable_output_execution_handoff_target(
    boundary_status: &str,
    host_native_output: bool,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "host-runner",
        "ready" => "entrypoint-materializer",
        _ => "none",
    }
}

fn nsld_final_executable_output_execution_handoff_evidence_status(
    boundary_status: &str,
    host_native_output: bool,
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "host-invoke-plan-ready",
        "ready" => "image-header-and-hash-ready",
        _ => "blocked",
    }
}

fn nsld_final_executable_output_boundary_status(
    ready: bool,
    path_present: bool,
    nsld_owned: Option<bool>,
    final_output_present: Option<bool>,
    final_output_runnable_candidate: Option<bool>,
    blockers: &[String],
) -> &'static str {
    if ready {
        return "ready";
    }
    if !path_present {
        return "missing";
    }
    if nsld_owned.is_none() {
        return "ownership-unknown";
    }
    if nsld_owned == Some(false) {
        return "not-nsld-owned";
    }
    if final_output_present == Some(false) {
        return "unreadable";
    }
    if final_output_runnable_candidate == Some(false) || !blockers.is_empty() {
        return "invalid";
    }
    "blocked"
}
