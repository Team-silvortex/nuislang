use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowDomainReadiness {
    package_id: String,
    domain_family: String,
    ready: bool,
    selected_lowering_target_present: bool,
    payload_blob_present: bool,
    payload_format_present: bool,
    bridge_stub_present: bool,
    ir_sidecar_present: bool,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowDomainReadinessSummary {
    hetero_units: usize,
    ready_units: usize,
    ready: bool,
    domain_families: Vec<String>,
    first_unready: Option<String>,
    units: Vec<WorkflowDomainReadiness>,
}

pub(crate) fn artifact_doctor_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis artifact-doctor {}", output_dir.display())
}

pub(crate) fn run_artifact_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis run-artifact {}", output_dir.display())
}

const NSLD_PREPARED_ARTIFACT_STAGES: &[(&str, &str)] = &[
    ("link-inputs", "nuis.nsld.link-inputs.toml"),
    ("link-units", "nuis.nsld.link-units.toml"),
    ("link-bundle", "nuis.nsld.link-bundle.toml"),
    ("assemble-plan", "nuis.nsld.assemble-plan.toml"),
    ("section-manifest", "nuis.nsld.section-manifest.toml"),
    ("object-plan", "nuis.nsld.object-plan.toml"),
    ("object-writer-input", "nuis.nsld.object-writer-input.toml"),
    ("object-byte-layout", "nuis.nsld.object-byte-layout.toml"),
    ("object-file-layout", "nuis.nsld.object-file-layout.toml"),
    (
        "object-image-dry-run",
        "nuis.nsld.object-image-dry-run.toml",
    ),
    ("object-emit-blocked", "nuis.nsld.object.blocked.toml"),
    (
        "object-writer-dry-run",
        "nuis.nsld.object-writer-dry-run.toml",
    ),
    ("container-plan", "nuis.nsld.container-plan.toml"),
    ("container", "nuis.nsld.container"),
    ("closure", "nuis.nsld.closure.toml"),
    ("final-stage-plan", "nuis.nsld.final-stage-plan.toml"),
];

const NSLD_FINAL_EXECUTABLE_TAIL_STAGES: &[(&str, &str)] = &[
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

pub(crate) struct NsldPreparedArtifactChainSummary {
    pub(crate) ready: bool,
    pub(crate) stage_count: usize,
    pub(crate) present_count: usize,
    pub(crate) next_missing_stage: Option<String>,
    pub(crate) manifest_path: String,
    pub(crate) prepare_command: String,
}

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
    pub(crate) self_owned_image_ready: Option<bool>,
    pub(crate) self_owned_image_path: Option<String>,
    pub(crate) self_owned_image_present: Option<bool>,
    pub(crate) self_owned_image_hash: Option<String>,
    pub(crate) self_owned_image_header_valid: Option<bool>,
}

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
    pub(crate) recommended_next_action: String,
    pub(crate) path_present: bool,
    pub(crate) nsld_owned: Option<bool>,
    pub(crate) blockers: Vec<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) struct NsldNextActionSummary {
    pub(crate) source: String,
    pub(crate) action: String,
    pub(crate) command: Option<String>,
    pub(crate) reason: String,
}

pub(crate) struct NsldArtifactChainNextActionMirror {
    pub(crate) source: Option<String>,
    pub(crate) command_id: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) command_resolved: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) available: bool,
}

pub(crate) struct NsldDriveRecommendation {
    pub(crate) available: bool,
    pub(crate) mode: String,
    pub(crate) command: Option<String>,
    pub(crate) mutates_artifacts: bool,
    pub(crate) reason: String,
}

pub(crate) struct NsldDriveCommandSet {
    pub(crate) protocol: String,
    pub(crate) recommended_first_json_command: String,
    pub(crate) dry_run_command: String,
    pub(crate) dry_run_json_command: String,
    pub(crate) dry_run_mutates_artifacts: bool,
    pub(crate) apply_next_command: String,
    pub(crate) apply_next_json_command: String,
    pub(crate) apply_next_mutates_artifacts: bool,
    pub(crate) apply_until_clean_command: String,
    pub(crate) apply_until_clean_json_command: String,
    pub(crate) apply_until_clean_mutates_artifacts: bool,
}

