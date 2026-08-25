use super::{fresh_reg, ResolvedLoopControlExpr};

pub(crate) fn emit_loop_flow_control_expr(
    expr: &ResolvedLoopControlExpr,
    next_current: &String,
    next_carries: &[String],
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    emit_loop_flow_control_expr_with_previous(
        expr,
        next_current,
        next_carries,
        None,
        None,
        body,
        next_reg,
        node_name,
        loop_instruction,
    )
}

pub(crate) fn emit_post_loop_flow_control_expr(
    expr: &ResolvedLoopControlExpr,
    next_current: &String,
    next_carries: &[String],
    previous_current: &String,
    previous_carries: &[String],
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    emit_loop_flow_control_expr_with_previous(
        expr,
        next_current,
        next_carries,
        Some(previous_current),
        Some(previous_carries),
        body,
        next_reg,
        node_name,
        loop_instruction,
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_loop_flow_control_expr_with_previous(
    expr: &ResolvedLoopControlExpr,
    next_current: &String,
    next_carries: &[String],
    previous_current: Option<&String>,
    previous_carries: Option<&[String]>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    match expr {
        ResolvedLoopControlExpr::Cond { kind, rhs } => {
            let (lhs, pred) = resolve_loop_flow_control_operand(
                kind,
                next_current,
                next_carries,
                previous_current,
                previous_carries,
                node_name,
                loop_instruction,
            )?;
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
            Ok(reg)
        }
        ResolvedLoopControlExpr::And(lhs, rhs) => {
            let lhs_reg = emit_loop_flow_control_expr_with_previous(
                lhs,
                next_current,
                next_carries,
                previous_current,
                previous_carries,
                body,
                next_reg,
                node_name,
                loop_instruction,
            )?;
            let rhs_reg = emit_loop_flow_control_expr_with_previous(
                rhs,
                next_current,
                next_carries,
                previous_current,
                previous_carries,
                body,
                next_reg,
                node_name,
                loop_instruction,
            )?;
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = and i1 {lhs_reg}, {rhs_reg}"));
            Ok(reg)
        }
        ResolvedLoopControlExpr::Or(lhs, rhs) => {
            let lhs_reg = emit_loop_flow_control_expr_with_previous(
                lhs,
                next_current,
                next_carries,
                previous_current,
                previous_carries,
                body,
                next_reg,
                node_name,
                loop_instruction,
            )?;
            let rhs_reg = emit_loop_flow_control_expr_with_previous(
                rhs,
                next_current,
                next_carries,
                previous_current,
                previous_carries,
                body,
                next_reg,
                node_name,
                loop_instruction,
            )?;
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = or i1 {lhs_reg}, {rhs_reg}"));
            Ok(reg)
        }
    }
}

fn resolve_loop_flow_control_operand(
    kind: &str,
    next_current: &String,
    next_carries: &[String],
    previous_current: Option<&String>,
    previous_carries: Option<&[String]>,
    node_name: &str,
    loop_instruction: &str,
) -> Result<(String, &'static str), String> {
    match kind {
        "current_eq" => Ok((next_current.clone(), "eq")),
        "current_ne" => Ok((next_current.clone(), "ne")),
        "current_lt" => Ok((next_current.clone(), "slt")),
        "current_le" => Ok((next_current.clone(), "sle")),
        "current_gt" => Ok((next_current.clone(), "sgt")),
        "current_ge" => Ok((next_current.clone(), "sge")),
        "prev_current_eq" => previous_control_operand(
            previous_current,
            kind,
            node_name,
            loop_instruction,
            "eq",
        ),
        "prev_current_ne" => previous_control_operand(
            previous_current,
            kind,
            node_name,
            loop_instruction,
            "ne",
        ),
        "prev_current_lt" => previous_control_operand(
            previous_current,
            kind,
            node_name,
            loop_instruction,
            "slt",
        ),
        "prev_current_le" => previous_control_operand(
            previous_current,
            kind,
            node_name,
            loop_instruction,
            "sle",
        ),
        "prev_current_gt" => previous_control_operand(
            previous_current,
            kind,
            node_name,
            loop_instruction,
            "sgt",
        ),
        "prev_current_ge" => previous_control_operand(
            previous_current,
            kind,
            node_name,
            loop_instruction,
            "sge",
        ),
        other if other.starts_with("prev_carry") && other.ends_with("_eq") => {
            resolve_previous_carry_operand(
                other,
                "_eq",
                previous_carries,
                node_name,
                loop_instruction,
                "eq",
            )
        }
        other if other.starts_with("prev_carry") && other.ends_with("_ne") => {
            resolve_previous_carry_operand(
                other,
                "_ne",
                previous_carries,
                node_name,
                loop_instruction,
                "ne",
            )
        }
        other if other.starts_with("prev_carry") && other.ends_with("_lt") => {
            resolve_previous_carry_operand(
                other,
                "_lt",
                previous_carries,
                node_name,
                loop_instruction,
                "slt",
            )
        }
        other if other.starts_with("prev_carry") && other.ends_with("_le") => {
            resolve_previous_carry_operand(
                other,
                "_le",
                previous_carries,
                node_name,
                loop_instruction,
                "sle",
            )
        }
        other if other.starts_with("prev_carry") && other.ends_with("_gt") => {
            resolve_previous_carry_operand(
                other,
                "_gt",
                previous_carries,
                node_name,
                loop_instruction,
                "sgt",
            )
        }
        other if other.starts_with("prev_carry") && other.ends_with("_ge") => {
            resolve_previous_carry_operand(
                other,
                "_ge",
                previous_carries,
                node_name,
                loop_instruction,
                "sge",
            )
        }
        other if other.starts_with("carry") && other.ends_with("_eq") => {
            resolve_carry_operand(other, "_eq", next_carries, node_name, loop_instruction, "eq")
        }
        other if other.starts_with("carry") && other.ends_with("_ne") => {
            resolve_carry_operand(other, "_ne", next_carries, node_name, loop_instruction, "ne")
        }
        other if other.starts_with("carry") && other.ends_with("_lt") => {
            resolve_carry_operand(other, "_lt", next_carries, node_name, loop_instruction, "slt")
        }
        other if other.starts_with("carry") && other.ends_with("_le") => {
            resolve_carry_operand(other, "_le", next_carries, node_name, loop_instruction, "sle")
        }
        other if other.starts_with("carry") && other.ends_with("_gt") => {
            resolve_carry_operand(other, "_gt", next_carries, node_name, loop_instruction, "sgt")
        }
        other if other.starts_with("carry") && other.ends_with("_ge") => {
            resolve_carry_operand(other, "_ge", next_carries, node_name, loop_instruction, "sge")
        }
        other => Err(format!(
            "cpu.{loop_instruction} `{node_name}` has unsupported control kind `{other}` during LLVM lowering",
        )),
    }
}

fn previous_control_operand(
    previous: Option<&String>,
    kind: &str,
    node_name: &str,
    loop_instruction: &str,
    pred: &'static str,
) -> Result<(String, &'static str), String> {
    previous.cloned().map(|value| (value, pred)).ok_or_else(|| {
        format!(
            "cpu.{loop_instruction} `{node_name}` references previous control source `{kind}` outside a post-flow loop",
        )
    })
}

fn resolve_previous_carry_operand(
    kind: &str,
    suffix: &str,
    previous_carries: Option<&[String]>,
    node_name: &str,
    loop_instruction: &str,
    pred: &'static str,
) -> Result<(String, &'static str), String> {
    let previous_carries = previous_carries.ok_or_else(|| {
        format!(
            "cpu.{loop_instruction} `{node_name}` references previous control source `{kind}` outside a post-flow loop",
        )
    })?;
    let source_index = kind[10..kind.len() - suffix.len()]
        .parse::<usize>()
        .map_err(|_| {
            format!(
                "cpu.{loop_instruction} `{node_name}` has unsupported control kind `{kind}` during LLVM lowering",
            )
        })?;
    let lhs = previous_carries
        .get(source_index)
        .cloned()
        .ok_or_else(|| {
            format!(
                "cpu.{loop_instruction} `{node_name}` references unavailable control source `{kind}` during LLVM lowering",
            )
        })?;
    Ok((lhs, pred))
}

fn resolve_carry_operand(
    kind: &str,
    suffix: &str,
    current_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
    pred: &'static str,
) -> Result<(String, &'static str), String> {
    let source_index = kind[5..kind.len() - suffix.len()]
        .parse::<usize>()
        .map_err(|_| {
            format!(
                "cpu.{loop_instruction} `{node_name}` has unsupported control kind `{kind}` during LLVM lowering",
            )
        })?;
    let lhs = current_carries
        .get(source_index)
        .cloned()
        .ok_or_else(|| {
            format!(
                "cpu.{loop_instruction} `{node_name}` references unavailable control source `{kind}` during LLVM lowering",
            )
        })?;
    Ok((lhs, pred))
}
