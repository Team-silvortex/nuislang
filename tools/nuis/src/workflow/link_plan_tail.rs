use super::link_plan::{
    nsld_final_executable_pipeline_command_for_output_dir, parse_bool_field,
    parse_first_string_array_item, parse_non_empty_string_field, parse_string_field,
    parse_usize_field,
};
use std::{fs, path::Path};

pub(super) const NSLD_FINAL_EXECUTABLE_TAIL_STAGES: &[(&str, &str)] = &[
    (
        "final-executable-writer-input",
        "nuis.nsld.final-executable-writer-input.toml",
    ),
    (
        "final-executable-host-invoke-plan",
        "nuis.nsld.final-executable-host-invoke-plan.toml",
    ),
    (
        "final-executable-layout",
        "nuis.nsld.final-executable-layout.toml",
    ),
    (
        "final-executable-image-dry-run",
        "nuis.nsld.final-executable-image-dry-run.toml",
    ),
    (
        "final-executable-image-dry-run-bytes",
        "nuis.nsld.final-executable-image-dry-run.bin",
    ),
    (
        "final-executable-blocked",
        "nuis.nsld.final-executable.blocked.toml",
    ),
    (
        "final-executable-launcher",
        "nuis.nsld.final-executable-launcher.toml",
    ),
    (
        "final-executable-launcher-dry-run",
        "nuis.nsld.final-executable-launcher-dry-run.toml",
    ),
    (
        "final-executable-pipeline",
        "nuis.nsld.final-executable-pipeline.toml",
    ),
];

pub(crate) struct NsldFinalExecutableTailSummary {
    pub(crate) ready: bool,
    pub(crate) stage_count: usize,
    pub(crate) present_count: usize,
    pub(crate) next_missing_stage: Option<String>,
    pub(crate) pipeline_command: String,
    pub(crate) pipeline_valid: Option<bool>,
    pub(crate) final_executable_emitted: Option<bool>,
    pub(crate) launcher_manifest_ready: Option<bool>,
    pub(crate) launcher_dry_run_ready: Option<bool>,
    pub(crate) would_enter_lifecycle_hook: Option<bool>,
    pub(crate) blocker_count: Option<usize>,
    pub(crate) first_blocker: Option<String>,
    pub(crate) execution_handoff_contract: Option<String>,
    pub(crate) execution_handoff_ready: Option<bool>,
    pub(crate) execution_handoff_status: Option<String>,
    pub(crate) execution_handoff_target: Option<String>,
    pub(crate) execution_handoff_evidence_status: Option<String>,
    pub(crate) execution_handoff_first_blocker: Option<String>,
    pub(crate) execution_handoff_decision_code: Option<String>,
    pub(crate) scheduler_metadata_payload_id: Option<String>,
    pub(crate) scheduler_metadata_present: Option<bool>,
    pub(crate) scheduler_metadata_hash: Option<String>,
    pub(crate) required_stage_path_count: Option<usize>,
    pub(crate) required_stage_path_present_count: Option<usize>,
    pub(crate) first_missing_required_stage_path: Option<String>,
    pub(crate) self_owned_image_status: String,
    pub(crate) entrypoint_materialization_status: String,
    pub(crate) entrypoint_materialization_kind: Option<String>,
    pub(crate) entrypoint_materialization_path: Option<String>,
    pub(crate) entrypoint_materialization_ready: Option<bool>,
    pub(crate) entrypoint_materialization_first_blocker: Option<String>,
    pub(crate) entrypoint_materialization_present: Option<bool>,
    pub(crate) entrypoint_materialization_hash: Option<String>,
    pub(crate) entrypoint_materialization_runner_command: Option<String>,
    pub(crate) self_owned_image_ready: Option<bool>,
    pub(crate) self_owned_image_path: Option<String>,
    pub(crate) self_owned_image_present: Option<bool>,
    pub(crate) self_owned_image_hash: Option<String>,
    pub(crate) self_owned_image_header_valid: Option<bool>,
}