pub(crate) fn nsld_next_action_summary(
    prepared: Option<&NsldPreparedArtifactChainSummary>,
    final_tail: Option<&NsldFinalExecutableTailSummary>,
    final_output: Option<&NsldFinalExecutableOutputBoundarySummary>,
) -> NsldNextActionSummary {
    let Some(prepared) = prepared else {
        return NsldNextActionSummary {
            source: "nuis-summary".to_owned(),
            action: "unavailable".to_owned(),
            command: None,
            reason: "link plan is unavailable, so nsld cannot resolve an artifact chain yet"
                .to_owned(),
        };
    };
    if !prepared.ready {
        return NsldNextActionSummary {
            source: "nuis-summary".to_owned(),
            action: "prepare".to_owned(),
            command: Some(prepared.prepare_command.clone()),
            reason: prepared
                .next_missing_stage
                .as_ref()
                .map(|stage| format!("prepared artifact chain is missing `{stage}`"))
                .unwrap_or_else(|| "prepared artifact chain is incomplete".to_owned()),
        };
    }
    let Some(final_tail) = final_tail else {
        return NsldNextActionSummary {
            source: "nuis-summary".to_owned(),
            action: "emit-final-executable-pipeline".to_owned(),
            command: None,
            reason: "final executable tail summary is unavailable after prepare".to_owned(),
        };
    };
    if !final_tail.ready {
        return NsldNextActionSummary {
            source: "nuis-summary".to_owned(),
            action: "emit-final-executable-pipeline".to_owned(),
            command: Some(final_tail.pipeline_command.clone()),
            reason: final_tail
                .next_missing_stage
                .as_ref()
                .map(|stage| format!("final executable tail is missing `{stage}`"))
                .or_else(|| {
                    final_tail.first_blocker.as_ref().map(|blocker| {
                        format!("final executable pipeline is blocked by `{blocker}`")
                    })
                })
                .unwrap_or_else(|| "final executable pipeline is not ready".to_owned()),
        };
    }
    let Some(final_output) = final_output else {
        return NsldNextActionSummary {
            source: "nuis-summary".to_owned(),
            action: "inspect-final-executable-output".to_owned(),
            command: Some(format!(
                "nsld final-executable-output {}",
                prepared.manifest_path
            )),
            reason:
                "final executable tail is ready, but final output boundary summary is unavailable"
                    .to_owned(),
        };
    };
    if !final_output.ready {
        return NsldNextActionSummary {
            source: "nuis-summary".to_owned(),
            action: "inspect-final-executable-output".to_owned(),
            command: Some(format!(
                "nsld final-executable-output {}",
                prepared.manifest_path
            )),
            reason: final_output
                .first_blocker
                .as_ref()
                .map(|blocker| {
                    format!("final executable output boundary is blocked by `{blocker}`")
                })
                .unwrap_or_else(|| "final executable output boundary is not ready".to_owned()),
        };
    }
    NsldNextActionSummary {
        source: "nuis-summary".to_owned(),
        action: "ready".to_owned(),
        command: None,
        reason: "nsld prepared chain, final executable tail, and final output boundary are ready"
            .to_owned(),
    }
}

pub(crate) fn nsld_artifact_chain_next_action_mirror(
    prepared: Option<&NsldPreparedArtifactChainSummary>,
    final_tail: Option<&NsldFinalExecutableTailSummary>,
) -> NsldArtifactChainNextActionMirror {
    let Some(prepared) = prepared else {
        return NsldArtifactChainNextActionMirror::unavailable();
    };
    if let Some(stage) = prepared.next_missing_stage.as_deref() {
        let command_id = nsld_artifact_chain_command_id_for_stage(stage).to_owned();
        let command = format!("nsld {command_id} <input>");
        let command_resolved = format!("nsld {command_id} {}", prepared.manifest_path);
        return NsldArtifactChainNextActionMirror {
            source: Some("required".to_owned()),
            command_id: Some(command_id),
            command: Some(command),
            command_resolved: Some(command_resolved),
            reason: Some(format!("first missing required artifact stage `{stage}`")),
            available: true,
        };
    }
    let Some(final_tail) = final_tail else {
        return NsldArtifactChainNextActionMirror::unavailable();
    };
    if let Some(blocker) = final_tail.first_blocker.as_deref() {
        return final_tail_command_mirror(
            prepared,
            "advisory",
            format!("first artifact-chain advisory: {blocker}"),
        );
    }
    if let Some(stage) = final_tail.next_missing_stage.as_deref() {
        return final_tail_command_mirror(
            prepared,
            "optional",
            format!("first missing optional artifact stage `{stage}`"),
        );
    }
    NsldArtifactChainNextActionMirror::unavailable()
}

