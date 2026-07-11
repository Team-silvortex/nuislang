macro_rules! lower_loop_async_post_flow_cond_chain {
    ($node:expr, $body:expr, $registers:expr, $next_reg:expr, $next_block:expr, $last_cpu_value:expr, $helper_signatures:expr) => {{
        let node = $node;
        let mut body = $body;
        let registers = $registers;
        let mut next_reg = $next_reg;
        let mut next_block = $next_block;
        let last_cpu_value = $last_cpu_value;
        let helper_signatures = $helper_signatures;
let loop_instruction = canonical_loop_instruction(&node.op.instruction);
let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
let initial_value = registers.get(&node.op.args[0]).cloned();
let limit_value = registers.get(&node.op.args[1]).cloned();
let (flow_expr, carry_start_index) =
    parse_loop_flow_expr_for_llvm(&node.op.args, 4, &node.name, loop_instruction)?;
let callee = &node.op.args[2];
let Some(signature) = helper_signatures.get(callee) else {
    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable", node.name, callee));
    continue;
};
if signature.params.as_slice() != [CpuCallScalarKind::I64]
    || signature.ret != CpuCallScalarKind::I64
{
    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`", node.name, callee));
    continue;
}
let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice", node.name));
    continue;
};
let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64", node.name));
    continue;
};
let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64", node.name));
    continue;
};
let cmp_kind = node.op.args[3].as_str();
let Some(resolved_flow_expr) = resolve_loop_flow_expr_for_llvm(
    &flow_expr,
    &registers,
    &mut body,
    &mut next_reg,
    &node.name,
    loop_instruction,
) else {
    continue;
};

let mut carry_initials = Vec::new();
let mut carry_specs =
    Vec::<(String, Option<String>, Vec<String>, Vec<String>)>::new();
let mut deferred = false;
let mut cursor = carry_start_index;
while cursor < node.op.args.len() {
    let chunk0 = node.op.args.get(cursor);
    let chunk1 = node.op.args.get(cursor + 1);
    let chunk2 = node.op.args.get(cursor + 2);
    let Some(initial_name) = chunk0 else {
        break;
    };
    let Some(cond_kind_name) = chunk1 else {
        return Err(format!("cpu.{loop_instruction} `{}` has truncated carry spec during LLVM lowering", node.name));
    };
    let Some(cond_rhs_name) = chunk2 else {
        return Err(format!("cpu.{loop_instruction} `{}` has truncated carry spec during LLVM lowering", node.name));
    };
    let Some(iv) = registers.get(initial_name).cloned() else {
        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice", node.name));
        deferred = true;
        break;
    };
    let Some(init) = coerce_to_i64(&iv, &mut body, &mut next_reg) else {
        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64", node.name));
        deferred = true;
        break;
    };
    let cond_rhs = if cond_kind_name == "always" {
        None
    } else {
        let Some(v) = registers.get(cond_rhs_name).cloned() else {
            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice", node.name));
            deferred = true;
            break;
        };
        let Some(rhs) = coerce_to_i64(&v, &mut body, &mut next_reg) else {
            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64", node.name));
            deferred = true;
            break;
        };
        Some(rhs)
    };
    let then_start = cursor + 3;
    let Some(then_kind) = node.op.args.get(then_start).cloned() else {
        return Err(format!("cpu.{loop_instruction} `{}` has truncated then carry source during LLVM lowering", node.name));
    };
    let Some(then_payload_len) =
        async_post_flow_carry_source_payload_len(&then_kind)
    else {
        return Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{then_kind}` during LLVM lowering", node.name));
    };
    let then_end = then_start + 1 + then_payload_len;
    if then_end > node.op.args.len() {
        return Err(format!("cpu.{loop_instruction} `{}` is missing payload for carry kind `{then_kind}` during LLVM lowering", node.name));
    }
    let mut then_source = vec![then_kind.clone()];
    for payload_name in &node.op.args[then_start + 1..then_end] {
        let Some(payload_value) = registers.get(payload_name).cloned() else {
            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is outside the current CPU LLVM slice", node.name));
            deferred = true;
            break;
        };
        let Some(payload) = coerce_to_i64(&payload_value, &mut body, &mut next_reg)
        else {
            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is not coercible to i64", node.name));
            deferred = true;
            break;
        };
        then_source.push(payload);
    }
    if deferred {
        break;
    }
    let Some(else_kind) = node.op.args.get(then_end).cloned() else {
        return Err(format!("cpu.{loop_instruction} `{}` has truncated else carry source during LLVM lowering", node.name));
    };
    let Some(else_payload_len) =
        async_post_flow_carry_source_payload_len(&else_kind)
    else {
        return Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{else_kind}` during LLVM lowering", node.name));
    };
    let else_end = then_end + 1 + else_payload_len;
    if else_end > node.op.args.len() {
        return Err(format!("cpu.{loop_instruction} `{}` is missing payload for carry kind `{else_kind}` during LLVM lowering", node.name));
    }
    let mut else_source = vec![else_kind.clone()];
    for payload_name in &node.op.args[then_end + 1..else_end] {
        let Some(payload_value) = registers.get(payload_name).cloned() else {
            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is outside the current CPU LLVM slice", node.name));
            deferred = true;
            break;
        };
        let Some(payload) = coerce_to_i64(&payload_value, &mut body, &mut next_reg)
        else {
            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is not coercible to i64", node.name));
            deferred = true;
            break;
        };
        else_source.push(payload);
    }
    if deferred {
        break;
    }
    carry_initials.push(init);
    carry_specs.push((cond_kind_name.clone(), cond_rhs, then_source, else_source));
    cursor = else_end;
}
if deferred {
    continue;
}
let current_slot = fresh_reg(&mut next_reg);
body.push(format!("  {current_slot} = alloca i64"));
body.push(format!("  store i64 {initial}, ptr {current_slot}"));
let carry_slots = carry_initials
    .iter()
    .map(|init| {
        let s = fresh_reg(&mut next_reg);
        body.push(format!("  {s} = alloca i64"));
        body.push(format!("  store i64 {init}, ptr {s}"));
        s
    })
    .collect::<Vec<_>>();
let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
let loop_continue =
    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
body.push(format!("  br label %{loop_cond}"));
body.push(format!("{loop_cond}:"));
let current = fresh_reg(&mut next_reg);
body.push(format!("  {current} = load i64, ptr {current_slot}"));
let cmp = fresh_reg(&mut next_reg);
let pred=match cmp_kind { "eq"=>"eq","ne"=>"ne","lt"=>"slt","le"=>"sle","gt"=>"sgt","ge"=>"sge",other=>return Err(format!("cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering", node.name)), };
body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
body.push(format!(
    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
));
body.push(format!("{loop_body}:"));
let next_current = fresh_reg(&mut next_reg);
body.push(format!(
    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
));
let mut current_carries = Vec::new();
for slot in &carry_slots {
    let r = fresh_reg(&mut next_reg);
    body.push(format!("  {r} = load i64, ptr {slot}"));
    current_carries.push(r);
}
let mut next_carries = Vec::new();
for (index, (cond_kind, cond_rhs, then_source_spec, else_source_spec)) in
    carry_specs.iter().enumerate()
{
    let then_value =
        if matches!(then_source_spec[0].as_str(), "keep" | "keep_prev_carry") {
            current_carries[index].clone()
        } else {
            let src = resolve_source_for_async_post_flow(
                then_source_spec,
                &current,
                &next_current,
                &current_carries,
                &next_carries,
                &mut body,
                &mut next_reg,
                &node.name,
                loop_instruction,
            )?;
            let r = fresh_reg(&mut next_reg);
            body.push(format!(
                "  {r} = add i64 {}, {}",
                current_carries[index], src
            ));
            r
        };
    let else_value =
        if matches!(else_source_spec[0].as_str(), "keep" | "keep_prev_carry") {
            current_carries[index].clone()
        } else {
            let src = resolve_source_for_async_post_flow(
                else_source_spec,
                &current,
                &next_current,
                &current_carries,
                &next_carries,
                &mut body,
                &mut next_reg,
                &node.name,
                loop_instruction,
            )?;
            let r = fresh_reg(&mut next_reg);
            body.push(format!(
                "  {r} = add i64 {}, {}",
                current_carries[index], src
            ));
            r
        };
    let next_carry = if cond_kind == "always" {
        then_value
    } else {
        let rhs=cond_rhs.clone().ok_or_else(|| format!("cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering", node.name))?;
        let (lhs,pred)=match cond_kind.as_str() {
            "current_eq"=>(next_current.clone(),"eq"), "current_ne"=>(next_current.clone(),"ne"), "current_lt"=>(next_current.clone(),"slt"), "current_le"=>(next_current.clone(),"sle"), "current_gt"=>(next_current.clone(),"sgt"), "current_ge"=>(next_current.clone(),"sge"),
            "prev_current_eq"=>(current.clone(),"eq"), "prev_current_ne"=>(current.clone(),"ne"), "prev_current_lt"=>(current.clone(),"slt"), "prev_current_le"=>(current.clone(),"sle"), "prev_current_gt"=>(current.clone(),"sgt"), "prev_current_ge"=>(current.clone(),"sge"),
            other if other.starts_with("prev_carry") && other.ends_with("_eq") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
            other if other.starts_with("prev_carry") && other.ends_with("_ne") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
            other if other.starts_with("prev_carry") && other.ends_with("_lt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
            other if other.starts_with("prev_carry") && other.ends_with("_le") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
            other if other.starts_with("prev_carry") && other.ends_with("_gt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
            other if other.starts_with("prev_carry") && other.ends_with("_ge") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
            other if other.starts_with("carry") && other.ends_with("_eq") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
            other if other.starts_with("carry") && other.ends_with("_ne") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
            other if other.starts_with("carry") && other.ends_with("_lt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
            other if other.starts_with("carry") && other.ends_with("_le") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
            other if other.starts_with("carry") && other.ends_with("_gt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
            other if other.starts_with("carry") && other.ends_with("_ge") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
            other => return Err(format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name)),
        };
        let c = fresh_reg(&mut next_reg);
        body.push(format!("  {c} = icmp {pred} i64 {lhs}, {rhs}"));
        let s = fresh_reg(&mut next_reg);
        body.push(format!(
            "  {s} = select i1 {c}, i64 {then_value}, i64 {else_value}"
        ));
        s
    };
    next_carries.push(next_carry);
}
let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
let condition_blocks = (0..flow_leaves.len())
    .map(|index| {
        if index == 0 {
            None
        } else {
            Some(fresh_block(&mut next_block, "loop_async_post_flow_rhs"))
        }
    })
    .collect::<Vec<_>>();
for (index, (condition, action)) in flow_leaves.iter().enumerate() {
    if let Some(block) = &condition_blocks[index] {
        body.push(format!("{block}:"));
    }
    let no_match_block = condition_blocks
        .get(index + 1)
        .and_then(|block| block.clone())
        .unwrap_or_else(|| loop_continue.clone());
    let control_cond = emit_loop_flow_control_expr(
        condition,
        &next_current,
        &next_carries,
        &mut body,
        &mut next_reg,
        &node.name,
        loop_instruction,
    )?;
    let action_block = fresh_block(&mut next_block, "loop_async_post_flow_action");
    body.push(format!(
        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
    ));
    body.push(format!("{action_block}:"));
    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
    for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
        body.push(format!("  store i64 {val}, ptr {slot}"));
    }
    match *action {
        "break" => body.push(format!("  br label %{loop_exit}")),
        "continue" => body.push(format!("  br label %{loop_cond}")),
        other => {
            return Err(format!(
                "cpu.{loop_instruction} `{}` has unsupported flow action `{other}` during LLVM lowering",
                node.name,
            ));
        }
    }
}
body.push(format!("{loop_continue}:"));
body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
    body.push(format!("  store i64 {val}, ptr {slot}"));
}
body.push(format!("  br label %{loop_cond}"));
body.push(format!("{loop_exit}:"));
let final_current = fresh_reg(&mut next_reg);
body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
let final_carries = carry_slots
    .iter()
    .map(|slot| {
        let r = fresh_reg(&mut next_reg);
        body.push(format!("  {r} = load i64, ptr {slot}"));
        r
    })
    .collect::<Vec<_>>();
insert_i64_loop_chain_result(
    registers,
    &node.name,
    final_current,
    final_carries,
    last_cpu_value,
);
    }};
}
