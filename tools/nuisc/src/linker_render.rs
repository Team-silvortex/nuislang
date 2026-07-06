use super::*;
use crate::{
    json_bool_field, json_optional_string_field, json_string_array_field, json_string_field,
    json_usize_field,
};

pub fn render_link_plan_summary(plan: &LinkPlan) -> Vec<String> {
    let mut lines = vec![
        format!("schema: {}", plan.schema),
        format!("input: {}", plan.input),
        format!("output_dir: {}", plan.output_dir),
        format!("packaging_mode: {}", plan.packaging_mode),
        format!(
            "cpu_target: abi={} arch={} os={} object={} calling={} clang={} cross={}",
            plan.cpu_target.abi,
            plan.cpu_target.machine_arch,
            plan.cpu_target.machine_os,
            plan.cpu_target.object_format,
            plan.cpu_target.calling_abi,
            plan.cpu_target.clang_target,
            plan.cpu_target.cross_compile
        ),
        format!(
            "envelope: schema={} packages={} families={} domains={}",
            plan.envelope.schema,
            plan.envelope.package_count,
            plan.envelope.contract_families.join(","),
            plan.envelope.domain_families.join(",")
        ),
        format!(
            "artifact: path={} binary={} bytes={}",
            plan.compiled_artifact.path,
            plan.compiled_artifact.binary_path,
            plan.compiled_artifact.binary_bytes
        ),
        format!(
            "final_stage: kind={} driver={} link_mode={} output={}",
            plan.final_stage.kind,
            plan.final_stage.driver,
            plan.final_stage.link_mode,
            plan.final_stage.output_path
        ),
    ];
    if let Some(path) = &plan.bridge_registry_path {
        lines.push(format!("bridge_registry: {path}"));
    }
    if let Some(path) = &plan.host_bridge_plan_index_path {
        lines.push(format!("host_bridge_plan_index: {path}"));
    }
    if let Some(path) = &plan.lowering_plan_index_path {
        lines.push(format!("lowering_plan_index: {path}"));
    }
    lines.push(format!(
        "host_ffi: symbols={} policies={} policy={} validation={} index={}",
        plan.host_ffi.symbol_count,
        plan.host_ffi.policy_count,
        plan.host_ffi.policy,
        if plan.host_ffi.validation.valid {
            "valid"
        } else {
            "invalid"
        },
        plan.host_ffi.index_path.as_deref().unwrap_or("none")
    ));
    for group in &plan.host_ffi.abi_groups {
        lines.push(format!(
            "host_ffi_abi: abi={} symbols={} policies={} valid={} entries={}",
            group.abi,
            group.symbol_count,
            group.policy_count,
            group.validation.valid,
            group.symbols.join(",")
        ));
    }
    if let Some(kind) = &plan.compiled_artifact.container_kind {
        lines.push(format!(
            "artifact_container: kind={} version={} sections={} valid={}",
            kind,
            plan.compiled_artifact
                .container_version
                .map(|version| version.to_string())
                .unwrap_or_else(|| "unknown".to_owned()),
            plan.compiled_artifact
                .section_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unknown".to_owned()),
            plan.compiled_artifact
                .section_table_valid
                .map(|valid| valid.to_string())
                .unwrap_or_else(|| "unknown".to_owned())
        ));
    }
    lines.push(format!(
        "artifact_lowering_alignment: checked={} mismatches={} consistent={}",
        plan.artifact_lowering_alignment.checked,
        plan.artifact_lowering_alignment.mismatches,
        plan.artifact_lowering_alignment.consistent
    ));
    lines.push(format!(
        "clock_protocol: schema={} mode={} source={} default_time={} lifecycle_tick={} domains={} edges={}",
        plan.clock_protocol.schema,
        plan.clock_protocol.mode,
        plan.clock_protocol.source,
        plan.clock_protocol.default_time_mode,
        plan.clock_protocol.lifecycle_tick_policy,
        plan.clock_protocol.domains.len(),
        plan.clock_protocol.edges.len()
    ));
    lines.push(format!(
        "clock_validation: checked={} valid={} issues={}",
        plan.clock_protocol.validation.checked,
        plan.clock_protocol.validation.valid,
        if plan.clock_protocol.validation.issues.is_empty() {
            "none".to_owned()
        } else {
            plan.clock_protocol.validation.issues.join(";")
        }
    ));
    for domain in &plan.clock_protocol.domains {
        lines.push(format!(
            "clock_domain: index={} domain={} package={} clock={} kind={} epoch={} resolution={} bridge={} hook={}",
            domain.index,
            domain.domain_family,
            domain.package_id,
            domain.clock_domain_id,
            domain.clock_kind,
            domain.clock_epoch_kind,
            domain.clock_resolution,
            domain.clock_bridge_default,
            domain.lifecycle_hook
        ));
    }
    for edge in &plan.clock_protocol.edges {
        lines.push(format!(
            "clock_edge: index={} from={} to={} relation={} source={}",
            edge.index, edge.from, edge.to, edge.relation, edge.source
        ));
    }
    lines.push(format!(
        "hetero_calculate: schema={} mode={} static_link={} lifecycle_driven={} time_order={} data_order={} c_world={}",
        plan.hetero_calculate.schema,
        plan.hetero_calculate.mode,
        plan.hetero_calculate.static_link,
        plan.hetero_calculate.lifecycle_driven,
        plan.hetero_calculate.time_order_model,
        plan.hetero_calculate.data_order_model,
        plan.hetero_calculate.c_world_policy
    ));
    lines.push(format!(
        "hetero_validation: checked={} valid={} issues={}",
        plan.hetero_calculate.validation.checked,
        plan.hetero_calculate.validation.valid,
        if plan.hetero_calculate.validation.issues.is_empty() {
            "none".to_owned()
        } else {
            plan.hetero_calculate.validation.issues.join(";")
        }
    ));
    for node in &plan.hetero_calculate.nodes {
        lines.push(format!(
            "hetero_node: index={} timestamp={} domain={} package={} hook={} wait_on={} emits={} input={} c_wrapper={}",
            node.index,
            node.timestamp,
            node.domain_family,
            node.package_id,
            node.lifecycle_hook,
            node.wait_on.join(","),
            node.emits.join(","),
            node.link_input,
            node.c_world_wrapper
        ));
    }
    for segment in &plan.hetero_calculate.data_segments {
        lines.push(format!(
            "data_segment: index={} id={} domain={} owner={} order={} phase={} source={}",
            segment.index,
            segment.segment_id,
            segment.domain_family,
            segment.owner_package,
            segment.order_key,
            segment.access_phase,
            segment.source_path.as_deref().unwrap_or("none")
        ));
    }
    for unit in &plan.domain_units {
        lines.push(format!(
            "domain_unit: kind={} domain={} package={} lowering={} backend={} target_device={} ir_format={} dispatch_abi={} backend_priority={} verification={} role={}",
            unit.kind,
            unit.domain_family,
            unit.package_id,
            unit.selected_lowering_target.as_deref().unwrap_or("none"),
            unit.backend_family.as_deref().unwrap_or("none"),
            unit.target_device.as_deref().unwrap_or("none"),
            unit.ir_format.as_deref().unwrap_or("none"),
            unit.dispatch_abi.as_deref().unwrap_or("none"),
            unit.backend_priority
                .map(|priority| priority.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            unit.verification.as_deref().unwrap_or("none"),
            unit.packaging_role
        ));
    }
    lines
}

pub fn render_link_plan_json(plan: &LinkPlan) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("toolchain_phase", "alpha-0.6.0-linker-boundary"),
        json_string_field("schema", &plan.schema),
        json_string_field("input", &plan.input),
        json_string_field("output_dir", &plan.output_dir),
        json_string_field("packaging_mode", &plan.packaging_mode),
        json_string_field("final_stage_kind", &plan.final_stage.kind),
        json_string_field("final_stage_driver", &plan.final_stage.driver),
        json_string_field("final_stage_link_mode", &plan.final_stage.link_mode),
        json_string_field("final_stage_output", &plan.final_stage.output_path),
        json_string_array_field("final_stage_inputs", &plan.final_stage.inputs),
        json_string_array_field("domain_families", &plan.envelope.domain_families),
        json_usize_field("domain_unit_count", plan.domain_units.len()),
        json_optional_string_field("host_ffi_index_path", plan.host_ffi.index_path.as_deref()),
        json_usize_field("host_ffi_symbol_count", plan.host_ffi.symbol_count),
        json_usize_field("host_ffi_policy_count", plan.host_ffi.policy_count),
        json_string_field("host_ffi_policy", &plan.host_ffi.policy),
        json_usize_field(
            "host_ffi_validation_checked",
            plan.host_ffi.validation.checked,
        ),
        json_bool_field("host_ffi_validation_valid", plan.host_ffi.validation.valid),
        json_bool_field(
            "host_ffi_link_allowed",
            plan.host_ffi.validation.link_allowed,
        ),
        json_string_array_field(
            "host_ffi_validation_issues",
            &plan.host_ffi.validation.issues,
        ),
        json_string_array_field("host_ffi_validation_notes", &plan.host_ffi.validation.notes),
        format!(
            "\"host_ffi_abi_groups\":[{}]",
            plan.host_ffi
                .abi_groups
                .iter()
                .map(render_host_ffi_abi_group_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!(
            "\"host_ffi_entries\":[{}]",
            plan.host_ffi
                .entries
                .iter()
                .map(render_host_ffi_entry_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_optional_string_field(
            "artifact_container_kind",
            plan.compiled_artifact.container_kind.as_deref(),
        ),
        json_usize_field(
            "artifact_lowering_alignment_checked",
            plan.artifact_lowering_alignment.checked,
        ),
        json_bool_field(
            "artifact_lowering_alignment_consistent",
            plan.artifact_lowering_alignment.consistent,
        ),
        json_string_field("clock_protocol_schema", &plan.clock_protocol.schema),
        json_string_field("clock_protocol_mode", &plan.clock_protocol.mode),
        json_usize_field("clock_protocol_domains", plan.clock_protocol.domains.len()),
        json_usize_field("clock_protocol_edges", plan.clock_protocol.edges.len()),
        json_bool_field("clock_protocol_valid", plan.clock_protocol.validation.valid),
        json_string_field("hetero_calculate_schema", &plan.hetero_calculate.schema),
        json_bool_field(
            "hetero_calculate_static_link",
            plan.hetero_calculate.static_link,
        ),
        json_bool_field(
            "hetero_calculate_lifecycle_driven",
            plan.hetero_calculate.lifecycle_driven,
        ),
        json_string_field(
            "hetero_calculate_time_order_model",
            &plan.hetero_calculate.time_order_model,
        ),
        json_string_field(
            "hetero_calculate_data_order_model",
            &plan.hetero_calculate.data_order_model,
        ),
        json_bool_field(
            "hetero_calculate_valid",
            plan.hetero_calculate.validation.valid,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn render_host_ffi_abi_group_json(group: &LinkPlanHostFfiAbiGroup) -> String {
    let entries = group
        .entries
        .iter()
        .map(render_host_ffi_abi_entry_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("abi", &group.abi),
        json_usize_field("symbol_count", group.symbol_count),
        json_usize_field("policy_count", group.policy_count),
        json_string_array_field("symbols", &group.symbols),
        format!(
            "\"validation\":{}",
            render_host_ffi_validation_json(&group.validation)
        ),
        format!("\"entries\":[{}]", entries),
    ];
    format!("{{{}}}", fields.join(","))
}

fn render_host_ffi_validation_json(validation: &LinkPlanHostFfiValidationSummary) -> String {
    let fields = vec![
        json_usize_field("checked", validation.checked),
        json_bool_field("valid", validation.valid),
        json_bool_field("link_allowed", validation.link_allowed),
        json_string_array_field("issues", &validation.issues),
        json_string_array_field("notes", &validation.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

fn render_host_ffi_abi_entry_json(entry: &LinkPlanHostFfiAbiEntry) -> String {
    let fields = vec![
        json_string_field("symbol", &entry.symbol),
        json_string_field("signature_pattern", &entry.signature_pattern),
        json_string_field("signature_hash", &entry.signature_hash),
        json_string_field("policy", &entry.policy),
    ];
    format!("{{{}}}", fields.join(","))
}

fn render_host_ffi_entry_json(entry: &LinkPlanHostFfiEntry) -> String {
    let fields = vec![
        json_string_field("abi", &entry.abi),
        json_string_field("symbol", &entry.symbol),
        json_string_field("signature_pattern", &entry.signature_pattern),
        json_string_field("signature_hash", &entry.signature_hash),
        json_string_field("policy", &entry.policy),
    ];
    format!("{{{}}}", fields.join(","))
}