pub(crate) fn nsld_drive_recommendation_for_output_dir(
    output_dir: Option<&Path>,
    next_action: &NsldArtifactChainNextActionMirror,
    final_output: Option<&NsldFinalExecutableOutputBoundarySummary>,
) -> NsldDriveRecommendation {
    let Some(output_dir) = output_dir else {
        return NsldDriveRecommendation {
            available: false,
            mode: "unavailable".to_owned(),
            command: None,
            mutates_artifacts: false,
            reason: "link plan is unavailable, so nsld drive cannot resolve an input".to_owned(),
        };
    };
    if next_action.available {
        return NsldDriveRecommendation {
            available: true,
            mode: "apply-next".to_owned(),
            command: Some(nsld_drive_apply_next_command_for_output_dir(output_dir)),
            mutates_artifacts: true,
            reason: next_action
                .reason
                .as_ref()
                .map(|reason| {
                    format!("apply the current nsld artifact-chain next action: {reason}")
                })
                .unwrap_or_else(|| "apply the current nsld artifact-chain next action".to_owned()),
        };
    }
    if let Some(final_output) = final_output {
        if !final_output.ready {
            return NsldDriveRecommendation {
                available: true,
                mode: "dry-run".to_owned(),
                command: Some(nsld_drive_dry_run_command_for_output_dir(output_dir)),
                mutates_artifacts: false,
                reason: final_output
                    .first_blocker
                    .as_ref()
                    .map(|blocker| {
                        format!(
                            "artifact-chain has no mutating next action; inspect the final executable output boundary blocked by `{blocker}`"
                        )
                    })
                    .unwrap_or_else(|| {
                        "artifact-chain has no mutating next action; inspect the final executable output boundary".to_owned()
                    }),
            };
        }
    }
    NsldDriveRecommendation {
        available: true,
        mode: "dry-run".to_owned(),
        command: Some(nsld_drive_dry_run_command_for_output_dir(output_dir)),
        mutates_artifacts: false,
        reason: "no artifact-chain next action is currently available; dry-run verifies nsld drive state without mutating artifacts".to_owned(),
    }
}

impl NsldArtifactChainNextActionMirror {
    fn unavailable() -> Self {
        Self {
            source: None,
            command_id: None,
            command: None,
            command_resolved: None,
            reason: None,
            available: false,
        }
    }
}

fn final_tail_command_mirror(
    prepared: &NsldPreparedArtifactChainSummary,
    source: &str,
    reason: String,
) -> NsldArtifactChainNextActionMirror {
    let command_id = "emit-final-executable-pipeline";
    NsldArtifactChainNextActionMirror {
        source: Some(source.to_owned()),
        command_id: Some(command_id.to_owned()),
        command: Some(format!("nsld {command_id} <input>")),
        command_resolved: Some(format!("nsld {command_id} {}", prepared.manifest_path)),
        reason: Some(reason),
        available: true,
    }
}

fn nsld_artifact_chain_command_id_for_stage(stage: &str) -> &'static str {
    match stage {
        "link-inputs" => "emit-inputs",
        "link-units" => "emit-units",
        "link-bundle" => "emit-bundle",
        "assemble-plan" => "emit-assemble-plan",
        "section-manifest" => "emit-section-manifest",
        "object-plan" => "emit-object-plan",
        "object-byte-layout" => "emit-object-byte-layout",
        "object-file-layout" => "emit-object-file-layout",
        "object-image-dry-run" => "emit-object-image-dry-run",
        "object-emit-blocked" => "emit-object",
        "object-writer-dry-run" => "emit-object-writer-dry-run",
        "container-plan" => "emit-container-plan",
        "container" => "emit-container",
        "closure" => "emit-closure",
        "final-stage-plan" => "emit-final-executable-pipeline",
        "object-writer-input" => "prepare",
        _ => "prepare",
    }
}

pub(crate) fn nsld_prepare_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld prepare {}",
        output_dir.join("nuis.build.manifest.toml").display()
    )
}

