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
    !matches!(kind, NsldArtifactStageKind::ObjectOutput)
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
