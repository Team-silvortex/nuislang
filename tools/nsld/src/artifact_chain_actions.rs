use super::artifact_chain::{nsld_artifact_stage_id, NsldArtifactStage, NsldArtifactStageKind};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldArtifactChainActionPlan {
    pub(crate) next_required_stage: Option<String>,
    pub(crate) suggested_command_id: Option<String>,
    pub(crate) suggested_command: Option<String>,
    pub(crate) suggested_command_resolved: Option<String>,
    pub(crate) suggested_command_reason: Option<String>,
    pub(crate) next_optional_stage: Option<String>,
    pub(crate) next_optional_command_id: Option<String>,
    pub(crate) next_optional_command: Option<String>,
    pub(crate) next_optional_command_resolved: Option<String>,
    pub(crate) next_optional_command_reason: Option<String>,
    pub(crate) advisory_command_id: Option<String>,
    pub(crate) advisory_command: Option<String>,
    pub(crate) advisory_command_resolved: Option<String>,
    pub(crate) advisory_command_reason: Option<String>,
    pub(crate) next_action_command_id: Option<String>,
    pub(crate) next_action_command: Option<String>,
    pub(crate) next_action_command_resolved: Option<String>,
    pub(crate) next_action_command_reason: Option<String>,
    pub(crate) next_action_source: Option<String>,
    pub(crate) next_action_available: bool,
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
        NsldArtifactStageKind::FinalStagePlan
        | NsldArtifactStageKind::FinalExecutableWriterInput
        | NsldArtifactStageKind::FinalExecutableHostInvokePlan
        | NsldArtifactStageKind::FinalExecutableLayoutPlan
        | NsldArtifactStageKind::FinalExecutableBlocked
        | NsldArtifactStageKind::FinalExecutableOutput
        | NsldArtifactStageKind::FinalExecutableLauncherManifest
        | NsldArtifactStageKind::FinalExecutableLauncherDryRun
        | NsldArtifactStageKind::FinalExecutablePipeline => "emit-final-executable-pipeline",
        NsldArtifactStageKind::FinalExecutableImageDryRun
        | NsldArtifactStageKind::FinalExecutableImageDryRunBytes => {
            "emit-final-executable-pipeline"
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

pub(crate) fn nsld_artifact_chain_action_plan(
    manifest: &Path,
    stages: &[NsldArtifactStage],
    first_missing_required_stage: Option<String>,
    advisories: &[String],
) -> NsldArtifactChainActionPlan {
    let first_missing_required = stages.iter().find(|stage| stage.required && !stage.present);
    let next_required_stage = first_missing_required_stage.clone();
    let suggested_command_id = first_missing_required
        .map(|stage| nsld_artifact_stage_suggested_command(stage.kind).to_owned());
    let suggested_command = first_missing_required
        .map(|stage| nsld_artifact_stage_suggested_command_template(stage.kind));
    let suggested_command_resolved = resolve_command(manifest, suggested_command.as_deref());
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
    let next_optional_command_resolved =
        resolve_command(manifest, next_optional_command.as_deref());
    let next_optional_command_reason = next_optional_stage
        .as_ref()
        .map(|stage_id| format!("first missing optional artifact stage `{stage_id}`"));
    let advisory_command_id =
        (!advisories.is_empty()).then(|| "emit-final-executable-pipeline".to_owned());
    let advisory_command = advisory_command_id
        .as_ref()
        .map(|command| format!("nsld {command} <input>"));
    let advisory_command_resolved = resolve_command(manifest, advisory_command.as_deref());
    let advisory_command_reason = advisories
        .first()
        .map(|advisory| format!("first artifact-chain advisory: {advisory}"));
    let next_action_command_id = suggested_command_id
        .clone()
        .or_else(|| advisory_command_id.clone())
        .or_else(|| next_optional_command_id.clone());
    let next_action_command = suggested_command
        .clone()
        .or_else(|| advisory_command.clone())
        .or_else(|| next_optional_command.clone());
    let next_action_command_resolved = suggested_command_resolved
        .clone()
        .or_else(|| advisory_command_resolved.clone())
        .or_else(|| next_optional_command_resolved.clone());
    let next_action_command_reason = suggested_command_reason
        .clone()
        .or_else(|| advisory_command_reason.clone())
        .or_else(|| next_optional_command_reason.clone());
    let next_action_source = next_action_source(
        suggested_command_id.as_ref(),
        advisory_command_id.as_ref(),
        next_optional_command_id.as_ref(),
    );
    let next_action_available = next_action_command_id.is_some();

    NsldArtifactChainActionPlan {
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
        advisory_command_id,
        advisory_command,
        advisory_command_resolved,
        advisory_command_reason,
        next_action_command_id,
        next_action_command,
        next_action_command_resolved,
        next_action_command_reason,
        next_action_source,
        next_action_available,
    }
}

fn resolve_command(manifest: &Path, command: Option<&str>) -> Option<String> {
    command.map(|command| command.replace("<input>", &manifest.display().to_string()))
}

fn next_action_source(
    suggested_command_id: Option<&String>,
    advisory_command_id: Option<&String>,
    next_optional_command_id: Option<&String>,
) -> Option<String> {
    if suggested_command_id.is_some() {
        Some("required".to_owned())
    } else if advisory_command_id.is_some() {
        Some("advisory".to_owned())
    } else if next_optional_command_id.is_some() {
        Some("optional".to_owned())
    } else {
        None
    }
}