pub(crate) fn nsld_drive_dry_run_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {}",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_dry_run_json_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --json",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_next_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --apply",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_next_json_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --apply --json",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_until_clean_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --apply --until-clean",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_until_clean_json_command_for_output_dir(
    output_dir: &Path,
) -> String {
    format!(
        "nsld drive {} --apply --until-clean --json",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_command_set_for_output_dir(output_dir: &Path) -> NsldDriveCommandSet {
    let dry_run_json_command = nsld_drive_dry_run_json_command_for_output_dir(output_dir);
    NsldDriveCommandSet {
        protocol: "nsld-drive-command-set-v1".to_owned(),
        recommended_first_json_command: dry_run_json_command.clone(),
        dry_run_command: nsld_drive_dry_run_command_for_output_dir(output_dir),
        dry_run_json_command,
        dry_run_mutates_artifacts: false,
        apply_next_command: nsld_drive_apply_next_command_for_output_dir(output_dir),
        apply_next_json_command: nsld_drive_apply_next_json_command_for_output_dir(output_dir),
        apply_next_mutates_artifacts: true,
        apply_until_clean_command: nsld_drive_apply_until_clean_command_for_output_dir(output_dir),
        apply_until_clean_json_command: nsld_drive_apply_until_clean_json_command_for_output_dir(
            output_dir,
        ),
        apply_until_clean_mutates_artifacts: true,
    }
}

pub(crate) fn nsld_drive_command_set_json_field(
    name: &str,
    command_set: Option<&NsldDriveCommandSet>,
) -> String {
    let Some(command_set) = command_set else {
        return format!("\"{name}\":null");
    };
    let fields = [
        json_field("protocol", &command_set.protocol),
        json_field(
            "recommended_first_json_command",
            &command_set.recommended_first_json_command,
        ),
        json_field("dry_run_command", &command_set.dry_run_command),
        json_field("dry_run_json_command", &command_set.dry_run_json_command),
        json_bool_field(
            "dry_run_mutates_artifacts",
            command_set.dry_run_mutates_artifacts,
        ),
        json_field("apply_next_command", &command_set.apply_next_command),
        json_field(
            "apply_next_json_command",
            &command_set.apply_next_json_command,
        ),
        json_bool_field(
            "apply_next_mutates_artifacts",
            command_set.apply_next_mutates_artifacts,
        ),
        json_field(
            "apply_until_clean_command",
            &command_set.apply_until_clean_command,
        ),
        json_field(
            "apply_until_clean_json_command",
            &command_set.apply_until_clean_json_command,
        ),
        json_bool_field(
            "apply_until_clean_mutates_artifacts",
            command_set.apply_until_clean_mutates_artifacts,
        ),
    ];
    format!("\"{name}\":{{{}}}", fields.join(","))
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_command_for_output_dir(output_dir: &Path) -> String {
    nsld_drive_apply_next_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_json_command_for_output_dir(output_dir: &Path) -> String {
    nsld_drive_apply_next_json_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_dry_run_command_for_output_dir(output_dir: &Path) -> String {
    nsld_drive_dry_run_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_dry_run_json_command_for_output_dir(
    output_dir: &Path,
) -> String {
    nsld_drive_dry_run_json_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_until_clean_command_for_output_dir(
    output_dir: &Path,
) -> String {
    nsld_drive_apply_until_clean_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_until_clean_json_command_for_output_dir(
    output_dir: &Path,
) -> String {
    nsld_drive_apply_until_clean_json_command_for_output_dir(output_dir)
}

fn nsld_manifest_path_for_output_dir(output_dir: &Path) -> String {
    output_dir
        .join("nuis.build.manifest.toml")
        .display()
        .to_string()
}

pub(crate) fn nsld_prepared_artifact_chain_summary(
    output_dir: &Path,
) -> NsldPreparedArtifactChainSummary {
    let mut present_count = 0usize;
    let mut next_missing_stage = None;
    for (stage, file_name) in NSLD_PREPARED_ARTIFACT_STAGES {
        if output_dir.join(file_name).exists() {
            present_count += 1;
        } else if next_missing_stage.is_none() {
            next_missing_stage = Some((*stage).to_owned());
        }
    }
    let stage_count = NSLD_PREPARED_ARTIFACT_STAGES.len();
    NsldPreparedArtifactChainSummary {
        ready: present_count == stage_count,
        stage_count,
        present_count,
        next_missing_stage,
        manifest_path: nsld_manifest_path_for_output_dir(output_dir),
        prepare_command: nsld_prepare_command_for_output_dir(output_dir),
    }
}

pub(crate) fn nsld_final_executable_pipeline_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld emit-final-executable-pipeline {}",
        output_dir.join("nuis.build.manifest.toml").display()
    )
}

pub(crate) fn nsld_prepared_artifact_stage_records_json(output_dir: &Path) -> Vec<String> {
    NSLD_PREPARED_ARTIFACT_STAGES
        .iter()
        .map(|(stage, file_name)| {
            nsld_artifact_stage_record_json(
                output_dir,
                stage,
                file_name,
                true,
                "required",
                nsld_artifact_chain_command_id_for_stage(stage),
            )
        })
        .collect()
}

pub(crate) fn nsld_final_executable_tail_stage_records_json(output_dir: &Path) -> Vec<String> {
    NSLD_FINAL_EXECUTABLE_TAIL_STAGES
        .iter()
        .map(|(stage, file_name)| {
            nsld_artifact_stage_record_json(
                output_dir,
                stage,
                file_name,
                false,
                "optional",
                "emit-final-executable-pipeline",
            )
        })
        .collect()
}

fn nsld_artifact_stage_record_json(
    output_dir: &Path,
    stage: &str,
    file_name: &str,
    required: bool,
    next_action_source: &str,
    command_id: &str,
) -> String {
    let fields = [
        json_field("stage", stage),
        json_field("file", file_name),
        json_bool_field("present", output_dir.join(file_name).exists()),
        json_bool_field("required", required),
        json_field("next_action_source", next_action_source),
        json_field("command_id", command_id),
    ];
    format!("{{{}}}", fields.join(","))
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
            )
        })
        .unwrap_or((
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
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
    let recommended_next_action = nsld_final_executable_output_recommended_next_action(
        boundary_status.as_str(),
        host_native_output,
    )
    .to_owned();

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
        recommended_next_action,
        path_present,
        nsld_owned,
        blockers,
        first_blocker,
    }
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
) -> &'static str {
    match boundary_status {
        "ready" if host_native_output => "handoff-to-runner",
        "ready" => "materialize-host-shell-or-os-entrypoint",
        "missing" => "emit-final-executable-pipeline",
        "not-nsld-owned" | "ownership-unknown" => "run-nsld-drive-or-inspect-output-boundary",
        "unreadable" => "inspect-final-output-permissions",
        "invalid" | "blocked" => "inspect-final-output-diagnostics",
        _ => "inspect-final-output-boundary",
    }
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

pub(crate) fn load_link_plan_for_output_dir(output_dir: &Path) -> Option<nuisc::linker::LinkPlan> {
    let manifest = output_dir.join("nuis.build.manifest.toml");
    if !manifest.exists() {
        return None;
    }
    nuisc::linker::build_link_plan_from_manifest(&manifest).ok()
}

fn workflow_link_plan_domain_unit_record(unit: &nuisc::linker::LinkPlanDomainUnit) -> String {
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", &unit.kind),
            json_field("package_id", &unit.package_id),
            json_field("domain_family", &unit.domain_family),
            json_field("contract_family", &unit.contract_family),
            json_field("packaging_role", &unit.packaging_role),
        ],
    );
    if let Some(value) = unit.abi.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("abi", value)]);
    }
    if let Some(value) = unit.backend_family.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("backend_family", value)]);
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        append_json_field_strings(
            &mut out,
            vec![json_field("selected_lowering_target", value)],
        );
    }
    if let Some(value) = unit.machine_arch.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_arch", value)]);
    }
    if let Some(value) = unit.machine_os.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_os", value)]);
    }
    out.push('}');
    out
}

