use std::collections::BTreeMap;

use yir_core::{Node, SemanticOp, YirModule};

pub(crate) fn verify_result_state_nodes(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        match node.op.semantic_op() {
            SemanticOp::DataObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                if !node.op.observe_state_matches_source(&source.op, actual)? {
                    return Err(format!(
                        "node `{}` observes data state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::DataValue => {
                require_observe_source(&nodes, node, SemanticOp::DataObserve)?;
            }
            SemanticOp::ShaderObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                if !node.op.observe_state_matches_source(&source.op, actual)? {
                    return Err(format!(
                        "node `{}` observes shader state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::ShaderIsPassReady
            | SemanticOp::ShaderIsFrameReady
            | SemanticOp::ShaderValue => {
                require_observe_source(&nodes, node, SemanticOp::ShaderObserve)?;
            }
            SemanticOp::KernelObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                let direct_project_ref =
                    source.op.semantic_op() == SemanticOp::CpuProjectProfileRef;
                let resolved_kernel_profile_slot = is_resolved_kernel_profile_slot(source);
                let direct_kernel_scalar_source = is_direct_kernel_scalar_source(source);
                if !direct_project_ref
                    && !resolved_kernel_profile_slot
                    && !direct_kernel_scalar_source
                {
                    return Err(format!(
                        "node `{}` expects cpu.project_profile_ref or direct kernel scalar input for kernel observe, got `{}`",
                        node.name,
                        source.op.full_name()
                    ));
                }
                let state_matches = if resolved_kernel_profile_slot || direct_kernel_scalar_source {
                    actual == "config_ready"
                } else {
                    node.op.observe_state_matches_source(&source.op, actual)?
                };
                if !state_matches {
                    return Err(format!(
                        "node `{}` observes kernel state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::KernelIsConfigReady | SemanticOp::KernelValue => {
                require_observe_source(&nodes, node, SemanticOp::KernelObserve)?;
            }
            SemanticOp::NetworkObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                let direct_project_ref =
                    source.op.semantic_op() == SemanticOp::CpuProjectProfileRef;
                let resolved_network_profile_slot = is_resolved_network_profile_slot(source);
                let host_network_transport_probe = is_host_network_transport_probe_source(source);
                if !direct_project_ref
                    && !resolved_network_profile_slot
                    && !host_network_transport_probe
                {
                    return Err(format!(
                        "node `{}` expects cpu.project_profile_ref or host network transport probe input for network observe, got `{}`",
                        node.name,
                        source.op.full_name()
                    ));
                }
                let state_matches = if resolved_network_profile_slot {
                    actual == "config_ready"
                } else if host_network_transport_probe {
                    match source.op.args[1].as_str() {
                        "host_network_send_probe" => actual == "send_ready",
                        "host_network_send_owned" => actual == "send_ready",
                        "host_network_accept_probe" => actual == "accept_ready",
                        "host_network_accept_owned" => actual == "accept_ready",
                        "host_network_recv_probe" => actual == "recv_ready",
                        "host_network_recv_owned" => actual == "recv_ready",
                        "host_network_recv_http_status_owned" => actual == "recv_ready",
                        "host_network_close" => actual == "closed",
                        _ => false,
                    }
                } else {
                    node.op.observe_state_matches_source(&source.op, actual)?
                };
                if !state_matches {
                    return Err(format!(
                        "node `{}` observes network state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::NetworkIsConfigReady => {
                require_observe_source(&nodes, node, SemanticOp::NetworkObserve)?;
            }
            SemanticOp::NetworkIsSendReady => {
                require_observe_source(&nodes, node, SemanticOp::NetworkObserve)?;
            }
            SemanticOp::NetworkIsRecvReady => {
                require_observe_source(&nodes, node, SemanticOp::NetworkObserve)?;
            }
            _ if node.op.result_source_semantic_op().is_some() => {
                require_expected_result_source(&nodes, node)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn is_resolved_kernel_profile_slot(node: &Node) -> bool {
    node.name.starts_with("project_profile_kernel_")
        && node.op.module == "cpu"
        && node.op.instruction == "const_i64"
}

fn is_resolved_network_profile_slot(node: &Node) -> bool {
    node.name.starts_with("project_profile_network_")
        && node.op.module == "network"
        && node.op.instruction == "const_i64"
}

fn is_host_network_transport_probe_source(node: &Node) -> bool {
    node.op.module == "cpu"
        && matches!(
            node.op.instruction.as_str(),
            "extern_call_i64" | "extern_call_i32"
        )
        && node.op.args.len() >= 2
        && matches!(
            node.op.args[1].as_str(),
            "host_network_accept_probe"
                | "host_network_accept_owned"
                | "host_network_send_probe"
                | "host_network_send_owned"
                | "host_network_recv_probe"
                | "host_network_recv_owned"
                | "host_network_recv_http_status_owned"
                | "host_network_close"
        )
}

fn is_direct_kernel_scalar_source(node: &Node) -> bool {
    node.op.module == "kernel"
        && matches!(
            node.op.instruction.as_str(),
            "reduce_sum" | "reduce_max" | "reduce_mean" | "argmax" | "argmin"
        )
}

fn require_expected_result_source(
    nodes: &BTreeMap<&str, &Node>,
    node: &Node,
) -> Result<(), String> {
    if node.op.semantic_op() == SemanticOp::NetworkValue {
        let source = node
            .op
            .args
            .first()
            .ok_or_else(|| format!("node `{}` is missing result source arg", node.name))
            .and_then(|name| {
                nodes.get(name.as_str()).copied().ok_or_else(|| {
                    format!(
                        "node `{}` references unknown result source `{name}`",
                        node.name
                    )
                })
            })?;
        let actual = source.op.semantic_op();
        if matches!(
            actual,
            SemanticOp::NetworkObserve
                | SemanticOp::NetworkConnect
                | SemanticOp::NetworkAccept
                | SemanticOp::NetworkClose
        ) {
            return Ok(());
        }
        return Err(format!(
            "node `{}` expects one of `network.observe`, `network.connect`, `network.accept`, or `network.close`, got `{}`",
            node.name,
            source.op.full_name()
        ));
    }
    let expected = node.op.result_source_semantic_op().ok_or_else(|| {
        format!(
            "node `{}` has no expected result source contract",
            node.name
        )
    })?;
    require_observe_source(nodes, node, expected)
}

fn observe_source_node<'a>(
    nodes: &'a BTreeMap<&str, &'a Node>,
    node: &Node,
) -> Result<&'a Node, String> {
    let source_name = node
        .op
        .args
        .first()
        .ok_or_else(|| format!("node `{}` is missing observe source arg", node.name))?;
    nodes.get(source_name.as_str()).copied().ok_or_else(|| {
        format!(
            "node `{}` references unknown observe source `{source_name}`",
            node.name
        )
    })
}

fn observe_state_arg(node: &Node) -> Result<&str, String> {
    node.op
        .args
        .get(1)
        .map(|value| value.as_str())
        .ok_or_else(|| format!("node `{}` is missing observe state arg", node.name))
}

fn require_observe_source(
    nodes: &BTreeMap<&str, &Node>,
    node: &Node,
    expected: SemanticOp,
) -> Result<(), String> {
    let source = node
        .op
        .args
        .first()
        .ok_or_else(|| format!("node `{}` is missing result source arg", node.name))
        .and_then(|name| {
            nodes.get(name.as_str()).copied().ok_or_else(|| {
                format!(
                    "node `{}` references unknown result source `{name}`",
                    node.name
                )
            })
        })?;
    if source.op.semantic_op() != expected {
        return Err(format!(
            "node `{}` expects `{}` input, got `{}`",
            node.name,
            semantic_op_name(expected),
            source.op.full_name()
        ));
    }
    Ok(())
}

fn semantic_op_name(op: SemanticOp) -> &'static str {
    match op {
        SemanticOp::CpuProjectProfileRef => "cpu.project_profile_ref",
        SemanticOp::CpuJoinResult => "cpu.join_result",
        SemanticOp::CpuTaskCompleted => "cpu.task_completed",
        SemanticOp::CpuTaskTimedOut => "cpu.task_timed_out",
        SemanticOp::CpuTaskCancelled => "cpu.task_cancelled",
        SemanticOp::CpuTaskValue => "cpu.task_value",
        SemanticOp::DataObserve => "data.observe",
        SemanticOp::DataIsReady => "data.is_ready",
        SemanticOp::DataIsMoved => "data.is_moved",
        SemanticOp::DataIsWindowed => "data.is_windowed",
        SemanticOp::DataValue => "data.value",
        SemanticOp::ShaderObserve => "shader.observe",
        SemanticOp::ShaderIsPassReady => "shader.is_pass_ready",
        SemanticOp::ShaderIsFrameReady => "shader.is_frame_ready",
        SemanticOp::ShaderValue => "shader.value",
        SemanticOp::KernelObserve => "kernel.observe",
        SemanticOp::KernelIsConfigReady => "kernel.is_config_ready",
        SemanticOp::KernelValue => "kernel.value",
        SemanticOp::NetworkObserve => "network.observe",
        SemanticOp::NetworkConnect => "network.connect",
        SemanticOp::NetworkAccept => "network.accept",
        SemanticOp::NetworkClose => "network.close",
        SemanticOp::NetworkIsConfigReady => "network.is_config_ready",
        SemanticOp::NetworkIsConnectReady => "network.is_connect_ready",
        SemanticOp::NetworkIsAcceptReady => "network.is_accept_ready",
        SemanticOp::NetworkIsClosed => "network.is_closed",
        SemanticOp::NetworkValue => "network.value",
        SemanticOp::DataBindCore => "data.bind_core",
        SemanticOp::DataMarker => "data.marker",
        SemanticOp::DataHandleTable => "data.handle_table",
        SemanticOp::DataOutputPipe => "data.output_pipe",
        SemanticOp::DataInputPipe => "data.input_pipe",
        SemanticOp::DataCopyWindow => "data.copy_window",
        SemanticOp::DataReadWindow => "data.read_window",
        SemanticOp::DataWriteWindow => "data.write_window",
        SemanticOp::DataImmutableWindow => "data.immutable_window",
        SemanticOp::ShaderBeginPass => "shader.begin_pass",
        SemanticOp::ShaderDrawInstanced => "shader.draw_instanced",
        _ => "other",
    }
}
