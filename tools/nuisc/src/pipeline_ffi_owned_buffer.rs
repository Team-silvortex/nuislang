use std::collections::BTreeMap;

use yir_core::{EdgeKind, Node, YirModule};

pub(super) fn validate_owned_return_buffer_yir(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    for producer in module
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "extern_call_owned_buffer")
    {
        let contract = yir_core::ffi::parse_owned_buffer_return_contract(&producer.op.args)
            .map_err(|error| {
                format!(
                    "owned extern buffer node `{}` has an invalid contract: {error}",
                    producer.name
                )
            })?;
        let functions = module
            .functions
            .iter()
            .filter(|function| {
                function
                    .body_nodes
                    .iter()
                    .any(|name| name == &producer.name)
            })
            .collect::<Vec<_>>();
        let [function] = functions.as_slice() else {
            return Err(format!(
                "owned extern buffer `{}` must belong to exactly one YIR function body",
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
        let free_nodes = consumers
            .iter()
            .copied()
            .filter(|node| is_exact_free(node, &producer.name))
            .collect::<Vec<_>>();
        let [free] = free_nodes.as_slice() else {
            return Err(format!(
                "owned extern buffer `{}` must be consumed by exactly one direct free(...) in the same linear block; found {}",
                contract.symbol,
                free_nodes.len()
            ));
        };
        let Some(&free_index) = positions.get(free.name.as_str()) else {
            return Err(format!(
                "owned extern buffer `{}` destructor transfer escapes YIR function `{}`",
                contract.symbol, function.name
            ));
        };
        if free_index <= producer_index {
            return Err(format!(
                "owned extern buffer `{}` is released before its producing call",
                contract.symbol
            ));
        }
        for consumer in consumers {
            if is_exact_free(consumer, &producer.name) {
                continue;
            }
            if !is_direct_buffer_access(consumer, &producer.name) {
                return Err(format!(
                    "owned extern buffer `{}` escapes through unsupported `{}.{}`; only buffer_len/load_at/store_at and one direct free are open",
                    contract.symbol, consumer.op.module, consumer.op.instruction
                ));
            }
            let Some(&consumer_index) = positions.get(consumer.name.as_str()) else {
                return Err(format!(
                    "owned extern buffer `{}` access escapes YIR function `{}`",
                    contract.symbol, function.name
                ));
            };
            if consumer_index >= free_index {
                return Err(format!(
                    "owned extern buffer `{}` is accessed after its registered destructor transfer",
                    contract.symbol
                ));
            }
        }
        for name in &function.body_nodes[producer_index + 1..free_index] {
            let Some(node) = nodes.get(name.as_str()).copied() else {
                continue;
            };
            if node.op.async_core_op().is_some() {
                return Err(format!(
                    "owned extern buffer `{}` remains live across async operation `{}.{}`; async escape remains closed",
                    contract.symbol, node.op.module, node.op.instruction
                ));
            }
            if is_control_flow_boundary(node) {
                return Err(format!(
                    "owned extern buffer `{}` remains live across control-flow operation `{}.{}`; branch escape remains closed",
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

fn is_direct_buffer_access(node: &Node, producer: &str) -> bool {
    node.op.module == "cpu"
        && matches!(
            node.op.instruction.as_str(),
            "buffer_len" | "load_at" | "store_at"
        )
        && node.op.args.first().map(String::as_str) == Some(producer)
}

fn is_control_flow_boundary(node: &Node) -> bool {
    let instruction = node.op.instruction.as_str();
    instruction.contains("branch")
        || instruction.starts_with("loop_")
        || instruction.starts_with("guard_")
        || instruction.starts_with("return")
}

#[cfg(test)]
mod tests {
    use super::validate_owned_return_buffer_yir;
    use yir_core::{
        ffi::{
            ffi_memory_capability_hash, ffi_symbol_signature_hash, owned_buffer_return_descriptor,
            OWNED_BUFFER_RETURN_LENGTH_POLICY, OWNED_BUFFER_RETURN_PROTOCOL,
        },
        Edge, EdgeKind, Node, Operation, YirFunction, YirFunctionRole, YirModule,
    };

    fn module_with_middle(middle: Option<Node>) -> YirModule {
        let signature = "ref_Buffer(i64)";
        let signature_hash = ffi_symbol_signature_hash("c", "host_owned_buffer_make", signature);
        let destructor_hash =
            ffi_symbol_signature_hash("c", "host_owned_buffer_destroy", "i64(ref_Buffer)");
        let descriptor =
            owned_buffer_return_descriptor("host_owned_buffer_destroy", &destructor_hash);
        let capability_hash =
            ffi_memory_capability_hash("c", "host_owned_buffer_make", &signature_hash, &descriptor);
        let producer = Node {
            name: "owned".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "extern_call_owned_buffer".to_owned(),
                args: vec![
                    OWNED_BUFFER_RETURN_PROTOCOL.to_owned(),
                    "c".to_owned(),
                    "host_owned_buffer_make".to_owned(),
                    signature.to_owned(),
                    signature_hash,
                    capability_hash,
                    OWNED_BUFFER_RETURN_LENGTH_POLICY.to_owned(),
                    "host_owned_buffer_destroy".to_owned(),
                    destructor_hash,
                ],
            },
        };
        let free = Node {
            name: "release".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "free".to_owned(),
                args: vec!["owned".to_owned()],
            },
        };
        let mut nodes = vec![producer];
        let mut body_nodes = vec!["owned".to_owned()];
        if let Some(middle) = middle {
            body_nodes.push(middle.name.clone());
            nodes.push(middle);
        }
        body_nodes.push("release".to_owned());
        nodes.push(free);
        let mut module = YirModule::new("0.1");
        module.nodes = nodes;
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: "owned".to_owned(),
            to: "release".to_owned(),
        });
        module.functions.push(YirFunction {
            name: "main".to_owned(),
            domain: "cffi".to_owned(),
            role: YirFunctionRole::Entry,
            parameters: Vec::new(),
            result: None,
            body_nodes,
        });
        module
    }

    fn middle_node(instruction: &str) -> Node {
        Node {
            name: "middle".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: instruction.to_owned(),
                args: Vec::new(),
            },
        }
    }

    #[test]
    fn accepts_linear_exact_destructor_transfer() {
        validate_owned_return_buffer_yir(&module_with_middle(None)).unwrap();
    }

    #[test]
    fn rejects_branch_and_async_lifetime_crossings() {
        let branch = validate_owned_return_buffer_yir(&module_with_middle(Some(middle_node(
            "branch_effect",
        ))))
        .unwrap_err();
        assert!(branch.contains("branch escape"));

        let asynchronous =
            validate_owned_return_buffer_yir(&module_with_middle(Some(middle_node("await"))))
                .unwrap_err();
        assert!(asynchronous.contains("async escape"));
    }

    #[test]
    fn rejects_secondary_extern_escape() {
        let mut module = module_with_middle(Some(Node {
            name: "escape".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "extern_call_i64".to_owned(),
                args: vec!["c".to_owned(), "host_sink".to_owned(), "owned".to_owned()],
            },
        }));
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: "owned".to_owned(),
            to: "escape".to_owned(),
        });

        let error = validate_owned_return_buffer_yir(&module).unwrap_err();
        assert!(error.contains("escapes through unsupported"));
    }
}
