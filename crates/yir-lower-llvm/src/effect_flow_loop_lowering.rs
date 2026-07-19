use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_block, fresh_reg,
    loop_async_post_flow_source::try_resolve_loop_carry_add_scaled_source,
    loop_carry_payload::loop_carry_payload_len,
    loop_carry_scaled_source::try_resolve_loop_carry_scaled_source,
    loop_carry_source::resolve_loop_carry_term,
    loop_effect_action::{begin_loop_effect_action, finish_loop_effect_action},
    loop_expr::{
        collect_resolved_loop_flow_leaves, parse_loop_flow_expr_for_llvm,
        resolve_loop_flow_expr_for_llvm, ResolvedLoopControlExpr,
    },
    loop_flow_control_lowering::emit_loop_flow_control_expr,
    value_ref::coerce_to_i64,
    CpuHelperSignature, CpuLoopScalarKind, LlvmValueRef, StructLlvmValueRef,
};

pub(crate) fn lower_cpu_effect_flow_loop_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &BTreeMap<String, String>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    next_reg: &mut usize,
    next_block: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" || node.op.instruction != "loop_while_i64_effect_flow" {
        return Ok(false);
    }
    let values = node.op.args[..3]
        .iter()
        .map(|name| {
            registers
                .get(name)
                .and_then(|value| coerce_to_i64(value, body, next_reg))
        })
        .collect::<Option<Vec<_>>>();
    let control_count = node.op.args[5].parse::<usize>().map_err(|_| {
        format!(
            "cpu.loop_while_i64_effect_flow `{}` has invalid control token count `{}`",
            node.name, node.op.args[5]
        )
    })?;
    let control_end = 6 + control_count;
    let (control_expr, after_control) =
        parse_loop_flow_expr_for_llvm(&node.op.args, 6, &node.name, "loop_while_i64_effect_flow")?;
    if after_control != control_end {
        return Err(unsupported(
            node,
            "control payload",
            &control_count.to_string(),
        ));
    }
    let carry_count = node.op.args[control_end].parse::<usize>().map_err(|_| {
        format!(
            "cpu.loop_while_i64_effect_flow `{}` has invalid carry count `{}`",
            node.name, node.op.args[control_end]
        )
    })?;
    let carry_offset = control_end + 1;
    let mut action_offset = carry_offset;
    let mut carry_values = Vec::with_capacity(carry_count);
    let mut carry_specs = Vec::with_capacity(carry_count);
    let mut carries_available = true;
    for _ in 0..carry_count {
        let initial_name = &node.op.args[action_offset];
        let kind = node.op.args[action_offset + 1].clone();
        let payload_len = loop_carry_payload_len(&kind);
        let payload_start = action_offset + 2;
        let payload_end = payload_start + payload_len;
        let initial = registers
            .get(initial_name)
            .and_then(|value| coerce_to_i64(value, body, next_reg));
        let payloads = node.op.args[payload_start..payload_end]
            .iter()
            .map(|name| {
                registers
                    .get(name)
                    .and_then(|value| coerce_to_i64(value, body, next_reg))
            })
            .collect::<Option<Vec<_>>>();
        let (Some(initial), Some(payloads)) = (initial, payloads) else {
            carries_available = false;
            break;
        };
        carry_values.push(initial);
        carry_specs.push((kind, payloads));
        action_offset = payload_end;
    }
    let resolved_control = resolve_loop_flow_expr_for_llvm(
        &control_expr,
        registers,
        body,
        next_reg,
        &node.name,
        "loop_while_i64_effect_flow",
    );
    let (Some(values), Some(resolved_control)) = (values, resolved_control) else {
        body.push(format!(
            "  ; deferred lowering for cpu.loop_while_i64_effect_flow `{}` because scalar inputs are unavailable",
            node.name
        ));
        return Ok(true);
    };
    if !carries_available {
        body.push(format!(
            "  ; deferred lowering for cpu.loop_while_i64_effect_flow `{}` because carry inputs are unavailable",
            node.name
        ));
        return Ok(true);
    }
    let slot = fresh_reg(next_reg);
    body.push(format!("  {slot} = alloca i64"));
    body.push(format!("  store i64 {}, ptr {slot}", values[0]));
    let carry_slots = carry_values
        .iter()
        .map(|initial| {
            let carry_slot = fresh_reg(next_reg);
            body.push(format!("  {carry_slot} = alloca i64"));
            body.push(format!("  store i64 {initial}, ptr {carry_slot}"));
            carry_slot
        })
        .collect::<Vec<_>>();
    let cond_block = fresh_block(next_block, "loop_effect_flow_cond");
    let body_block = fresh_block(next_block, "loop_effect_flow_body");
    let update_block = fresh_block(next_block, "loop_effect_flow_update");
    let exit_block = fresh_block(next_block, "loop_effect_flow_exit");
    body.push(format!("  br label %{cond_block}"));
    body.push(format!("{cond_block}:"));
    let current = fresh_reg(next_reg);
    body.push(format!("  {current} = load i64, ptr {slot}"));
    let loop_cmp = emit_compare(body, next_reg, &current, &values[1], &node.op.args[3], node)?;
    body.push(format!(
        "  br i1 {loop_cmp}, label %{body_block}, label %{exit_block}"
    ));
    body.push(format!("{body_block}:"));
    let cleanup = begin_loop_effect_action(
        node,
        action_offset,
        body,
        registers,
        buffer_lengths,
        helper_signatures,
        &BTreeMap::new(),
        &current,
        next_reg,
    )?;
    let next = emit_step(body, next_reg, &current, &values[2], &node.op.args[4], node)?;
    body.push(format!("  store i64 {next}, ptr {slot}"));
    let current_carries = carry_slots
        .iter()
        .map(|carry_slot| {
            let current_carry = fresh_reg(next_reg);
            body.push(format!("  {current_carry} = load i64, ptr {carry_slot}"));
            current_carry
        })
        .collect::<Vec<_>>();
    let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
    collect_resolved_loop_flow_leaves(&resolved_control, &mut flow_leaves);
    let condition_blocks = (0..flow_leaves.len())
        .map(|index| {
            if index == 0 {
                None
            } else {
                Some(fresh_block(next_block, "loop_effect_flow_rhs"))
            }
        })
        .collect::<Vec<_>>();
    for (index, (condition, action)) in flow_leaves.iter().enumerate() {
        if let Some(block) = &condition_blocks[index] {
            body.push(format!("{block}:"));
        }
        let no_match_block = condition_blocks
            .get(index + 1)
            .and_then(Clone::clone)
            .unwrap_or_else(|| update_block.clone());
        let control_cmp = emit_loop_flow_control_expr(
            condition,
            &next,
            &current_carries,
            body,
            next_reg,
            &node.name,
            "loop_while_i64_effect_flow",
        )?;
        let action_block = fresh_block(next_block, "loop_effect_flow_action");
        body.push(format!(
            "  br i1 {control_cmp}, label %{action_block}, label %{no_match_block}"
        ));
        body.push(format!("{action_block}:"));
        finish_loop_effect_action(&cleanup, body);
        match *action {
            "break" => body.push(format!("  br label %{exit_block}")),
            "continue" => body.push(format!("  br label %{cond_block}")),
            other => return Err(unsupported(node, "control action", other)),
        }
    }
    body.push(format!("{update_block}:"));
    let mut next_carries = Vec::with_capacity(carry_slots.len());
    for (carry_slot, (kind, payloads)) in carry_slots.iter().zip(carry_specs.iter()) {
        let (source, op) = resolve_effect_flow_carry_source(
            kind,
            payloads,
            &current,
            &next,
            &current_carries,
            &next_carries,
            body,
            next_reg,
            node,
        )?;
        let current_carry = fresh_reg(next_reg);
        body.push(format!("  {current_carry} = load i64, ptr {carry_slot}"));
        let next_carry = fresh_reg(next_reg);
        body.push(format!(
            "  {next_carry} = {op} i64 {current_carry}, {source}"
        ));
        body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
        next_carries.push(next_carry);
    }
    finish_loop_effect_action(&cleanup, body);
    body.push(format!("  br label %{cond_block}"));
    body.push(format!("{exit_block}:"));
    let final_current = fresh_reg(next_reg);
    body.push(format!("  {final_current} = load i64, ptr {slot}"));
    let mut fields = vec![(
        "current".to_owned(),
        LlvmValueRef::I64(final_current.clone()),
    )];
    for (index, carry_slot) in carry_slots.iter().enumerate() {
        let final_carry = fresh_reg(next_reg);
        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
        fields.push((format!("carry{index}"), LlvmValueRef::I64(final_carry)));
    }
    registers.insert(
        node.name.clone(),
        LlvmValueRef::Struct(StructLlvmValueRef {
            type_name: "LoopEffectFlow".to_owned(),
            fields,
        }),
    );
    *last_cpu_value = Some(final_current);
    Ok(true)
}