fn workflow_domain_readiness_summary(
    plan: &nuisc::linker::LinkPlan,
) -> WorkflowDomainReadinessSummary {
    let units = plan
        .domain_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .map(workflow_domain_readiness)
        .collect::<Vec<_>>();
    let hetero_units = units.len();
    let ready_units = units.iter().filter(|unit| unit.ready).count();
    let domain_families = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let first_unready = units
        .iter()
        .find(|unit| !unit.ready)
        .map(|unit| format!("{}[{}]", unit.package_id, unit.domain_family));
    WorkflowDomainReadinessSummary {
        hetero_units,
        ready_units,
        ready: hetero_units == ready_units,
        domain_families,
        first_unready,
        units,
    }
}

fn workflow_domain_readiness(unit: &nuisc::linker::LinkPlanDomainUnit) -> WorkflowDomainReadiness {
    let selected_lowering_target_present = unit.selected_lowering_target.is_some();
    let payload_blob_present = unit.artifact_payload_blob_path.is_some();
    let payload_format_present = unit.artifact_payload_format.is_some();
    let bridge_stub_present = unit.artifact_bridge_stub_path.is_some();
    let ir_sidecar_present = unit.artifact_ir_sidecar_path.is_some();
    let mut issues = Vec::new();
    if !payload_blob_present {
        issues.push("payload_blob_missing".to_owned());
    }
    if !payload_format_present {
        issues.push("payload_format_missing".to_owned());
    }
    if !bridge_stub_present {
        issues.push("bridge_stub_missing".to_owned());
    }
    WorkflowDomainReadiness {
        package_id: unit.package_id.clone(),
        domain_family: unit.domain_family.clone(),
        ready: issues.is_empty(),
        selected_lowering_target_present,
        payload_blob_present,
        payload_format_present,
        bridge_stub_present,
        ir_sidecar_present,
        issues,
    }
}

fn workflow_domain_readiness_units_json(summary: &WorkflowDomainReadinessSummary) -> Vec<String> {
    summary
        .units
        .iter()
        .map(workflow_domain_readiness_json)
        .collect()
}

