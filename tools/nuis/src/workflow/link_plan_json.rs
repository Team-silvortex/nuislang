use super::link_plan_domain::{
    workflow_domain_readiness_summary, workflow_domain_readiness_units_json,
    workflow_link_plan_domain_unit_record,
};
use super::*;
use crate::{artifact_doctor::probe_artifact_doctor, run_artifact::run_artifact_prelaunch_summary};
use std::path::{Path, PathBuf};

fn workflow_link_plan_json_fields(link_plan: Option<&nuisc::linker::LinkPlan>) -> Vec<String> {
    let domain_unit_records = link_plan
        .map(|plan| {
            plan.domain_units
                .iter()
                .map(workflow_link_plan_domain_unit_record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let nsld_chain =
        link_plan.map(|plan| nsld_prepared_artifact_chain_summary(Path::new(&plan.output_dir)));
    let nsld_tail =
        link_plan.map(|plan| nsld_final_executable_tail_summary(Path::new(&plan.output_dir)));
    let prepared_stage_records = link_plan
        .map(|plan| nsld_prepared_artifact_stage_records_json(Path::new(&plan.output_dir)))
        .unwrap_or_default();
    let final_tail_stage_records = link_plan
        .map(|plan| nsld_final_executable_tail_stage_records_json(Path::new(&plan.output_dir)))
        .unwrap_or_default();
    let nsld_final_output = link_plan.map(nsld_final_executable_output_boundary_summary);
    let nsld_next = nsld_next_action_summary(
        nsld_chain.as_ref(),
        nsld_tail.as_ref(),
        nsld_final_output.as_ref(),
    );
    let nsld_chain_next =
        nsld_artifact_chain_next_action_mirror(nsld_chain.as_ref(), nsld_tail.as_ref());
    let nsld_drive_recommendation = nsld_drive_recommendation_for_output_dir(
        link_plan.map(|plan| Path::new(&plan.output_dir)),
        &nsld_chain_next,
        nsld_final_output.as_ref(),
    );
    let nsld_drive_command_set =
        link_plan.map(|plan| nsld_drive_command_set_for_output_dir(Path::new(&plan.output_dir)));
    let domain_readiness = link_plan.map(workflow_domain_readiness_summary);
    let workflow_prelaunch =
        link_plan.map(|plan| workflow_run_artifact_prelaunch_summary(Path::new(&plan.output_dir)));
    vec![
        json_bool_field("link_plan_available", link_plan.is_some()),
        json_optional_string_field(
            "link_plan_final_stage",
            link_plan.map(|plan| plan.final_stage.kind.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_driver",
            link_plan.map(|plan| plan.final_stage.driver.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_link_mode",
            link_plan.map(|plan| plan.final_stage.link_mode.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_output",
            link_plan.map(|plan| plan.final_stage.output_path.as_str()),
        ),
        json_optional_string_field(
            "link_plan_lowering_plan_index_path",
            link_plan.and_then(|plan| plan.lowering_plan_index_path.as_deref()),
        ),
        json_optional_string_field(
            "link_plan_lowering_plan_index_source",
            link_plan.map(|plan| plan.lowering_plan_index_source.as_str()),
        ),
        json_optional_string_field(
            "workflow_run_artifact_prelaunch_kind",
            workflow_prelaunch
                .as_ref()
                .map(|prelaunch| prelaunch.kind.as_str()),
        ),
        json_optional_string_field(
            "workflow_run_artifact_prelaunch_status",
            workflow_prelaunch
                .as_ref()
                .map(|prelaunch| prelaunch.status.as_str()),
        ),
        json_optional_string_field(
            "workflow_run_artifact_prelaunch_evidence_status",
            workflow_prelaunch
                .as_ref()
                .map(|prelaunch| prelaunch.evidence_status.as_str()),
        ),
        json_optional_string_field(
            "workflow_run_artifact_prelaunch_command",
            workflow_prelaunch
                .as_ref()
                .and_then(|prelaunch| prelaunch.command.as_deref()),
        ),
        json_optional_string_field(
            "workflow_run_artifact_prelaunch_reason",
            workflow_prelaunch
                .as_ref()
                .map(|prelaunch| prelaunch.reason.as_str()),
        ),
        json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
        json_usize_field(
            "link_plan_heterogeneous_domain_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.hetero_units)
                .unwrap_or(0),
        ),
        json_usize_field(
            "link_plan_heterogeneous_domain_ready_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.ready_units)
                .unwrap_or(0),
        ),
        json_usize_field(
            "link_plan_heterogeneous_domain_registry_dispatch_ready_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.registry_dispatch_ready_units)
                .unwrap_or(0),
        ),
        json_bool_field(
            "link_plan_heterogeneous_domain_readiness_ready",
            domain_readiness
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        json_string_array_field(
            "link_plan_heterogeneous_domain_families",
            &domain_readiness
                .as_ref()
                .map(|summary| summary.domain_families.clone())
                .unwrap_or_default(),
        ),
        json_optional_string_field(
            "link_plan_heterogeneous_domain_first_unready",
            domain_readiness
                .as_ref()
                .and_then(|summary| summary.first_unready.as_deref()),
        ),
        json_optional_string_field(
            "link_plan_heterogeneous_domain_registry_dispatch_first_blocked",
            domain_readiness
                .as_ref()
                .and_then(|summary| summary.registry_dispatch_first_blocked.as_deref()),
        ),
        json_object_array_field(
            "link_plan_heterogeneous_domain_readiness",
            &domain_readiness
                .as_ref()
                .map(workflow_domain_readiness_units_json)
                .unwrap_or_default(),
        ),
        json_object_array_field("link_plan_domain_unit_records", &domain_unit_records),
        json_optional_string_field(
            "nsld_prepare_command",
            nsld_chain
                .as_ref()
                .map(|summary| summary.prepare_command.as_str()),
        ),
        json_optional_string_field(
            "nsld_drive_dry_run_command",
            link_plan
                .map(|plan| nsld_drive_dry_run_command_for_output_dir(Path::new(&plan.output_dir)))
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_dry_run_json_command",
            link_plan
                .map(|plan| {
                    nsld_drive_dry_run_json_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_next_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_next_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_next_json_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_next_json_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_until_clean_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_until_clean_command_for_output_dir(Path::new(&plan.output_dir))
                })
                .as_deref(),
        ),
        json_optional_string_field(
            "nsld_drive_apply_until_clean_json_command",
            link_plan
                .map(|plan| {
                    nsld_drive_apply_until_clean_json_command_for_output_dir(Path::new(
                        &plan.output_dir,
                    ))
                })
                .as_deref(),
        ),
        nsld_drive_command_set_json_field(
            "nsld_drive_command_set",
            nsld_drive_command_set.as_ref(),
        ),
        json_bool_field(
            "nsld_drive_recommended_available",
            nsld_drive_recommendation.available,
        ),
        json_field(
            "nsld_drive_recommended_mode",
            &nsld_drive_recommendation.mode,
        ),
        json_optional_string_field(
            "nsld_drive_recommended_command",
            nsld_drive_recommendation.command.as_deref(),
        ),
        json_bool_field(
            "nsld_drive_recommended_mutates_artifacts",
            nsld_drive_recommendation.mutates_artifacts,
        ),
        json_field(
            "nsld_drive_recommended_reason",
            &nsld_drive_recommendation.reason,
        ),
        json_bool_field(
            "nsld_prepared_artifact_chain_ready",
            nsld_chain.as_ref().is_some_and(|summary| summary.ready),
        ),
        json_usize_field(
            "nsld_prepared_artifact_stage_count",
            nsld_chain
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_prepared_artifact_present_count",
            nsld_chain
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_prepared_artifact_next_missing_stage",
            nsld_chain
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        json_object_array_field(
            "nsld_prepared_artifact_stage_records",
            &prepared_stage_records,
        ),
        json_field("nsld_next_action_source", &nsld_next.source),
        json_field("nsld_next_action", &nsld_next.action),
        json_optional_string_field("nsld_next_action_command", nsld_next.command.as_deref()),
        json_field("nsld_next_action_reason", &nsld_next.reason),
        json_bool_field(
            "nsld_artifact_chain_next_action_available",
            nsld_chain_next.available,
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_source",
            nsld_chain_next.source.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_command_id",
            nsld_chain_next.command_id.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_command",
            nsld_chain_next.command.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_command_resolved",
            nsld_chain_next.command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "nsld_artifact_chain_next_action_reason",
            nsld_chain_next.reason.as_deref(),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_command",
            nsld_tail
                .as_ref()
                .map(|summary| summary.pipeline_command.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_tail_ready",
            nsld_tail.as_ref().is_some_and(|summary| summary.ready),
        ),
        json_usize_field(
            "nsld_final_executable_tail_stage_count",
            nsld_tail
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_tail_present_count",
            nsld_tail
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_final_executable_tail_next_missing_stage",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        json_object_array_field(
            "nsld_final_executable_tail_stage_records",
            &final_tail_stage_records,
        ),
        match nsld_tail
            .as_ref()
            .and_then(|summary| summary.pipeline_valid)
        {
            Some(valid) => json_bool_field("nsld_final_executable_pipeline_valid", valid),
            None => "\"nsld_final_executable_pipeline_valid\":null".to_owned(),
        },
        json_optional_bool_field(
            "nsld_final_executable_pipeline_final_executable_emitted",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.final_executable_emitted),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_manifest_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.launcher_manifest_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_dry_run_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_would_enter_lifecycle_hook",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.would_enter_lifecycle_hook),
        ),
        match nsld_tail.as_ref().and_then(|summary| summary.blocker_count) {
            Some(count) => json_usize_field("nsld_final_executable_pipeline_blocker_count", count),
            None => "\"nsld_final_executable_pipeline_blocker_count\":null".to_owned(),
        },
        json_optional_string_field(
            "nsld_final_executable_pipeline_first_blocker",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_contract",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_contract.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_execution_handoff_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_status",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_status.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_target",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_target.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_evidence_status.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.execution_handoff_decision_code.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_kind",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_kind.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_path",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_first_blocker",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_first_blocker.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_present",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_present),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_hash",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_hash.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_entrypoint_materialization_runner_command",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.entrypoint_materialization_runner_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_payload_id",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_payload_id.as_deref()),
        ),
        match nsld_tail
            .as_ref()
            .and_then(|summary| summary.scheduler_metadata_present)
        {
            Some(present) => json_bool_field(
                "nsld_final_executable_pipeline_scheduler_metadata_present",
                present,
            ),
            None => "\"nsld_final_executable_pipeline_scheduler_metadata_present\":null".to_owned(),
        },
        json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_hash",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_hash.as_deref()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_count",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.required_stage_path_count),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_present_count",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.required_stage_path_present_count),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_first_missing_required_stage_path",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.first_missing_required_stage_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_ready),
        ),
        json_optional_string_field(
            "nsld_self_owned_image_status",
            nsld_tail
                .as_ref()
                .map(|summary| summary.self_owned_image_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_entrypoint_materialization_status",
            nsld_tail
                .as_ref()
                .map(|summary| summary.entrypoint_materialization_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_self_owned_image_path",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_present",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_present),
        ),
        json_optional_string_field(
            "nsld_self_owned_image_hash",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_hash.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_header_valid",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.self_owned_image_header_valid),
        ),
        json_bool_field(
            "nsld_final_executable_output_ready",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_boundary_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.boundary_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_materialization_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.materialization_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_contract",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_contract.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_execution_handoff_ready",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.execution_handoff_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_target",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_target.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_evidence_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_evidence_status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_first_blocker",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_decision_code",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.execution_handoff_decision_code.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_entrypoint_materialization_evidence_status",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.entrypoint_materialization_evidence_status.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_launcher_manifest_present",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.launcher_manifest_present),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_launcher_manifest_ready",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.launcher_manifest_ready),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_launcher_manifest_blocker_count",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.launcher_manifest_blocker_count),
        ),
        json_bool_field(
            "nsld_final_executable_output_launcher_dry_run_present",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.launcher_dry_run_present),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_launcher_dry_run_ready",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_launcher_dry_run_would_enter_lifecycle_hook",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_would_enter_lifecycle_hook),
        ),
        json_optional_usize_field(
            "nsld_final_executable_output_launcher_dry_run_blocker_count",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_blocker_count),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_recommended_next_action",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.recommended_next_action.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_path_present",
            nsld_final_output
                .as_ref()
                .is_some_and(|summary| summary.path_present),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_nsld_owned",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.nsld_owned),
        ),
        json_usize_field(
            "nsld_final_executable_output_blocker_count",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.blockers.len())
                .unwrap_or(0),
        ),
        json_string_array_field(
            "nsld_final_executable_output_blockers",
            nsld_final_output
                .as_ref()
                .map(|summary| summary.blockers.as_slice())
                .unwrap_or(&[]),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_first_blocker",
            nsld_final_output
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
    ]
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn workflow_run_artifact_prelaunch_summary(
    output_dir: &Path,
) -> crate::run_artifact::RunArtifactPrelaunchSummary {
    let doctor = probe_artifact_doctor(output_dir);
    let resolved_binary = doctor.binary_path.filter(|path| path.exists());
    run_artifact_prelaunch_summary(
        Some(output_dir),
        resolved_binary.as_ref().map(PathBuf::as_path),
    )
}

pub(crate) fn append_workflow_link_plan_json_fields(
    out: &mut String,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    append_json_field_strings(out, workflow_link_plan_json_fields(link_plan));
}
