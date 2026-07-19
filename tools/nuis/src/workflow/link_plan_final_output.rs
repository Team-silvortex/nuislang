use super::{
    link_plan::{parse_bool_field, parse_string_field, parse_usize_field},
    object_identity::workflow_object_identity,
};
use crate::artifact_doctor_mirrors::collect_device_provider_sample_manifest_mirror;
use crate::artifact_nsdb_handoff::read_persisted_nsdb_handoff;
use crate::artifact_nsdb_replay_cursor::read_debugger_cursor_handoff;
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
    pub(crate) device_provider_sample_manifest_available: bool,
    pub(crate) device_provider_sample_manifest_status: String,
    pub(crate) device_provider_sample_manifest_record_count: usize,
    pub(crate) device_provider_sample_manifest_pending_record_count: usize,
    pub(crate) device_provider_sample_manifest_blocked_record_count: usize,
    pub(crate) device_provider_sample_manifest_first_provider_family: String,
    pub(crate) device_provider_sample_manifest_first_materialization_status: String,
    pub(crate) nsdb_replay_contract: String,
    pub(crate) nsdb_replay_ready: bool,
    pub(crate) nsdb_replay_status: String,
    pub(crate) nsdb_replay_checkpoint_count: usize,
    pub(crate) nsdb_replayable_checkpoint_count: usize,
    pub(crate) nsdb_replay_command: Option<String>,
    pub(crate) nsdb_replay_next_action: String,
    pub(crate) nsdb_replay_next_command: Option<String>,
    pub(crate) nsdb_replay_first_blocker: Option<String>,
    pub(crate) object_package_summary_contract: String,
    pub(crate) object_package_summary_ready: bool,
    pub(crate) object_package_summary_status: String,
    pub(crate) object_package_summary_next_action: String,
    pub(crate) object_package_summary_next_command: Option<String>,
    pub(crate) debugger_transcript_contract: String,
    pub(crate) debugger_transcript_ready: bool,
    pub(crate) debugger_transcript_status: String,
    pub(crate) debugger_transcript_next_action: String,
    pub(crate) debugger_transcript_first_blocker: Option<String>,
    pub(crate) debugger_cursor_handoff_contract: String,
    pub(crate) debugger_cursor_path: String,
    pub(crate) debugger_cursor_ready: bool,
    pub(crate) debugger_cursor_status: String,
    pub(crate) debugger_cursor_next_command: Option<String>,
    pub(crate) recommended_next_action: String,
    pub(crate) path_present: bool,
    pub(crate) nsld_owned: Option<bool>,
    pub(crate) object_valid: bool,
    pub(crate) object_path: String,
    pub(crate) object_family: String,
    pub(crate) object_magic_status: String,
    pub(crate) object_magic: Option<String>,
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
    let provider_sample_manifest =
        collect_device_provider_sample_manifest_mirror(Some(Path::new(&plan.output_dir)));
    let nsdb_replay = nsld_final_executable_output_nsdb_replay(plan);
    let object_evidence = nsld_final_executable_output_object_evidence(plan);
    let object_package_summary = nsld_final_executable_output_object_package_summary(&nsdb_replay);
    let debugger_transcript = nsld_final_executable_output_debugger_transcript(&nsdb_replay);
    let debugger_cursor = read_debugger_cursor_handoff(
        Path::new(&plan.output_dir),
        &Path::new(&plan.output_dir).join("nuis.build.manifest.toml"),
    );

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
        device_provider_sample_manifest_available: provider_sample_manifest.available,
        device_provider_sample_manifest_status: provider_sample_manifest.status,
        device_provider_sample_manifest_record_count: provider_sample_manifest.record_count,
        device_provider_sample_manifest_pending_record_count: provider_sample_manifest
            .pending_record_count,
        device_provider_sample_manifest_blocked_record_count: provider_sample_manifest
            .blocked_record_count,
        device_provider_sample_manifest_first_provider_family: provider_sample_manifest
            .first_provider_family,
        device_provider_sample_manifest_first_materialization_status: provider_sample_manifest
            .first_materialization_status,
        nsdb_replay_contract: nsdb_replay.contract,
        nsdb_replay_ready: nsdb_replay.ready,
        nsdb_replay_status: nsdb_replay.status,
        nsdb_replay_checkpoint_count: nsdb_replay.checkpoint_count,
        nsdb_replayable_checkpoint_count: nsdb_replay.replayable_checkpoint_count,
        nsdb_replay_command: nsdb_replay.command,
        nsdb_replay_next_action: nsdb_replay.next_action,
        nsdb_replay_next_command: nsdb_replay.next_command,
        nsdb_replay_first_blocker: nsdb_replay.first_blocker,
        object_package_summary_contract: object_package_summary.contract,
        object_package_summary_ready: object_package_summary.ready,
        object_package_summary_status: object_package_summary.status,
        object_package_summary_next_action: object_package_summary.next_action,
        object_package_summary_next_command: object_package_summary.next_command,
        debugger_transcript_contract: debugger_transcript.contract,
        debugger_transcript_ready: debugger_transcript.ready,
        debugger_transcript_status: debugger_transcript.status,
        debugger_transcript_next_action: debugger_transcript.next_action,
        debugger_transcript_first_blocker: debugger_transcript.first_blocker,
        debugger_cursor_handoff_contract: debugger_cursor.contract.to_owned(),
        debugger_cursor_path: debugger_cursor.path,
        debugger_cursor_ready: debugger_cursor.ready,
        debugger_cursor_status: debugger_cursor.status.to_owned(),
        debugger_cursor_next_command: debugger_cursor.next_command,
        recommended_next_action,
        path_present,
        nsld_owned,
        object_valid: object_evidence.valid,
        object_path: object_evidence.object_path,
        object_family: object_evidence.object_family,
        object_magic_status: object_evidence.object_magic_status,
        object_magic: object_evidence.object_magic,
        object_expected_size_bytes: object_evidence.expected_size_bytes,
        object_actual_size_bytes: object_evidence.actual_size_bytes,
        object_expected_hash: object_evidence.expected_hash,
        object_actual_hash: object_evidence.actual_hash,
        object_issues: object_evidence.issues,
        blockers,
        first_blocker,
    }
}

