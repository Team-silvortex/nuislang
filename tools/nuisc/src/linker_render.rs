use super::*;

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
            "domain_unit: kind={} domain={} package={} lowering={} backend={} role={}",
            unit.kind,
            unit.domain_family,
            unit.package_id,
            unit.selected_lowering_target.as_deref().unwrap_or("none"),
            unit.backend_family.as_deref().unwrap_or("none"),
            unit.packaging_role
        ));
    }
    lines
}
