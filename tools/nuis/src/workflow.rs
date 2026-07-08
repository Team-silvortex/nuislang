use super::*;

mod command;
mod frontdoor;
mod json_bits;
mod link_plan;
mod render;

pub(crate) use command::{
    default_build_output_dir, default_release_check_output_dir, handle_workflow,
};
#[cfg(test)]
pub(crate) use frontdoor::{
    build_workflow_frontdoor_surface, project_compile_workflow_source_profile,
    recommend_project_workflow_step, single_source_workflow_source_profile, WorkflowRecommendation,
};
pub(crate) use frontdoor::{
    print_workflow_frontdoor_surface, project_frontdoor_surface, single_source_frontdoor_surface,
    toolchain_frontdoor_surface, workflow_frontdoor_json_object_field, WorkflowFrontdoorSurface,
};
pub(crate) use json_bits::{
    append_json_object_fields, artifact_lowering_units_json, json_object_array_field,
    project_abi_checks_json, project_domain_registry_checks_json, project_lowering_checks_json,
};
pub(crate) use link_plan::{
    append_workflow_link_plan_json_fields, artifact_doctor_command_for_output_dir,
    load_link_plan_for_output_dir, run_artifact_command_for_output_dir,
};
pub(crate) use render::render_workflow_json;

pub(crate) fn debug_workflow_brief() -> &'static str {
    "dump-ast -> dump-nir -> dump-yir -> scheduler-view"
}

pub(crate) fn debug_workflow_samples_brief() -> &'static str {
    "ast=nuis dump-ast <input>; nir=nuis dump-nir <input>; yir=nuis dump-yir <input>; scheduler=nuis scheduler-view <input>"
}

pub(crate) fn single_source_compile_workflow_brief() -> &'static str {
    "check -> test -> build -> artifact_doctor -> run_artifact -> release_check"
}

pub(crate) fn single_source_compile_samples_brief() -> &'static str {
    "check=nuis check <input.ns>; test=nuis test <input.ns>; build=nuis build <input.ns> <output-dir>; artifact=nuis artifact-doctor <output-dir>; run=nuis run-artifact <output-dir>; release=nuis release-check <input.ns> <output-dir>"
}

pub(crate) fn artifact_workflow_brief() -> &'static str {
    "build -> inspect_artifact -> verify_artifact -> artifact_doctor -> verify_build_manifest -> run_artifact"
}
