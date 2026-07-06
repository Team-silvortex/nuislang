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
    if let Some(value) = unit.target_device.as_deref() {
        fields.push(json_string_field("target_device", value));
    }
    if let Some(value) = unit.ir_format.as_deref() {
        fields.push(json_string_field("ir_format", value));
    }
    if let Some(value) = unit.dispatch_abi.as_deref() {
        fields.push(json_string_field("dispatch_abi", value));
    }
    if let Some(value) = unit.backend_priority {
        fields.push(json_usize_field("backend_priority", value));
    }
    if let Some(value) = unit.verification.as_deref() {
        fields.push(json_string_field("verification", value));
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        fields.push(json_string_field("selected_lowering_target", value));
    }
    if let Some(value) = unit.artifact_ir_sidecar_path.as_deref() {
        fields.push(json_string_field("artifact_ir_sidecar_path", value));
    }
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn link_plan_hetero_node_json(node: &linker::LinkPlanHeteroNode) -> String {
    let fields = vec![
        json_usize_field("index", node.index),
        json_string_field("timestamp", &node.timestamp),
        json_string_field("domain_family", &node.domain_family),
        json_string_field("package_id", &node.package_id),
        json_string_field("lifecycle_hook", &node.lifecycle_hook),
        json_string_array_field("wait_on", &node.wait_on),
        json_string_array_field("emits", &node.emits),
        json_string_field("link_input", &node.link_input),
        json_bool_field("c_world_wrapper", node.c_world_wrapper),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn link_plan_data_segment_json(segment: &linker::LinkPlanDataSegment) -> String {
    let fields = vec![
        json_usize_field("index", segment.index),
        json_string_field("segment_id", &segment.segment_id),
        json_string_field("domain_family", &segment.domain_family),
        json_string_field("owner_package", &segment.owner_package),
        json_string_field("order_key", &segment.order_key),
        json_string_field("access_phase", &segment.access_phase),
        json_optional_string_field("source_path", segment.source_path.as_deref()),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn link_plan_hetero_validation_json(
    validation: &linker::LinkPlanHeteroValidationSummary,
) -> String {
    let fields = vec![
        json_usize_field("checked", validation.checked),
        json_bool_field("valid", validation.valid),
        json_string_array_field("issues", &validation.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn link_plan_hetero_calculate_json(plan: &linker::LinkPlanHeteroCalculate) -> String {
    let nodes = plan
        .nodes
        .iter()
        .map(link_plan_hetero_node_json)
        .collect::<Vec<_>>()
        .join(",");
    let segments = plan
        .data_segments
        .iter()
        .map(link_plan_data_segment_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("schema", &plan.schema),
        json_string_field("mode", &plan.mode),
        json_bool_field("static_link", plan.static_link),
        json_bool_field("lifecycle_driven", plan.lifecycle_driven),
        json_string_field("time_order_model", &plan.time_order_model),
        json_string_field("data_order_model", &plan.data_order_model),
        json_string_field("c_world_policy", &plan.c_world_policy),
        format!(
            "\"validation\":{}",
            link_plan_hetero_validation_json(&plan.validation)
        ),
        format!("\"nodes\":[{}]", nodes),
        format!("\"data_segments\":[{}]", segments),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn link_plan_host_ffi_json(footprint: &linker::LinkPlanHostFfiFootprint) -> String {
    let entries = footprint
        .entries
        .iter()
        .map(link_plan_host_ffi_entry_json)
        .collect::<Vec<_>>()
        .join(",");
    let abi_groups = footprint
        .abi_groups
        .iter()
        .map(link_plan_host_ffi_abi_group_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_optional_string_field("index_path", footprint.index_path.as_deref()),
        json_usize_field("symbol_count", footprint.symbol_count),
        json_usize_field("policy_count", footprint.policy_count),
        json_string_field("policy", &footprint.policy),
        format!(
            "\"validation\":{}",
            link_plan_host_ffi_validation_json(&footprint.validation)
        ),
        format!("\"abi_groups\":[{}]", abi_groups),
        format!("\"entries\":[{}]", entries),
    ];
    format!("{{{}}}", fields.join(","))
}

fn link_plan_host_ffi_abi_group_json(group: &linker::LinkPlanHostFfiAbiGroup) -> String {
    let entries = group
        .entries
        .iter()
        .map(link_plan_host_ffi_abi_entry_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("abi", &group.abi),
        json_usize_field("symbol_count", group.symbol_count),
        json_usize_field("policy_count", group.policy_count),
        json_string_array_field("symbols", &group.symbols),
        format!(
            "\"validation\":{}",
            link_plan_host_ffi_validation_json(&group.validation)
        ),
        format!("\"entries\":[{}]", entries),
    ];
    format!("{{{}}}", fields.join(","))
}

fn link_plan_host_ffi_abi_entry_json(entry: &linker::LinkPlanHostFfiAbiEntry) -> String {
    let fields = vec![
        json_string_field("symbol", &entry.symbol),
        json_string_field("signature_pattern", &entry.signature_pattern),
        json_string_field("signature_hash", &entry.signature_hash),
        json_string_field("policy", &entry.policy),
    ];
    format!("{{{}}}", fields.join(","))
}

fn link_plan_host_ffi_validation_json(
    validation: &linker::LinkPlanHostFfiValidationSummary,
) -> String {
    let fields = vec![
        json_usize_field("checked", validation.checked),
        json_bool_field("valid", validation.valid),
        json_bool_field("link_allowed", validation.link_allowed),
        json_string_array_field("issues", &validation.issues),
        json_string_array_field("notes", &validation.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

fn link_plan_host_ffi_entry_json(entry: &linker::LinkPlanHostFfiEntry) -> String {
    let fields = vec![
        json_string_field("abi", &entry.abi),
        json_string_field("symbol", &entry.symbol),
        json_string_field("signature_pattern", &entry.signature_pattern),
        json_string_field("signature_hash", &entry.signature_hash),
        json_string_field("policy", &entry.policy),
    ];
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
        format!(
            "\"hetero_calculate\":{}",
            link_plan_hetero_calculate_json(&plan.hetero_calculate)
        ),
        format!("\"host_ffi\":{}", link_plan_host_ffi_json(&plan.host_ffi)),
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
