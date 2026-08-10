use std::collections::BTreeMap;

use yir_core::{
    parse_branch_effect_args, BranchEffectAccess, BranchEffectResult, EdgeKind, Node, YirFunction,
    YirModule,
};

#[path = "pipeline_ffi_owned_buffer_return.rs"]
mod function_return;

pub(super) fn validate_owned_return_buffer_yir(module: &YirModule) -> Result<(), String> {
    function_return::validate_owned_buffer_function_transfers(module)?;
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
        let transfer_nodes = consumers
            .iter()
            .copied()
            .filter(|node| {
                owned_buffer_transfer_inputs(node)
                    .is_some_and(|inputs| inputs.contains(&producer.name.as_str()))
            })
            .collect::<Vec<_>>();
        let return_nodes = consumers
            .iter()
            .copied()
            .filter(|node| is_exact_function_return(node, &producer.name))
            .collect::<Vec<_>>();
        match (
            free_nodes.as_slice(),
            transfer_nodes.as_slice(),
            return_nodes.as_slice(),
        ) {
            ([free], [], []) => validate_owner_tail(
                contract.symbol,
                &producer.name,
                producer_index,
                free,
                &consumers,
                function,
                &positions,
                &nodes,
            )?,
            ([], [transfer], []) => {
                let Some(&transfer_index) = positions.get(transfer.name.as_str()) else {
                    return Err(format!(
                        "owned extern buffer `{}` transfer escapes YIR function `{}`",
                        contract.symbol, function.name
                    ));
                };
                if transfer_index <= producer_index {
                    return Err(format!(
                        "owned extern buffer `{}` is transferred before its producing call",
                        contract.symbol
                    ));
                }
                validate_owner_consumers_before(
                    contract.symbol,
                    &producer.name,
                    transfer,
                    transfer_index,
                    &consumers,
                    function,
                    &positions,
                )?;
                validate_live_interval(
                    contract.symbol,
                    producer_index,
                    transfer_index,
                    function,
                    &nodes,
                )?;
                validate_registered_transfer(module, transfer, function, &positions, &nodes)?;
                validate_transferred_owner_tail(
                    module,
                    contract.symbol,
                    transfer,
                    transfer_index,
                    function,
                    &positions,
                    &nodes,
                )?;
            }
            ([], [], [returned]) => validate_owner_tail(
                contract.symbol,
                &producer.name,
                producer_index,
                returned,
                &consumers,
                function,
                &positions,
                &nodes,
            )?,
            _ => {
                return Err(format!(
                    "owned extern buffer `{}` must be consumed by exactly one direct free(...), registered branch transfer, or registered helper return; found {} free(s), {} branch transfer(s), and {} return transfer(s)",
                    contract.symbol,
                    free_nodes.len(),
                    transfer_nodes.len(),
                    return_nodes.len()
                ));
            }
        }
    }
    validate_returned_call_owners(module, &nodes)?;
    Ok(())
}

fn validate_returned_call_owners(
    module: &YirModule,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<(), String> {
    for owner in module.nodes.iter().filter(|node| {
        node.op.module == "cpu" && node.op.instruction == "call_owned_external_buffer"
    }) {
        let contract =
            yir_core::ffi::parse_owned_buffer_function_transfer_contract(&owner.op.args[1..])
                .map_err(|error| format!("returned owned buffer `{}`: {error}", owner.name))?;
        let function = module
            .functions
            .iter()
            .find(|function| function.body_nodes.contains(&owner.name))
            .ok_or_else(|| {
                format!(
                    "returned owned buffer `{}` must belong to one caller function",
                    owner.name
                )
            })?;
        let positions = function
            .body_nodes
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect::<BTreeMap<_, _>>();
        let owner_index = positions[owner.name.as_str()];
        let consumers = module
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::Dep && edge.from == owner.name)
            .filter_map(|edge| nodes.get(edge.to.as_str()).copied())
            .collect::<Vec<_>>();
        let free_nodes = consumers
            .iter()
            .copied()
            .filter(|node| is_exact_free(node, &owner.name))
            .collect::<Vec<_>>();
        let [free] = free_nodes.as_slice() else {
            return Err(format!(
                "returned owned buffer from `{}` must be consumed by exactly one direct caller free(...); found {}",
                owner.op.args[0],
                free_nodes.len()
            ));
        };
        validate_owner_tail(
            &format!("{} via {}", owner.op.args[0], contract.destructor_symbol),
            &owner.name,
            owner_index,
            free,
            &consumers,
            function,
            &positions,
            nodes,
        )?;
    }
    Ok(())
}