pub(crate) fn nsld_final_executable_tail_summary(
    output_dir: &Path,
) -> NsldFinalExecutableTailSummary {
    let mut present_count = 0usize;
    let mut next_missing_stage = None;
    for (stage, file_name) in NSLD_FINAL_EXECUTABLE_TAIL_STAGES {
        if output_dir.join(file_name).exists() {
            present_count += 1;
        } else if next_missing_stage.is_none() {
            next_missing_stage = Some((*stage).to_owned());
        }
    }
    let pipeline = output_dir.join("nuis.nsld.final-executable-pipeline.toml");
    let launcher_manifest = output_dir.join("nuis.nsld.final-executable-launcher.toml");
    let (
        pipeline_valid,
        final_executable_emitted,
        launcher_manifest_ready,
        launcher_dry_run_ready,
        would_enter_lifecycle_hook,
        blocker_count,
        first_blocker,
        pipeline_execution_handoff_contract,
        pipeline_execution_handoff_ready,
        pipeline_execution_handoff_status,
        pipeline_execution_handoff_target,
        pipeline_execution_handoff_evidence_status,
        pipeline_execution_handoff_first_blocker,
        pipeline_execution_handoff_decision_code,
        scheduler_metadata_payload_id,
        scheduler_metadata_present,
        scheduler_metadata_hash,
        required_stage_path_count,
        required_stage_path_present_count,
        first_missing_required_stage_path,
        pipeline_self_owned_image_status,
        pipeline_entrypoint_materialization_status,
        pipeline_entrypoint_materialization_kind,
        pipeline_entrypoint_materialization_path,
        pipeline_entrypoint_materialization_ready,
        pipeline_entrypoint_materialization_first_blocker,
        pipeline_entrypoint_materialization_present,
        pipeline_entrypoint_materialization_hash,
        pipeline_entrypoint_materialization_runner_command,
    ) = fs::read_to_string(&pipeline)
        .ok()
        .map(|source| {
            (
                parse_bool_field(&source, "valid"),
                parse_bool_field(&source, "final_executable_emitted"),
                parse_bool_field(&source, "launcher_manifest_ready"),
                parse_bool_field(&source, "launcher_dry_run_ready"),
                parse_bool_field(&source, "would_enter_lifecycle_hook"),
                parse_usize_field(&source, "blocker_count"),
                parse_first_string_array_item(&source, "blockers"),
                parse_string_field(&source, "execution_handoff_contract"),
                parse_bool_field(&source, "execution_handoff_ready"),
                parse_string_field(&source, "execution_handoff_status"),
                parse_string_field(&source, "execution_handoff_target"),
                parse_string_field(&source, "execution_handoff_evidence_status"),
                parse_string_field(&source, "execution_handoff_first_blocker"),
                parse_string_field(&source, "execution_handoff_decision_code"),
                parse_string_field(&source, "scheduler_metadata_payload_id"),
                parse_bool_field(&source, "scheduler_metadata_present"),
                parse_string_field(&source, "scheduler_metadata_hash"),
                parse_usize_field(&source, "required_stage_path_count"),
                parse_usize_field(&source, "required_stage_path_present_count"),
                parse_first_string_array_item(&source, "missing_required_stage_paths"),
                parse_string_field(&source, "self_owned_image_status"),
                parse_string_field(&source, "entrypoint_materialization_status"),
                parse_string_field(&source, "entrypoint_materialization_kind"),
                parse_non_empty_string_field(&source, "entrypoint_materialization_path"),
                parse_bool_field(&source, "entrypoint_materialization_ready"),
                parse_non_empty_string_field(&source, "entrypoint_materialization_first_blocker"),
                parse_bool_field(&source, "entrypoint_materialization_present"),
                parse_non_empty_string_field(&source, "entrypoint_materialization_hash"),
                parse_non_empty_string_field(&source, "entrypoint_materialization_runner_command"),
            )
        })
        .unwrap_or((
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None,
        ));
    let (
        self_owned_image_path,
        self_owned_image_present,
        self_owned_image_hash,
        self_owned_image_header_valid,
    ) = fs::read_to_string(&launcher_manifest)
        .ok()
        .map(|source| {
            (
                parse_string_field(&source, "nsb_path"),
                parse_bool_field(&source, "nsb_present"),
                parse_string_field(&source, "nsb_hash"),
                parse_bool_field(&source, "image_header_valid"),
            )
        })
        .unwrap_or((None, None, None, None));
    let launcher_manifest_present = launcher_manifest.exists();
    let self_owned_image_ready = self_owned_image_present
        .map(|present| present && self_owned_image_header_valid == Some(true));
    let self_owned_image_status = pipeline_self_owned_image_status.unwrap_or_else(|| {
        nsld_self_owned_image_status(
            launcher_manifest_present,
            self_owned_image_ready,
            self_owned_image_path.as_deref(),
            self_owned_image_present,
            self_owned_image_hash.as_deref(),
            self_owned_image_header_valid,
        )
        .to_owned()
    });
    let entrypoint_materialization_status = pipeline_entrypoint_materialization_status
        .unwrap_or_else(|| {
            nsld_entrypoint_materialization_status(self_owned_image_status.as_str())
        });
    let stage_count = NSLD_FINAL_EXECUTABLE_TAIL_STAGES.len();
    NsldFinalExecutableTailSummary {
        ready: present_count == stage_count && pipeline_valid == Some(true),
        stage_count,
        present_count,
        next_missing_stage,
        pipeline_command: nsld_final_executable_pipeline_command_for_output_dir(output_dir),
        pipeline_valid,
        final_executable_emitted,
        launcher_manifest_ready,
        launcher_dry_run_ready,
        would_enter_lifecycle_hook,
        blocker_count,
        first_blocker,
        execution_handoff_contract: pipeline_execution_handoff_contract,
        execution_handoff_ready: pipeline_execution_handoff_ready,
        execution_handoff_status: pipeline_execution_handoff_status,
        execution_handoff_target: pipeline_execution_handoff_target,
        execution_handoff_evidence_status: pipeline_execution_handoff_evidence_status,
        execution_handoff_first_blocker: pipeline_execution_handoff_first_blocker,
        execution_handoff_decision_code: pipeline_execution_handoff_decision_code,
        scheduler_metadata_payload_id,
        scheduler_metadata_present,
        scheduler_metadata_hash,
        required_stage_path_count,
        required_stage_path_present_count,
        first_missing_required_stage_path,
        self_owned_image_status,
        entrypoint_materialization_status,
        entrypoint_materialization_kind: pipeline_entrypoint_materialization_kind,
        entrypoint_materialization_path: pipeline_entrypoint_materialization_path,
        entrypoint_materialization_ready: pipeline_entrypoint_materialization_ready,
        entrypoint_materialization_first_blocker: pipeline_entrypoint_materialization_first_blocker,
        entrypoint_materialization_present: pipeline_entrypoint_materialization_present,
        entrypoint_materialization_hash: pipeline_entrypoint_materialization_hash,
        entrypoint_materialization_runner_command:
            pipeline_entrypoint_materialization_runner_command,
        self_owned_image_ready,
        self_owned_image_path,
        self_owned_image_present,
        self_owned_image_hash,
        self_owned_image_header_valid,
    }
}

fn nsld_self_owned_image_status(
    launcher_manifest_present: bool,
    ready: Option<bool>,
    path: Option<&str>,
    present: Option<bool>,
    hash: Option<&str>,
    header_valid: Option<bool>,
) -> &'static str {
    if ready == Some(true) {
        return "ready";
    }
    if !launcher_manifest_present {
        return "manifest-missing";
    }
    if path.is_none() {
        return "path-missing";
    }
    if present == Some(false) {
        return "missing";
    }
    if header_valid == Some(false) {
        return "header-invalid";
    }
    if hash.is_none() && present == Some(true) {
        return "hash-missing";
    }
    "unknown"
}

fn nsld_entrypoint_materialization_status(self_owned_image_status: &str) -> String {
    if self_owned_image_status == "ready" {
        "image-ready-entrypoint-pending".to_owned()
    } else {
        "blocked".to_owned()
    }
}
