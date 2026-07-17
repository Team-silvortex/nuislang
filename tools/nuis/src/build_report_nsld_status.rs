use crate::workflow::{
    self, nsld_drive_apply_next_command_for_output_dir,
    nsld_drive_apply_next_json_command_for_output_dir,
    nsld_drive_apply_until_clean_command_for_output_dir,
    nsld_drive_apply_until_clean_json_command_for_output_dir,
    nsld_drive_dry_run_command_for_output_dir, nsld_drive_dry_run_json_command_for_output_dir,
    nsld_drive_recommendation_for_output_dir, nsld_final_executable_output_boundary_summary,
    nsld_final_executable_tail_summary, nsld_prepare_command_for_output_dir,
    nsld_prepared_artifact_chain_summary,
};
use std::path::Path;

pub(crate) fn print_nsld_artifact_chain_status(plan: &nuisc::linker::LinkPlan) {
    let output_dir = Path::new(&plan.output_dir);
    let nsld_chain = nsld_prepared_artifact_chain_summary(output_dir);
    println!(
        "  nsld_prepare_command: {}",
        nsld_prepare_command_for_output_dir(output_dir)
    );
    println!(
        "  nsld_drive_dry_run_command: {}",
        nsld_drive_dry_run_command_for_output_dir(output_dir)
    );
    println!(
        "  nsld_drive_dry_run_json_command: {}",
        nsld_drive_dry_run_json_command_for_output_dir(output_dir)
    );
    println!(
        "  nsld_drive_apply_next_command: {}",
        nsld_drive_apply_next_command_for_output_dir(output_dir)
    );
    println!(
        "  nsld_drive_apply_next_json_command: {}",
        nsld_drive_apply_next_json_command_for_output_dir(output_dir)
    );
    println!(
        "  nsld_drive_apply_until_clean_command: {}",
        nsld_drive_apply_until_clean_command_for_output_dir(output_dir)
    );
    println!(
        "  nsld_drive_apply_until_clean_json_command: {}",
        nsld_drive_apply_until_clean_json_command_for_output_dir(output_dir)
    );
    let nsld_tail = nsld_final_executable_tail_summary(output_dir);
    let nsld_final_output = nsld_final_executable_output_boundary_summary(plan);
    let nsld_next = workflow::nsld_next_action_summary(
        Some(&nsld_chain),
        Some(&nsld_tail),
        Some(&nsld_final_output),
    );
    let nsld_chain_next =
        workflow::nsld_artifact_chain_next_action_mirror(Some(&nsld_chain), Some(&nsld_tail));
    let nsld_drive_recommendation = nsld_drive_recommendation_for_output_dir(
        Some(output_dir),
        &nsld_chain_next,
        Some(&nsld_final_output),
    );
    println!(
        "  nsld_drive_recommended_available: {}",
        nsld_drive_recommendation.available
    );
    println!(
        "  nsld_drive_recommended_mode: {}",
        nsld_drive_recommendation.mode
    );
    println!(
        "  nsld_drive_recommended_command: {}",
        nsld_drive_recommendation
            .command
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsld_drive_recommended_mutates_artifacts: {}",
        nsld_drive_recommendation.mutates_artifacts
    );
    println!(
        "  nsld_drive_recommended_reason: {}",
        nsld_drive_recommendation.reason
    );
    println!("  nsld_prepared_artifact_chain_ready: {}", nsld_chain.ready);
    println!(
        "  nsld_prepared_artifact_stages: {}/{}",
        nsld_chain.present_count, nsld_chain.stage_count
    );
    println!(
        "  nsld_prepared_artifact_next_missing_stage: {}",
        nsld_chain.next_missing_stage.as_deref().unwrap_or("<none>")
    );
    println!("  nsld_next_action_source: {}", nsld_next.source);
    println!("  nsld_next_action: {}", nsld_next.action);
    println!(
        "  nsld_next_action_command: {}",
        nsld_next.command.as_deref().unwrap_or("<none>")
    );
    println!("  nsld_next_action_reason: {}", nsld_next.reason);
    println!(
        "  nsld_artifact_chain_next_action_available: {}",
        nsld_chain_next.available
    );
    println!(
        "  nsld_artifact_chain_next_action_source: {}",
        nsld_chain_next.source.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsld_artifact_chain_next_action_command_id: {}",
        nsld_chain_next.command_id.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsld_artifact_chain_next_action_command: {}",
        nsld_chain_next.command.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsld_artifact_chain_next_action_command_resolved: {}",
        nsld_chain_next
            .command_resolved
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsld_artifact_chain_next_action_reason: {}",
        nsld_chain_next.reason.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsld_final_executable_pipeline_command: {}",
        nsld_tail.pipeline_command
    );
    println!("  nsld_final_executable_tail_ready: {}", nsld_tail.ready);
    println!(
        "  nsld_final_executable_tail_stages: {}/{}",
        nsld_tail.present_count, nsld_tail.stage_count
    );
    println!(
        "  nsld_final_executable_tail_next_missing_stage: {}",
        nsld_tail.next_missing_stage.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsld_final_executable_pipeline_valid: {}",
        nsld_tail
            .pipeline_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_final_executable_emitted: {}",
        nsld_tail
            .final_executable_emitted
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_launcher_manifest_ready: {}",
        nsld_tail
            .launcher_manifest_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_launcher_dry_run_ready: {}",
        nsld_tail
            .launcher_dry_run_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_would_enter_lifecycle_hook: {}",
        nsld_tail
            .would_enter_lifecycle_hook
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_blocker_count: {}",
        nsld_tail
            .blocker_count
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_first_blocker: {}",
        nsld_tail.first_blocker.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsld_final_executable_pipeline_execution_handoff_contract: {}",
        nsld_tail
            .execution_handoff_contract
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_final_executable_pipeline_execution_handoff_ready: {}",
        nsld_tail
            .execution_handoff_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_execution_handoff_status: {}",
        nsld_tail
            .execution_handoff_status
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_final_executable_pipeline_execution_handoff_target: {}",
        nsld_tail
            .execution_handoff_target
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_final_executable_pipeline_execution_handoff_evidence_status: {}",
        nsld_tail
            .execution_handoff_evidence_status
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_final_executable_pipeline_execution_handoff_first_blocker: {}",
        nsld_tail
            .execution_handoff_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsld_final_executable_pipeline_execution_handoff_decision_code: {}",
        nsld_tail
            .execution_handoff_decision_code
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_final_executable_pipeline_scheduler_metadata_payload_id: {}",
        nsld_tail
            .scheduler_metadata_payload_id
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_final_executable_pipeline_scheduler_metadata_present: {}",
        nsld_tail
            .scheduler_metadata_present
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_scheduler_metadata_hash: {}",
        nsld_tail
            .scheduler_metadata_hash
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_final_executable_pipeline_required_stage_paths: {}/{}",
        nsld_tail
            .required_stage_path_present_count
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned()),
        nsld_tail
            .required_stage_path_count
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_pipeline_first_missing_required_stage_path: {}",
        nsld_tail
            .first_missing_required_stage_path
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsld_self_owned_image_ready: {}",
        nsld_tail
            .self_owned_image_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_self_owned_image_status: {}",
        nsld_tail.self_owned_image_status
    );
    println!(
        "  nsld_entrypoint_materialization_status: {}",
        nsld_tail.entrypoint_materialization_status
    );
    println!(
        "  nsld_self_owned_image_path: {}",
        nsld_tail
            .self_owned_image_path
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_self_owned_image_present: {}",
        nsld_tail
            .self_owned_image_present
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_self_owned_image_hash: {}",
        nsld_tail
            .self_owned_image_hash
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  nsld_self_owned_image_header_valid: {}",
        nsld_tail
            .self_owned_image_header_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_output_ready: {}",
        nsld_final_output.ready
    );
    println!(
        "  nsld_final_executable_output_boundary_status: {}",
        nsld_final_output.boundary_status
    );
    println!(
        "  nsld_final_executable_output_materialization_status: {}",
        nsld_final_output.materialization_status
    );
    println!(
        "  nsld_final_executable_output_execution_handoff_contract: {}",
        nsld_final_output.execution_handoff_contract
    );
    println!(
        "  nsld_final_executable_output_execution_handoff_ready: {}",
        nsld_final_output.execution_handoff_ready
    );
    println!(
        "  nsld_final_executable_output_execution_handoff_status: {}",
        nsld_final_output.execution_handoff_status
    );
    println!(
        "  nsld_final_executable_output_execution_handoff_target: {}",
        nsld_final_output.execution_handoff_target
    );
    println!(
        "  nsld_final_executable_output_execution_handoff_evidence_status: {}",
        nsld_final_output.execution_handoff_evidence_status
    );
    println!(
        "  nsld_final_executable_output_execution_handoff_first_blocker: {}",
        nsld_final_output
            .execution_handoff_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsld_final_executable_output_execution_handoff_decision_code: {}",
        nsld_final_output.execution_handoff_decision_code
    );
    println!(
        "  nsld_final_executable_output_nsdb_replay_contract: {}",
        nsld_final_output.nsdb_replay_contract
    );
    println!(
        "  nsld_final_executable_output_nsdb_replay_ready: {}",
        nsld_final_output.nsdb_replay_ready
    );
    println!(
        "  nsld_final_executable_output_nsdb_replay_status: {}",
        nsld_final_output.nsdb_replay_status
    );
    println!(
        "  nsld_final_executable_output_nsdb_replay_checkpoints: {}/{}",
        nsld_final_output.nsdb_replayable_checkpoint_count,
        nsld_final_output.nsdb_replay_checkpoint_count
    );
    println!(
        "  nsld_final_executable_output_nsdb_replay_next_action: {}",
        nsld_final_output.nsdb_replay_next_action
    );
    println!(
        "  nsld_final_executable_output_nsdb_replay_next_command: {}",
        nsld_final_output
            .nsdb_replay_next_command
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsld_final_executable_output_nsdb_replay_first_blocker: {}",
        nsld_final_output
            .nsdb_replay_first_blocker
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsld_final_executable_output_recommended_next_action: {}",
        nsld_final_output.recommended_next_action
    );
    println!(
        "  nsld_final_executable_output_path_present: {}",
        nsld_final_output.path_present
    );
    println!(
        "  nsld_final_executable_output_nsld_owned: {}",
        nsld_final_output
            .nsld_owned
            .map(|owned| owned.to_string())
            .unwrap_or_else(|| "<unknown>".to_owned())
    );
    println!(
        "  nsld_final_executable_output_blocker_count: {}",
        nsld_final_output.blockers.len()
    );
    println!(
        "  nsld_final_executable_output_first_blocker: {}",
        nsld_final_output
            .first_blocker
            .as_deref()
            .unwrap_or("<none>")
    );
    for blocker in &nsld_final_output.blockers {
        println!("  nsld_final_executable_output_blocker: {blocker}");
    }
}
