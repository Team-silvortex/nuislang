use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LinkPlanDomainReadiness {
    package_id: String,
    domain_family: String,
    ready: bool,
    selected_lowering_target_present: bool,
    payload_blob_present: bool,
    payload_format_present: bool,
    bridge_stub_present: bool,
    ir_sidecar_present: bool,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LinkPlanDomainReadinessSummary {
    hetero_units: usize,
    ready_units: usize,
    ready: bool,
    domain_families: Vec<String>,
    first_unready: Option<String>,
    units: Vec<LinkPlanDomainReadiness>,
}

pub(super) fn append_json_object_strings(out: &mut String, values: &[String]) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(value);
    }
}

pub(super) fn load_link_plan(output_dir: &Path) -> Option<nuisc::linker::LinkPlan> {
    let manifest = output_dir.join("nuis.build.manifest.toml");
    if !manifest.exists() {
        return None;
    }
    nuisc::linker::build_link_plan_from_manifest(&manifest).ok()
}

pub(super) fn write_link_plan_text_fields<W: fmt::Write>(
    out: &mut W,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) -> fmt::Result {
    writeln!(
        out,
        "  link_plan_available: {}",
        crate::yes_no(link_plan.is_some())
    )?;
    if let Some(plan) = link_plan {
        writeln!(out, "  link_plan_final_stage: {}", plan.final_stage.kind)?;
        writeln!(out, "  link_plan_final_driver: {}", plan.final_stage.driver)?;
        writeln!(
            out,
            "  link_plan_final_link_mode: {}",
            plan.final_stage.link_mode
        )?;
        writeln!(
            out,
            "  link_plan_final_output: {}",
            plan.final_stage.output_path
        )?;
        writeln!(
            out,
            "  link_plan_lowering_plan_index_path: {}",
            plan.lowering_plan_index_path.as_deref().unwrap_or("<none>")
        )?;
        writeln!(
            out,
            "  link_plan_lowering_plan_index_source: {}",
            plan.lowering_plan_index_source
        )?;
        writeln!(out, "  link_plan_domain_units: {}", plan.domain_units.len())?;
        let domain_readiness = link_plan_domain_readiness_summary(plan);
        writeln!(
            out,
            "  link_plan_heterogeneous_domain_units: {}",
            domain_readiness.hetero_units
        )?;
        writeln!(
            out,
            "  link_plan_heterogeneous_domain_ready_units: {}",
            domain_readiness.ready_units
        )?;
        writeln!(
            out,
            "  link_plan_heterogeneous_domain_readiness_ready: {}",
            crate::yes_no(domain_readiness.ready)
        )?;
        writeln!(
            out,
            "  link_plan_heterogeneous_domain_families: {}",
            if domain_readiness.domain_families.is_empty() {
                "<none>".to_owned()
            } else {
                domain_readiness.domain_families.join(", ")
            }
        )?;
        writeln!(
            out,
            "  link_plan_heterogeneous_domain_first_unready: {}",
            domain_readiness
                .first_unready
                .as_deref()
                .unwrap_or("<none>")
        )?;
        write_nsld_artifact_chain_text_fields(out, plan)?;
    } else {
        writeln!(out, "  link_plan_final_stage: <unavailable>")?;
        writeln!(out, "  link_plan_final_driver: <unavailable>")?;
        writeln!(out, "  link_plan_final_link_mode: <unavailable>")?;
        writeln!(out, "  link_plan_final_output: <unavailable>")?;
        writeln!(out, "  link_plan_lowering_plan_index_path: <unavailable>")?;
        writeln!(out, "  link_plan_lowering_plan_index_source: <unavailable>")?;
        writeln!(out, "  link_plan_domain_units: 0")?;
        writeln!(out, "  link_plan_heterogeneous_domain_units: 0")?;
        writeln!(out, "  link_plan_heterogeneous_domain_ready_units: 0")?;
        writeln!(out, "  link_plan_heterogeneous_domain_readiness_ready: no")?;
        writeln!(out, "  link_plan_heterogeneous_domain_families: <none>")?;
        writeln!(
            out,
            "  link_plan_heterogeneous_domain_first_unready: <none>"
        )?;
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
        writeln!(out, "  nsld_final_executable_output_first_blocker: <none>")?;
    }
    Ok(())
}

