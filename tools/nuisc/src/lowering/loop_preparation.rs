use super::*;
use crate::lowering::loop_carries::{
    loop_compare_from_binary_op, parse_loop_carry_branch_source, parse_loop_carry_condition,
    parse_loop_carry_linear,
};
use crate::lowering::loop_purity::substitute_stmt_bindings;

fn parse_loop_flow_condition_atom(
    expr: &NirExpr,
    binding_name: &str,
    carry_binding_names: &[String],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedLoopCarryCondition> {
    let (lhs, compare, rhs) = match expr {
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Eq, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Ne,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Ne, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Lt,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Lt, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Le,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Le, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Gt,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Gt, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Ge,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Ge, rhs.as_ref().clone())
        }
        _ => return None,
    };
    let lhs = match lhs {
        NirExpr::Var(name) if name == binding_name => PreparedCarryCondSource::Current,
        NirExpr::Var(name) if name == TAIL_RECURSIVE_PREV_CURRENT_BINDING => {
            PreparedCarryCondSource::PreviousCurrent
        }
        NirExpr::Var(name) if name.starts_with(TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX) => {
            PreparedCarryCondSource::PreviousCarry(
                name[TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX.len()..]
                    .parse::<usize>()
                    .ok()?,
            )
        }
        NirExpr::Var(name) => PreparedCarryCondSource::Carry(
            carry_binding_names
                .iter()
                .position(|carry_name| carry_name == name)?,
        ),
        _ => return None,
    };
    Some(PreparedLoopCarryCondition { lhs, compare, rhs })
}

fn parse_loop_flow_condition(
    expr: &NirExpr,
    binding_name: &str,
    carry_binding_names: &[String],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedLoopFlowCondition> {
    match expr {
        NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs,
            rhs,
        } => Some(PreparedLoopFlowCondition::Compound {
            op: PreparedLoopLogicOp::And,
            lhs: parse_loop_flow_condition_atom(
                lhs,
                binding_name,
                carry_binding_names,
                pure_helpers,
            )?,
            rhs: parse_loop_flow_condition_atom(
                rhs,
                binding_name,
                carry_binding_names,
                pure_helpers,
            )?,
        }),
        NirExpr::Binary {
            op: NirBinaryOp::Or,
            lhs,
            rhs,
        } => Some(PreparedLoopFlowCondition::Compound {
            op: PreparedLoopLogicOp::Or,
            lhs: parse_loop_flow_condition_atom(
                lhs,
                binding_name,
                carry_binding_names,
                pure_helpers,
            )?,
            rhs: parse_loop_flow_condition_atom(
                rhs,
                binding_name,
                carry_binding_names,
                pure_helpers,
            )?,
        }),
        _ => Some(PreparedLoopFlowCondition::Simple(
            parse_loop_flow_condition_atom(expr, binding_name, carry_binding_names, pure_helpers)?,
        )),
    }
}

fn combine_loop_flow_conditions(
    lhs: PreparedLoopFlowCondition,
    op: PreparedLoopLogicOp,
    rhs: PreparedLoopFlowCondition,
) -> Option<PreparedLoopFlowCondition> {
    match (lhs, rhs) {
        (PreparedLoopFlowCondition::Simple(lhs), PreparedLoopFlowCondition::Simple(rhs)) => {
            Some(PreparedLoopFlowCondition::Compound {
                op,
                lhs,
                rhs,
            })
        }
        _ => None,
    }
}