struct NsldFinalExecutableOutputClosureMirror {
    contract: String,
    ready: bool,
    status: String,
    next_action: String,
    next_command: Option<String>,
    first_blocker: Option<String>,
}

fn nsld_final_executable_output_object_package_summary(
    nsdb_replay: &NsldFinalExecutableOutputNsdbReplay,
) -> NsldFinalExecutableOutputClosureMirror {
    let ready = nsdb_replay.ready;
    NsldFinalExecutableOutputClosureMirror {
        contract: "nsld-object-package-summary-v1".to_owned(),
        ready,
        status: if ready {
            "replay-ready"
        } else {
            "replay-blocked"
        }
        .to_owned(),
        next_action: if ready {
            "consume-object-package-summary"
        } else {
            "resolve-object-package-replay-evidence"
        }
        .to_owned(),
        next_command: nsdb_replay.next_command.clone(),
        first_blocker: (!ready).then(|| {
            nsdb_replay
                .first_blocker
                .clone()
                .unwrap_or_else(|| "payload-execution-replay:unknown".to_owned())
        }),
    }
}

fn nsld_final_executable_output_debugger_transcript(
    nsdb_replay: &NsldFinalExecutableOutputNsdbReplay,
) -> NsldFinalExecutableOutputClosureMirror {
    let ready = nsdb_replay.ready;
    NsldFinalExecutableOutputClosureMirror {
        contract: "nsdb-yir-replay-transcript-v1".to_owned(),
        ready,
        status: if ready {
            "transcript-ready"
        } else {
            "transcript-blocked"
        }
        .to_owned(),
        next_action: if ready {
            "consume-nsdb-yir-replay-transcript"
        } else {
            "resolve-nsdb-yir-replay-transcript"
        }
        .to_owned(),
        next_command: nsdb_replay.next_command.clone(),
        first_blocker: (!ready).then(|| {
            nsdb_replay
                .first_blocker
                .clone()
                .unwrap_or_else(|| "payload-execution-replay:unknown".to_owned())
        }),
    }
}

struct NsldFinalExecutableOutputNsdbReplay {
    contract: String,
    ready: bool,
    status: String,
    checkpoint_count: usize,
    replayable_checkpoint_count: usize,
    command: Option<String>,
    next_action: String,
    next_command: Option<String>,
    first_blocker: Option<String>,
}

fn nsld_final_executable_output_nsdb_replay(
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableOutputNsdbReplay {
    let handoff = read_persisted_nsdb_handoff(Some(Path::new(&plan.output_dir)));
    let checkpoint_count = handoff.record_count();
    let replayable_checkpoint_count = handoff.ready_record_count();
    let first_blocker = if !handoff.available() {
        Some("payload-execution-handoff-missing".to_owned())
    } else if !handoff.hetero_execution_closure_ready() {
        handoff.hetero_execution_closure_blocker()
    } else if checkpoint_count == 0 {
        Some("payload-execution-replay:no-checkpoints".to_owned())
    } else if checkpoint_count != replayable_checkpoint_count {
        Some("payload-execution-replay:blocked-checkpoint".to_owned())
    } else {
        handoff.error().map(ToOwned::to_owned)
    };
    let ready = first_blocker.is_none();
    let command = ready.then(|| format!("nsdb replay {} --json", plan.output_dir));
    let next_action = if ready {
        "replay-nsdb-payload-execution"
    } else {
        "resolve-final-output-nsdb-replay"
    }
    .to_owned();
    let next_command = command.clone().or_else(|| {
        Some(format!(
            "nsld final-executable-output {} --json",
            Path::new(&plan.output_dir)
                .join("nuis.build.manifest.toml")
                .display()
        ))
    });
    NsldFinalExecutableOutputNsdbReplay {
        contract: "nsdb-payload-execution-replay-plan-v1".to_owned(),
        ready,
        status: if ready {
            "replay-evidence-ready"
        } else {
            "blocked"
        }
        .to_owned(),
        checkpoint_count,
        replayable_checkpoint_count,
        command,
        next_action,
        next_command,
        first_blocker: if ready {
            None
        } else {
            first_blocker.or_else(|| Some("final-output-nsdb-replay-not-ready".to_owned()))
        },
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
    object_family: String,
    object_magic_status: String,
    object_magic: Option<String>,
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
    let object_identity = workflow_object_identity(
        &plan.cpu_target.object_format,
        object_bytes.as_ref().ok().map(Vec::as_slice),
    );
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
    if object_identity.magic_status == "invalid" {
        issues.push(format!(
            "object_output_magic invalid for {}: found {}",
            object_identity.family,
            object_identity.magic.as_deref().unwrap_or("missing")
        ));
    }

    NsldFinalExecutableOutputObjectEvidence {
        valid: issues.is_empty(),
        object_path: object_path.display().to_string(),
        object_family: object_identity.family,
        object_magic_status: object_identity.magic_status,
        object_magic: object_identity.magic,
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
