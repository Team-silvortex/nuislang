use super::*;

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
        write_nsld_artifact_chain_text_fields(out, plan)?;
    } else {
        writeln!(out, "  link_plan_final_stage: <unavailable>")?;
        writeln!(out, "  link_plan_final_driver: <unavailable>")?;
        writeln!(out, "  link_plan_final_link_mode: <unavailable>")?;
        writeln!(out, "  link_plan_final_output: <unavailable>")?;
        writeln!(out, "  link_plan_lowering_plan_index_path: <unavailable>")?;
        writeln!(out, "  link_plan_lowering_plan_index_source: <unavailable>")?;
        writeln!(out, "  link_plan_domain_units: 0")?;
        writeln!(out, "  nsld_prepare_command: <unavailable>")?;
        writeln!(out, "  nsld_prepared_artifact_chain_ready: no")?;
        writeln!(out, "  nsld_prepared_artifact_stages: 0/0")?;
        writeln!(
            out,
            "  nsld_prepared_artifact_next_missing_stage: <unavailable>"
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
            "  nsld_final_executable_pipeline_blocker_count: <unavailable>"
        )?;
        writeln!(
            out,
            "  nsld_final_executable_pipeline_first_blocker: <none>"
        )?;
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
        crate::json_optional_string_field(
            "nsld_prepare_command",
            prepared_summary
                .as_ref()
                .map(|summary| summary.prepare_command.as_str()),
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
        json_optional_bool_field(
            "nsld_final_executable_pipeline_valid",
            final_tail_summary
                .as_ref()
                .and_then(|summary| summary.pipeline_valid),
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
    ]
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
    writeln!(out, "  nsld_prepare_command: {}", prepared.prepare_command)?;
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
