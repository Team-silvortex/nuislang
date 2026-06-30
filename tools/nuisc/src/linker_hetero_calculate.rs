use super::{
    LinkPlanDataSegment, LinkPlanDomainUnit, LinkPlanHeteroCalculate, LinkPlanHeteroNode,
    LinkPlanHeteroValidationSummary, LinkPlanLifecycle,
};
use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn derive_hetero_calculate_plan(
    lifecycle: &LinkPlanLifecycle,
    domain_units: &[LinkPlanDomainUnit],
) -> LinkPlanHeteroCalculate {
    let mut hetero_units = domain_units
        .iter()
        .filter(|unit| unit.kind == "heterogeneous")
        .collect::<Vec<_>>();
    hetero_units.sort_by(|left, right| {
        (
            left.domain_family.as_str(),
            left.selected_lowering_target.as_deref().unwrap_or(""),
            left.package_id.as_str(),
        )
            .cmp(&(
                right.domain_family.as_str(),
                right.selected_lowering_target.as_deref().unwrap_or(""),
                right.package_id.as_str(),
            ))
    });

    let nodes = hetero_units
        .iter()
        .enumerate()
        .map(|(index, unit)| {
            let timestamp = format!("t{:04}.{}", index + 1, unit.domain_family);
            let wait_on = if index == 0 {
                vec![format!("t0000.{}", lifecycle.bootstrap_entry)]
            } else {
                vec![format!(
                    "t{:04}.{}",
                    index,
                    hetero_units[index - 1].domain_family
                )]
            };
            LinkPlanHeteroNode {
                index,
                timestamp: timestamp.clone(),
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                lifecycle_hook: lifecycle_hook_for_domain(unit),
                wait_on,
                emits: vec![
                    format!("{timestamp}.submit"),
                    format!("{timestamp}.complete"),
                    format!("{timestamp}.data_commit"),
                ],
                link_input: unit
                    .artifact_payload_blob_path
                    .clone()
                    .or_else(|| unit.artifact_payload_path.clone())
                    .or_else(|| unit.artifact_ir_sidecar_path.clone())
                    .unwrap_or_else(|| "<embedded-domain-sidecar>".to_owned()),
                c_world_wrapper: uses_c_world_wrapper(unit),
            }
        })
        .collect::<Vec<_>>();

    let data_segments = hetero_units
        .iter()
        .enumerate()
        .map(|(index, unit)| LinkPlanDataSegment {
            index,
            segment_id: format!("seg{:04}.{}", index + 1, unit.domain_family),
            domain_family: unit.domain_family.clone(),
            owner_package: unit.package_id.clone(),
            order_key: format!("data:{:04}:{}", index + 1, unit.domain_family),
            access_phase: if unit.domain_family == "network" {
                "recv-finalize".to_owned()
            } else {
                "bind-submit-wait-finalize".to_owned()
            },
            source_path: unit
                .artifact_payload_blob_path
                .clone()
                .or_else(|| unit.artifact_payload_path.clone()),
        })
        .collect::<Vec<_>>();

    let mut plan = LinkPlanHeteroCalculate {
        schema: "nuis-hetero-calculate-link-plan-v1".to_owned(),
        mode: if hetero_units.is_empty() {
            "host-only".to_owned()
        } else {
            "heterogeneous-static-lifecycle".to_owned()
        },
        static_link: true,
        lifecycle_driven: true,
        time_order_model: "timestamped-partial-order".to_owned(),
        data_order_model: "deterministic-segment-order".to_owned(),
        c_world_policy: "wrapped-ordinary-node-no-linker-fast-path".to_owned(),
        nodes,
        data_segments,
        validation: LinkPlanHeteroValidationSummary {
            checked: 0,
            valid: true,
            issues: Vec::new(),
        },
    };
    plan.validation = validate_hetero_calculate_plan(&plan);
    plan
}

