use super::reports::{NsldArtifactChainReport, NsldArtifactStageDiagnostic};
use std::path::{Path, PathBuf};

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
    FinalExecutableBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldArtifactStage {
    pub(crate) kind: NsldArtifactStageKind,
    pub(crate) file_name: &'static str,
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
        NsldArtifactStageKind::FinalExecutableBlocked,
        "nuis.nsld.final-executable.blocked.toml",
    ),
];

pub(crate) fn nsld_artifact_stage_path(output_dir: impl AsRef<Path>, file_name: &str) -> PathBuf {
    output_dir.as_ref().join(file_name)
}

pub(crate) fn nsld_artifact_stage_file_name(kind: NsldArtifactStageKind) -> &'static str {
    ARTIFACT_STAGE_DEFINITIONS
        .iter()
        .find(|(stage_kind, _)| *stage_kind == kind)
        .map(|(_, file_name)| *file_name)
        .expect("nsld artifact stage kind must have a file name")
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
        NsldArtifactStageKind::FinalExecutableBlocked => "final-executable-blocked",
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
        NsldArtifactStageKind::FinalExecutableBlocked => "emit-final-executable",
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

pub(crate) fn nsld_artifact_stages(output_dir: impl AsRef<Path>) -> Vec<NsldArtifactStage> {
    ARTIFACT_STAGE_DEFINITIONS
        .iter()
        .map(|(kind, file_name)| NsldArtifactStage {
            kind: *kind,
            file_name,
            present: nsld_artifact_stage_path(&output_dir, file_name).exists(),
            required: nsld_artifact_stage_required(*kind),
        })
        .collect()
}

pub(crate) fn nsld_artifact_stage_required(kind: NsldArtifactStageKind) -> bool {
    !matches!(
        kind,
        NsldArtifactStageKind::ObjectOutput
            | NsldArtifactStageKind::ClosureSnapshot
            | NsldArtifactStageKind::FinalStagePlan
            | NsldArtifactStageKind::FinalExecutableBlocked
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
            if let Some(missing) = first_missing_before_present {
                issues.push(format!(
                    "artifact `{}` is present but prerequisite `{missing}` is missing",
                    stage.file_name
                ));
            }
        } else if stage.required && first_missing_before_present.is_none() {
            first_missing_before_present = Some(stage.file_name);
        }
    }

    issues
}

pub(crate) fn nsld_artifact_chain_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldArtifactChainReport {
    let stages = nsld_artifact_stages(&plan.output_dir);
    let issues = nsld_artifact_chain_issues(&stages);
    let diagnostics = stages
        .iter()
        .enumerate()
        .map(|(order_index, stage)| NsldArtifactStageDiagnostic {
            order_index,
            stage_id: nsld_artifact_stage_id(stage.kind).to_owned(),
            file_name: stage.file_name.to_owned(),
            path: nsld_artifact_stage_path(&plan.output_dir, stage.file_name)
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
        stages: diagnostics,
        issues,
    }
}
