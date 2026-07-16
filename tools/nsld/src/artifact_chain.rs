use super::{
    artifact_chain_actions::nsld_artifact_chain_action_plan,
    final_executable_output::nsld_final_executable_output_report,
    reports::{NsldArtifactChainReport, NsldArtifactStageDiagnostic},
};
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "main_artifact_chain_next_action_tests.rs"]
mod next_action_tests;
#[cfg(test)]
#[path = "main_artifact_chain_tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NsldArtifactStageKind {
    LinkInputs,
    LinkUnits,
    LinkBundle,
    AssemblePlan,
    SectionManifest,
    ObjectPlan,
    ObjectWriterInput,
    ObjectByteLayout,
    ObjectFileLayout,
    ObjectImageDryRun,
    ObjectImageDryRunBytes,
    ObjectEmitBlocked,
    ObjectOutput,
    ObjectWriterDryRun,
    ContainerPlan,
    Container,
    ContainerPayload,
    ClosureSnapshot,
    FinalStagePlan,
    FinalExecutableWriterInput,
    FinalExecutableHostInvokePlan,
    FinalExecutableLayoutPlan,
    FinalExecutableImageDryRun,
    FinalExecutableImageDryRunBytes,
    FinalExecutableBlocked,
    FinalExecutableOutput,
    FinalExecutableLauncherManifest,
    FinalExecutableLauncherDryRun,
    FinalExecutablePipeline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldArtifactStage {
    pub(crate) kind: NsldArtifactStageKind,
    pub(crate) file_name: String,
    pub(crate) present: bool,
    pub(crate) required: bool,
}

const ARTIFACT_STAGE_DEFINITIONS: &[(NsldArtifactStageKind, &str)] = &[
    (
        NsldArtifactStageKind::LinkInputs,
        "nuis.nsld.link-inputs.toml",
    ),
    (
        NsldArtifactStageKind::LinkUnits,
        "nuis.nsld.link-units.toml",
    ),
    (
        NsldArtifactStageKind::LinkBundle,
        "nuis.nsld.link-bundle.toml",
    ),
    (
        NsldArtifactStageKind::AssemblePlan,
        "nuis.nsld.assemble-plan.toml",
    ),
    (
        NsldArtifactStageKind::SectionManifest,
        "nuis.nsld.section-manifest.toml",
    ),
    (
        NsldArtifactStageKind::ObjectPlan,
        "nuis.nsld.object-plan.toml",
    ),
    (
        NsldArtifactStageKind::ObjectWriterInput,
        "nuis.nsld.object-writer-input.toml",
    ),
    (
        NsldArtifactStageKind::ObjectByteLayout,
        "nuis.nsld.object-byte-layout.toml",
    ),
    (
        NsldArtifactStageKind::ObjectFileLayout,
        "nuis.nsld.object-file-layout.toml",
    ),
    (
        NsldArtifactStageKind::ObjectImageDryRun,
        "nuis.nsld.object-image-dry-run.toml",
    ),
    (
        NsldArtifactStageKind::ObjectImageDryRunBytes,
        "nuis.nsld.object-image-dry-run.bin",
    ),
    (
        NsldArtifactStageKind::ObjectEmitBlocked,
        "nuis.nsld.object.blocked.toml",
    ),
    (NsldArtifactStageKind::ObjectOutput, "nuis.nsld.mach-o"),
    (
        NsldArtifactStageKind::ObjectWriterDryRun,
        "nuis.nsld.object-writer-dry-run.toml",
    ),
    (
        NsldArtifactStageKind::ContainerPlan,
        "nuis.nsld.container-plan.toml",
    ),
    (NsldArtifactStageKind::Container, "nuis.nsld.container"),
    (
        NsldArtifactStageKind::ContainerPayload,
        "nuis.nsld.container.payload",
    ),
    (
        NsldArtifactStageKind::ClosureSnapshot,
        "nuis.nsld.closure.toml",
    ),
    (
        NsldArtifactStageKind::FinalStagePlan,
        "nuis.nsld.final-stage-plan.toml",
    ),
    (
        NsldArtifactStageKind::FinalExecutableWriterInput,
        "nuis.nsld.final-executable-writer-input.toml",
    ),
    (
        NsldArtifactStageKind::FinalExecutableHostInvokePlan,
        "nuis.nsld.final-executable-host-invoke-plan.toml",
    ),
    (
        NsldArtifactStageKind::FinalExecutableLayoutPlan,
        "nuis.nsld.final-executable-layout.toml",
    ),
    (
        NsldArtifactStageKind::FinalExecutableImageDryRun,
        "nuis.nsld.final-executable-image-dry-run.toml",
    ),
    (
        NsldArtifactStageKind::FinalExecutableImageDryRunBytes,
        "nuis.nsld.final-executable-image-dry-run.bin",
    ),
    (
        NsldArtifactStageKind::FinalExecutableBlocked,
        "nuis.nsld.final-executable.blocked.toml",
    ),
    (NsldArtifactStageKind::FinalExecutableOutput, ""),
    (
        NsldArtifactStageKind::FinalExecutableLauncherManifest,
        "nuis.nsld.final-executable-launcher.toml",
    ),
    (
        NsldArtifactStageKind::FinalExecutableLauncherDryRun,
        "nuis.nsld.final-executable-launcher-dry-run.toml",
    ),
    (
        NsldArtifactStageKind::FinalExecutablePipeline,
        "nuis.nsld.final-executable-pipeline.toml",
    ),
];

