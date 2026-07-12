use std::collections::BTreeMap;

use yir_core::{Node, YirModule};

use super::*;

pub fn analyze_kernel_lowering(module: &YirModule) -> KernelLoweringContract {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let fabric_handle_tables = collect_fabric_handle_tables(module);
    let fabric_core_bindings = collect_fabric_core_bindings(module);
    let target_profiles = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "kernel"
                && node.op.instruction == "target_config"
                && node.op.args.len() == 3
        })
        .map(|node| {
            (
                node.resource.clone(),
                KernelTargetProfile {
                    arch: Some(node.op.args[0].clone()),
                    runtime: Some(node.op.args[1].clone()),
                    lane_width: node.op.args[2].parse::<usize>().ok(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let stages = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "kernel")
        .filter_map(|node| {
            analyze_kernel_stage(node, &nodes, &target_profiles, &fabric_handle_tables)
        })
        .collect::<Vec<_>>();
    let graphs = build_kernel_graphs(&stages);

    KernelLoweringContract {
        stages,
        graphs,
        fabric_handle_tables,
        fabric_core_bindings,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KernelTargetProfile {
    arch: Option<String>,
    runtime: Option<String>,
    lane_width: Option<usize>,
}

fn analyze_kernel_stage(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
    target_profiles: &BTreeMap<String, KernelTargetProfile>,
    fabric_handle_tables: &[FabricHandleTableContract],
) -> Option<KernelStageContract> {
    if matches!(
        node.op.instruction.as_str(),
        "target_config"
            | "observe"
            | "is_config_ready"
            | "value"
            | "const_bool"
            | "const_i32"
            | "const_i64"
            | "const_f32"
            | "const_f64"
            | "print"
    ) {
        return None;
    }

    let target_profile = target_profiles.get(&node.resource);
    let runtime = target_profile.and_then(|profile| profile.runtime.clone());
    let arch = target_profile.and_then(|profile| profile.arch.clone());
    let lane_width = target_profile.and_then(|profile| profile.lane_width);
    let rows = infer_kernel_rows(node, nodes);
    let cols = infer_kernel_cols(node, nodes);
    let axis = infer_kernel_axis(node);
    let topk = infer_kernel_topk(node);
    let inputs = infer_kernel_inputs(node);
    let fabric_handle_table = fabric_handle_tables
        .iter()
        .find(|table| {
            table
                .entries
                .iter()
                .any(|entry| entry.resource == node.resource)
        })
        .map(|table| table.node.clone());

    let (lowering, reason) = classify_kernel_backend_eligibility(
        node.op.instruction.as_str(),
        runtime.as_deref(),
        rows,
        cols,
        axis.as_deref(),
    );

    Some(KernelStageContract {
        node: node.name.clone(),
        function: format!("kernel.{}", node.name),
        node_kind: "function-node".to_owned(),
        execution_domain: "kernel".to_owned(),
        time_mode: "logical".to_owned(),
        op: node.op.full_name(),
        resource: node.resource.clone(),
        lowering,
        reason: reason.to_owned(),
        target_arch: arch,
        target_runtime: runtime,
        lane_width,
        rows,
        cols,
        axis,
        topk,
        inputs,
        fabric_handle_table,
    })
}

fn infer_kernel_rows(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Option<usize> {
    match node.op.instruction.as_str() {
        "tensor" | "fill" | "splat" if node.op.args.len() >= 2 => {
            node.op.args[0].parse::<usize>().ok()
        }
        "reshape" if node.op.args.len() >= 3 => node.op.args[1].parse::<usize>().ok(),
        "slice" if node.op.args.len() >= 5 => node.op.args[3].parse::<usize>().ok(),
        "broadcast" if node.op.args.len() >= 3 => node.op.args[1].parse::<usize>().ok(),
        "row" => Some(1),
        "col" => nodes
            .get(node.op.args.first()?.as_str())
            .copied()
            .and_then(|source| infer_kernel_rows(source, nodes)),
        "shape" => Some(1),
        "rows" | "cols" | "reduce_sum" | "reduce_mean" | "reduce_max" | "reduce_min" => Some(1),
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "topk_axis" | "sort_axis" => {
            let source = nodes.get(node.op.args.first()?.as_str()).copied()?;
            match node.op.args.get(1).map(|value| value.as_str()) {
                Some("rows") => Some(1),
                Some("cols") => infer_kernel_rows(source, nodes),
                _ => None,
            }
        }
        "argmax" | "argmin" | "element_at" => Some(1),
        _ => None,
    }
}

fn infer_kernel_cols(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Option<usize> {
    match node.op.instruction.as_str() {
        "tensor" | "fill" | "splat" if node.op.args.len() >= 2 => {
            node.op.args[1].parse::<usize>().ok()
        }
        "reshape" if node.op.args.len() >= 3 => node.op.args[2].parse::<usize>().ok(),
        "slice" if node.op.args.len() >= 5 => node.op.args[4].parse::<usize>().ok(),
        "broadcast" if node.op.args.len() >= 3 => node.op.args[2].parse::<usize>().ok(),
        "row" => nodes
            .get(node.op.args.first()?.as_str())
            .copied()
            .and_then(|source| infer_kernel_cols(source, nodes)),
        "col" => Some(1),
        "shape" => Some(2),
        "rows" | "cols" | "reduce_sum" | "reduce_mean" | "reduce_max" | "reduce_min" => Some(1),
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "topk_axis" | "sort_axis" => {
            let source = nodes.get(node.op.args.first()?.as_str()).copied()?;
            match node.op.args.get(1).map(|value| value.as_str()) {
                Some("rows") => infer_kernel_cols(source, nodes),
                Some("cols") => Some(1),
                _ => None,
            }
        }
        "argmax" | "argmin" | "element_at" => Some(1),
        "topk" => node.op.args.get(1).and_then(|k| k.parse::<usize>().ok()),
        _ => None,
    }
}

fn infer_kernel_axis(node: &Node) -> Option<String> {
    match node.op.instruction.as_str() {
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "topk_axis" | "sort_axis" | "relu_axis"
        | "add_scalar_axis" | "mul_scalar_axis" => node.op.args.last().cloned(),
        _ => None,
    }
}

fn infer_kernel_topk(node: &Node) -> Option<usize> {
    match node.op.instruction.as_str() {
        "topk" => node
            .op
            .args
            .get(1)
            .and_then(|value| value.parse::<usize>().ok()),
        "topk_axis" => node
            .op
            .args
            .get(1)
            .and_then(|value| value.parse::<usize>().ok()),
        _ => None,
    }
}

fn infer_kernel_inputs(node: &Node) -> Vec<String> {
    match node.op.instruction.as_str() {
        "tensor" | "fill" | "splat" | "const_bool" | "const_i32" | "const_i64" | "const_f32"
        | "const_f64" | "target_config" => Vec::new(),
        "reshape" => node.op.args.iter().take(1).cloned().collect(),
        "slice" => node.op.args.iter().take(1).cloned().collect(),
        "broadcast" => node.op.args.iter().take(1).cloned().collect(),
        "topk" => node.op.args.iter().take(1).cloned().collect(),
        "topk_axis" => node.op.args.iter().take(1).cloned().collect(),
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "sort_axis" | "relu_axis" | "add_scalar_axis"
        | "mul_scalar_axis" => node.op.args.iter().take(1).cloned().collect(),
        "element_at" => node.op.args.iter().take(1).cloned().collect(),
        "print" => node.op.args.iter().take(1).cloned().collect(),
        _ => node.op.args.clone(),
    }
}

fn classify_kernel_backend_eligibility(
    op: &str,
    runtime: Option<&str>,
    rows: Option<usize>,
    cols: Option<usize>,
    axis: Option<&str>,
) -> (KernelLoweringMode, &'static str) {
    let Some(runtime) = runtime else {
        return (
            KernelLoweringMode::CpuFallbackOnly,
            "missing kernel.target_config runtime contract",
        );
    };

    let portable_tensor_subset = matches!(
        op,
        "tensor"
            | "fill"
            | "splat"
            | "add"
            | "mul"
            | "add_scalar"
            | "mul_scalar"
            | "matmul"
            | "add_bias"
            | "relu"
            | "reshape"
            | "slice"
            | "broadcast"
            | "reduce_sum"
            | "reduce_mean"
            | "reduce_max"
            | "reduce_min"
            | "reduce_sum_axis"
            | "reduce_mean_axis"
            | "reduce_max_axis"
            | "reduce_min_axis"
            | "row"
            | "col"
            | "shape"
            | "rows"
            | "cols"
            | "element_at"
    );

    if !portable_tensor_subset {
        return (
            KernelLoweringMode::CpuFallbackOnly,
            "op is outside the current portable kernel lowering subset",
        );
    }

    if rows == Some(0) || cols == Some(0) {
        return (
            KernelLoweringMode::CpuFallbackOnly,
            "zero-shaped kernel work cannot be lowered portably",
        );
    }

    if let Some(axis) = axis {
        if !matches!(axis, "rows" | "cols") {
            return (
                KernelLoweringMode::CpuFallbackOnly,
                "axis contract is outside the current portable kernel lowering subset",
            );
        }
    }

    match runtime {
        "coreml" => (
            KernelLoweringMode::BackendEligible,
            "stage fits the current CoreML/MPS graph lowering subset",
        ),
        "vulkan" => (
            KernelLoweringMode::BackendEligible,
            "stage fits the current Vulkan compute lowering subset",
        ),
        _ => (
            KernelLoweringMode::CpuFallbackOnly,
            "runtime is outside the current portable kernel lowering subset",
        ),
    }
}

fn build_kernel_graphs(stages: &[KernelStageContract]) -> Vec<KernelComputeGraphContract> {
    let mut by_resource = BTreeMap::<String, Vec<&KernelStageContract>>::new();
    for stage in stages {
        by_resource
            .entry(stage.resource.clone())
            .or_default()
            .push(stage);
    }

    by_resource
        .into_iter()
        .enumerate()
        .map(|(index, (resource, resource_stages))| {
            let lowering = if resource_stages
                .iter()
                .all(|stage| stage.lowering == KernelLoweringMode::BackendEligible)
            {
                KernelLoweringMode::BackendEligible
            } else {
                KernelLoweringMode::CpuFallbackOnly
            };
            let reason = if lowering == KernelLoweringMode::BackendEligible {
                "graph fits the current fused kernel backend portability subset".to_owned()
            } else {
                "graph includes one or more stages outside the current fused kernel backend portability subset".to_owned()
            };
            let target_arch = resource_stages
                .iter()
                .find_map(|stage| stage.target_arch.clone());
            let target_runtime = resource_stages
                .iter()
                .find_map(|stage| stage.target_runtime.clone());
            let lane_width = resource_stages.iter().find_map(|stage| stage.lane_width);
            let graph_name = resource
                .rsplit('@')
                .next()
                .unwrap_or(resource.as_str())
                .replace('.', "_");
            let stages = resource_stages
                .iter()
                .map(|stage| stage.node.clone())
                .collect::<Vec<_>>();

            KernelComputeGraphContract {
                id: format!("kernel_graph_{}_{}", index + 1, graph_name),
                function: format!("kernel.graph.{}", graph_name),
                node_kind: "function-graph".to_owned(),
                execution_domain: "kernel".to_owned(),
                time_mode: "logical".to_owned(),
                resource,
                lowering,
                reason,
                target_arch,
                target_runtime,
                lane_width,
                stages,
            }
        })
        .collect()
}
