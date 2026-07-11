use super::reports::{NsldArtifactChainReport, NsldArtifactStageDiagnostic};
use std::path::{Path, PathBuf};

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
    }
}

pub(crate) fn nsld_artifact_stage_suggested_command(kind: NsldArtifactStageKind) -> &'static str {
    match kind {
        NsldArtifactStageKind::LinkInputs => "emit-inputs",
        NsldArtifactStageKind::LinkUnits => "emit-units",
        NsldArtifactStageKind::LinkBundle => "emit-bundle",
        NsldArtifactStageKind::AssemblePlan => "emit-assemble-plan",
        NsldArtifactStageKind::SectionManifest => "emit-section-manifest",
        NsldArtifactStageKind::ObjectPlan => "emit-object-plan",
        NsldArtifactStageKind::ObjectWriterInput
        | NsldArtifactStageKind::ObjectImageDryRunBytes
        | NsldArtifactStageKind::ContainerPayload => "prepare",
        NsldArtifactStageKind::ObjectByteLayout => "emit-object-byte-layout",
        NsldArtifactStageKind::ObjectFileLayout => "emit-object-file-layout",
        NsldArtifactStageKind::ObjectImageDryRun => "emit-object-image-dry-run",
        NsldArtifactStageKind::ObjectEmitBlocked => "emit-object",
        NsldArtifactStageKind::ObjectOutput => "emit-object",
        NsldArtifactStageKind::ObjectWriterDryRun => "emit-object-writer-dry-run",
        NsldArtifactStageKind::ContainerPlan => "emit-container-plan",
        NsldArtifactStageKind::Container => "emit-container",
        NsldArtifactStageKind::ClosureSnapshot => "emit-closure",
        NsldArtifactStageKind::FinalStagePlan => "emit-final-stage-plan",
        NsldArtifactStageKind::FinalExecutableWriterInput => "emit-final-executable-writer-input",
        NsldArtifactStageKind::FinalExecutableHostInvokePlan => {
            "emit-final-executable-host-invoke-plan"
        }
        NsldArtifactStageKind::FinalExecutableLayoutPlan => "emit-final-executable-layout",
        NsldArtifactStageKind::FinalExecutableImageDryRun
        | NsldArtifactStageKind::FinalExecutableImageDryRunBytes => {
            "emit-final-executable-image-dry-run"
        }
        NsldArtifactStageKind::FinalExecutableBlocked
        | NsldArtifactStageKind::FinalExecutableOutput => "emit-final-executable",
        NsldArtifactStageKind::FinalExecutableLauncherManifest => {
            "emit-final-executable-launcher-manifest"
        }
    }
}

pub(crate) fn nsld_artifact_stage_suggested_command_template(
    kind: NsldArtifactStageKind,
) -> String {
    format!(
        "nsld {} <input>",
        nsld_artifact_stage_suggested_command(kind)
    )
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
    let mut stages = Vec::with_capacity(ARTIFACT_STAGE_DEFINITIONS.len());
    for (kind, _) in ARTIFACT_STAGE_DEFINITIONS {
        let file_name = nsld_artifact_stage_file_name_for_plan(*kind, plan);
        stages.push(NsldArtifactStage {
            kind: *kind,
            present: nsld_artifact_stage_path_for_plan(plan, &file_name).exists(),
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

pub(crate) fn nsld_artifact_chain_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldArtifactChainReport {
    let stages = nsld_artifact_stages_for_plan(plan);
    let issues = nsld_artifact_chain_issues(&stages);
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
    let next_required_stage = first_missing_required_stage.clone();
    let suggested_command_id = first_missing_required
        .map(|stage| nsld_artifact_stage_suggested_command(stage.kind).to_owned());
    let suggested_command = first_missing_required
        .map(|stage| nsld_artifact_stage_suggested_command_template(stage.kind));
    let suggested_command_resolved = suggested_command
        .as_ref()
        .map(|command| command.replace("<input>", &manifest.display().to_string()));
    let suggested_command_reason = first_missing_required_stage
        .as_ref()
        .map(|stage_id| format!("first missing required artifact stage `{stage_id}`"));
    let first_missing_optional = first_missing_required
        .is_none()
        .then(|| {
            stages
                .iter()
                .find(|stage| !stage.required && !stage.present)
        })
        .flatten();
    let next_optional_stage =
        first_missing_optional.map(|stage| nsld_artifact_stage_id(stage.kind).to_owned());
    let next_optional_command_id = first_missing_optional
        .map(|stage| nsld_artifact_stage_suggested_command(stage.kind).to_owned());
    let next_optional_command = first_missing_optional
        .map(|stage| nsld_artifact_stage_suggested_command_template(stage.kind));
    let next_optional_command_resolved = next_optional_command
        .as_ref()
        .map(|command| command.replace("<input>", &manifest.display().to_string()));
    let next_optional_command_reason = next_optional_stage
        .as_ref()
        .map(|stage_id| format!("first missing optional artifact stage `{stage_id}`"));

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
        next_required_stage,
        suggested_command_id,
        suggested_command,
        suggested_command_resolved,
        suggested_command_reason,
        next_optional_stage,
        next_optional_command_id,
        next_optional_command,
        next_optional_command_resolved,
        next_optional_command_reason,
        stages: diagnostics,
        issues,
    }
}