pub(super) fn link_plan_json_fields(link_plan: Option<&nuisc::linker::LinkPlan>) -> Vec<String> {
    let prepared_summary = link_plan.map(|plan| {
        crate::workflow::nsld_prepared_artifact_chain_summary(Path::new(&plan.output_dir))
    });
    let final_tail_summary = link_plan.map(|plan| {
        crate::workflow::nsld_final_executable_tail_summary(Path::new(&plan.output_dir))
    });
    let final_output_summary =
        link_plan.map(crate::workflow::nsld_final_executable_output_boundary_summary);
    let prepared_stage_records = link_plan
        .map(|plan| {
            crate::workflow::nsld_prepared_artifact_stage_records_json(Path::new(&plan.output_dir))
        })
        .unwrap_or_default();
    let final_tail_stage_records = link_plan
        .map(|plan| {
            crate::workflow::nsld_final_executable_tail_stage_records_json(Path::new(
                &plan.output_dir,
            ))
        })
        .unwrap_or_default();
    let nsld_next = crate::workflow::nsld_next_action_summary(
        prepared_summary.as_ref(),
        final_tail_summary.as_ref(),
        final_output_summary.as_ref(),
    );
    let nsld_chain_next = crate::workflow::nsld_artifact_chain_next_action_mirror(
        prepared_summary.as_ref(),
        final_tail_summary.as_ref(),
    );
    let nsld_drive_recommendation = crate::workflow::nsld_drive_recommendation_for_output_dir(
        link_plan.map(|plan| Path::new(&plan.output_dir)),
        &nsld_chain_next,
        final_output_summary.as_ref(),
    );
    let nsld_drive_command_set = link_plan.map(|plan| {
        crate::workflow::nsld_drive_command_set_for_output_dir(Path::new(&plan.output_dir))
    });
    let domain_readiness = link_plan.map(link_plan_domain_readiness_summary);

    vec![
        crate::json_bool_field("link_plan_available", link_plan.is_some()),
        crate::json_optional_string_field(
            "link_plan_final_stage",
            link_plan.map(|plan| plan.final_stage.kind.as_str()),
        ),
        crate::json_optional_string_field(
            "link_plan_final_driver",
            link_plan.map(|plan| plan.final_stage.driver.as_str()),
        ),
        crate::json_optional_string_field(
            "link_plan_final_link_mode",
            link_plan.map(|plan| plan.final_stage.link_mode.as_str()),
        ),
        crate::json_optional_string_field(
            "link_plan_final_output",
            link_plan.map(|plan| plan.final_stage.output_path.as_str()),
        ),
        crate::json_optional_string_field(
            "link_plan_lowering_plan_index_path",
            link_plan.and_then(|plan| plan.lowering_plan_index_path.as_deref()),
        ),
        crate::json_optional_string_field(
            "link_plan_lowering_plan_index_source",
            link_plan.map(|plan| plan.lowering_plan_index_source.as_str()),
        ),
        crate::json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
        crate::json_usize_field(
            "link_plan_heterogeneous_domain_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.hetero_units)
                .unwrap_or(0),
        ),
        crate::json_usize_field(
            "link_plan_heterogeneous_domain_ready_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.ready_units)
                .unwrap_or(0),
        ),
        crate::json_bool_field(
            "link_plan_heterogeneous_domain_readiness_ready",
            domain_readiness
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        crate::json_string_array_field(
            "link_plan_heterogeneous_domain_families",
            &domain_readiness
                .as_ref()
                .map(|summary| summary.domain_families.clone())
                .unwrap_or_default(),
        ),
        crate::json_optional_string_field(
            "link_plan_heterogeneous_domain_first_unready",
            domain_readiness
                .as_ref()
                .and_then(|summary| summary.first_unready.as_deref()),
        ),
        format!(
            "\"link_plan_heterogeneous_domain_readiness\":[{}]",
            domain_readiness
                .as_ref()
                .map(link_plan_domain_readiness_units_json)
                .unwrap_or_default()
        ),
        crate::json_optional_string_field(
            "nsld_prepare_command",
            prepared_summary
                .as_ref()
                .map(|summary| summary.prepare_command.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_drive_dry_run_command",
            link_plan
                .map(|plan| {
                    crate::workflow::nsld_drive_dry_run_command_for_output_dir(Path::new(
                        &plan.output_dir,
                    ))
                })
                .as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_drive_dry_run_json_command",
            link_plan
                .map(|plan| {
                    crate::workflow::nsld_drive_dry_run_json_command_for_output_dir(Path::new(
                        &plan.output_dir,
                    ))
                })
                .as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_drive_apply_next_command",
            link_plan
                .map(|plan| {
                    crate::workflow::nsld_drive_apply_next_command_for_output_dir(Path::new(
                        &plan.output_dir,
                    ))
                })
                .as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_drive_apply_next_json_command",
            link_plan
                .map(|plan| {
                    crate::workflow::nsld_drive_apply_next_json_command_for_output_dir(Path::new(
                        &plan.output_dir,
                    ))
                })
                .as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_drive_apply_until_clean_command",
            link_plan
                .map(|plan| {
                    crate::workflow::nsld_drive_apply_until_clean_command_for_output_dir(Path::new(
                        &plan.output_dir,
                    ))
                })
                .as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_drive_apply_until_clean_json_command",
            link_plan
                .map(|plan| {
                    crate::workflow::nsld_drive_apply_until_clean_json_command_for_output_dir(
                        Path::new(&plan.output_dir),
                    )
                })
                .as_deref(),
        ),
        crate::workflow::nsld_drive_command_set_json_field(
            "nsld_drive_command_set",
            nsld_drive_command_set.as_ref(),
        ),
        crate::json_bool_field(
            "nsld_drive_recommended_available",
            nsld_drive_recommendation.available,
        ),
        crate::json_field(
            "nsld_drive_recommended_mode",
            &nsld_drive_recommendation.mode,
        ),
        crate::json_optional_string_field(
            "nsld_drive_recommended_command",
            nsld_drive_recommendation.command.as_deref(),
        ),
        crate::json_bool_field(
            "nsld_drive_recommended_mutates_artifacts",
            nsld_drive_recommendation.mutates_artifacts,
        ),
        crate::json_field(
            "nsld_drive_recommended_reason",
            &nsld_drive_recommendation.reason,
        ),
        crate::json_bool_field(
            "nsld_prepared_artifact_chain_ready",
            prepared_summary
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        crate::json_usize_field(
            "nsld_prepared_artifact_stage_count",
            prepared_summary
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        crate::json_usize_field(
            "nsld_prepared_artifact_present_count",
            prepared_summary
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        crate::json_optional_string_field(
            "nsld_prepared_artifact_next_missing_stage",
            prepared_summary
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        crate::json_object_array_field(
            "nsld_prepared_artifact_stage_records",
            &prepared_stage_records,
        ),
        crate::json_field("nsld_next_action_source", &nsld_next.source),
        crate::json_field("nsld_next_action", &nsld_next.action),
        crate::json_optional_string_field("nsld_next_action_command", nsld_next.command.as_deref()),
        crate::json_field("nsld_next_action_reason", &nsld_next.reason),
        crate::json_bool_field(
            "nsld_artifact_chain_next_action_available",
            nsld_chain_next.available,
        ),
        crate::json_optional_string_field(
            "nsld_artifact_chain_next_action_source",
            nsld_chain_next.source.as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_artifact_chain_next_action_command_id",
            nsld_chain_next.command_id.as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_artifact_chain_next_action_command",
            nsld_chain_next.command.as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_artifact_chain_next_action_command_resolved",
            nsld_chain_next.command_resolved.as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_artifact_chain_next_action_reason",
            nsld_chain_next.reason.as_deref(),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_command",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.pipeline_command.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_tail_ready",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        crate::json_usize_field(
            "nsld_final_executable_tail_stage_count",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        crate::json_usize_field(
            "nsld_final_executable_tail_present_count",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_tail_next_missing_stage",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        crate::json_object_array_field(
            "nsld_final_executable_tail_stage_records",
            &final_tail_stage_records,
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_valid",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.pipeline_valid),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_final_executable_emitted",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.final_executable_emitted),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_manifest_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.launcher_manifest_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_dry_run_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_would_enter_lifecycle_hook",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.would_enter_lifecycle_hook),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_blocker_count",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.blocker_count),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_first_blocker",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_contract",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_contract.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_execution_handoff_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_status",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_status.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_target",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_target.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_evidence_status.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_decision_code.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_payload_id",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_payload_id.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_scheduler_metadata_present",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_present),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_hash",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_hash.as_deref()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_count",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.required_stage_path_count),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_present_count",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.required_stage_path_present_count),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_pipeline_first_missing_required_stage_path",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.first_missing_required_stage_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_ready",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_ready),
        ),
        crate::json_optional_string_field(
            "nsld_self_owned_image_status",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.self_owned_image_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_entrypoint_materialization_status",
            final_tail_summary
                .as_ref()
                .map(|summary| summary.entrypoint_materialization_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_self_owned_image_path",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_path.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_present",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_present),
        ),
        crate::json_optional_string_field(
            "nsld_self_owned_image_hash",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_hash.as_deref()),
        ),
        json_optional_bool_field(
            "nsld_self_owned_image_header_valid",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.self_owned_image_header_valid),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_ready",
            final_output_summary
                .as_ref()
                .map(|summary| summary.ready)
                .unwrap_or(false),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_boundary_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.boundary_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_materialization_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.materialization_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_contract",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_contract.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_execution_handoff_ready",
            final_output_summary
                .as_ref()
                .is_some_and(|summary| summary.execution_handoff_ready),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_target",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_target.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_evidence_status",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_evidence_status.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_first_blocker",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.execution_handoff_first_blocker.as_deref()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_execution_handoff_decision_code",
            final_output_summary
                .as_ref()
                .map(|summary| summary.execution_handoff_decision_code.as_str()),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_recommended_next_action",
            final_output_summary
                .as_ref()
                .map(|summary| summary.recommended_next_action.as_str()),
        ),
        crate::json_bool_field(
            "nsld_final_executable_output_path_present",
            final_output_summary
                .as_ref()
                .map(|summary| summary.path_present)
                .unwrap_or(false),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_nsld_owned",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.nsld_owned),
        ),
        crate::json_usize_field(
            "nsld_final_executable_output_blocker_count",
            final_output_summary
                .as_ref()
                .map(|summary| summary.blockers.len())
                .unwrap_or(0),
        ),
        crate::json_string_array_field(
            "nsld_final_executable_output_blockers",
            final_output_summary
                .as_ref()
                .map(|summary| summary.blockers.as_slice())
                .unwrap_or(&[]),
        ),
        crate::json_optional_string_field(
            "nsld_final_executable_output_first_blocker",
            final_output_summary
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
    ]
}

fn link_plan_domain_readiness_summary(
    plan: &nuisc::linker::LinkPlan,
) -> LinkPlanDomainReadinessSummary {
    let units = plan
        .domain_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .map(link_plan_domain_readiness)
        .collect::<Vec<_>>();
    let hetero_units = units.len();
    let ready_units = units.iter().filter(|unit| unit.ready).count();
    let mut domain_families = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    domain_families.sort();
    let first_unready = units
        .iter()
        .find(|unit| !unit.ready)
        .map(|unit| format!("{}[{}]", unit.package_id, unit.domain_family));
    LinkPlanDomainReadinessSummary {
        hetero_units,
        ready_units,
        ready: hetero_units == ready_units,
        domain_families,
        first_unready,
        units,
    }
}

fn link_plan_domain_readiness(unit: &nuisc::linker::LinkPlanDomainUnit) -> LinkPlanDomainReadiness {
    let selected_lowering_target_present = unit.selected_lowering_target.is_some();
    let payload_blob_present = unit.artifact_payload_blob_path.is_some();
    let payload_format_present = unit.artifact_payload_format.is_some();
    let bridge_stub_present = unit.artifact_bridge_stub_path.is_some();
    let ir_sidecar_present = unit.artifact_ir_sidecar_path.is_some();
    let mut issues = Vec::new();
    if !payload_blob_present {
        issues.push("payload_blob_missing".to_owned());
    }
    if !payload_format_present {
        issues.push("payload_format_missing".to_owned());
    }
    if !bridge_stub_present {
        issues.push("bridge_stub_missing".to_owned());
    }
    LinkPlanDomainReadiness {
        package_id: unit.package_id.clone(),
        domain_family: unit.domain_family.clone(),
        ready: issues.is_empty(),
        selected_lowering_target_present,
        payload_blob_present,
        payload_format_present,
        bridge_stub_present,
        ir_sidecar_present,
        issues,
    }
}

fn link_plan_domain_readiness_units_json(summary: &LinkPlanDomainReadinessSummary) -> String {
    summary
        .units
        .iter()
        .map(link_plan_domain_readiness_json)
        .collect::<Vec<_>>()
        .join(",")
}

fn link_plan_domain_readiness_json(unit: &LinkPlanDomainReadiness) -> String {
    let fields = [
        crate::json_field("package_id", &unit.package_id),
        crate::json_field("domain_family", &unit.domain_family),
        crate::json_bool_field("ready", unit.ready),
        crate::json_bool_field(
            "selected_lowering_target_present",
            unit.selected_lowering_target_present,
        ),
        crate::json_bool_field("payload_blob_present", unit.payload_blob_present),
        crate::json_bool_field("payload_format_present", unit.payload_format_present),
        crate::json_bool_field("bridge_stub_present", unit.bridge_stub_present),
        crate::json_bool_field("ir_sidecar_present", unit.ir_sidecar_present),
        crate::json_string_array_field("issues", &unit.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(super) fn append_link_plan_json_fields(
    out: &mut String,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    append_json_field_strings(out, link_plan_json_fields(link_plan));
}

fn write_nsld_artifact_chain_text_fields<W: fmt::Write>(
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

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => crate::json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => crate::json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}