fn workflow_domain_readiness_json(unit: &WorkflowDomainReadiness) -> String {
    let fields = [
        json_field("package_id", &unit.package_id),
        json_field("domain_family", &unit.domain_family),
        json_bool_field("ready", unit.ready),
        json_bool_field(
            "selected_lowering_target_present",
            unit.selected_lowering_target_present,
        ),
        json_bool_field("payload_blob_present", unit.payload_blob_present),
        json_bool_field("payload_format_present", unit.payload_format_present),
        json_bool_field("bridge_stub_present", unit.bridge_stub_present),
        json_bool_field("ir_sidecar_present", unit.ir_sidecar_present),
        json_string_array_field("issues", &unit.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn workflow_link_plan_json_fields(link_plan: Option<&nuisc::linker::LinkPlan>) -> Vec<String> {
    let domain_unit_records = link_plan
        .map(|plan| {
            plan.domain_units
                .iter()
                .map(workflow_link_plan_domain_unit_record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let nsld_chain =
        link_plan.map(|plan| nsld_prepared_artifact_chain_summary(Path::new(&plan.output_dir)));
    let nsld_tail =
        link_plan.map(|plan| nsld_final_executable_tail_summary(Path::new(&plan.output_dir)));
    let prepared_stage_records = link_plan
        .map(|plan| nsld_prepared_artifact_stage_records_json(Path::new(&plan.output_dir)))
        .unwrap_or_default();
    let final_tail_stage_records = link_plan
        .map(|plan| nsld_final_executable_tail_stage_records_json(Path::new(&plan.output_dir)))
        .unwrap_or_default();
    let nsld_final_output = link_plan.map(nsld_final_executable_output_boundary_summary);
    let nsld_next = nsld_next_action_summary(
        nsld_chain.as_ref(),
        nsld_tail.as_ref(),
        nsld_final_output.as_ref(),
    );
    let nsld_chain_next =
        nsld_artifact_chain_next_action_mirror(nsld_chain.as_ref(), nsld_tail.as_ref());
    let nsld_drive_recommendation = nsld_drive_recommendation_for_output_dir(
        link_plan.map(|plan| Path::new(&plan.output_dir)),
        &nsld_chain_next,
        nsld_final_output.as_ref(),
    );
    let nsld_drive_command_set =
        link_plan.map(|plan| nsld_drive_command_set_for_output_dir(Path::new(&plan.output_dir)));
    let domain_readiness = link_plan.map(workflow_domain_readiness_summary);
    vec![
        json_bool_field("link_plan_available", link_plan.is_some()),
        json_optional_string_field(
            "link_plan_final_stage",
            link_plan.map(|plan| plan.final_stage.kind.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_driver",
            link_plan.map(|plan| plan.final_stage.driver.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_link_mode",
            link_plan.map(|plan| plan.final_stage.link_mode.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_output",
            link_plan.map(|plan| plan.final_stage.output_path.as_str()),
        ),
        json_optional_string_field(
            "link_plan_lowering_plan_index_path",
            link_plan.and_then(|plan| plan.lowering_plan_index_path.as_deref()),
        ),
        json_optional_string_field(
            "link_plan_lowering_plan_index_source",
            link_plan.map(|plan| plan.lowering_plan_index_source.as_str()),
        ),
        json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
        json_usize_field(
            "link_plan_heterogeneous_domain_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.hetero_units)
                .unwrap_or(0),
        ),
        json_usize_field(
            "link_plan_heterogeneous_domain_ready_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.ready_units)
                .unwrap_or(0),
        ),
        json_bool_field(
            "link_plan_heterogeneous_domain_readiness_ready",
            domain_readiness
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        json_string_array_field(
            "link_plan_heterogeneous_domain_families",
            &domain_readiness
                .as_ref()
                .map(|summary| summary.domain_families.clone())
                .unwrap_or_default(),
        ),
        json_optional_string_field(
            "link_plan_heterogeneous_domain_first_unready",
            domain_readiness
                .as_ref()
                .and_then(|summary| summary.first_unready.as_deref()),
        ),
        json_object_array_field(
            "link_plan_heterogeneous_domain_readiness",
            &domain_readiness
                .as_ref()
                .map(workflow_domain_readiness_units_json)
                .unwrap_or_default(),
        ),
        json_object_array_field("link_plan_domain_unit_records", &domain_unit_records),
        json_optional_string_field(
            "nsld_prepare_command",
            nsld_chain
                .as_ref()
                .map(|summary| summary.prepare_command.as_str()),
        ),
        json_optional_string_field(
            "nsld_drive_dry_run_command",
            link_plan
                .map(|plan| nsld_drive_dry_run_command_for_output_dir(Path::new(&plan.output_dir)))
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_dry_run_json_command",
            link_plan
                .map(|plan| {
                    nsld_drive_dry_run_json_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_next_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_next_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_next_json_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_next_json_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_until_clean_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_until_clean_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_until_clean_json_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_until_clean_json_command_for_output_dir(Path::new(
                        &plan.output_dir,
                    ))
                })
                .as_deref(),
        ),
        nsld_drive_command_set_json_field(
            "nsld_drive_command_set",
            nsld_drive_command_set.as_ref(),
        ),
        json_bool_field(
            "nsld_drive_recommended_available",
            nsld_drive_recommendation.available,
        ),
        json_field(
            "nsld_drive_recommended_mode",
            &nsld_drive_recommendation.mode,
        ),
        json_optional_string_field(
            "nsld_drive_recommended_command",
            nsld_drive_recommendation.command.as_deref(),
        ),
        json_bool_field(
            "nsld_drive_recommended_mutates_artifacts",
            nsld_drive_recommendation.mutates_artifacts,
        ),
        json_field(
            "nsld_drive_recommended_reason",
            &nsld_drive_recommendation.reason,
        ),
        json_bool_field(
            "nsld_prepared_artifact_chain_ready",
            nsld_chain.as_ref().is_some_and(|summary| summary.ready),
        ),
        json_usize_field(
            "nsld_prepared_artifact_stage_count",
            nsld_chain
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_prepared_artifact_present_count",
            nsld_chain
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_prepared_artifact_next_missing_stage",
            nsld_chain
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        json_object_array_field(
            "nsld_prepared_artifact_stage_records",
            &prepared_stage_records,
        ),
        json_field("nsld_next_action_source", &nsld_next.source),
        json_field("nsld_next_action", &nsld_next.action),
        json_optional_string_field("nsld_next_action_command", nsld_next.command.as_deref()),
        json_field("nsld_next_action_reason", &nsld_next.reason),
        json_bool_field(
            "nsld_artifact_chain_next_action_available",
            nsld_chain_next.available,
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_source",
            nsld_chain_next.source.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_command_id",
            nsld_chain_next.command_id.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_command",
            nsld_chain_next.command.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_command_resolved",
            nsld_chain_next.command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_reason",
            nsld_chain_next.reason.as_deref(),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_command",
            nsld_tail
                .as_ref()
                .map(|summary| summary.pipeline_command.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_tail_ready",
            nsld_tail.as_ref().is_some_and(|summary| summary.ready),
        ),
        json_usize_field(
            "nsld_final_executable_tail_stage_count",
            nsld_tail
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_tail_present_count",
            nsld_tail
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_final_executable_tail_next_missing_stage",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        json_object_array_field(
            "nsld_final_executable_tail_stage_records",
            &final_tail_stage_records,
        ),
        match nsld_tail
            .as_ref()
            .and_then(|summary| summary.pipeline_valid)
        {
            Some(valid) => json_bool_field("nsld_final_executable_pipeline_valid", valid),
            None => "\"nsld_final_executable_pipeline_valid\":null".to_owned(),
        },
        json_optional_bool_field(
            "nsld_final_executable_pipeline_final_executable_emitted",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.final_executable_emitted),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_manifest_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.launcher_manifest_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_dry_run_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_would_enter_lifecycle_hook",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.would_enter_lifecycle_hook),
        ),
        match nsld_tail.as_ref().and_then(|summary| summary.blocker_count) {
            Some(count) => json_usize_field("nsld_final_executable_pipeline_blocker_count", count),
            None => "\"nsld_final_executable_pipeline_blocker_count\":null".to_owned(),
        },
        json_optional_string_field(
            "nsld_final_executable_pipeline_first_blocker",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_contract",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_contract.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_execution_handoff_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_status",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_status.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_target",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_target.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_evidence_status.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_decision_code.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_payload_id",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_payload_id.as_deref()),
        ),
        match nsld_tail
            .as_ref()
            .and_then(|summary| summary.scheduler_metadata_present)
        {
            Some(present) => json_bool_field(
                "nsld_final_executable_pipeline_scheduler_metadata_present",
                present,
            ),
            None => "\"nsld_final_executable_pipeline_scheduler_metadata_present\":null".to_owned(),
        },
        json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_hash",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_hash.as_deref()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_count",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.required_stage_path_count),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_present_count",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.required_stage_path_present_count),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_first_missing_required_stage_path",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.first_missing_required_stage_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_ready),
        ),
        json_optional_string_field(
            "nsld_self_owned_image_status",
            nsld_tail
                .as_ref()
                .map(|summary| summary.self_owned_image_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_entrypoint_materialization_status",
            nsld_tail
                .as_ref()
                .map(|summary| summary.entrypoint_materialization_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_self_owned_image_path",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_present",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_present),
        ),
        json_optional_string_field(
            "nsld_self_owned_image_hash",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_hash.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_header_valid",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_header_valid),
        ),
        json_bool_field(
            "nsld_final_executable_output_ready",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_boundary_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.boundary_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_materialization_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.materialization_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_contract",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_contract.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_execution_handoff_ready",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.execution_handoff_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_target",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_target.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_evidence_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_evidence_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_first_blocker",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_decision_code",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_decision_code.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_recommended_next_action",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.recommended_next_action.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_path_present",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.path_present),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_nsld_owned",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.nsld_owned),
        ),
        json_usize_field(
            "nsld_final_executable_output_blocker_count",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.blockers.len())
                .unwrap_or(0),
        ),
        json_string_array_field(
            "nsld_final_executable_output_blockers",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.blockers.as_slice())
                .unwrap_or(&[]),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_first_blocker",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
    ]
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn parse_bool_field(source: &str, key: &str) -> Option<bool> {
    parse_scalar_field(source, key).and_then(|value| match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

fn parse_usize_field(source: &str, key: &str) -> Option<usize> {
    parse_scalar_field(source, key).and_then(|value| value.trim().parse().ok())
}

fn parse_string_field(source: &str, key: &str) -> Option<String> {
    parse_scalar_field(source, key)
        .and_then(|value| value.trim().strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
}

fn parse_first_string_array_item(source: &str, key: &str) -> Option<String> {
    let value = parse_scalar_field(source, key)?;
    let value = value.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    if value.is_empty() {
        return None;
    }
    value
        .split(',')
        .next()
        .map(str::trim)
        .and_then(|item| item.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
}

fn parse_scalar_field<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    source.lines().find_map(|line| {
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim())
    })
}

fn compile_pipeline_stage_json(stage: &nuisc::pipeline::CompilePipelineStage) -> String {
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("id", stage.id),
            json_field("status", stage.status),
            json_field("detail", &stage.detail),
        ],
    );
    out.push('}');
    out
}

pub(super) fn workflow_compile_pipeline_json_fields(input: &Path) -> Vec<String> {
    match nuisc::pipeline::resolve_compile_input(input).and_then(|resolved| {
        let artifacts = resolved.compile()?;
        Ok(resolved.compile_report(&artifacts))
    }) {
        Ok(report) => {
            let stage_records = report
                .stages
                .iter()
                .map(compile_pipeline_stage_json)
                .collect::<Vec<_>>();
            vec![
                json_bool_field("compile_pipeline_available", true),
                json_field("compile_pipeline_source_kind", report.source_kind),
                json_field("compile_pipeline_input", &report.input_path),
                json_field(
                    "compile_pipeline_effective_input",
                    &report.effective_input_path,
                ),
                json_optional_string_field(
                    "compile_pipeline_project",
                    report.project_name.as_deref(),
                ),
                json_field("compile_pipeline_domain", &report.domain),
                json_field("compile_pipeline_unit", &report.unit),
                json_usize_field("compile_pipeline_stage_count", report.stage_count()),
                json_usize_field("compile_pipeline_ok_stage_count", report.ok_stage_count()),
                json_usize_field("compile_pipeline_ast_functions", report.ast_functions),
                json_usize_field("compile_pipeline_nir_functions", report.nir_functions),
                json_usize_field("compile_pipeline_yir_nodes", report.yir_nodes),
                json_usize_field("compile_pipeline_yir_resources", report.yir_resources),
                json_usize_field("compile_pipeline_yir_edges", report.yir_edges),
                json_usize_field("compile_pipeline_llvm_ir_bytes", report.llvm_ir_bytes),
                json_usize_field(
                    "compile_pipeline_loaded_nustar_count",
                    report.loaded_nustar.len(),
                ),
                json_string_array_field("compile_pipeline_loaded_nustar", &report.loaded_nustar),
                json_object_array_field("compile_pipeline_stages", &stage_records),
                json_bool_field("compile_pipeline_ready_for_aot", report.ready_for_aot),
                json_field(
                    "compile_pipeline_recommended_next_step",
                    report.recommended_next_step,
                ),
                json_field(
                    "compile_pipeline_recommended_reason",
                    &report.recommended_reason,
                ),
                json_field("compile_pipeline_summary", &report.summary_line()),
            ]
        }
        Err(error) => vec![
            json_bool_field("compile_pipeline_available", false),
            json_field("compile_pipeline_error", &error),
        ],
    }
}

pub(crate) fn append_workflow_link_plan_json_fields(
    out: &mut String,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    append_json_field_strings(out, workflow_link_plan_json_fields(link_plan));
}
