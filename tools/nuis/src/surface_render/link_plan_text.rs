use std::{fmt, path::Path};

pub(super) fn write_unavailable_link_plan_text_fields<W: fmt::Write>(out: &mut W) -> fmt::Result {
    writeln!(out, "  link_plan_final_stage: <unavailable>")?;
    writeln!(out, "  link_plan_final_driver: <unavailable>")?;
    writeln!(out, "  link_plan_final_link_mode: <unavailable>")?;
    writeln!(out, "  link_plan_final_output: <unavailable>")?;
    writeln!(out, "  link_plan_lowering_plan_index_path: <unavailable>")?;
    writeln!(out, "  link_plan_lowering_plan_index_source: <unavailable>")?;
    writeln!(out, "  link_plan_domain_units: 0")?;
    writeln!(out, "  link_plan_heterogeneous_domain_units: 0")?;
    writeln!(out, "  link_plan_heterogeneous_domain_ready_units: 0")?;
    writeln!(
        out,
        "  link_plan_heterogeneous_domain_registry_dispatch_ready_units: 0"
    )?;
    writeln!(
        out,
        "  link_plan_heterogeneous_backend_artifact_ready_units: 0/0"
    )?;
    writeln!(out, "  link_plan_heterogeneous_domain_readiness_ready: no")?;
    writeln!(out, "  link_plan_heterogeneous_domain_families: <none>")?;
    writeln!(
        out,
        "  link_plan_heterogeneous_domain_first_unready: <none>"
    )?;
    writeln!(
        out,
        "  link_plan_heterogeneous_backend_artifact_first_unready: <none>"
    )?;
    writeln!(
        out,
        "  link_plan_heterogeneous_domain_registry_dispatch_first_blocked: <none>"
    )?;
    write_unavailable_nsld_text_fields(out)
}

fn write_unavailable_nsld_text_fields<W: fmt::Write>(out: &mut W) -> fmt::Result {
    writeln!(out, "  nsld_prepare_command: <unavailable>")?;
    writeln!(out, "  nsld_drive_dry_run_command: <unavailable>")?;
    writeln!(out, "  nsld_drive_dry_run_json_command: <unavailable>")?;
    writeln!(out, "  nsld_drive_apply_next_command: <unavailable>")?;
    writeln!(out, "  nsld_drive_apply_next_json_command: <unavailable>")?;
    writeln!(out, "  nsld_drive_apply_until_clean_command: <unavailable>")?;
    writeln!(
        out,
        "  nsld_drive_apply_until_clean_json_command: <unavailable>"
    )?;
    writeln!(out, "  nsld_drive_recommended_available: no")?;
    writeln!(out, "  nsld_drive_recommended_mode: unavailable")?;
    writeln!(out, "  nsld_drive_recommended_command: <unavailable>")?;
    writeln!(out, "  nsld_drive_recommended_mutates_artifacts: no")?;
    writeln!(
        out,
        "  nsld_drive_recommended_reason: link plan is unavailable"
    )?;
    writeln!(out, "  nsld_prepared_artifact_chain_ready: no")?;
    writeln!(out, "  nsld_prepared_artifact_stages: 0/0")?;
    writeln!(
        out,
        "  nsld_prepared_artifact_next_missing_stage: <unavailable>"
    )?;
    writeln!(out, "  nsld_next_action_source: nuis-summary")?;
    writeln!(out, "  nsld_next_action: unavailable")?;
    writeln!(out, "  nsld_next_action_command: <unavailable>")?;
    writeln!(out, "  nsld_next_action_reason: link plan is unavailable")?;
    writeln!(out, "  nsld_artifact_chain_next_action_available: no")?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_source: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_command_id: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_command: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_command_resolved: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_reason: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_command: <unavailable>"
    )?;
    writeln!(out, "  nsld_final_executable_tail_ready: no")?;
    writeln!(out, "  nsld_final_executable_tail_stages: 0/0")?;
    writeln!(
        out,
        "  nsld_final_executable_tail_next_missing_stage: <unavailable>"
    )?;
    writeln!(out, "  nsld_final_executable_pipeline_valid: <unavailable>")?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_final_executable_emitted: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_launcher_manifest_ready: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_launcher_dry_run_ready: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_would_enter_lifecycle_hook: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_blocker_count: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_first_blocker: <none>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_contract: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_ready: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_target: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_evidence_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_first_blocker: <none>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_decision_code: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_kind: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_path: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_ready: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_first_blocker: <none>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_present: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_hash: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_runner_command: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_scheduler_metadata_payload_id: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_scheduler_metadata_present: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_scheduler_metadata_hash: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_required_stage_paths: <unavailable>/<unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_first_missing_required_stage_path: <none>"
    )?;
    writeln!(out, "  nsld_self_owned_image_ready: <unavailable>")?;
    writeln!(out, "  nsld_self_owned_image_status: <unavailable>")?;
    writeln!(
        out,
        "  nsld_entrypoint_materialization_status: <unavailable>"
    )?;
    writeln!(out, "  nsld_self_owned_image_path: <unavailable>")?;
    writeln!(out, "  nsld_self_owned_image_present: <unavailable>")?;
    writeln!(out, "  nsld_self_owned_image_hash: <unavailable>")?;
    writeln!(out, "  nsld_self_owned_image_header_valid: <unavailable>")?;
    writeln!(out, "  nsld_final_executable_output_ready: <unavailable>")?;
    writeln!(
        out,
        "  nsld_final_executable_output_boundary_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_materialization_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_contract: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_ready: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_target: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_evidence_status: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_first_blocker: <none>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_decision_code: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_recommended_next_action: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_path_present: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsld_owned: <unavailable>"
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_blocker_count: <unavailable>"
    )?;
    writeln!(out, "  nsld_final_executable_output_first_blocker: <none>")
}