pub(crate) fn nsld_artifact_stage_path(output_dir: impl AsRef<Path>, file_name: &str) -> PathBuf {
    output_dir.as_ref().join(file_name)
}

pub(crate) fn nsld_artifact_stage_file_name(kind: NsldArtifactStageKind) -> &'static str {
    match kind {
        NsldArtifactStageKind::LinkInputs => "nuis.nsld.link-inputs.toml",
        NsldArtifactStageKind::LinkUnits => "nuis.nsld.link-units.toml",
        NsldArtifactStageKind::LinkBundle => "nuis.nsld.link-bundle.toml",
        NsldArtifactStageKind::AssemblePlan => "nuis.nsld.assemble-plan.toml",
        NsldArtifactStageKind::SectionManifest => "nuis.nsld.section-manifest.toml",
        NsldArtifactStageKind::ObjectPlan => "nuis.nsld.object-plan.toml",
        NsldArtifactStageKind::ObjectWriterInput => "nuis.nsld.object-writer-input.toml",
        NsldArtifactStageKind::ObjectByteLayout => "nuis.nsld.object-byte-layout.toml",
        NsldArtifactStageKind::ObjectFileLayout => "nuis.nsld.object-file-layout.toml",
        NsldArtifactStageKind::ObjectImageDryRun => "nuis.nsld.object-image-dry-run.toml",
        NsldArtifactStageKind::ObjectImageDryRunBytes => "nuis.nsld.object-image-dry-run.bin",
        NsldArtifactStageKind::ObjectEmitBlocked => "nuis.nsld.object.blocked.toml",
        NsldArtifactStageKind::ObjectOutput => "nuis.nsld.mach-o",
        NsldArtifactStageKind::ObjectWriterDryRun => "nuis.nsld.object-writer-dry-run.toml",
        NsldArtifactStageKind::ContainerPlan => "nuis.nsld.container-plan.toml",
        NsldArtifactStageKind::Container => "nuis.nsld.container",
        NsldArtifactStageKind::ContainerPayload => "nuis.nsld.container.payload",
        NsldArtifactStageKind::ClosureSnapshot => "nuis.nsld.closure.toml",
        NsldArtifactStageKind::FinalStagePlan => "nuis.nsld.final-stage-plan.toml",
        NsldArtifactStageKind::FinalExecutableWriterInput => {
            "nuis.nsld.final-executable-writer-input.toml"
        }
        NsldArtifactStageKind::FinalExecutableHostInvokePlan => {
            "nuis.nsld.final-executable-host-invoke-plan.toml"
        }
        NsldArtifactStageKind::FinalExecutableLayoutPlan => {
            "nuis.nsld.final-executable-layout.toml"
        }
        NsldArtifactStageKind::FinalExecutableImageDryRun => {
            "nuis.nsld.final-executable-image-dry-run.toml"
        }
        NsldArtifactStageKind::FinalExecutableImageDryRunBytes => {
            "nuis.nsld.final-executable-image-dry-run.bin"
        }
        NsldArtifactStageKind::FinalExecutableBlocked => "nuis.nsld.final-executable.blocked.toml",
        NsldArtifactStageKind::FinalExecutableOutput => "final-executable-output",
        NsldArtifactStageKind::FinalExecutableLauncherManifest => {
            "nuis.nsld.final-executable-launcher.toml"
        }
        NsldArtifactStageKind::FinalExecutableLauncherDryRun => {
            "nuis.nsld.final-executable-launcher-dry-run.toml"
        }
        NsldArtifactStageKind::FinalExecutablePipeline => {
            "nuis.nsld.final-executable-pipeline.toml"
        }
    }
}

