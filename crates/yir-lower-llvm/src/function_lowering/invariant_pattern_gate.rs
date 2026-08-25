use super::super::*;
use yir_core::Node;

pub(super) fn lower_false_invariant_post_flow_loop(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    facts: &KnownFacts,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    let conditional = match (node.op.module.as_str(), node.op.instruction.as_str()) {
        ("cpu", "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain") => false,
        (
            "cpu",
            "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain",
        ) => true,
        _ => return Ok(false),
    };
    let Some(compare_kind) = node.op.args.get(3).map(String::as_str) else {
        return Ok(false);
    };
    if !matches!(compare_kind, "invariant_true" | "pattern_exit") {
        return Ok(false);
    }
    let carry_start = if conditional {
        let instruction = canonical_loop_instruction(&node.op.instruction);
        parse_loop_flow_expr_for_llvm(&node.op.args, 5, &node.name, instruction)?.1
    } else {
        8
    };
    let carry_names = if conditional {
        yir_domain_cpu::parse_conditional_carries(&node.op.args, carry_start, &node.name, true)?
            .into_iter()
            .map(|carry| carry.initial)
            .collect::<Vec<_>>()
    } else {
        node.op.args[carry_start..]
            .chunks(2)
            .filter_map(|chunk| chunk.first().cloned())
            .collect::<Vec<_>>()
    };
    let gate_name = &node.op.args[1];
    let gate_is_false = facts.get_bool(gate_name) == Some(false)
        || facts.get_i64(gate_name) == Some(0)
        || matches!(
            registers.get(gate_name),
            Some(LlvmValueRef::Bool { i1, .. }) if i1 == "false"
        )
        || matches!(registers.get(gate_name), Some(LlvmValueRef::I64(value)) if value == "0");
    if !gate_is_false {
        return Ok(false);
    }

    let Some(initial_value) = registers.get(&node.op.args[0]).cloned() else {
        return Ok(false);
    };
    let Some(initial) = coerce_to_i64(&initial_value, body, next_reg) else {
        return Ok(false);
    };
    let mut final_value = initial.clone();
    let mut fields = vec![("current".to_owned(), LlvmValueRef::I64(initial))];
    for (index, carry_name) in carry_names.iter().enumerate() {
        let Some(carry_value) = registers.get(carry_name).cloned() else {
            return Ok(false);
        };
        let Some(carry) = coerce_to_i64(&carry_value, body, next_reg) else {
            return Ok(false);
        };
        final_value = carry.clone();
        fields.push((format!("carry{index}"), LlvmValueRef::I64(carry)));
    }

    *last_cpu_value = Some(final_value);
    registers.insert(
        node.name.clone(),
        LlvmValueRef::Struct(StructLlvmValueRef {
            type_name: "LoopChain".to_owned(),
            fields,
        }),
    );
    Ok(true)
}
