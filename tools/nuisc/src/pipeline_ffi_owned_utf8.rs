use std::collections::BTreeMap;

use yir_core::{EdgeKind, Node, YirModule};

pub(super) fn validate_owned_return_utf8_yir(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for producer in module
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "extern_call_owned_utf8")
    {
        let contract = yir_core::ffi::parse_owned_utf8_return_contract(&producer.op.args).map_err(
            |error| {
                format!(
                    "owned extern UTF-8 node `{}` has an invalid contract: {error}",
                    producer.name
                )
            },
        )?;
        let functions = module
            .functions
            .iter()
            .filter(|function| function.body_nodes.contains(&producer.name))
            .collect::<Vec<_>>();
        let [function] = functions.as_slice() else {
            return Err(format!(
                "owned extern UTF-8 `{}` must belong to exactly one YIR function body",
                contract.symbol
            ));
        };
        let positions = function
            .body_nodes
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect::<BTreeMap<_, _>>();
        let producer_index = positions[producer.name.as_str()];
        let consumers = module
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::Dep && edge.from == producer.name)
            .filter_map(|edge| nodes.get(edge.to.as_str()).copied())
            .collect::<Vec<_>>();
        let frees = consumers
            .iter()
            .copied()
            .filter(|node| is_exact_free(node, &producer.name))
            .collect::<Vec<_>>();
        let [free] = frees.as_slice() else {
            return Err(format!(
                "owned extern UTF-8 `{}` must be consumed by exactly one direct free(...); found {}",
                contract.symbol,
                frees.len()
            ));
        };
        let Some(&free_index) = positions.get(free.name.as_str()) else {
            return Err(format!(
                "owned extern UTF-8 `{}` destructor escapes YIR function `{}`",
                contract.symbol, function.name
            ));
        };
        if free_index <= producer_index {
            return Err(format!(
                "owned extern UTF-8 `{}` is released before its producing call",
                contract.symbol
            ));
        }
        if !module.edges.iter().any(|edge| {
            edge.kind == EdgeKind::Lifetime && edge.from == producer.name && edge.to == free.name
        }) {
            return Err(format!(
                "owned extern UTF-8 `{}` lacks a producer-to-destructor lifetime edge",
                contract.symbol
            ));
        }
        for consumer in &consumers {
            if consumer.name == free.name {
                continue;
            }
            if !is_direct_read(consumer, &producer.name) {
                return Err(format!(
                    "owned extern UTF-8 `{}` escapes through unsupported `{}.{}`; only owned_utf8_len/owned_utf8_byte_at and one direct free are open",
                    contract.symbol, consumer.op.module, consumer.op.instruction
                ));
            }
            let Some(&consumer_index) = positions.get(consumer.name.as_str()) else {
                return Err(format!(
                    "owned extern UTF-8 `{}` read escapes YIR function `{}`",
                    contract.symbol, function.name
                ));
            };
            if consumer_index >= free_index {
                return Err(format!(
                    "owned extern UTF-8 `{}` is read after its registered destructor",
                    contract.symbol
                ));
            }
        }
        for name in &function.body_nodes[producer_index + 1..free_index] {
            let Some(node) = nodes.get(name.as_str()).copied() else {
                continue;
            };
            if node.op.async_core_op().is_some() || is_control_flow_boundary(node) {
                return Err(format!(
                    "owned extern UTF-8 `{}` remains live across unsupported `{}.{}` boundary",
                    contract.symbol, node.op.module, node.op.instruction
                ));
            }
        }
    }
    Ok(())
}

fn is_exact_free(node: &Node, producer: &str) -> bool {
    node.op.module == "cpu"
        && node.op.instruction == "free"
        && node.op.args.as_slice() == [producer]
}

fn is_direct_read(node: &Node, producer: &str) -> bool {
    node.op.module == "cpu"
        && matches!(node.op.instruction.as_str(), "buffer_len" | "load_at")
        && node.op.args.first().map(String::as_str) == Some(producer)
}

fn is_control_flow_boundary(node: &Node) -> bool {
    let instruction = node.op.instruction.as_str();
    instruction.contains("branch")
        || instruction.starts_with("loop_")
        || instruction.starts_with("guard_")
        || instruction.starts_with("return")
}