pub(crate) fn nsld_object_output_file_name(object_format: &str) -> String {
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

pub(crate) fn nsld_artifact_stage_file_name_for_plan(
    kind: NsldArtifactStageKind,
    plan: &nuisc::linker::LinkPlan,
) -> String {
    match kind {
        NsldArtifactStageKind::ObjectOutput => {
            nsld_object_output_file_name(&plan.cpu_target.object_format)
        }
        NsldArtifactStageKind::FinalExecutableOutput => plan.final_stage.output_path.clone(),
        _ => nsld_artifact_stage_file_name(kind).to_owned(),
    }
}

pub(crate) fn nsld_artifact_stage_id(kind: NsldArtifactStageKind) -> &'static str {
    match kind {
        NsldArtifactStageKind::LinkInputs => "link-inputs",
        NsldArtifactStageKind::LinkUnits => "link-units",
        NsldArtifactStageKind::LinkBundle => "link-bundle",
        NsldArtifactStageKind::AssemblePlan => "assemble-plan",
        NsldArtifactStageKind::SectionManifest => "section-manifest",
        NsldArtifactStageKind::ObjectPlan => "object-plan",
        NsldArtifactStageKind::ObjectWriterInput => "object-writer-input",
        NsldArtifactStageKind::ObjectByteLayout => "object-byte-layout",
        NsldArtifactStageKind::ObjectFileLayout => "object-file-layout",
        NsldArtifactStageKind::ObjectImageDryRun => "object-image-dry-run",
        NsldArtifactStageKind::ObjectImageDryRunBytes => "object-image-dry-run-bytes",
        NsldArtifactStageKind::ObjectEmitBlocked => "object-emit-blocked",
        NsldArtifactStageKind::ObjectOutput => "object-output",
        NsldArtifactStageKind::ObjectWriterDryRun => "object-writer-dry-run",
        NsldArtifactStageKind::ContainerPlan => "container-plan",
        NsldArtifactStageKind::Container => "container",
        NsldArtifactStageKind::ContainerPayload => "container-payload",
        NsldArtifactStageKind::ClosureSnapshot => "closure-snapshot",
        NsldArtifactStageKind::FinalStagePlan => "final-stage-plan",
        NsldArtifactStageKind::FinalExecutableWriterInput => "final-executable-writer-input",
        NsldArtifactStageKind::FinalExecutableHostInvokePlan => "final-executable-host-invoke-plan",
        NsldArtifactStageKind::FinalExecutableLayoutPlan => "final-executable-layout",
        NsldArtifactStageKind::FinalExecutableImageDryRun => "final-executable-image-dry-run",
        NsldArtifactStageKind::FinalExecutableImageDryRunBytes => {
            "final-executable-image-dry-run-bytes"
        }
        NsldArtifactStageKind::FinalExecutableBlocked => "final-executable-blocked",
        NsldArtifactStageKind::FinalExecutableOutput => "final-executable-output",
        NsldArtifactStageKind::FinalExecutableLauncherManifest => "final-executable-launcher",
        NsldArtifactStageKind::FinalExecutableLauncherDryRun => "final-executable-launcher-dry-run",
        NsldArtifactStageKind::FinalExecutablePipeline => "final-executable-pipeline",
    }
}