fn validate_transferred_owner_tail(
    module: &YirModule,
    symbol: &str,
    transfer: &Node,
    transfer_index: usize,
    function: &YirFunction,
    positions: &BTreeMap<&str, usize>,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<(), String> {
    let consumers = module
        .edges
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Dep && edge.from == transfer.name)
        .filter_map(|edge| nodes.get(edge.to.as_str()).copied())
        .collect::<Vec<_>>();
    let free_nodes = consumers
        .iter()
        .copied()
        .filter(|node| is_exact_free(node, &transfer.name))
        .collect::<Vec<_>>();
    let [free] = free_nodes.as_slice() else {
        return Err(format!(
            "transferred owned extern buffer `{symbol}` must be consumed by exactly one direct free(...) after merge; found {}",
            free_nodes.len()
        ));
    };
    validate_owner_tail(
        symbol,
        &transfer.name,
        transfer_index,
        free,
        &consumers,
        function,
        positions,
        nodes,
    )
}

fn validate_owner_tail(
    symbol: &str,
    owner: &str,
    owner_index: usize,
    free: &Node,
    consumers: &[&Node],
    function: &YirFunction,
    positions: &BTreeMap<&str, usize>,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<(), String> {
    let Some(&free_index) = positions.get(free.name.as_str()) else {
        return Err(format!(
            "owned extern buffer `{symbol}` destructor transfer escapes YIR function `{}`",
            function.name
        ));
    };
    if free_index <= owner_index {
        return Err(format!(
            "owned extern buffer `{symbol}` is released before its producing or transfer operation"
        ));
    }
    validate_owner_consumers_before(
        symbol, owner, free, free_index, consumers, function, positions,
    )?;
    validate_live_interval(symbol, owner_index, free_index, function, nodes)
}

fn validate_owner_consumers_before(
    symbol: &str,
    owner: &str,
    terminal: &Node,
    terminal_index: usize,
    consumers: &[&Node],
    function: &YirFunction,
    positions: &BTreeMap<&str, usize>,
) -> Result<(), String> {
    for consumer in consumers {
        if consumer.name == terminal.name {
            continue;
        }
        if !is_direct_buffer_access(consumer, owner) {
            return Err(format!(
                "owned extern buffer `{symbol}` escapes through unsupported `{}.{}`; only buffer_len/load_at/store_at, one registered branch transfer, and one direct free are open",
                consumer.op.module, consumer.op.instruction
            ));
        }
        let Some(&consumer_index) = positions.get(consumer.name.as_str()) else {
            return Err(format!(
                "owned extern buffer `{symbol}` access escapes YIR function `{}`",
                function.name
            ));
        };
        if consumer_index >= terminal_index {
            return Err(format!(
                "owned extern buffer `{symbol}` is accessed after its registered destructor transfer"
            ));
        }
    }
    Ok(())
}

fn validate_live_interval(
    symbol: &str,
    start: usize,
    end: usize,
    function: &YirFunction,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<(), String> {
    for name in &function.body_nodes[start + 1..end] {
        let Some(node) = nodes.get(name.as_str()).copied() else {
            continue;
        };
        if node.op.async_core_op().is_some() {
            return Err(format!(
                "owned extern buffer `{symbol}` remains live across async operation `{}.{}`; async escape remains closed",
                node.op.module, node.op.instruction
            ));
        }
        if is_control_flow_boundary(node) {
            return Err(format!(
                "owned extern buffer `{symbol}` remains live across control-flow operation `{}.{}`; branch escape remains closed except for one registered owner transfer",
                node.op.module, node.op.instruction
            ));
        }
    }
    Ok(())
}

fn validate_registered_transfer(
    module: &YirModule,
    transfer: &Node,
    function: &YirFunction,
    positions: &BTreeMap<&str, usize>,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<(), String> {
    let inputs = owned_buffer_transfer_inputs(transfer)
        .ok_or_else(|| format!("owned-buffer transfer `{}` is malformed", transfer.name))?;
    let mut identity = None::<(String, String, String)>;
    for input in inputs {
        if !positions.contains_key(input) {
            return Err(format!(
                "owned-buffer transfer `{}` input `{input}` escapes YIR function `{}`",
                transfer.name, function.name
            ));
        }
        if !module.edges.iter().any(|edge| {
            edge.kind == EdgeKind::Dep && edge.from == input && edge.to == transfer.name
        }) {
            return Err(format!(
                "owned-buffer transfer `{}` is missing dependency `{input}`",
                transfer.name
            ));
        }
        let producer = nodes.get(input).copied().ok_or_else(|| {
            format!(
                "owned-buffer transfer `{}` references missing input `{input}`",
                transfer.name
            )
        })?;
        if producer.op.module != "cpu" || producer.op.instruction != "extern_call_owned_buffer" {
            return Err(format!(
                "owned-buffer transfer `{}` input `{input}` is not a direct registered owner producer",
                transfer.name
            ));
        }
        let contract = yir_core::ffi::parse_owned_buffer_return_contract(&producer.op.args)
            .map_err(|error| {
                format!(
                    "owned-buffer transfer `{}` input `{input}` has invalid authority: {error}",
                    transfer.name
                )
            })?;
        let current = (
            contract.abi.to_owned(),
            contract.destructor_symbol.to_owned(),
            contract.destructor_signature_hash.to_owned(),
        );
        if identity.as_ref().is_some_and(|known| known != &current) {
            return Err(format!(
                "owned-buffer transfer `{}` requires one exact ABI/destructor/hash identity",
                transfer.name
            ));
        }
        identity = Some(current);
    }
    Ok(())
}

fn owned_buffer_transfer_inputs(node: &Node) -> Option<[&str; 2]> {
    if node.op.module != "cpu" || node.op.instruction != "branch_effect" {
        return None;
    }
    let args = parse_branch_effect_args(&node.op.args)?;
    if args.merge_result != BranchEffectResult::OwnedPointer
        || args.address_kind != Some("buffer")
        || args.nullable
    {
        return None;
    }
    let [then_action] = args.then_actions.as_slice() else {
        return None;
    };
    let [else_action] = args.else_actions.as_slice() else {
        return None;
    };
    let action_is_valid = |action: &yir_core::BranchEffectAction<'_>| {
        action.module == "cpu"
            && action.instruction == yir_core::ffi::OWNED_BUFFER_BRANCH_TRANSFER_ACTION
            && action.result == BranchEffectResult::OwnedPointer
            && matches!(
                action.operands.as_slice(),
                [selected, discarded]
                    if selected.access == BranchEffectAccess::ResourceOwn
                        && discarded.access == BranchEffectAccess::ResourceOwn
            )
    };
    if !action_is_valid(then_action) || !action_is_valid(else_action) {
        return None;
    }
    let then_selected = then_action.operands[0].value;
    let then_discarded = then_action.operands[1].value;
    let else_selected = else_action.operands[0].value;
    let else_discarded = else_action.operands[1].value;
    (then_selected != then_discarded
        && then_selected == else_discarded
        && then_discarded == else_selected)
        .then_some([then_selected, then_discarded])
}

fn is_exact_free(node: &Node, producer: &str) -> bool {
    node.op.module == "cpu"
        && node.op.instruction == "free"
        && node.op.args.as_slice() == [producer]
}

fn is_exact_function_return(node: &Node, producer: &str) -> bool {
    node.op.module == "cpu"
        && node.op.instruction == "return_owned_external_buffer"
        && node.op.args.first().map(String::as_str) == Some(producer)
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
            OWNED_BUFFER_BRANCH_TRANSFER_ACTION, OWNED_BUFFER_RETURN_LENGTH_POLICY,
            OWNED_BUFFER_RETURN_PROTOCOL,
        },
        Edge, EdgeKind, Node, Operation, YirFunction, YirFunctionRole, YirModule,
    };

    fn module_with_middle(middle: Option<Node>) -> YirModule {
        let producer = registered_owner(
            "owned",
            "host_owned_buffer_make",
            "host_owned_buffer_destroy",
        );
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

    fn registered_owner(name: &str, symbol: &str, destructor: &str) -> Node {
        let signature = "ref_Buffer(i64)";
        let signature_hash = ffi_symbol_signature_hash("c", symbol, signature);
        let destructor_hash = ffi_symbol_signature_hash("c", destructor, "i64(ref_Buffer)");
        let descriptor = owned_buffer_return_descriptor(destructor, &destructor_hash);
        let capability_hash = ffi_memory_capability_hash("c", symbol, &signature_hash, &descriptor);
        Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "extern_call_owned_buffer".to_owned(),
                args: vec![
                    OWNED_BUFFER_RETURN_PROTOCOL.to_owned(),
                    "c".to_owned(),
                    symbol.to_owned(),
                    signature.to_owned(),
                    signature_hash,
                    capability_hash,
                    OWNED_BUFFER_RETURN_LENGTH_POLICY.to_owned(),
                    destructor.to_owned(),
                    destructor_hash,
                ],
            },
        }
    }

    fn branch_transfer_module(right_destructor: &str) -> YirModule {
        let left = registered_owner(
            "left",
            "host_owned_buffer_make",
            "host_owned_buffer_destroy",
        );
        let right = registered_owner("right", "host_owned_buffer_make", right_destructor);
        let choose = Node {
            name: "choose".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "const_bool".to_owned(),
                args: vec!["true".to_owned()],
            },
        };
        let transfer = Node {
            name: "selected".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "branch_effect".to_owned(),
                args: vec![
                    "choose".to_owned(),
                    "owned_ptr".to_owned(),
                    "address_kind=buffer".to_owned(),
                    "nullable=false".to_owned(),
                    "1".to_owned(),
                    "cpu".to_owned(),
                    OWNED_BUFFER_BRANCH_TRANSFER_ACTION.to_owned(),
                    "owned_ptr".to_owned(),
                    "2".to_owned(),
                    "resource_own".to_owned(),
                    "left".to_owned(),
                    "resource_own".to_owned(),
                    "right".to_owned(),
                    "1".to_owned(),
                    "cpu".to_owned(),
                    OWNED_BUFFER_BRANCH_TRANSFER_ACTION.to_owned(),
                    "owned_ptr".to_owned(),
                    "2".to_owned(),
                    "resource_own".to_owned(),
                    "right".to_owned(),
                    "resource_own".to_owned(),
                    "left".to_owned(),
                ],
            },
        };
        let free = Node {
            name: "release".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "free".to_owned(),
                args: vec!["selected".to_owned()],
            },
        };
        let mut module = YirModule::new("0.1");
        module.nodes = vec![choose, left, right, transfer, free];
        for (from, to) in [
            ("choose", "selected"),
            ("left", "selected"),
            ("right", "selected"),
            ("selected", "release"),
        ] {
            module.edges.push(Edge {
                kind: EdgeKind::Dep,
                from: from.to_owned(),
                to: to.to_owned(),
            });
        }
        module.functions.push(YirFunction {
            name: "main".to_owned(),
            domain: "cffi".to_owned(),
            role: YirFunctionRole::Entry,
            parameters: Vec::new(),
            result: None,
            body_nodes: vec![
                "choose".to_owned(),
                "left".to_owned(),
                "right".to_owned(),
                "selected".to_owned(),
                "release".to_owned(),
            ],
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

    #[test]
    fn accepts_one_registered_branch_transfer_with_matching_destructor_identity() {
        validate_owned_return_buffer_yir(&branch_transfer_module("host_owned_buffer_destroy"))
            .unwrap();
    }

    #[test]
    fn rejects_registered_branch_transfer_with_destructor_identity_drift() {
        let error =
            validate_owned_return_buffer_yir(&branch_transfer_module("other_destroy")).unwrap_err();
        assert!(error.contains("one exact ABI/destructor/hash identity"));
    }

    #[test]
    fn rejects_registered_branch_transfer_before_its_producers() {
        let mut module = branch_transfer_module("host_owned_buffer_destroy");
        module.functions[0].body_nodes.swap(1, 3);
        let error = validate_owned_return_buffer_yir(&module).unwrap_err();
        assert!(error.contains("transferred before its producing call"));
    }
}
