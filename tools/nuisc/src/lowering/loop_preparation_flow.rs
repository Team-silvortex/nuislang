use super::*;

fn parse_loop_flow_condition_atom(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedLoopCarryCondition> {
    let normalized =
        normalize_pure_bool_test_expr(inline_pure_helper_calls(expr, inlineable_pure_helpers));
    let (lhs, compare, rhs) = match &normalized {
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
        _ => loop_state_ref_into_cond_source(parse_prepared_loop_state_ref_expr(
            lhs,
            binding_name,
            carries,
        )?),
    };
    Some(PreparedLoopCarryCondition { lhs, compare, rhs })
}

pub(super) fn parse_loop_flow_condition(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedLoopFlowCondition> {
    match expr {
        NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs,
            rhs,
        } => Some(PreparedLoopFlowCondition::Compound {
            op: PreparedLoopLogicOp::And,
            lhs: Box::new(parse_loop_flow_condition(
                lhs,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
            rhs: Box::new(parse_loop_flow_condition(
                rhs,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
        }),
        NirExpr::Binary {
            op: NirBinaryOp::Or,
            lhs,
            rhs,
        } => Some(PreparedLoopFlowCondition::Compound {
            op: PreparedLoopLogicOp::Or,
            lhs: Box::new(parse_loop_flow_condition(
                lhs,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
            rhs: Box::new(parse_loop_flow_condition(
                rhs,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
        }),
        _ => Some(PreparedLoopFlowCondition::Simple(
            parse_loop_flow_condition_atom(
                expr,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?,
        )),
    }
}

pub(super) fn parse_prepared_loop_header(
    condition: &NirExpr,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(String, NirExpr, PreparedLoopCompare)> {
    let normalized_condition = inline_pure_helper_calls(condition, inlineable_pure_helpers);
    match &normalized_condition {
        NirExpr::Binary { op, lhs, rhs } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            let compare = loop_compare_from_binary_op(*op)?;
            match lhs.as_ref() {
                NirExpr::Var(name) => Some((name.clone(), (**rhs).clone(), compare)),
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn parse_prepared_loop_step(
    stmt: &NirStmt,
    binding_name: &str,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(NirExpr, PreparedLoopStepKind)> {
    let (step_name, step_expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    let step_expr = inline_pure_helper_calls(&step_expr, inlineable_pure_helpers);
    if step_name != binding_name {
        return None;
    }
    match step_expr {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                Some(((*rhs).clone(), PreparedLoopStepKind::Add))
            }
            _ => None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Sub,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(name)
                if name == binding_name && is_terminal_branch_pure_expr(&rhs, pure_helpers) =>
            {
                Some(((*rhs).clone(), PreparedLoopStepKind::Sub))
            }
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn parse_prepared_async_loop_step(stmt: &NirStmt, binding_name: &str) -> Option<String> {
    let (step_name, step_expr) = match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            (name.as_str(), value)
        }
        _ => return None,
    };
    if step_name != binding_name {
        return None;
    }
    match step_expr {
        NirExpr::Await(inner) => match inner.as_ref() {
            NirExpr::Call { callee, args } if matches!(args.as_slice(), [NirExpr::Var(arg_name)] if arg_name == binding_name) => {
                Some(callee.clone())
            }
            _ => None,
        },
        _ => None,
    }
}

fn combine_loop_flow_conditions(
    lhs: PreparedLoopFlowCondition,
    op: PreparedLoopLogicOp,
    rhs: PreparedLoopFlowCondition,
) -> Option<PreparedLoopFlowCondition> {
    Some(PreparedLoopFlowCondition::Compound {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

pub(super) fn parse_loop_flow_control(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedLoopFlowControl> {
    fn terminal(
        condition: PreparedLoopFlowCondition,
        action: PreparedLoopFlowAction,
    ) -> PreparedLoopFlowControl {
        PreparedLoopFlowControl::Terminal { condition, action }
    }

    fn compound(
        op: PreparedLoopLogicOp,
        lhs: PreparedLoopFlowControl,
        rhs: PreparedLoopFlowControl,
    ) -> PreparedLoopFlowControl {
        PreparedLoopFlowControl::Compound {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    fn prefix_condition(
        control: PreparedLoopFlowControl,
        op: PreparedLoopLogicOp,
        condition: PreparedLoopFlowCondition,
    ) -> PreparedLoopFlowControl {
        match control {
            PreparedLoopFlowControl::Terminal {
                condition: leaf_condition,
                action,
            } => {
                let merged = combine_loop_flow_conditions(condition, op, leaf_condition)
                    .expect("loop flow control prefix condition should always combine");
                terminal(merged, action)
            }
            PreparedLoopFlowControl::Compound {
                op: branch_op,
                lhs,
                rhs,
            } => compound(
                branch_op,
                prefix_condition(*lhs, op, condition.clone()),
                prefix_condition(*rhs, op, condition),
            ),
        }
    }

    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    let outer_condition = parse_loop_flow_condition(
        condition,
        binding_name,
        carries,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if then_body.is_empty() {
        let [action_stmt] = else_body.as_slice() else {
            return None;
        };
        let inverted_condition = normalize_pure_bool_test_expr(NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(condition.clone()),
            rhs: Box::new(NirExpr::Bool(false)),
        });
        let inverted_condition = parse_loop_flow_condition(
            &inverted_condition,
            binding_name,
            carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        match action_stmt {
            NirStmt::Break => {
                return Some(terminal(inverted_condition, PreparedLoopFlowAction::Break));
            }
            NirStmt::Continue => {
                return Some(terminal(
                    inverted_condition,
                    PreparedLoopFlowAction::Continue,
                ));
            }
            NirStmt::If { .. } => {
                let nested = parse_loop_flow_control(
                    action_stmt,
                    binding_name,
                    carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?;
                return Some(prefix_condition(
                    nested,
                    PreparedLoopLogicOp::And,
                    inverted_condition,
                ));
            }
            _ => return None,
        }
    }
    if else_body.is_empty() {
        let [action_stmt] = then_body.as_slice() else {
            return None;
        };
        match action_stmt {
            NirStmt::Break => {
                return Some(terminal(outer_condition, PreparedLoopFlowAction::Break));
            }
            NirStmt::Continue => {
                return Some(terminal(outer_condition, PreparedLoopFlowAction::Continue));
            }
            NirStmt::If { .. } => {
                let nested = parse_loop_flow_control(
                    action_stmt,
                    binding_name,
                    carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?;
                return Some(prefix_condition(
                    nested,
                    PreparedLoopLogicOp::And,
                    outer_condition,
                ));
            }
            _ => return None,
        }
    }
    let [then_stmt] = then_body.as_slice() else {
        return None;
    };
    let [else_stmt] = else_body.as_slice() else {
        return None;
    };
    let direct_action = |stmt: &NirStmt| match stmt {
        NirStmt::Break => Some(PreparedLoopFlowAction::Break),
        NirStmt::Continue => Some(PreparedLoopFlowAction::Continue),
        _ => None,
    };
    if let (Some(then_action), Some(else_action)) =
        (direct_action(then_stmt), direct_action(else_stmt))
    {
        let inverted_condition = normalize_pure_bool_test_expr(NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(condition.clone()),
            rhs: Box::new(NirExpr::Bool(false)),
        });
        let inverted_condition = parse_loop_flow_condition(
            &inverted_condition,
            binding_name,
            carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        return Some(compound(
            PreparedLoopLogicOp::Or,
            terminal(outer_condition, then_action),
            terminal(inverted_condition, else_action),
        ));
    }
    if let Some(then_action) = direct_action(then_stmt) {
        let nested = parse_loop_flow_control(
            else_stmt,
            binding_name,
            carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        return Some(compound(
            PreparedLoopLogicOp::Or,
            terminal(outer_condition, then_action),
            nested,
        ));
    }
    if let Some(else_action) = direct_action(else_stmt) {
        let nested = parse_loop_flow_control(
            then_stmt,
            binding_name,
            carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        let inverted_condition = normalize_pure_bool_test_expr(NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(condition.clone()),
            rhs: Box::new(NirExpr::Bool(false)),
        });
        let inverted_condition = parse_loop_flow_condition(
            &inverted_condition,
            binding_name,
            carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        return Some(compound(
            PreparedLoopLogicOp::Or,
            nested,
            terminal(inverted_condition, else_action),
        ));
    }
    None
}

fn stmt_contains_terminal_loop_control_action(stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Break | NirStmt::Continue => true,
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            then_body
                .iter()
                .any(stmt_contains_terminal_loop_control_action)
                || else_body
                    .iter()
                    .any(stmt_contains_terminal_loop_control_action)
        }
        _ => false,
    }
}

fn collect_terminal_loop_control_actions(
    stmt: &PreparedLoopFlowControl,
    actions: &mut BTreeSet<&'static str>,
) {
    match stmt {
        PreparedLoopFlowControl::Terminal { action, .. } => {
            actions.insert(match action {
                PreparedLoopFlowAction::Break => "break",
                PreparedLoopFlowAction::Continue => "continue",
            });
        }
        PreparedLoopFlowControl::Compound { lhs, rhs, .. } => {
            collect_terminal_loop_control_actions(lhs, actions);
            collect_terminal_loop_control_actions(rhs, actions);
        }
    }
}

pub(super) fn diagnose_unstructured_loop_flow_control(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<String> {
    let NirStmt::If { condition, .. } = stmt else {
        return None;
    };
    if !stmt_contains_terminal_loop_control_action(stmt) {
        return None;
    }
    if parse_loop_flow_condition(
        condition,
        binding_name,
        carries,
        pure_helpers,
        inlineable_pure_helpers,
    )
    .is_none()
    {
        return Some(format!(
            "structured `while` lowering recognized loop state `{binding_name}` and a loop-control `if`, but its control condition is not reducible to supported loop-state/carry boolean tests"
        ));
    }
    let Some(control) = parse_loop_flow_control(
        stmt,
        binding_name,
        carries,
        pure_helpers,
        inlineable_pure_helpers,
    ) else {
        return Some(format!(
            "structured `while` lowering recognized loop state `{binding_name}` and a loop-control `if`, but the control branches do not match a supported break/continue flow shape"
        ));
    };
    let mut actions = BTreeSet::new();
    collect_terminal_loop_control_actions(&control, &mut actions);
    if actions.len() > 1 {
        return Some(format!(
            "structured `while` lowering recognized loop state `{binding_name}` and a loop-control `if`, but this control tree mixes `break` and `continue`; current flow/post-flow loop lowering requires one terminal loop action kind per structured control chain"
        ));
    }
    None
}