pub(crate) fn nsld_artifact_stage_kind_path(
    output_dir: impl AsRef<Path>,
    kind: NsldArtifactStageKind,
) -> PathBuf {
    nsld_artifact_stage_path(output_dir, nsld_artifact_stage_file_name(kind))
}

pub(crate) fn nsld_artifact_stage_kind_path_for_plan(
    plan: &nuisc::linker::LinkPlan,
    kind: NsldArtifactStageKind,
) -> PathBuf {
    nsld_artifact_stage_path_for_plan(plan, &nsld_artifact_stage_file_name_for_plan(kind, plan))
}

fn nsld_artifact_stage_path_for_plan(plan: &nuisc::linker::LinkPlan, file_name: &str) -> PathBuf {
    if file_name == plan.final_stage.output_path {
        PathBuf::from(file_name)
    } else {
        nsld_artifact_stage_path(&plan.output_dir, file_name)
    }
}

pub(crate) fn nsld_artifact_stages_for_plan(
    plan: &nuisc::linker::LinkPlan,
) -> Vec<NsldArtifactStage> {
    nsld_artifact_stages_for_plan_with_final_output_present(plan, None)
}

fn nsld_artifact_stages_for_plan_with_final_output_present(
    plan: &nuisc::linker::LinkPlan,
    final_output_present: Option<bool>,
) -> Vec<NsldArtifactStage> {
    let mut stages = Vec::with_capacity(ARTIFACT_STAGE_DEFINITIONS.len());
    for (kind, _) in ARTIFACT_STAGE_DEFINITIONS {
        let file_name = nsld_artifact_stage_file_name_for_plan(*kind, plan);
        let present = if *kind == NsldArtifactStageKind::FinalExecutableOutput {
            final_output_present
                .unwrap_or_else(|| nsld_artifact_stage_path_for_plan(plan, &file_name).exists())
        } else {
            nsld_artifact_stage_path_for_plan(plan, &file_name).exists()
        };
        stages.push(NsldArtifactStage {
            kind: *kind,
            present,
            file_name,
            required: nsld_artifact_stage_required(*kind),
        });
    }
    stages
}

pub(crate) fn nsld_artifact_stage_required(kind: NsldArtifactStageKind) -> bool {
    !matches!(
        kind,
        NsldArtifactStageKind::ObjectOutput
            | NsldArtifactStageKind::ClosureSnapshot
            | NsldArtifactStageKind::FinalStagePlan
            | NsldArtifactStageKind::FinalExecutableWriterInput
            | NsldArtifactStageKind::FinalExecutableHostInvokePlan
            | NsldArtifactStageKind::FinalExecutableLayoutPlan
            | NsldArtifactStageKind::FinalExecutableImageDryRun
            | NsldArtifactStageKind::FinalExecutableImageDryRunBytes
            | NsldArtifactStageKind::FinalExecutableBlocked
            | NsldArtifactStageKind::FinalExecutableOutput
            | NsldArtifactStageKind::FinalExecutableLauncherManifest
            | NsldArtifactStageKind::FinalExecutableLauncherDryRun
            | NsldArtifactStageKind::FinalExecutablePipeline
    )
}

pub(crate) fn nsld_artifact_stage_present(
    stages: &[NsldArtifactStage],
    kind: NsldArtifactStageKind,
) -> bool {
    stages
        .iter()
        .find(|stage| stage.kind == kind)
        .is_some_and(|stage| stage.present)
}