fn resolve_effect_flow_carry_source(
    kind: &str,
    payloads: &[String],
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node: &Node,
) -> Result<(String, &'static str), String> {
    if kind == "add_current" {
        return Ok((next_current.to_owned(), "add"));
    }
    if matches!(kind, "add_invariant" | "mul_invariant") {
        let source = payloads
            .first()
            .cloned()
            .ok_or_else(|| unsupported(node, "carry payload", kind))?;
        return Ok((
            source,
            if kind == "add_invariant" {
                "add"
            } else {
                "mul"
            },
        ));
    }
    if let Some(resolved) = try_resolve_loop_carry_scaled_source(
        body,
        next_reg,
        CpuLoopScalarKind::I64,
        kind,
        payloads,
        current,
        next_current,
        current_carries,
        next_carries,
        &node.name,
        "loop_while_i64_effect_flow",
    )? {
        return Ok(resolved);
    }
    if let Some(source) = try_resolve_loop_carry_add_scaled_source(
        kind,
        payloads,
        current,
        next_current,
        current_carries,
        next_carries,
        body,
        next_reg,
        &node.name,
        "loop_while_i64_effect_flow",
    )? {
        return Ok((source, "add"));
    }
    if let Some((terms, op)) = kind
        .strip_prefix("add_")
        .map(|terms| (terms, "add"))
        .or_else(|| kind.strip_prefix("mul_").map(|terms| (terms, "mul")))
    {
        let (terms, has_invariant) = terms
            .strip_suffix("_plus_invariant")
            .map_or((terms, false), |terms| (terms, true));
        let terms = terms.split("_plus_").collect::<Vec<_>>();
        if terms.len() >= 2 {
            let mut source = resolve_loop_carry_term(
                terms[0],
                kind,
                current,
                next_current,
                current_carries,
                next_carries,
                &node.name,
                "loop_while_i64_effect_flow",
            )?;
            for term in &terms[1..] {
                let rhs = resolve_loop_carry_term(
                    term,
                    kind,
                    current,
                    next_current,
                    current_carries,
                    next_carries,
                    &node.name,
                    "loop_while_i64_effect_flow",
                )?;
                source = emit_effect_flow_add(body, next_reg, &source, &rhs);
            }
            if has_invariant {
                let invariant = payloads
                    .first()
                    .ok_or_else(|| unsupported(node, "carry payload", kind))?;
                source = emit_effect_flow_add(body, next_reg, &source, invariant);
            }
            return Ok((source, op));
        }
    }
    if matches!(
        kind,
        "add_current_plus_invariant" | "mul_current_plus_invariant"
    ) {
        let invariant = payloads
            .first()
            .ok_or_else(|| unsupported(node, "carry payload", kind))?;
        let source = fresh_reg(next_reg);
        body.push(format!("  {source} = add i64 {next_current}, {invariant}"));
        return Ok((
            source,
            if kind.starts_with("add_") {
                "add"
            } else {
                "mul"
            },
        ));
    }
    if let Some(source_index) = kind
        .strip_prefix("add_carry")
        .and_then(|value| value.parse::<usize>().ok())
    {
        return next_carries
            .get(source_index)
            .cloned()
            .map(|source| (source, "add"))
            .ok_or_else(|| unsupported(node, "carry kind", kind));
    }
    Err(unsupported(node, "carry kind", kind))
}

