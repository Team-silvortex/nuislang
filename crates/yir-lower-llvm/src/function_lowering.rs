use super::*;

#[macro_use]
mod loop_async_cond_chain;
#[macro_use]
mod loop_cond_chain;
#[macro_use]
mod loop_chain;
#[macro_use]
mod loop_async_chain;
#[macro_use]
mod loop_flow_chain;
#[macro_use]
mod loop_async_flow_chain;
#[macro_use]
mod loop_async_flow_cond_chain;
#[macro_use]
mod loop_post_flow_chain;
#[macro_use]
mod loop_async_post_flow_chain;
#[macro_use]
mod loop_async_post_flow_cond_chain;
#[macro_use]
mod loop_post_flow_cond_chain;
#[macro_use]
mod loop_flow_cond_chain;

pub(super) fn emit_cpu_function(
    module: &YirModule,
    resources: &BTreeMap<String, &Resource>,
    ordered_node_names: &[String],
    param_bindings: &BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    function_return_kind: CpuCallScalarKind,
    global_counter: &mut usize,
) -> Result<EmittedCpuFunction, String> {
    let ordered_names = ordered_node_names.iter().collect::<BTreeSet<_>>();
    let deferred_task_calls = ordered_node_names
        .iter()
        .filter_map(|node_name| module.nodes.iter().find(|node| &node.name == node_name))
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .filter_map(|node| node.op.args.get(1))
        .filter(|call_name| ordered_names.contains(call_name))
        .filter(|call_name| {
            module.nodes.iter().any(|candidate| {
                &candidate.name == *call_name
                    && candidate.op.module == "cpu"
                    && candidate.op.instruction == "call_i64"
            })
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut state = LlvmLoweringState {
        body: Vec::new(),
        globals: Vec::new(),
        registers: BTreeMap::new(),
        delayed_registers: BTreeMap::new(),
        facts: KnownFacts::new(),
        buffer_lengths: BTreeMap::new(),
        next_reg: 0,
        next_global: *global_counter,
        next_block: 0,
        last_cpu_value: None,
        ends_with_terminal_return: false,
    };

    for (node_name, value) in param_bindings {
        state.registers.insert(node_name.clone(), value.clone());
        match value {
            LlvmValueRef::Bool { i1, i64 } => {
                state.body.push(format!("  {i64} = zext i1 {i1} to i64"));
                state.last_cpu_value = Some(i64.clone());
            }
            LlvmValueRef::I32(reg) => {
                let widened = fresh_reg(&mut state.next_reg);
                state
                    .body
                    .push(format!("  {widened} = sext i32 {reg} to i64"));
                state.last_cpu_value = Some(widened);
            }
            LlvmValueRef::I64(reg) => state.last_cpu_value = Some(reg.clone()),
            _ => {}
        }
    }

    for node_name in ordered_node_names {
        let node = module
            .nodes
            .iter()
            .find(|node| &node.name == node_name)
            .ok_or_else(|| format!("lowering references unknown node `{node_name}`"))?;
        let resource = resources
            .get(&node.resource)
            .copied()
            .ok_or_else(|| format!("unknown resource `{}`", node.resource))?;
        if state.ends_with_terminal_return {
            continue;
        }

        if resource.kind.is_family("network") && lower_network_observer_node(node, &mut state) {
            continue;
        }

        if !resource.kind.is_family("cpu") {
            state.body.push(format!(
                "  ; deferred lowering for {} on {} ({})",
                node.op.full_name(),
                node.resource,
                resource.kind.raw
            ));
            continue;
        }

        if lower_cpu_async_resource_node(node, &mut state) {
            continue;
        }

        if node.op.module != "cpu" || node.op.instruction != "select" {
            if let Some((input, reason)) =
                first_delayed_input(&node.op.args, &state.delayed_registers)
            {
                state.delayed_registers.insert(
                    node.name.clone(),
                    format!("depends on delayed `{input}`: {reason}"),
                );
                continue;
            }
        }

        match node.op.cpu_llvm_lowering_class() {
            CpuLlvmLoweringClass::Literal => {
                if lower_cpu_literal_node(node, &mut state) {
                    continue;
                }
            }
            CpuLlvmLoweringClass::Aggregate => {
                if lower_cpu_aggregate_node(node, &mut state) {
                    continue;
                }
            }
            CpuLlvmLoweringClass::Pointer => {
                if lower_cpu_pointer_node(node, &mut state) {
                    continue;
                }
            }
            _ => {}
        }

        let body = &mut state.body;
        let _globals = &mut state.globals;
        let registers = &mut state.registers;
        let buffer_lengths = &mut state.buffer_lengths;
        let mut next_reg = &mut state.next_reg;
        let mut next_block = &mut state.next_block;
        let _next_global = &mut state.next_global;
        let last_cpu_value = &mut state.last_cpu_value;

        if lower_cpu_extern_call_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_memory_node(
            node,
            body,
            registers,
            buffer_lengths,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_cast_node(
            node,
            body,
            registers,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_static_node(
            node,
            body,
            registers,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_call_node(
            node,
            body,
            registers,
            helper_signatures,
            &deferred_task_calls,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        match lower_cpu_return_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            ReturnLoweringOutcome::NotReturn => {}
            ReturnLoweringOutcome::Deferred => continue,
            ReturnLoweringOutcome::Returned => {
                state.ends_with_terminal_return = true;
                break;
            }
        }

        if lower_cpu_print_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_bitwise_node(
            node,
            body,
            registers,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_param_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_select_node(
            node,
            body,
            registers,
            &mut state.delayed_registers,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_scalar_equality_node(
            node,
            body,
            registers,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_scalar_order_node(
            node,
            body,
            registers,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_scalar_node(
            node,
            body,
            registers,
            &mut state.facts,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        match lower_cpu_guard_return_node(
            node,
            body,
            registers,
            &mut next_reg,
            &mut next_block,
            function_return_kind,
        )? {
            GuardReturnLoweringOutcome::NotGuard => {}
            GuardReturnLoweringOutcome::Continue => continue,
            GuardReturnLoweringOutcome::TerminalReturn => {
                state.ends_with_terminal_return = true;
                continue;
            }
        }

        if lower_cpu_simple_loop_node(
            node,
            body,
            registers,
            &mut next_reg,
            &mut next_block,
            last_cpu_value,
        )? {
            continue;
        }

        match (node.op.module.as_str(), node.op.instruction.as_str()) {
            ("cpu", "text")
            | ("cpu", "const_bool")
            | ("cpu", "const_i32")
            | ("cpu", "const")
            | ("cpu", "const_i64")
            | ("cpu", "const_f32")
            | ("cpu", "const_f64")
            | ("cpu", "struct")
            | ("cpu", "field")
            | ("cpu", "variant_is")
            | ("cpu", "variant_field")
            | ("cpu", "null")
            | ("cpu", "borrow")
            | ("cpu", "borrow_end")
            | ("cpu", "move_ptr") => unreachable!(
                "preclassified CPU LLVM lowering op `{}` should have been handled earlier",
                node.op.full_name()
            ),
            ("cpu", "loop_while_i64_chain" | "loop_while_scalar_chain") => {
                lower_loop_chain!(node, body, registers, next_reg, next_block, last_cpu_value);
            }
            ("cpu", "loop_while_i64_async_chain" | "loop_while_scalar_async_chain") => {
                lower_loop_async_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            ("cpu", "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain") => {
                lower_loop_async_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            ("cpu", "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain") => {
                lower_loop_cond_chain!(node, body, registers, next_reg, next_block, last_cpu_value);
            }
            ("cpu", "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain") => {
                lower_loop_flow_chain!(node, body, registers, next_reg, next_block, last_cpu_value);
            }
            ("cpu", "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain") => {
                lower_loop_async_flow_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            (
                "cpu",
                "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain",
            ) => {
                lower_loop_async_flow_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            ("cpu", "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain") => {
                lower_loop_post_flow_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value
                );
            }
            (
                "cpu",
                "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain",
            ) => {
                lower_loop_async_post_flow_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            (
                "cpu",
                "loop_while_i64_async_post_flow_cond_chain"
                | "loop_while_scalar_async_post_flow_cond_chain",
            ) => {
                lower_loop_async_post_flow_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            (
                "cpu",
                "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain",
            ) => {
                lower_loop_post_flow_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value
                );
            }
            ("cpu", "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain") => {
                lower_loop_flow_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value
                );
            }
            _ => {
                body.push(format!(
                    "  ; deferred lowering for {} on {} ({})",
                    node.op.full_name(),
                    node.resource,
                    resource.kind.raw
                ));
            }
        }
    }

    *global_counter = state.next_global;
    let ret = state.last_cpu_value.unwrap_or_else(|| "0".to_owned());
    let body = if state.ends_with_terminal_return {
        state.body.join("\n")
    } else {
        let mut body = state.body;
        emit_typed_return_from_last_value(
            &mut body,
            &mut state.next_reg,
            function_return_kind,
            &ret,
        );
        body.join("\n")
    };

    Ok(EmittedCpuFunction {
        globals: state.globals,
        body,
    })
}

fn first_delayed_input<'a>(
    args: &'a [String],
    delayed: &'a BTreeMap<String, String>,
) -> Option<(&'a str, &'a str)> {
    args.iter().find_map(|arg| {
        let value_name = arg.split_once('=').map_or(arg.as_str(), |(_, value)| value);
        delayed
            .get(value_name.trim())
            .map(|reason| (value_name.trim(), reason.as_str()))
    })
}
