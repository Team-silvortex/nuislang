use super::*;

pub(super) fn parse_loop_carry_update(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedCarryUpdate> {
    parse_linear_loop_carry_update_stmt(
        stmt,
        binding_name,
        carries,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )
    .or_else(|| {
        parse_conditional_loop_carry_update_stmt(
            stmt,
            binding_name,
            carries,
            pure_helpers,
            inlineable_pure_helpers,
        )
    })
}

pub(super) fn diagnose_unsupported_loop_carry_update(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<String> {
    match stmt {
        NirStmt::Let { .. } | NirStmt::Const { .. } => {
            let (carry_name, carry_expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
            diagnose_unsupported_loop_carry_expr(
                &carry_name,
                &carry_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )
            .or_else(|| {
                parse_helper_conditional_carry_update(
                    &carry_name,
                    &carry_expr,
                    binding_name,
                    carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                )
                .and_then(|kind| match kind {
                    PreparedCarryUpdateKind::Linear { .. } => None,
                    PreparedCarryUpdateKind::Conditional {
                        then_source,
                        else_source,
                        ..
                    } => unsupported_loop_carry_branch_source_message(&then_source)
                        .or_else(|| unsupported_loop_carry_branch_source_message(&else_source)),
                })
            })
        }
        NirStmt::If { .. } => {
            let carry_name =
                extract_single_stmt_carry_name(stmt, pure_helpers, inlineable_pure_helpers)?;
            parse_stmt_carry_decision_tree(
                stmt,
                &carry_name,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?;
            diagnose_unsupported_stmt_carry_tree(
                stmt,
                &carry_name,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )
        }
        _ => None,
    }
}

fn parse_linear_loop_carry_update_stmt(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedCarryUpdate> {
    let carry_stmt = match stmt {
        NirStmt::Let { .. } | NirStmt::Const { .. } => stmt,
        _ => return None,
    };
    let (carry_name, carry_expr) = extract_pure_branch_binding(carry_stmt, pure_helpers)?;
    if let Some((op, source)) = parse_loop_carry_linear(
        &carry_name,
        &carry_expr,
        binding_name,
        carries,
        inlineable_pure_helpers,
    ) {
        Some(PreparedCarryUpdate {
            binding_name: carry_name,
            kind: PreparedCarryUpdateKind::Linear {
                op,
                source: Box::new(source),
            },
        })
    } else {
        Some(PreparedCarryUpdate {
            binding_name: carry_name.clone(),
            kind: parse_helper_conditional_carry_update(
                &carry_name,
                &carry_expr,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
                pure_helper_blocks,
            )?,
        })
    }
}

fn parse_conditional_loop_carry_update_stmt(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryUpdate> {
    let NirStmt::If { .. } = stmt else {
        return None;
    };
    let carry_name = extract_single_stmt_carry_name(stmt, pure_helpers, inlineable_pure_helpers)?;
    let carry_tree = parse_stmt_carry_decision_tree(
        stmt,
        &carry_name,
        binding_name,
        carries,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let (condition, then_source, else_source) = collapse_carry_decision_tree(&carry_tree)?;
    Some(PreparedCarryUpdate {
        binding_name: carry_name,
        kind: PreparedCarryUpdateKind::Conditional {
            condition,
            then_source: Box::new(then_source),
            else_source: Box::new(else_source),
        },
    })
}

fn extract_final_pure_binding_expr(
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(String, NirExpr)> {
    let normalized = normalize_pure_stmt_prefix_body(body, pure_helpers, inlineable_pure_helpers)?;
    let [stmt] = normalized.as_slice() else {
        return None;
    };
    extract_pure_branch_binding(stmt, pure_helpers)
}

pub(super) fn parse_conditional_temp_binding(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedConditionalTempBinding> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    let (then_name, then_expr) =
        extract_final_pure_binding_expr(then_body, pure_helpers, inlineable_pure_helpers)?;
    let (else_name, else_expr) =
        extract_final_pure_binding_expr(else_body, pure_helpers, inlineable_pure_helpers)?;
    if then_name != else_name {
        return None;
    }
    Some(PreparedConditionalTempBinding {
        binding_name: then_name,
        condition: parse_loop_flow_condition(
            condition,
            binding_name,
            carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?,
        then_expr,
        else_expr,
    })
}

pub(super) fn parse_derived_conditional_temp_binding(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    conditional_temps: &BTreeMap<String, PreparedConditionalTempBinding>,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedConditionalTempBinding> {
    let (derived_binding_name, expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    let normalized = inline_pure_helper_calls(&expr, inlineable_pure_helpers);
    let (source_temp_name, make_branch_expr): (&str, ConditionalTempExprBuilder) = match &normalized
    {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => match lhs.as_ref() {
            NirExpr::Var(name) => {
                let rhs = (**rhs).clone();
                (
                    name.as_str(),
                    Box::new(move |base| NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(base.clone()),
                        rhs: Box::new(rhs.clone()),
                    }),
                )
            }
            _ => return None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(name), other)
                if parse_prepared_loop_state_ref_expr(other, binding_name, carries).is_some() =>
            {
                let rhs = other.clone();
                (
                    name.as_str(),
                    Box::new(move |base| NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(base.clone()),
                        rhs: Box::new(rhs.clone()),
                    }),
                )
            }
            (other, NirExpr::Var(name))
                if parse_prepared_loop_state_ref_expr(other, binding_name, carries).is_some() =>
            {
                let lhs = other.clone();
                (
                    name.as_str(),
                    Box::new(move |base| NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(lhs.clone()),
                        rhs: Box::new(base.clone()),
                    }),
                )
            }
            _ => return None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => match lhs.as_ref() {
            NirExpr::Var(name) => {
                let rhs = (**rhs).clone();
                (
                    name.as_str(),
                    Box::new(move |base| NirExpr::Binary {
                        op: NirBinaryOp::Mul,
                        lhs: Box::new(base.clone()),
                        rhs: Box::new(rhs.clone()),
                    }),
                )
            }
            _ => return None,
        },
        _ => return None,
    };
    let source = conditional_temps.get(source_temp_name)?;
    Some(PreparedConditionalTempBinding {
        binding_name: derived_binding_name,
        condition: source.condition.clone(),
        then_expr: make_branch_expr(&source.then_expr),
        else_expr: make_branch_expr(&source.else_expr),
    })
}
