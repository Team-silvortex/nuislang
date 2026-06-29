use crate::{
    artifact_lowering_units_json, json_bool_field, json_optional_string_field,
    json_string_array_field, json_string_field, json_usize_field, linker,
};

pub(crate) fn artifact_lowering_alignment_check_json(
    check: &linker::ArtifactLoweringAlignmentCheck,
) -> String {
    let fields = vec![
        json_usize_field("index", check.index),
        json_string_field("package_id", &check.package_id),
        json_string_field("domain_family", &check.domain_family),
        json_bool_field("consistent", check.consistent),
        json_string_array_field("issues", &check.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn artifact_lowering_alignment_summary_json(
    summary: &linker::ArtifactLoweringAlignmentSummary,
) -> String {
    let checks = summary
        .checks
        .iter()
        .map(artifact_lowering_alignment_check_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_usize_field("checked", summary.checked),
        json_usize_field("mismatches", summary.mismatches),
        json_bool_field("consistent", summary.consistent),
        format!("\"checks\":[{}]", checks),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn link_plan_domain_unit_json(unit: &linker::LinkPlanDomainUnit) -> String {
    let mut fields = vec![
        json_string_field("kind", &unit.kind),
        json_string_field("package_id", &unit.package_id),
        json_string_field("domain_family", &unit.domain_family),
        json_string_field("contract_family", &unit.contract_family),
        json_string_field("packaging_role", &unit.packaging_role),
    ];
    if let Some(value) = unit.abi.as_deref() {
        fields.push(json_string_field("abi", value));
    }
    if let Some(value) = unit.backend_family.as_deref() {
        fields.push(json_string_field("backend_family", value));
    }
    if let Some(value) = unit.vendor.as_deref() {
        fields.push(json_string_field("vendor", value));
    }
    if let Some(value) = unit.device_class.as_deref() {
        fields.push(json_string_field("device_class", value));
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        fields.push(json_string_field("selected_lowering_target", value));
    }
    if let Some(value) = unit.artifact_ir_sidecar_path.as_deref() {
        fields.push(json_string_field("artifact_ir_sidecar_path", value));
    }
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn link_plan_json(plan: &linker::LinkPlan) -> String {
    let domain_units = plan
        .domain_units
        .iter()
        .map(link_plan_domain_unit_json)
        .collect::<Vec<_>>()
        .join(",");
    let mut fields = vec![
        json_string_field("schema", &plan.schema),
        json_string_field("packaging_mode", &plan.packaging_mode),
        json_string_field("final_stage_kind", &plan.final_stage.kind),
        json_string_field("final_stage_driver", &plan.final_stage.driver),
        json_string_field("final_stage_link_mode", &plan.final_stage.link_mode),
        json_string_field("final_stage_output", &plan.final_stage.output_path),
        json_string_array_field("final_stage_inputs", &plan.final_stage.inputs),
        json_string_array_field("final_stage_notes", &plan.final_stage.notes),
        json_optional_string_field(
            "artifact_container_kind",
            plan.compiled_artifact.container_kind.as_deref(),
        ),
        match plan.compiled_artifact.container_version {
            Some(version) => format!("\"artifact_container_version\":{}", version),
            None => "\"artifact_container_version\":null".to_owned(),
        },
        match plan.compiled_artifact.section_count {
            Some(count) => json_usize_field("artifact_section_count", count),
            None => "\"artifact_section_count\":null".to_owned(),
        },
        json_string_array_field(
            "artifact_section_names",
            &plan.compiled_artifact.section_names,
        ),
        match plan.compiled_artifact.section_table_valid {
            Some(valid) => json_bool_field("artifact_section_table_valid", valid),
            None => "\"artifact_section_table_valid\":null".to_owned(),
        },
        match plan.compiled_artifact.lowering_unit_count {
            Some(count) => json_usize_field("lowering_unit_count", count),
            None => "\"lowering_unit_count\":null".to_owned(),
        },
        json_string_array_field(
            "lowering_domain_families",
            &plan.compiled_artifact.lowering_domain_families,
        ),
        json_string_array_field("lowering_targets", &plan.compiled_artifact.lowering_targets),
        artifact_lowering_units_json(&plan.compiled_artifact.lowering_units),
        format!(
            "\"artifact_lowering_alignment\":{}",
            artifact_lowering_alignment_summary_json(&plan.artifact_lowering_alignment)
        ),
        json_usize_field("domain_unit_count", plan.domain_units.len()),
        format!("\"domain_units\":[{}]", domain_units),
    ];
    if let Some(path) = &plan.bridge_registry_path {
        fields.push(json_string_field("bridge_registry_path", path));
    }
    if let Some(path) = &plan.host_bridge_plan_index_path {
        fields.push(json_string_field("host_bridge_plan_index_path", path));
    }
    if let Some(path) = &plan.lowering_plan_index_path {
        fields.push(json_string_field("lowering_plan_index_path", path));
    }
    format!("{{{}}}", fields.join(","))
}