fn emit_effect_flow_add(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    lhs: &str,
    rhs: &str,
) -> String {
    let result = fresh_reg(next_reg);
    body.push(format!("  {result} = add i64 {lhs}, {rhs}"));
    result
}

fn emit_compare(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    lhs: &str,
    rhs: &str,
    kind: &str,
    node: &Node,
) -> Result<String, String> {
    let pred = match kind {
        "eq" => "eq",
        "ne" => "ne",
        "lt" => "slt",
        "le" => "sle",
        "gt" => "sgt",
        "ge" => "sge",
        other => return Err(unsupported(node, "compare kind", other)),
    };
    let reg = fresh_reg(next_reg);
    body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
    Ok(reg)
}

fn emit_step(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    current: &str,
    step: &str,
    kind: &str,
    node: &Node,
) -> Result<String, String> {
    let op = match kind {
        "add" => "add",
        "sub" => "sub",
        other => return Err(unsupported(node, "step kind", other)),
    };
    let reg = fresh_reg(next_reg);
    body.push(format!("  {reg} = {op} i64 {current}, {step}"));
    Ok(reg)
}

fn unsupported(node: &Node, field: &str, value: &str) -> String {
    format!(
        "cpu.loop_while_i64_effect_flow `{}` has unsupported {field} `{value}`",
        node.name
    )
}
