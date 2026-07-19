use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_block, fresh_reg,
    loop_effect_action::{begin_loop_effect_action, finish_loop_effect_action, LoopEffectCleanup},
    value_ref::coerce_to_i64,
    CpuHelperSignature, LlvmValueRef,
};

pub(crate) fn lower_cpu_simple_loop_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &BTreeMap<String, String>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    next_reg: &mut usize,
    next_block: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }

    match node.op.instruction.as_str() {
        "loop_while_i64" | "loop_while_i64_effect" => {
            let initial_value = registers.get(&node.op.args[0]).cloned();
            let limit_value = registers.get(&node.op.args[1]).cloned();
            let step_value = registers.get(&node.op.args[2]).cloned();
            let (Some(initial_value), Some(limit_value), Some(step_value)) =
                (initial_value, limit_value, step_value)
            else {
                body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let Some(initial) = coerce_to_i64(&initial_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because its initial value is not coercible to i64",
                        node.name
                    ));
                return Ok(true);
            };
            let Some(limit) = coerce_to_i64(&limit_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because its limit value is not coercible to i64",
                        node.name
                    ));
                return Ok(true);
            };
            let Some(step) = coerce_to_i64(&step_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because its step value is not coercible to i64",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp_kind = node.op.args[3].as_str();
            let step_kind = node.op.args[4].as_str();
            let owned_return = if node.op.instruction == "loop_while_i64_effect"
                && node.op.args.get(6).map(String::as_str) == Some("scoped_call_owned_return")
            {
                let result_name = node.op.args.get(9).cloned().ok_or_else(|| {
                    format!(
                        "cpu.loop_while_i64_effect `{}` has no owned result projection",
                        node.name
                    )
                })?;
                let source_name = node.op.args[10..]
                    .iter()
                    .find_map(|arg| arg.strip_prefix("move_owned:"))
                    .ok_or_else(|| {
                        format!(
                            "cpu.loop_while_i64_effect `{}` owned return has no moved owner",
                            node.name
                        )
                    })?
                    .to_owned();
                let Some(LlvmValueRef::OwnedBytes { blob }) = registers.get(&source_name) else {
                    return Err(format!(
                        "cpu.loop_while_i64_effect `{}` cannot resolve moved owner `{source_name}`",
                        node.name
                    ));
                };
                Some((result_name, source_name, blob.clone()))
            } else {
                None
            };
            let loop_slot = fresh_reg(next_reg);
            body.push(format!("  {loop_slot} = alloca i64"));
            body.push(format!("  store i64 {initial}, ptr {loop_slot}"));
            let owned_slot = owned_return.as_ref().map(|(_, _, blob)| {
                let slot = fresh_reg(next_reg);
                body.push(format!("  {slot} = alloca ptr"));
                body.push(format!("  store ptr {blob}, ptr {slot}"));
                slot
            });
            let loop_cond = fresh_block(next_block, "loop_while_i64_cond");
            let loop_body = fresh_block(next_block, "loop_while_i64_body");
            let loop_exit = fresh_block(next_block, "loop_while_i64_exit");
            body.push(format!("  br label %{loop_cond}"));
            body.push(format!("{loop_cond}:"));
            let current = fresh_reg(next_reg);
            body.push(format!("  {current} = load i64, ptr {loop_slot}"));
            let cmp = fresh_reg(next_reg);
            let pred = match cmp_kind {
                "eq" => "eq",
                "ne" => "ne",
                "lt" => "slt",
                "le" => "sle",
                "gt" => "sgt",
                "ge" => "sge",
                other => {
                    return Err(format!(
                            "cpu.loop_while_i64 `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name
                        ));
                }
            };
            body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
            body.push(format!(
                "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
            ));
            body.push(format!("{loop_body}:"));
            let mut owned_move_overrides = BTreeMap::new();
            if let (Some((_, source, _)), Some(slot)) = (&owned_return, &owned_slot) {
                let blob = fresh_reg(next_reg);
                body.push(format!("  {blob} = load ptr, ptr {slot}"));
                owned_move_overrides.insert(source.clone(), blob);
            }
            let effect_cleanup = (node.op.instruction == "loop_while_i64_effect")
                .then(|| {
                    begin_loop_effect_action(
                        node,
                        5,
                        body,
                        registers,
                        buffer_lengths,
                        helper_signatures,
                        &owned_move_overrides,
                        &current,
                        next_reg,
                    )
                })
                .transpose()?;
            let next_value = match step_kind {
                "add" => {
                    let reg = fresh_reg(next_reg);
                    body.push(format!("  {reg} = add i64 {current}, {step}"));
                    reg
                }
                "sub" => {
                    let reg = fresh_reg(next_reg);
                    body.push(format!("  {reg} = sub i64 {current}, {step}"));
                    reg
                }
                other => {
                    return Err(format!(
                            "cpu.loop_while_i64 `{}` has unsupported step kind `{other}` during LLVM lowering",
                            node.name
                        ));
                }
            };
            body.push(format!("  store i64 {next_value}, ptr {loop_slot}"));
            if let Some(cleanup) = effect_cleanup {
                if let (LoopEffectCleanup::OwnedResult(blob), Some(slot)) = (&cleanup, &owned_slot)
                {
                    body.push(format!("  store ptr {blob}, ptr {slot}"));
                }
                finish_loop_effect_action(&cleanup, body);
            }
            body.push(format!("  br label %{loop_cond}"));
            body.push(format!("{loop_exit}:"));
            registers.insert(node.name.clone(), LlvmValueRef::I64(current.clone()));
            if let (Some((result, _, _)), Some(slot)) = (owned_return, owned_slot) {
                let blob = fresh_reg(next_reg);
                body.push(format!("  {blob} = load ptr, ptr {slot}"));
                registers.insert(result, LlvmValueRef::OwnedBytes { blob });
            }
            *last_cpu_value = Some(current);
        }
        _ => return Ok(false),
    }

    Ok(true)
}
