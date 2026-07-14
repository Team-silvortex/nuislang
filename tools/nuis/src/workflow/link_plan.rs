use super::link_plan_tail::NSLD_FINAL_EXECUTABLE_TAIL_STAGES;
use super::*;

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

pub(crate) struct NsldPreparedArtifactChainSummary {
    pub(crate) ready: bool,
    pub(crate) stage_count: usize,
    pub(crate) present_count: usize,
    pub(crate) next_missing_stage: Option<String>,
    pub(crate) manifest_path: String,
    pub(crate) prepare_command: String,
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

pub(crate) fn load_link_plan_for_output_dir(output_dir: &Path) -> Option<nuisc::linker::LinkPlan> {
    let manifest = output_dir.join("nuis.build.manifest.toml");
    if !manifest.exists() {
        return None;
    }
    nuisc::linker::build_link_plan_from_manifest(&manifest).ok()
}

pub(super) fn parse_bool_field(source: &str, key: &str) -> Option<bool> {
    parse_scalar_field(source, key).and_then(|value| match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

pub(super) fn parse_usize_field(source: &str, key: &str) -> Option<usize> {
    parse_scalar_field(source, key).and_then(|value| value.trim().parse().ok())
}

pub(super) fn parse_string_field(source: &str, key: &str) -> Option<String> {
    parse_scalar_field(source, key)
        .and_then(|value| value.trim().strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
}

pub(super) fn parse_non_empty_string_field(source: &str, key: &str) -> Option<String> {
    parse_string_field(source, key).filter(|value| !value.is_empty())
}

pub(super) fn parse_first_string_array_item(source: &str, key: &str) -> Option<String> {
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