pub(crate) fn nsld_artifact_chain_issues(stages: &[NsldArtifactStage]) -> Vec<String> {
    let mut first_missing_before_present = None;
    let mut issues = Vec::new();

    for stage in stages {
        if stage.present {
            if let Some(missing) = first_missing_before_present.as_ref() {
                issues.push(format!(
                    "artifact `{}` is present but prerequisite `{missing}` is missing",
                    stage.file_name
                ));
            }
        } else if stage.required && first_missing_before_present.is_none() {
            first_missing_before_present = Some(stage.file_name.clone());
        }
    }

    issues
}

pub(crate) fn nsld_artifact_chain_advisories(stages: &[NsldArtifactStage]) -> Vec<String> {
    final_executable_tail_advisories(stages)
}

fn final_executable_tail_advisories(stages: &[NsldArtifactStage]) -> Vec<String> {
    let tail = [
        NsldArtifactStageKind::FinalExecutableWriterInput,
        NsldArtifactStageKind::FinalExecutableHostInvokePlan,
        NsldArtifactStageKind::FinalExecutableLayoutPlan,
        NsldArtifactStageKind::FinalExecutableImageDryRun,
        NsldArtifactStageKind::FinalExecutableBlocked,
        NsldArtifactStageKind::FinalExecutableLauncherManifest,
        NsldArtifactStageKind::FinalExecutableLauncherDryRun,
        NsldArtifactStageKind::FinalExecutablePipeline,
    ];
    let mut first_missing = None;
    let mut advisories = Vec::new();

    for kind in tail {
        let Some(stage) = stages.iter().find(|stage| stage.kind == kind) else {
            continue;
        };
        if stage.present {
            if let Some(missing) = first_missing.as_ref() {
                advisories.push(format!(
                    "optional final executable tail artifact `{}` is present but earlier tail artifact `{missing}` is missing; run `emit-final-executable-pipeline` to rebuild the tail",
                    nsld_artifact_stage_id(kind)
                ));
            }
        } else if first_missing.is_none() {
            first_missing = Some(nsld_artifact_stage_id(kind).to_owned());
        }
    }

    advisories
}

pub(crate) fn nsld_artifact_chain_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldArtifactChainReport {
    let final_output = nsld_final_executable_output_report(manifest, plan);
    let stages =
        nsld_artifact_stages_for_plan_with_final_output_present(plan, Some(final_output.present));
    let issues = nsld_artifact_chain_issues(&stages);
    let advisories = nsld_artifact_chain_advisories(&stages);
    let diagnostics = stages
        .iter()
        .enumerate()
        .map(|(order_index, stage)| NsldArtifactStageDiagnostic {
            order_index,
            stage_id: nsld_artifact_stage_id(stage.kind).to_owned(),
            file_name: stage.file_name.clone(),
            path: nsld_artifact_stage_path_for_plan(plan, &stage.file_name)
                .display()
                .to_string(),
            required: stage.required,
            present: stage.present,
        })
        .collect::<Vec<_>>();
    let stage_count = diagnostics.len();
    let present_count = diagnostics.iter().filter(|stage| stage.present).count();
    let required_count = diagnostics.iter().filter(|stage| stage.required).count();
    let missing_required_count = diagnostics
        .iter()
        .filter(|stage| stage.required && !stage.present)
        .count();
    let optional_present_count = diagnostics
        .iter()
        .filter(|stage| !stage.required && stage.present)
        .count();
    let first_missing_required = stages.iter().find(|stage| stage.required && !stage.present);
    let first_missing_required_stage =
        first_missing_required.map(|stage| nsld_artifact_stage_id(stage.kind).to_owned());
    let action_plan = nsld_artifact_chain_action_plan(
        manifest,
        &stages,
        first_missing_required_stage.clone(),
        &advisories,
        suppressed_optional_stage(&stages, &final_output),
    );
    let final_output_boundary_ready =
        final_output.runnable_candidate && final_output.blockers.is_empty();
    let final_output_boundary = final_output_boundary_action(
        manifest,
        plan,
        final_output_boundary_ready,
        &final_output.blockers,
    );

    NsldArtifactChainReport {
        manifest: manifest.display().to_string(),
        output_dir: plan.output_dir.clone(),
        valid: issues.is_empty(),
        stage_count,
        present_count,
        required_count,
        missing_required_count,
        optional_present_count,
        first_missing_required_stage,
        next_required_stage: action_plan.next_required_stage,
        suggested_command_id: action_plan.suggested_command_id,
        suggested_command: action_plan.suggested_command,
        suggested_command_resolved: action_plan.suggested_command_resolved,
        suggested_command_reason: action_plan.suggested_command_reason,
        next_optional_stage: action_plan.next_optional_stage,
        next_optional_command_id: action_plan.next_optional_command_id,
        next_optional_command: action_plan.next_optional_command,
        next_optional_command_resolved: action_plan.next_optional_command_resolved,
        next_optional_command_reason: action_plan.next_optional_command_reason,
        advisory_command_id: action_plan.advisory_command_id,
        advisory_command: action_plan.advisory_command,
        advisory_command_resolved: action_plan.advisory_command_resolved,
        advisory_command_reason: action_plan.advisory_command_reason,
        next_action_command_id: action_plan.next_action_command_id,
        next_action_command: action_plan.next_action_command,
        next_action_command_resolved: action_plan.next_action_command_resolved,
        next_action_command_reason: action_plan.next_action_command_reason,
        next_action_source: action_plan.next_action_source,
        next_action_available: action_plan.next_action_available,
        final_output_boundary_ready,
        final_output_boundary_command_id: final_output_boundary.command_id,
        final_output_boundary_command: final_output_boundary.command,
        final_output_boundary_command_resolved: final_output_boundary.command_resolved,
        final_output_boundary_reason: final_output_boundary.reason,
        final_output_boundary_blockers: final_output.blockers.clone(),
        stages: diagnostics,
        advisories,
        issues,
    }
}