pub(super) fn write_nsld_artifact_chain_text_fields<W: fmt::Write>(
    out: &mut W,
    plan: &nuisc::linker::LinkPlan,
) -> fmt::Result {
    let output_dir = Path::new(&plan.output_dir);
    let prepared = crate::workflow::nsld_prepared_artifact_chain_summary(output_dir);
    let final_tail = crate::workflow::nsld_final_executable_tail_summary(output_dir);
    let final_output = crate::workflow::nsld_final_executable_output_boundary_summary(plan);
    let nsld_next = crate::workflow::nsld_next_action_summary(
        Some(&prepared),
        Some(&final_tail),
        Some(&final_output),
    );
    let nsld_chain_next =
        crate::workflow::nsld_artifact_chain_next_action_mirror(Some(&prepared), Some(&final_tail));
    let nsld_drive_recommendation = crate::workflow::nsld_drive_recommendation_for_output_dir(
        Some(output_dir),
        &nsld_chain_next,
        Some(&final_output),
    );
    writeln!(out, "  nsld_prepare_command: {}", prepared.prepare_command)?;
    writeln!(
        out,
        "  nsld_drive_dry_run_command: {}",
        crate::workflow::nsld_drive_dry_run_command_for_output_dir(output_dir)
    )?;
    writeln!(
        out,
        "  nsld_drive_dry_run_json_command: {}",
        crate::workflow::nsld_drive_dry_run_json_command_for_output_dir(output_dir)
    )?;
    writeln!(
        out,
        "  nsld_drive_apply_next_command: {}",
        crate::workflow::nsld_drive_apply_next_command_for_output_dir(output_dir)
    )?;
    writeln!(
        out,
        "  nsld_drive_apply_next_json_command: {}",
        crate::workflow::nsld_drive_apply_next_json_command_for_output_dir(output_dir)
    )?;
    writeln!(
        out,
        "  nsld_drive_apply_until_clean_command: {}",
        crate::workflow::nsld_drive_apply_until_clean_command_for_output_dir(output_dir)
    )?;
    writeln!(
        out,
        "  nsld_drive_apply_until_clean_json_command: {}",
        crate::workflow::nsld_drive_apply_until_clean_json_command_for_output_dir(output_dir)
    )?;
    writeln!(
        out,
        "  nsld_drive_recommended_available: {}",
        crate::yes_no(nsld_drive_recommendation.available)
    )?;
    writeln!(
        out,
        "  nsld_drive_recommended_mode: {}",
        nsld_drive_recommendation.mode
    )?;
    writeln!(
        out,
        "  nsld_drive_recommended_command: {}",
        nsld_drive_recommendation
            .command
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_drive_recommended_mutates_artifacts: {}",
        crate::yes_no(nsld_drive_recommendation.mutates_artifacts)
    )?;
    writeln!(
        out,
        "  nsld_drive_recommended_reason: {}",
        nsld_drive_recommendation.reason
    )?;
    writeln!(
        out,
        "  nsld_prepared_artifact_chain_ready: {}",
        crate::yes_no(prepared.ready)
    )?;
    writeln!(
        out,
        "  nsld_prepared_artifact_stages: {}/{}",
        prepared.present_count, prepared.stage_count
    )?;
    writeln!(
        out,
        "  nsld_prepared_artifact_next_missing_stage: {}",
        prepared.next_missing_stage.as_deref().unwrap_or("<none>")
    )?;
    writeln!(out, "  nsld_next_action_source: {}", nsld_next.source)?;
    writeln!(out, "  nsld_next_action: {}", nsld_next.action)?;
    writeln!(
        out,
        "  nsld_next_action_command: {}",
        nsld_next.command.as_deref().unwrap_or("<none>")
    )?;
    writeln!(out, "  nsld_next_action_reason: {}", nsld_next.reason)?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_available: {}",
        crate::yes_no(nsld_chain_next.available)
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_source: {}",
        nsld_chain_next.source.as_deref().unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_command_id: {}",
        nsld_chain_next.command_id.as_deref().unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_command: {}",
        nsld_chain_next.command.as_deref().unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_command_resolved: {}",
        nsld_chain_next
            .command_resolved
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_artifact_chain_next_action_reason: {}",
        nsld_chain_next.reason.as_deref().unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_command: {}",
        final_tail.pipeline_command
    )?;
    writeln!(
        out,
        "  nsld_final_executable_tail_ready: {}",
        crate::yes_no(final_tail.ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_tail_stages: {}/{}",
        final_tail.present_count, final_tail.stage_count
    )?;
    writeln!(
        out,
        "  nsld_final_executable_tail_next_missing_stage: {}",
        final_tail.next_missing_stage.as_deref().unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_valid: {}",
        final_tail
            .pipeline_valid
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_final_executable_emitted: {}",
        final_tail
            .final_executable_emitted
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_launcher_manifest_ready: {}",
        final_tail
            .launcher_manifest_ready
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_launcher_dry_run_ready: {}",
        final_tail
            .launcher_dry_run_ready
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_would_enter_lifecycle_hook: {}",
        final_tail
            .would_enter_lifecycle_hook
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_blocker_count: {}",
        final_tail
            .blocker_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "<unavailable>".to_owned())
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_first_blocker: {}",
        final_tail.first_blocker.as_deref().unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_contract: {}",
        final_tail
            .execution_handoff_contract
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_ready: {}",
        final_tail
            .execution_handoff_ready
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_status: {}",
        final_tail
            .execution_handoff_status
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_target: {}",
        final_tail
            .execution_handoff_target
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_evidence_status: {}",
        final_tail
            .execution_handoff_evidence_status
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_first_blocker: {}",
        final_tail
            .execution_handoff_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_execution_handoff_decision_code: {}",
        final_tail
            .execution_handoff_decision_code
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_kind: {}",
        final_tail
            .entrypoint_materialization_kind
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_path: {}",
        final_tail
            .entrypoint_materialization_path
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_ready: {}",
        final_tail
            .entrypoint_materialization_ready
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_first_blocker: {}",
        final_tail
            .entrypoint_materialization_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_present: {}",
        final_tail
            .entrypoint_materialization_present
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_hash: {}",
        final_tail
            .entrypoint_materialization_hash
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_entrypoint_materialization_runner_command: {}",
        final_tail
            .entrypoint_materialization_runner_command
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_scheduler_metadata_payload_id: {}",
        final_tail
            .scheduler_metadata_payload_id
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_scheduler_metadata_present: {}",
        final_tail
            .scheduler_metadata_present
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_scheduler_metadata_hash: {}",
        final_tail
            .scheduler_metadata_hash
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_required_stage_paths: {}/{}",
        final_tail
            .required_stage_path_present_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "<unavailable>".to_owned()),
        final_tail
            .required_stage_path_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "<unavailable>".to_owned())
    )?;
    writeln!(
        out,
        "  nsld_final_executable_pipeline_first_missing_required_stage_path: {}",
        final_tail
            .first_missing_required_stage_path
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_self_owned_image_ready: {}",
        final_tail
            .self_owned_image_ready
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_self_owned_image_status: {}",
        final_tail.self_owned_image_status
    )?;
    writeln!(
        out,
        "  nsld_entrypoint_materialization_status: {}",
        final_tail.entrypoint_materialization_status
    )?;
    writeln!(
        out,
        "  nsld_self_owned_image_path: {}",
        final_tail
            .self_owned_image_path
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_self_owned_image_present: {}",
        final_tail
            .self_owned_image_present
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_self_owned_image_hash: {}",
        final_tail
            .self_owned_image_hash
            .as_deref()
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_self_owned_image_header_valid: {}",
        final_tail
            .self_owned_image_header_valid
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_ready: {}",
        crate::yes_no(final_output.ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_boundary_status: {}",
        final_output.boundary_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_materialization_status: {}",
        final_output.materialization_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_contract: {}",
        final_output.execution_handoff_contract
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_ready: {}",
        crate::yes_no(final_output.execution_handoff_ready)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_status: {}",
        final_output.execution_handoff_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_target: {}",
        final_output.execution_handoff_target
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_evidence_status: {}",
        final_output.execution_handoff_evidence_status
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_first_blocker: {}",
        final_output
            .execution_handoff_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_execution_handoff_decision_code: {}",
        final_output.execution_handoff_decision_code
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_recommended_next_action: {}",
        final_output.recommended_next_action
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_path_present: {}",
        crate::yes_no(final_output.path_present)
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_nsld_owned: {}",
        final_output
            .nsld_owned
            .map(crate::yes_no)
            .unwrap_or("<unavailable>")
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_blocker_count: {}",
        final_output.blockers.len()
    )?;
    writeln!(
        out,
        "  nsld_final_executable_output_first_blocker: {}",
        final_output.first_blocker.as_deref().unwrap_or("<none>")
    )?;
    for blocker in &final_output.blockers {
        writeln!(out, "  nsld_final_executable_output_blocker: {blocker}")?;
    }
    Ok(())
}