fn parse_loop_flow_control(
    stmt: &NirStmt,
    binding_name: &str,
    carry_binding_names: &[String],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedLoopFlowControl> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    let outer_condition =
        parse_loop_flow_condition(condition, binding_name, carry_binding_names, pure_helpers)?;
    if else_body.is_empty() {
        let [action_stmt] = then_body.as_slice() else {
            return None;
        };
        match action_stmt {
            NirStmt::Break => {
                return Some(PreparedLoopFlowControl {
                    condition: outer_condition,
                    action: PreparedLoopFlowAction::Break,
                });
            }
            NirStmt::Continue => {
                return Some(PreparedLoopFlowControl {
                    condition: outer_condition,
                    action: PreparedLoopFlowAction::Continue,
                });
            }
            NirStmt::If { .. } => {
                let nested = parse_loop_flow_control(
                    action_stmt,
                    binding_name,
                    carry_binding_names,
                    pure_helpers,
                )?;
                let condition = combine_loop_flow_conditions(
                    outer_condition,
                    PreparedLoopLogicOp::And,
                    nested.condition,
                )?;
                return Some(PreparedLoopFlowControl {
                    condition,
                    action: nested.action,
                });
            }
            _ => return None,
        }
    }
    let [then_stmt] = then_body.as_slice() else {
        return None;
    };
    let then_action = match then_stmt {
        NirStmt::Break => Some(PreparedLoopFlowAction::Break),
        NirStmt::Continue => Some(PreparedLoopFlowAction::Continue),
        _ => None,
    }?;
    let [else_stmt] = else_body.as_slice() else {
        return None;
    };
    let nested = parse_loop_flow_control(
        else_stmt,
        binding_name,
        carry_binding_names,
        pure_helpers,
    )?;
    if !matches!(
        (then_action, nested.action),
        (PreparedLoopFlowAction::Break, PreparedLoopFlowAction::Break)
            | (PreparedLoopFlowAction::Continue, PreparedLoopFlowAction::Continue)
    ) {
        return None;
    }
    let condition = combine_loop_flow_conditions(
        outer_condition,
        PreparedLoopLogicOp::Or,
        nested.condition,
    )?;
    Some(PreparedLoopFlowControl {
        condition,
        action: then_action,
    })
}

fn parse_loop_carry_update(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedCarryUpdate> {
    match stmt {
        carry_stmt @ (NirStmt::Let { .. } | NirStmt::Const { .. }) => {
            let (carry_name, carry_expr) = extract_pure_branch_binding(carry_stmt, pure_helpers)?;
            let (op, source) =
                parse_loop_carry_linear(&carry_name, &carry_expr, binding_name, carries)?;
            Some(PreparedCarryUpdate {
                binding_name: carry_name,
                kind: PreparedCarryUpdateKind::Linear { op, source },
            })
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let [then_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. })] =
                then_body.as_slice()
            else {
                return None;
            };
            let [else_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. })] =
                else_body.as_slice()
            else {
                return None;
            };
            let (then_name, then_expr) = extract_pure_branch_binding(then_binding, pure_helpers)?;
            let (else_name, else_expr) = extract_pure_branch_binding(else_binding, pure_helpers)?;
            if then_name != else_name {
                return None;
            }
            let condition =
                parse_loop_carry_condition(condition, binding_name, carries, pure_helpers)?;
            let then_source =
                parse_loop_carry_branch_source(&then_name, &then_expr, binding_name, carries)?;
            let else_source =
                parse_loop_carry_branch_source(&else_name, &else_expr, binding_name, carries)?;
            Some(PreparedCarryUpdate {
                binding_name: then_name,
                kind: PreparedCarryUpdateKind::Conditional {
                    condition,
                    then_source,
                    else_source,
                },
            })
        }
        _ => None,
    }
}

fn is_loop_match_scrutinee_temp_binding(name: &str) -> bool {
    name.starts_with("__match_scrutinee_")
}

fn extract_loop_match_scrutinee_temp_binding(
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
) -> Option<(String, NirExpr)> {
    let (name, expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    if is_loop_match_scrutinee_temp_binding(&name) {
        Some((name, expr))
    } else {
        None
    }
}

fn prepare_loop_carry_sequence(
    stmts: &[NirStmt],
    binding_name: &str,
    pure_helpers: &BTreeSet<String>,
) -> Option<Vec<PreparedCarryUpdate>> {
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    for stmt in stmts {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
        if let Some(prepared) =
            parse_loop_carry_update(&substituted, binding_name, &carries, pure_helpers)
        {
            carries.push(prepared);
            continue;
        }
        let (temp_name, temp_expr) = extract_pure_branch_binding(&substituted, pure_helpers)?;
        if !is_loop_match_scrutinee_temp_binding(&temp_name) {
            return None;
        }
        temp_bindings.push((temp_name, temp_expr));
    }
    Some(carries)
}

fn split_temp_prefixed_loop_flow_control<'a>(
    stmts: &'a [NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<(Vec<(String, NirExpr)>, &'a NirStmt, &'a [NirStmt])> {
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    for (index, stmt) in stmts.iter().enumerate() {
        if let Some((temp_name, temp_expr)) =
            extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers)
        {
            temp_bindings.push((temp_name, temp_expr));
            continue;
        }
        return Some((temp_bindings, stmt, &stmts[index + 1..]));
    }
    None
}