struct FinalOutputBoundaryAction {
    command_id: Option<String>,
    command: Option<String>,
    command_resolved: Option<String>,
    reason: Option<String>,
}

fn final_output_boundary_action(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    ready: bool,
    blockers: &[String],
) -> FinalOutputBoundaryAction {
    if ready {
        return FinalOutputBoundaryAction {
            command_id: None,
            command: None,
            command_resolved: None,
            reason: None,
        };
    }
    if let Some(blocker) = blockers
        .iter()
        .find(|blocker| blocker.starts_with("device-provider-sample:"))
    {
        return FinalOutputBoundaryAction {
            command_id: Some("materialize-provider-samples".to_owned()),
            command: Some("nsdb materialize-provider-samples <artifact-output-dir> --json".to_owned()),
            command_resolved: Some(format!(
                "nsdb materialize-provider-samples {} --json",
                plan.output_dir
            )),
            reason: Some(format!(
                "final executable output boundary is blocked by `{blocker}`; materialize provider samples before relinking"
            )),
        };
    }
    let blocker = blockers
        .iter()
        .find(|blocker| blocker.starts_with("final-executable-output:"))
        .or_else(|| blockers.first());
    FinalOutputBoundaryAction {
        command_id: Some("final-executable-output".to_owned()),
        command: Some("nsld final-executable-output <input>".to_owned()),
        command_resolved: Some(format!(
            "nsld final-executable-output {}",
            manifest.display()
        )),
        reason: Some(
            blocker
                .map(|blocker| {
                    format!("final executable output boundary is blocked by `{blocker}`")
                })
                .unwrap_or_else(|| "final executable output boundary is not ready".to_owned()),
        ),
    }
}

fn suppressed_optional_stage(
    stages: &[NsldArtifactStage],
    final_output: &super::reports::NsldFinalExecutableOutputReport,
) -> Option<NsldArtifactStageKind> {
    let final_pipeline_present =
        nsld_artifact_stage_present(stages, NsldArtifactStageKind::FinalExecutablePipeline);
    let host_native_boundary = final_output.output_kind == "host-native-executable"
        && !final_output.nsld_owned_output
        && final_pipeline_present;

    host_native_boundary.then_some(NsldArtifactStageKind::FinalExecutableOutput)
}
