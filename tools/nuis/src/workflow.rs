use super::*;

mod command;
mod command_nsld_text;
mod compile_pipeline;
mod frontdoor;
mod json_bits;
mod link_plan;
mod link_plan_commands;
mod link_plan_domain;
mod link_plan_final_output;
mod link_plan_final_output_summary;
mod link_plan_json;
mod link_plan_json_nsld_output;
mod link_plan_tail;
mod object_identity;
mod render;

pub(crate) use command::{
    default_build_output_dir, default_release_check_output_dir, handle_workflow,
};
pub(crate) use compile_pipeline::workflow_compile_pipeline_json_fields;
#[cfg(test)]
pub(crate) use frontdoor::{
    build_workflow_frontdoor_surface, project_compile_workflow_source_profile,
    recommend_project_workflow_step, single_source_workflow_source_profile, WorkflowRecommendation,
};
pub(crate) use frontdoor::{
    print_workflow_frontdoor_surface, project_frontdoor_surface, single_source_frontdoor_surface,
    toolchain_frontdoor_surface, workflow_frontdoor_json_object_field,
    write_workflow_frontdoor_reading_order, WorkflowFrontdoorSurface, FRONTDOOR_READING_ORDER,
    FRONTDOOR_READING_ORDER_CONTRACT, FRONTDOOR_SAMPLE_CLOSURE_SUMMARY,
    FRONTDOOR_SAMPLE_TENSOR_HANDOFF,
};
pub(crate) use json_bits::{
    append_json_object_fields, artifact_lowering_units_json, json_object_array_field,
    project_abi_checks_json, project_domain_registry_checks_json, project_lowering_checks_json,
};
pub(crate) use link_plan::{
    artifact_doctor_command_for_output_dir, load_link_plan_for_output_dir,
    nsld_artifact_chain_next_action_mirror, nsld_drive_recommendation_for_output_dir,
    nsld_final_executable_tail_stage_records_json, nsld_next_action_summary,
    nsld_prepared_artifact_chain_summary, nsld_prepared_artifact_stage_records_json,
    run_artifact_command_for_output_dir, NsldDriveRecommendation,
};
pub(crate) use link_plan_commands::{
    nsld_drive_apply_next_command_for_output_dir,
    nsld_drive_apply_next_json_command_for_output_dir,
    nsld_drive_apply_until_clean_command_for_output_dir,
    nsld_drive_apply_until_clean_json_command_for_output_dir,
    nsld_drive_command_set_for_output_dir, nsld_drive_command_set_json_field,
    nsld_drive_dry_run_command_for_output_dir, nsld_drive_dry_run_json_command_for_output_dir,
    nsld_prepare_command_for_output_dir, NsldDriveCommandSet,
};
#[cfg(test)]
pub(crate) use link_plan_commands::{
    release_check_nsld_drive_command_for_output_dir,
    release_check_nsld_drive_dry_run_command_for_output_dir,
    release_check_nsld_drive_dry_run_json_command_for_output_dir,
    release_check_nsld_drive_json_command_for_output_dir,
    release_check_nsld_drive_until_clean_command_for_output_dir,
    release_check_nsld_drive_until_clean_json_command_for_output_dir,
};
pub(crate) use link_plan_final_output::nsld_final_executable_output_boundary_summary;
pub(crate) use link_plan_final_output_summary::NsldFinalExecutableOutputBoundarySummary;
#[cfg(test)]
pub(crate) use link_plan_final_output_summary::ProviderCompletionBoundarySummary;
pub(crate) use link_plan_json::append_workflow_link_plan_json_fields;
pub(crate) use link_plan_tail::{
    nsld_final_executable_tail_summary, NsldFinalExecutableTailSummary,
};
pub(crate) use render::render_workflow_json;

pub(crate) fn debug_workflow_brief() -> &'static str {
    "dump-ast -> dump-nir -> dump-yir -> scheduler-view"
}

pub(crate) fn debug_workflow_samples_brief() -> &'static str {
    "ast=nuis dump-ast <input>; nir=nuis dump-nir <input>; yir=nuis dump-yir <input>; scheduler=nuis scheduler-view <input>"
}

pub(crate) fn single_source_compile_workflow_brief() -> &'static str {
    "check -> test -> build -> artifact_doctor -> nsld_drive -> run_artifact -> release_check"
}

pub(crate) fn single_source_compile_samples_brief() -> &'static str {
    "check=nuis check <input.ns>; test=nuis test <input.ns>; build=nuis build <input.ns> <output-dir>; artifact=nuis artifact-doctor <output-dir>; linker=nsld drive <output-dir>/nuis.build.manifest.toml --apply; run=nuis run-artifact <output-dir>; release=nuis release-check <input.ns> <output-dir>"
}

pub(crate) fn artifact_workflow_brief() -> &'static str {
    "build -> inspect_artifact -> verify_artifact -> artifact_doctor -> nsld_drive -> verify_build_manifest -> run_artifact"
}
