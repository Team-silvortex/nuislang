use super::artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind};
use std::path::PathBuf;

pub(crate) fn nsld_final_stage_plan_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::FinalStagePlan)
}

pub(crate) fn nsld_final_executable_writer_input_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableWriterInput,
    )
}

pub(crate) fn nsld_final_executable_blocked_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableBlocked,
    )
}

pub(crate) fn nsld_final_executable_host_invoke_plan_path(
    plan: &nuisc::linker::LinkPlan,
) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableHostInvokePlan,
    )
}

pub(crate) fn nsld_final_executable_layout_plan_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableLayoutPlan,
    )
}

pub(crate) fn nsld_final_executable_image_dry_run_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableImageDryRun,
    )
}

pub(crate) fn nsld_final_executable_image_dry_run_bytes_path(
    plan: &nuisc::linker::LinkPlan,
) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableImageDryRunBytes,
    )
}

pub(crate) fn nsld_final_executable_launcher_manifest_path(
    plan: &nuisc::linker::LinkPlan,
) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableLauncherManifest,
    )
}

pub(crate) fn nsld_final_executable_launcher_dry_run_path(
    plan: &nuisc::linker::LinkPlan,
) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableLauncherDryRun,
    )
}

pub(crate) fn nsld_final_executable_pipeline_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutablePipeline,
    )
}