pub(super) fn prepare_counted_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedCountedWhile> {
    let (binding_name, limit, compare) = match condition {
        NirExpr::Binary { op, lhs, rhs } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            let compare = loop_compare_from_binary_op(*op)?;
            match lhs.as_ref() {
                NirExpr::Var(name) => (name.clone(), (**rhs).clone(), compare),
                _ => return None,
            }
        }
        _ => return None,
    };

    match body {
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. })] => {
            let (name, step_expr) = extract_pure_branch_binding(binding, pure_helpers)?;
            if name != binding_name {
                return None;
            }
            match step_expr {
                NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs,
                    rhs,
                } => match lhs.as_ref() {
                    NirExpr::Var(name)
                        if name == &binding_name
                            && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
                    {
                        Some(PreparedCountedWhile {
                            binding_name,
                            limit,
                            step: (*rhs).clone(),
                            compare,
                            step_kind: PreparedLoopStepKind::Add,
                        })
                    }
                    _ => None,
                },
                NirExpr::Binary {
                    op: NirBinaryOp::Sub,
                    lhs,
                    rhs,
                } => match lhs.as_ref() {
                    NirExpr::Var(name)
                        if name == &binding_name
                            && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
                    {
                        Some(PreparedCountedWhile {
                            binding_name,
                            limit,
                            step: (*rhs).clone(),
                            compare,
                            step_kind: PreparedLoopStepKind::Sub,
                        })
                    }
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn prepare_chained_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedChainedWhile> {
    let (binding_name, limit, compare) = match condition {
        NirExpr::Binary { op, lhs, rhs } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            let compare = loop_compare_from_binary_op(*op)?;
            match lhs.as_ref() {
                NirExpr::Var(name) => (name.clone(), (**rhs).clone(), compare),
                _ => return None,
            }
        }
        _ => return None,
    };

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), carry_bindings @ ..] = body
    else {
        return None;
    };
    if carry_bindings.is_empty() {
        return None;
    }
    let (step_name, step_expr) = extract_pure_branch_binding(step_binding, pure_helpers)?;
    if step_name != binding_name {
        return None;
    }
    let (step, step_kind) = match step_expr {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == &binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                ((*rhs).clone(), PreparedLoopStepKind::Add)
            }
            _ => return None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Sub,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == &binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                ((*rhs).clone(), PreparedLoopStepKind::Sub)
            }
            _ => return None,
        },
        _ => return None,
    };

    let carries = prepare_loop_carry_sequence(carry_bindings, &binding_name, pure_helpers)?;
    if carries.is_empty() {
        return None;
    }

    Some(PreparedChainedWhile {
        binding_name,
        limit,
        step,
        compare,
        step_kind,
        carries,
    })
}

