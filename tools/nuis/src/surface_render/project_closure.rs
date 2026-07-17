use super::*;

pub(super) fn project_closure_summary(
    project_source: &'static str,
    link_plan_source: &'static str,
    artifact_ready_to_run: bool,
    missing_test_count: usize,
    frontdoor: &crate::workflow::WorkflowFrontdoorSurface,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) -> crate::closure_summary::FrontdoorClosureSummary {
    let Some(plan) = link_plan else {
        return crate::closure_summary::FrontdoorClosureSummary::from_project_surface(
            project_source,
            artifact_ready_to_run,
            missing_test_count,
            frontdoor.recommended_next_step,
            frontdoor.recommended_command,
        );
    };

    let output_dir = Path::new(&plan.output_dir);
    let prepared_summary = crate::workflow::nsld_prepared_artifact_chain_summary(output_dir);
    let final_tail_summary = crate::workflow::nsld_final_executable_tail_summary(output_dir);
    let final_output_summary = crate::workflow::nsld_final_executable_output_boundary_summary(plan);
    let nsld_next = crate::workflow::nsld_next_action_summary(
        Some(&prepared_summary),
        Some(&final_tail_summary),
        Some(&final_output_summary),
    );
    let nsld_chain_next = crate::workflow::nsld_artifact_chain_next_action_mirror(
        Some(&prepared_summary),
        Some(&final_tail_summary),
    );
    let drive_recommendation = crate::workflow::nsld_drive_recommendation_for_output_dir(
        Some(output_dir),
        &nsld_chain_next,
        Some(&final_output_summary),
    );
    let drive_command_set = crate::workflow::nsld_drive_command_set_for_output_dir(output_dir);
    crate::closure_summary::FrontdoorClosureSummary::from_nsld_final_output_closure(
        link_plan_source,
        &nsld_next.action,
        nsld_next.command.as_deref(),
        &nsld_next.reason,
        Some(&final_output_summary),
    )
    .with_nsld_drive_safe_next(Some(&drive_recommendation), Some(&drive_command_set))
}