pub(crate) fn validate_hetero_calculate_plan(
    plan: &LinkPlanHeteroCalculate,
) -> LinkPlanHeteroValidationSummary {
    let mut checked = 0;
    let mut issues = Vec::new();
    checked += 1;
    if !plan.static_link {
        issues.push("hetero calculate linker must be fully static".to_owned());
    }
    checked += 1;
    if !plan.lifecycle_driven {
        issues.push("hetero calculate linker must be lifecycle hook driven".to_owned());
    }
    checked += 1;
    if plan.time_order_model != "timestamped-partial-order" {
        issues.push(format!(
            "unexpected time order model `{}`",
            plan.time_order_model
        ));
    }
    checked += 1;
    if plan.data_order_model != "deterministic-segment-order" {
        issues.push(format!(
            "unexpected data order model `{}`",
            plan.data_order_model
        ));
    }
    checked += 1;
    if plan.c_world_policy != "wrapped-ordinary-node-no-linker-fast-path" {
        issues.push(format!(
            "unexpected C world policy `{}`",
            plan.c_world_policy
        ));
    }
    checked += 1;
    if plan.nodes.len() != plan.data_segments.len() {
        issues.push(format!(
            "node/data segment count mismatch: nodes={}, segments={}",
            plan.nodes.len(),
            plan.data_segments.len()
        ));
    }

    for (expected_index, node) in plan.nodes.iter().enumerate() {
        checked += 1;
        if node.index != expected_index {
            issues.push(format!(
                "node `{}` index mismatch: expected {}, found {}",
                node.timestamp, expected_index, node.index
            ));
        }
        let expected_timestamp = format!("t{:04}.{}", expected_index + 1, node.domain_family);
        checked += 1;
        if node.timestamp != expected_timestamp {
            issues.push(format!(
                "node timestamp mismatch: expected `{}`, found `{}`",
                expected_timestamp, node.timestamp
            ));
        }
        let expected_wait = if expected_index == 0 {
            "t0000.".to_owned()
        } else {
            plan.nodes[expected_index - 1].timestamp.clone()
        };
        checked += 1;
        if expected_index == 0 {
            if !node
                .wait_on
                .iter()
                .any(|wait| wait.starts_with(expected_wait.as_str()))
            {
                issues.push(format!(
                    "first node `{}` must wait on lifecycle bootstrap",
                    node.timestamp
                ));
            }
        } else if node.wait_on != vec![expected_wait.clone()] {
            issues.push(format!(
                "node `{}` wait_on mismatch: expected `{}`",
                node.timestamp, expected_wait
            ));
        }
        checked += 1;
        for suffix in ["submit", "complete", "data_commit"] {
            let expected_emit = format!("{}.{}", node.timestamp, suffix);
            if !node.emits.contains(&expected_emit) {
                issues.push(format!(
                    "node `{}` missing emit `{}`",
                    node.timestamp, expected_emit
                ));
            }
        }
        checked += 1;
        if node.link_input.is_empty() {
            issues.push(format!("node `{}` has empty link input", node.timestamp));
        }
    }

    for (expected_index, segment) in plan.data_segments.iter().enumerate() {
        checked += 1;
        if segment.index != expected_index {
            issues.push(format!(
                "segment `{}` index mismatch: expected {}, found {}",
                segment.segment_id, expected_index, segment.index
            ));
        }
        let expected_order = format!("data:{:04}:{}", expected_index + 1, segment.domain_family);
        checked += 1;
        if segment.order_key != expected_order {
            issues.push(format!(
                "segment `{}` order mismatch: expected `{}`, found `{}`",
                segment.segment_id, expected_order, segment.order_key
            ));
        }
        checked += 1;
        if let Some(node) = plan.nodes.get(expected_index) {
            if segment.domain_family != node.domain_family
                || segment.owner_package != node.package_id
            {
                issues.push(format!(
                    "segment `{}` does not align with node `{}`",
                    segment.segment_id, node.timestamp
                ));
            }
        }
    }

    LinkPlanHeteroValidationSummary {
        checked,
        valid: issues.is_empty(),
        issues,
    }
}

pub fn render_hetero_calculate_plan_toml(plan: &LinkPlanHeteroCalculate) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(&plan.schema)
    ));
    out.push_str(&format!("mode = \"{}\"\n", escape_toml_string(&plan.mode)));
    out.push_str(&format!("static_link = {}\n", plan.static_link));
    out.push_str(&format!("lifecycle_driven = {}\n", plan.lifecycle_driven));
    out.push_str(&format!(
        "time_order_model = \"{}\"\n",
        escape_toml_string(&plan.time_order_model)
    ));
    out.push_str(&format!(
        "data_order_model = \"{}\"\n",
        escape_toml_string(&plan.data_order_model)
    ));
    out.push_str(&format!(
        "c_world_policy = \"{}\"\n",
        escape_toml_string(&plan.c_world_policy)
    ));
    out.push_str("[validation]\n");
    out.push_str(&format!("checked = {}\n", plan.validation.checked));
    out.push_str(&format!("valid = {}\n", plan.validation.valid));
    out.push_str(&format!(
        "issues = {}\n",
        render_string_array(&plan.validation.issues)
    ));
    for node in &plan.nodes {
        out.push_str("[[node]]\n");
        out.push_str(&format!("index = {}\n", node.index));
        out.push_str(&format!(
            "timestamp = \"{}\"\n",
            escape_toml_string(&node.timestamp)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&node.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&node.package_id)
        ));
        out.push_str(&format!(
            "lifecycle_hook = \"{}\"\n",
            escape_toml_string(&node.lifecycle_hook)
        ));
        out.push_str(&format!(
            "wait_on = {}\n",
            render_string_array(&node.wait_on)
        ));
        out.push_str(&format!("emits = {}\n", render_string_array(&node.emits)));
        out.push_str(&format!(
            "link_input = \"{}\"\n",
            escape_toml_string(&node.link_input)
        ));
        out.push_str(&format!("c_world_wrapper = {}\n", node.c_world_wrapper));
    }
    for segment in &plan.data_segments {
        out.push_str("[[data_segment]]\n");
        out.push_str(&format!("index = {}\n", segment.index));
        out.push_str(&format!(
            "segment_id = \"{}\"\n",
            escape_toml_string(&segment.segment_id)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&segment.domain_family)
        ));
        out.push_str(&format!(
            "owner_package = \"{}\"\n",
            escape_toml_string(&segment.owner_package)
        ));
        out.push_str(&format!(
            "order_key = \"{}\"\n",
            escape_toml_string(&segment.order_key)
        ));
        out.push_str(&format!(
            "access_phase = \"{}\"\n",
            escape_toml_string(&segment.access_phase)
        ));
        if let Some(source_path) = &segment.source_path {
            out.push_str(&format!(
                "source_path = \"{}\"\n",
                escape_toml_string(source_path)
            ));
        }
    }
    out
}

fn lifecycle_hook_for_domain(unit: &LinkPlanDomainUnit) -> String {
    match unit.domain_family.as_str() {
        "network" => "on_network_bridge_progress".to_owned(),
        "kernel" | "shader" | "data" => "on_hetero_submission_progress".to_owned(),
        _ => "on_scheduler_tick".to_owned(),
    }
}

fn uses_c_world_wrapper(unit: &LinkPlanDomainUnit) -> bool {
    unit.artifact_bridge_stub_path
        .as_deref()
        .map(|path| path.ends_with(".c") || path.contains("cffi") || path.contains("bridge"))
        .unwrap_or(false)
}
