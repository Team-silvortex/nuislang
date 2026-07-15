use super::{
    link_plan_readiness::{
        link_plan_domain_readiness_summary, link_plan_domain_readiness_units_json,
    },
    *,
};

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
            "  link_plan_heterogeneous_domain_registry_dispatch_ready_units: {}",
            domain_readiness.registry_dispatch_ready_units
        )?;
        writeln!(
            out,
            "  link_plan_heterogeneous_backend_artifact_ready_units: {}/{}",
            domain_readiness.backend_artifact_ready_units, domain_readiness.backend_artifact_units
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
        writeln!(
            out,
            "  link_plan_heterogeneous_backend_artifact_first_unready: {}",
            domain_readiness
                .backend_artifact_first_unready
                .as_deref()
                .unwrap_or("<none>")
        )?;
        writeln!(
            out,
            "  link_plan_heterogeneous_domain_registry_dispatch_first_blocked: {}",
            domain_readiness
                .registry_dispatch_first_blocked
                .as_deref()
                .unwrap_or("<none>")
        )?;
        super::link_plan_text::write_nsld_artifact_chain_text_fields(out, plan)?;
    } else {
        super::link_plan_text::write_unavailable_link_plan_text_fields(out)?;
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

    let mut fields = vec![
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
        crate::json_usize_field(
            "link_plan_heterogeneous_domain_registry_dispatch_ready_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.registry_dispatch_ready_units)
                .unwrap_or(0),
        ),
        crate::json_usize_field(
            "link_plan_heterogeneous_backend_artifact_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.backend_artifact_units)
                .unwrap_or(0),
        ),
        crate::json_usize_field(
            "link_plan_heterogeneous_backend_artifact_ready_units",
            domain_readiness
                .as_ref()
                .map(|summary| summary.backend_artifact_ready_units)
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
        crate::json_string_array_field(
            "link_plan_heterogeneous_backend_families",
            &domain_readiness
                .as_ref()
                .map(|summary| summary.backend_families.clone())
                .unwrap_or_default(),
        ),
        crate::json_string_array_field(
            "link_plan_heterogeneous_target_devices",
            &domain_readiness
                .as_ref()
                .map(|summary| summary.target_devices.clone())
                .unwrap_or_default(),
        ),
        crate::json_optional_string_field(
            "link_plan_heterogeneous_domain_first_unready",
            domain_readiness
                .as_ref()
                .and_then(|summary| summary.first_unready.as_deref()),
        ),
        crate::json_optional_string_field(
            "link_plan_heterogeneous_backend_artifact_first_unready",
            domain_readiness
                .as_ref()
                .and_then(|summary| summary.backend_artifact_first_unready.as_deref()),
        ),
        crate::json_optional_string_field(
            "link_plan_heterogeneous_domain_registry_dispatch_first_blocked",
            domain_readiness
                .as_ref()
                .and_then(|summary| summary.registry_dispatch_first_blocked.as_deref()),
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
    ];
    fields.extend(super::link_plan_nsld_tail::nsld_tail_json_fields(
        final_tail_summary.as_ref(),
        &final_tail_stage_records,
        final_output_summary.as_ref(),
    ));
    fields
}

pub(super) fn append_link_plan_json_fields(
    out: &mut String,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    append_json_field_strings(out, link_plan_json_fields(link_plan));
}
