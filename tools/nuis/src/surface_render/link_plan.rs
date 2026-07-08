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
        writeln!(out, "  link_plan_domain_units: {}", plan.domain_units.len())?;
    } else {
        writeln!(out, "  link_plan_final_stage: <unavailable>")?;
        writeln!(out, "  link_plan_final_driver: <unavailable>")?;
        writeln!(out, "  link_plan_final_link_mode: <unavailable>")?;
        writeln!(out, "  link_plan_final_output: <unavailable>")?;
        writeln!(out, "  link_plan_lowering_plan_index_path: <unavailable>")?;
        writeln!(out, "  link_plan_domain_units: 0")?;
    }
    Ok(())
}

pub(super) fn link_plan_json_fields(link_plan: Option<&nuisc::linker::LinkPlan>) -> Vec<String> {
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
        crate::json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
    ]
}

pub(super) fn append_link_plan_json_fields(
    out: &mut String,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    append_json_field_strings(out, link_plan_json_fields(link_plan));
}