pub(super) fn prepare_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedFlowWhile> {
    let (binding_name, limit, compare) = match condition {
        NirExpr::Binary { op, lhs, rhs } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            let compare = loop_compare_from_binary_op(*op)?;
            match lhs.as_ref() {
                NirExpr::Var(name) => (name.clone(), (**rhs).clone(), compare),
                _ => return None,
            }
        }
        _ => return None,
    };

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), rest @ ..] = body else {
        return None;
    };
    if rest.is_empty() {
        return None;
    }
    let (step_name, step_expr) = extract_pure_branch_binding(step_binding, pure_helpers)?;
    if step_name != binding_name {
        return None;
    }
    let (step, step_kind) = match step_expr {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == &binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                ((*rhs).clone(), PreparedLoopStepKind::Add)
            }
            _ => return None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Sub,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == &binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                ((*rhs).clone(), PreparedLoopStepKind::Sub)
            }
            _ => return None,
        },
        _ => return None,
    };
    let (control_temp_bindings, raw_control_stmt, carry_bindings) =
        split_temp_prefixed_loop_flow_control(rest, pure_helpers)?;
    let substituted_control_stmt =
        substitute_stmt_bindings(raw_control_stmt, &control_temp_bindings);
    let (control, prepared_carries) = if let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &substituted_control_stmt
    {
        if carry_bindings.is_empty() && !else_body.is_empty() {
            let [action_stmt] = then_body.as_slice() else {
                return None;
            };
            if let Some(action) = match action_stmt {
                NirStmt::Break => Some(PreparedLoopFlowAction::Break),
                NirStmt::Continue => Some(PreparedLoopFlowAction::Continue),
                _ => None,
            } {
                let prepared_carries =
                    prepare_loop_carry_sequence(else_body, &binding_name, pure_helpers)?;
                let carry_binding_names = prepared_carries
                    .iter()
                    .map(|carry| carry.binding_name.clone())
                    .collect::<Vec<_>>();
                let control_condition = parse_loop_flow_condition(
                    condition,
                    &binding_name,
                    &carry_binding_names,
                    pure_helpers,
                )?;
                (
                    PreparedLoopFlowControl {
                        condition: control_condition,
                        action,
                    },
                    prepared_carries,
                )
            } else {
                let prepared_carries =
                    prepare_loop_carry_sequence(carry_bindings, &binding_name, pure_helpers)?;
                let carry_binding_names = prepared_carries
                    .iter()
                    .map(|carry| carry.binding_name.clone())
                    .collect::<Vec<_>>();
                let control = parse_loop_flow_control(
                    &substituted_control_stmt,
                    &binding_name,
                    &carry_binding_names,
                    pure_helpers,
                )?;
                (control, prepared_carries)
            }
        } else {
            let prepared_carries =
                prepare_loop_carry_sequence(carry_bindings, &binding_name, pure_helpers)?;
            let carry_binding_names = prepared_carries
                .iter()
                .map(|carry| carry.binding_name.clone())
                .collect::<Vec<_>>();
            let control = parse_loop_flow_control(
                &substituted_control_stmt,
                &binding_name,
                &carry_binding_names,
                pure_helpers,
            )?;
            (control, prepared_carries)
        }
    } else {
        let prepared_carries =
            prepare_loop_carry_sequence(carry_bindings, &binding_name, pure_helpers)?;
        let carry_binding_names = prepared_carries
            .iter()
            .map(|carry| carry.binding_name.clone())
            .collect::<Vec<_>>();
        let control = parse_loop_flow_control(
            &substituted_control_stmt,
            &binding_name,
            &carry_binding_names,
            pure_helpers,
        )?;
        (control, prepared_carries)
    };
    Some(PreparedFlowWhile {
        binding_name,
        limit,
        step,
        compare,
        step_kind,
        control,
        carries: prepared_carries,
    })
}

pub(super) fn prepare_post_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedPostFlowWhile> {
    let (binding_name, limit, compare) = match condition {
        NirExpr::Binary { op, lhs, rhs } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            let compare = loop_compare_from_binary_op(*op)?;
            match lhs.as_ref() {
                NirExpr::Var(name) => (name.clone(), (**rhs).clone(), compare),
                _ => return None,
            }
        }
        _ => return None,
    };

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), middle @ .., control_stmt] =
        body
    else {
        return None;
    };
    if middle.is_empty() {
        return None;
    }
    let (step_name, step_expr) = extract_pure_branch_binding(step_binding, pure_helpers)?;
    if step_name != binding_name {
        return None;
    }
    let (step, step_kind) = match step_expr {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == &binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                ((*rhs).clone(), PreparedLoopStepKind::Add)
            }
            _ => return None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Sub,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == &binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                ((*rhs).clone(), PreparedLoopStepKind::Sub)
            }
            _ => return None,
        },
        _ => return None,
    };

    let trailing_temp_count = middle
        .iter()
        .rev()
        .take_while(|stmt| extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers).is_some())
        .count();
    let split_index = middle.len().saturating_sub(trailing_temp_count);
    let (carry_bindings, control_temp_stmts) = middle.split_at(split_index);
    let prepared_carries =
        prepare_loop_carry_sequence(carry_bindings, &binding_name, pure_helpers)?;
    let carry_binding_names = prepared_carries
        .iter()
        .map(|carry| carry.binding_name.clone())
        .collect::<Vec<_>>();
    let control_temp_bindings = control_temp_stmts
        .iter()
        .map(|stmt| extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers))
        .collect::<Option<Vec<_>>>()?;
    let substituted_control_stmt = substitute_stmt_bindings(control_stmt, &control_temp_bindings);
    let control = parse_loop_flow_control(
        &substituted_control_stmt,
        &binding_name,
        &carry_binding_names,
        pure_helpers,
    )?;
    Some(PreparedPostFlowWhile {
        binding_name,
        limit,
        step,
        compare,
        step_kind,
        carries: prepared_carries,
        control,
    })
}
