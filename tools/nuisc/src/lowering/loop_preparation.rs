use super::*;
use crate::lowering::loop_carries::{
    diagnose_unsupported_loop_carry_expr, loop_compare_from_binary_op,
    loop_state_ref_into_carry_source, loop_state_ref_into_cond_source,
    parse_loop_carry_branch_source, parse_loop_carry_linear,
    parse_prepared_dynamic_read_carry_source, parse_prepared_fixed_read_carry_source,
    parse_prepared_loop_state_ref_expr, parse_prepared_loop_state_ref_name,
    unsupported_loop_carry_branch_source_message,
};
use crate::lowering::loop_purity::{
    normalize_pure_bool_test_expr, substitute_branch_binding, substitute_stmt_bindings,
};

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

fn parse_loop_flow_condition(
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

fn parse_prepared_loop_header(
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

fn parse_prepared_loop_step(
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

fn parse_prepared_async_loop_step(stmt: &NirStmt, binding_name: &str) -> Option<String> {
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

fn parse_loop_flow_control(
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

fn diagnose_unstructured_loop_flow_control(
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

enum PreparedReturnDecisionTree {
    Return(NirExpr),
    Branch {
        condition: NirExpr,
        then_tree: Box<PreparedReturnDecisionTree>,
        else_tree: Box<PreparedReturnDecisionTree>,
    },
}

enum PreparedCarryDecisionTree {
    Leaf(PreparedCarryBranchSource),
    Branch {
        condition: PreparedLoopFlowCondition,
        then_tree: Box<PreparedCarryDecisionTree>,
        else_tree: Box<PreparedCarryDecisionTree>,
    },
}

#[derive(Clone)]
struct PreparedConditionalTempBinding {
    binding_name: String,
    condition: PreparedLoopFlowCondition,
    then_expr: NirExpr,
    else_expr: NirExpr,
}

fn extract_pure_block_return_expr(
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<NirExpr> {
    let (NirStmt::Return(Some(expr)), prefix) = body.split_last()? else {
        return None;
    };
    let mut substituted = inline_pure_helper_calls(expr, inlineable_pure_helpers);
    for stmt in prefix.iter().rev() {
        let (name, value) = extract_pure_branch_binding(stmt, pure_helpers)?;
        let value = inline_pure_helper_calls(&value, inlineable_pure_helpers);
        substituted = substitute_branch_binding(&substituted, &name, &value);
    }
    Some(substituted)
}

fn normalize_pure_helper_body(
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<Vec<NirStmt>> {
    let mut current = body.to_vec();
    loop {
        let Some((first, tail)) = current.split_first() else {
            return None;
        };
        let Some((name, value)) = extract_pure_branch_binding(first, pure_helpers) else {
            return Some(current);
        };
        let value = inline_pure_helper_calls(&value, inlineable_pure_helpers);
        current = tail
            .iter()
            .map(|stmt| substitute_stmt_bindings(stmt, &[(name.clone(), value.clone())]))
            .collect();
    }
}

fn normalize_pure_stmt_prefix_body(
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<Vec<NirStmt>> {
    let mut current = body.to_vec();
    loop {
        if current.len() <= 1 {
            return Some(current);
        }
        let Some((first, tail)) = current.split_first() else {
            return Some(current);
        };
        let Some((name, value)) = extract_pure_branch_binding(first, pure_helpers) else {
            return Some(current);
        };
        let value = inline_pure_helper_calls(&value, inlineable_pure_helpers);
        current = tail
            .iter()
            .map(|stmt| substitute_stmt_bindings(stmt, &[(name.clone(), value.clone())]))
            .collect();
    }
}

fn instantiate_pure_helper_body(
    expr: &NirExpr,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<Vec<NirStmt>> {
    let NirExpr::Call { callee, args } = expr else {
        return None;
    };
    let helper = pure_helper_blocks.get(callee)?;
    if helper.params.len() != args.len() {
        return None;
    }
    let bindings = helper
        .params
        .iter()
        .cloned()
        .zip(
            args.iter()
                .map(|arg| inline_pure_helper_calls(arg, inlineable_pure_helpers)),
        )
        .collect::<Vec<_>>();
    Some(
        helper
            .body
            .iter()
            .map(|stmt| substitute_stmt_bindings(stmt, &bindings))
            .collect(),
    )
}

fn parse_helper_return_tree(
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedReturnDecisionTree> {
    let normalized_body = normalize_pure_helper_body(body, pure_helpers, inlineable_pure_helpers)?;
    if let Some(returned) =
        extract_pure_block_return_expr(&normalized_body, pure_helpers, inlineable_pure_helpers)
    {
        return Some(PreparedReturnDecisionTree::Return(returned));
    }
    match normalized_body.as_slice() {
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => Some(PreparedReturnDecisionTree::Branch {
            condition: condition.clone(),
            then_tree: Box::new(parse_helper_return_tree(
                then_body,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
            else_tree: Box::new(parse_helper_return_tree(
                else_body,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
        }),
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }, tail @ NirStmt::Return(Some(_))]
            if else_body.is_empty() =>
        {
            Some(PreparedReturnDecisionTree::Branch {
                condition: condition.clone(),
                then_tree: Box::new(parse_helper_return_tree(
                    then_body,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?),
                else_tree: Box::new(parse_helper_return_tree(
                    std::slice::from_ref(tail),
                    pure_helpers,
                    inlineable_pure_helpers,
                )?),
            })
        }
        _ => None,
    }
}

fn lower_helper_return_tree_to_carry_tree(
    tree: PreparedReturnDecisionTree,
    carry_name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryDecisionTree> {
    match tree {
        PreparedReturnDecisionTree::Return(expr) => Some(PreparedCarryDecisionTree::Leaf(
            parse_loop_carry_branch_source(
                carry_name,
                &expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )?,
        )),
        PreparedReturnDecisionTree::Branch {
            condition,
            then_tree,
            else_tree,
        } => Some(PreparedCarryDecisionTree::Branch {
            condition: parse_loop_flow_condition(
                &condition,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?,
            then_tree: Box::new(lower_helper_return_tree_to_carry_tree(
                *then_tree,
                carry_name,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
            else_tree: Box::new(lower_helper_return_tree_to_carry_tree(
                *else_tree,
                carry_name,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
        }),
    }
}

fn collapse_carry_decision_tree(
    tree: &PreparedCarryDecisionTree,
) -> Option<(
    PreparedLoopFlowCondition,
    PreparedCarryBranchSource,
    PreparedCarryBranchSource,
)> {
    fn leaf_source(tree: &PreparedCarryDecisionTree) -> Option<PreparedCarryBranchSource> {
        match tree {
            PreparedCarryDecisionTree::Leaf(source) => Some(source.clone()),
            PreparedCarryDecisionTree::Branch { .. } => None,
        }
    }

    match tree {
        PreparedCarryDecisionTree::Leaf(_) => None,
        PreparedCarryDecisionTree::Branch {
            condition,
            then_tree,
            else_tree,
        } => {
            if let (Some(then_source), Some(else_source)) =
                (leaf_source(then_tree), leaf_source(else_tree))
            {
                return Some((condition.clone(), then_source, else_source));
            }

            if let Some(then_source) = leaf_source(then_tree) {
                if let Some((nested_condition, nested_then, nested_else)) =
                    collapse_carry_decision_tree(else_tree)
                {
                    if nested_then == then_source {
                        return Some((
                            PreparedLoopFlowCondition::Compound {
                                op: PreparedLoopLogicOp::Or,
                                lhs: Box::new(condition.clone()),
                                rhs: Box::new(nested_condition),
                            },
                            then_source,
                            nested_else,
                        ));
                    }
                }
            }

            if let Some(else_source) = leaf_source(else_tree) {
                if let Some((nested_condition, nested_then, nested_else)) =
                    collapse_carry_decision_tree(then_tree)
                {
                    if nested_else == else_source {
                        return Some((
                            PreparedLoopFlowCondition::Compound {
                                op: PreparedLoopLogicOp::And,
                                lhs: Box::new(condition.clone()),
                                rhs: Box::new(nested_condition),
                            },
                            nested_then,
                            else_source,
                        ));
                    }
                }
            }

            None
        }
    }
}

fn parse_helper_conditional_carry_update(
    carry_name: &str,
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedCarryUpdateKind> {
    let instantiated =
        instantiate_pure_helper_body(expr, inlineable_pure_helpers, pure_helper_blocks)?;
    let return_tree =
        parse_helper_return_tree(&instantiated, pure_helpers, inlineable_pure_helpers)?;
    let carry_tree = lower_helper_return_tree_to_carry_tree(
        return_tree,
        carry_name,
        binding_name,
        carries,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let (condition, then_source, else_source) = collapse_carry_decision_tree(&carry_tree)?;
    Some(PreparedCarryUpdateKind::Conditional {
        condition,
        then_source,
        else_source,
    })
}

fn extract_single_stmt_carry_name(
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<String> {
    match stmt {
        NirStmt::Let { .. } | NirStmt::Const { .. } => {
            let (name, _) = extract_pure_branch_binding(stmt, pure_helpers)?;
            Some(name)
        }
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            let normalized_then =
                normalize_pure_stmt_prefix_body(then_body, pure_helpers, inlineable_pure_helpers)?;
            let normalized_else =
                normalize_pure_stmt_prefix_body(else_body, pure_helpers, inlineable_pure_helpers)?;
            let [then_stmt] = normalized_then.as_slice() else {
                return None;
            };
            let [else_stmt] = normalized_else.as_slice() else {
                return None;
            };
            let then_name =
                extract_single_stmt_carry_name(then_stmt, pure_helpers, inlineable_pure_helpers)?;
            let else_name =
                extract_single_stmt_carry_name(else_stmt, pure_helpers, inlineable_pure_helpers)?;
            if then_name == else_name {
                Some(then_name)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_non_temp_loop_carry_name(
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<String> {
    let name = extract_single_stmt_carry_name(stmt, pure_helpers, inlineable_pure_helpers)?;
    if is_loop_match_scrutinee_temp_binding(&name) {
        None
    } else {
        Some(name)
    }
}

fn expr_references_any_name(expr: &NirExpr, names: &BTreeSet<String>) -> bool {
    match expr {
        NirExpr::Var(name) => names.contains(name),
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner)
        | NirExpr::FieldAccess { base: inner, .. } => expr_references_any_name(inner, names),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_references_any_name(lhs, names) || expr_references_any_name(rhs, names)
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => expr_references_any_name(buffer, names) || expr_references_any_name(index, names),
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => expr_references_any_name(target, names) || expr_references_any_name(value, names),
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        }
        | NirExpr::DataWriteWindow {
            window: buffer,
            index,
            value,
        } => {
            expr_references_any_name(buffer, names)
                || expr_references_any_name(index, names)
                || expr_references_any_name(value, names)
        }
        NirExpr::AllocNode { value, next } => {
            expr_references_any_name(value, names) || expr_references_any_name(next, names)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_references_any_name(len, names) || expr_references_any_name(fill, names)
        }
        NirExpr::Call { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. } => {
            args.iter().any(|arg| expr_references_any_name(arg, names))
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_references_any_name(receiver, names)
                || args.iter().any(|arg| expr_references_any_name(arg, names))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_references_any_name(value, names)),
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => expr_references_any_name(value, names),
        _ => false,
    }
}

fn stmt_references_any_name(stmt: &NirStmt, names: &BTreeSet<String>) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value)
        | NirStmt::Await(value) => expr_references_any_name(value, names),
        NirStmt::Return(Some(value)) => expr_references_any_name(value, names),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_references_any_name(condition, names)
                || then_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
                || else_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
        }
        NirStmt::While { condition, body } => {
            expr_references_any_name(condition, names)
                || body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}

fn collect_loop_carry_binding_names(
    stmts: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for stmt in stmts {
        if extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers).is_some() {
            continue;
        }
        names.push(extract_non_temp_loop_carry_name(
            stmt,
            pure_helpers,
            inlineable_pure_helpers,
        )?);
    }
    Some(names)
}

fn diagnose_future_carry_reference(
    stmt: &NirStmt,
    current_carry_name: &str,
    future_carry_names: &[String],
) -> Option<String> {
    let referenced_name = future_carry_names
        .iter()
        .find(|name| stmt_references_any_name(stmt, &BTreeSet::from([(*name).clone()])))?;
    Some(format!(
        "loop carry update `{current_carry_name}` references sibling carry `{referenced_name}` before that carry is updated in the loop body; reorder the carry bindings or make the previous-state dependency explicit"
    ))
}

fn parse_stmt_carry_decision_tree(
    stmt: &NirStmt,
    carry_name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryDecisionTree> {
    match stmt {
        NirStmt::Let { .. } | NirStmt::Const { .. } => {
            let (branch_name, branch_expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
            if branch_name != carry_name {
                return None;
            }
            Some(PreparedCarryDecisionTree::Leaf(
                parse_loop_carry_branch_source(
                    &branch_name,
                    &branch_expr,
                    binding_name,
                    carries,
                    inlineable_pure_helpers,
                )?,
            ))
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let normalized_then =
                normalize_pure_stmt_prefix_body(then_body, pure_helpers, inlineable_pure_helpers)?;
            let normalized_else =
                normalize_pure_stmt_prefix_body(else_body, pure_helpers, inlineable_pure_helpers)?;
            let [then_stmt] = normalized_then.as_slice() else {
                return None;
            };
            let [else_stmt] = normalized_else.as_slice() else {
                return None;
            };
            Some(PreparedCarryDecisionTree::Branch {
                condition: parse_loop_flow_condition(
                    condition,
                    binding_name,
                    carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?,
                then_tree: Box::new(parse_stmt_carry_decision_tree(
                    then_stmt,
                    carry_name,
                    binding_name,
                    carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?),
                else_tree: Box::new(parse_stmt_carry_decision_tree(
                    else_stmt,
                    carry_name,
                    binding_name,
                    carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?),
            })
        }
        _ => None,
    }
}

fn diagnose_unsupported_stmt_carry_tree(
    stmt: &NirStmt,
    carry_name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<String> {
    match stmt {
        NirStmt::Let { .. } | NirStmt::Const { .. } => {
            let (branch_name, branch_expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
            if branch_name != carry_name {
                return None;
            }
            diagnose_unsupported_loop_carry_expr(
                &branch_name,
                &branch_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )
            .or_else(|| {
                parse_loop_carry_branch_source(
                    &branch_name,
                    &branch_expr,
                    binding_name,
                    carries,
                    inlineable_pure_helpers,
                )
                .and_then(|source| unsupported_loop_carry_branch_source_message(&source))
            })
        }
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            let [then_stmt] = then_body.as_slice() else {
                return None;
            };
            let [else_stmt] = else_body.as_slice() else {
                return None;
            };
            diagnose_unsupported_stmt_carry_tree(
                then_stmt,
                carry_name,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )
            .or_else(|| {
                diagnose_unsupported_stmt_carry_tree(
                    else_stmt,
                    carry_name,
                    binding_name,
                    carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )
            })
        }
        _ => None,
    }
}

fn parse_loop_carry_update(
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

fn diagnose_unsupported_loop_carry_update(
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
            if parse_stmt_carry_decision_tree(
                stmt,
                &carry_name,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )
            .is_none()
            {
                return None;
            }
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
            kind: PreparedCarryUpdateKind::Linear { op, source },
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
            then_source,
            else_source,
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

fn parse_conditional_temp_binding(
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

fn parse_derived_conditional_temp_binding(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    conditional_temps: &BTreeMap<String, PreparedConditionalTempBinding>,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedConditionalTempBinding> {
    let (derived_binding_name, expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    let normalized = inline_pure_helper_calls(&expr, inlineable_pure_helpers);
    let (source_temp_name, make_branch_expr): (&str, Box<dyn Fn(&NirExpr) -> NirExpr>) =
        match &normalized {
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
                    if parse_prepared_loop_state_ref_expr(other, binding_name, carries)
                        .is_some() =>
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
                    if parse_prepared_loop_state_ref_expr(other, binding_name, carries)
                        .is_some() =>
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

fn parse_loop_carry_delta_branch_source(
    op: PreparedCarryLinearOp,
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryBranchSource> {
    #[derive(Default)]
    struct ParsedAdditiveSource {
        terms: Vec<PreparedLoopStateRef>,
        offset: Option<NirExpr>,
    }

    fn expr_contains_loop_variant_ref(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> bool {
        match expr {
            NirExpr::Var(name) => {
                parse_prepared_loop_state_ref_name(name, binding_name, carries).is_some()
            }
            NirExpr::Binary { lhs, rhs, .. } => {
                expr_contains_loop_variant_ref(lhs, binding_name, carries)
                    || expr_contains_loop_variant_ref(rhs, binding_name, carries)
            }
            _ => false,
        }
    }

    fn combine_invariant_terms(terms: Vec<NirExpr>) -> Option<NirExpr> {
        let mut iter = terms.into_iter();
        let first = iter.next()?;
        Some(iter.fold(first, |lhs, rhs| NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }))
    }

    fn scale_invariant_expr(expr: NirExpr, factor: i64) -> NirExpr {
        match factor {
            0 => NirExpr::Int(0),
            1 => expr,
            _ => NirExpr::Binary {
                op: NirBinaryOp::Mul,
                lhs: Box::new(expr),
                rhs: Box::new(NirExpr::Int(factor)),
            },
        }
    }

    fn scale_additive_source(
        source: PreparedCarrySource,
        factor: NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<PreparedCarrySource> {
        let factor_state = parse_prepared_loop_state_ref_expr(&factor, binding_name, carries);
        let factor_affine = parse_additive_source_for_factor(&factor, binding_name, carries);
        let factor_scaled_affine =
            parse_scaled_additive_source_for_factor(&factor, binding_name, carries);
        let factor_group_product =
            parse_factor_group_product_for_factor(&factor, binding_name, carries);
        let factor_group_product_times_invariant =
            parse_scaled_factor_group_product_for_factor(&factor, binding_name, carries);
        match source {
            PreparedCarrySource::AddStateList { terms, offset } => {
                if let Some(factor_state) = factor_state {
                    Some(PreparedCarrySource::ScaledStateListByState {
                        terms,
                        factor: factor_state,
                        offset,
                    })
                } else if let Some((factor_terms, factor_offset)) = factor_affine {
                    match (factor_terms.as_slice(), factor_offset) {
                        ([factor_state], Some(factor_offset)) => {
                            Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                                terms,
                                factor: *factor_state,
                                factor_offset,
                                offset,
                            })
                        }
                        (factor_terms, factor_offset) => {
                            Some(PreparedCarrySource::ScaledStateListByFactorStateList {
                                terms,
                                factor_terms: factor_terms.to_vec(),
                                factor_offset,
                                offset,
                            })
                        }
                    }
                } else if let Some((factor_terms, factor_scale, factor_offset)) =
                    factor_scaled_affine
                {
                    Some(
                        PreparedCarrySource::ScaledStateListByFactorStateListTimesInvariant {
                            terms,
                            factor_terms,
                            factor_scale,
                            factor_offset,
                            offset,
                        },
                    )
                } else if let Some((
                    lhs_factor_terms,
                    lhs_factor_offset,
                    rhs_factor_terms,
                    rhs_factor_offset,
                )) = factor_group_product
                {
                    Some(PreparedCarrySource::ScaledStateListByFactorGroupProduct {
                        terms,
                        lhs_factor_terms,
                        lhs_factor_offset,
                        rhs_factor_terms,
                        rhs_factor_offset,
                        offset,
                    })
                } else if let Some((
                    lhs_factor_terms,
                    lhs_factor_offset,
                    rhs_factor_terms,
                    rhs_factor_offset,
                    factor_scale,
                )) = factor_group_product_times_invariant
                {
                    Some(
                        PreparedCarrySource::ScaledStateListByFactorGroupProductTimesInvariant {
                            terms,
                            lhs_factor_terms,
                            lhs_factor_offset,
                            rhs_factor_terms,
                            rhs_factor_offset,
                            factor_scale,
                            offset,
                        },
                    )
                } else {
                    let scaled_offset = offset.map(|offset| NirExpr::Binary {
                        op: NirBinaryOp::Mul,
                        lhs: Box::new(offset),
                        rhs: Box::new(factor.clone()),
                    });
                    Some(PreparedCarrySource::ScaledStateList {
                        terms,
                        factor,
                        offset: scaled_offset,
                    })
                }
            }
            PreparedCarrySource::Current
            | PreparedCarrySource::PreviousCurrent
            | PreparedCarrySource::PreviousCarry(_)
            | PreparedCarrySource::Carry(_)
            | PreparedCarrySource::ScaledStateList { .. }
            | PreparedCarrySource::ScaledStateListByState { .. }
            | PreparedCarrySource::ScaledStateListByStatePlusInvariant { .. }
            | PreparedCarrySource::ScaledStateListByFactorStateList { .. }
            | PreparedCarrySource::ScaledStateListByFactorStateListTimesInvariant { .. }
            | PreparedCarrySource::ScaledStateListByFactorGroupProduct { .. }
            | PreparedCarrySource::ScaledStateListByFactorGroupProductTimesInvariant { .. }
            | PreparedCarrySource::InvariantExpr(_)
            | PreparedCarrySource::AddInvariant { .. }
            | PreparedCarrySource::FixedRead(_)
            | PreparedCarrySource::DynamicReadAt { .. } => None,
        }
    }

    fn parse_additive_source_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<(Vec<PreparedLoopStateRef>, Option<NirExpr>)> {
        fn parse_inner(
            expr: &NirExpr,
            binding_name: &str,
            carries: &[PreparedCarryUpdate],
            expr_contains_loop_variant_ref: &impl Fn(&NirExpr, &str, &[PreparedCarryUpdate]) -> bool,
        ) -> Option<ParsedAdditiveSource> {
            if let Some(state_ref) = parse_prepared_loop_state_ref_expr(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
                    terms: vec![state_ref],
                    offset: None,
                });
            }
            if is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
                    terms: Vec::new(),
                    offset: Some(expr.clone()),
                });
            }
            match expr {
                NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs,
                    rhs,
                } => {
                    let lhs =
                        parse_inner(lhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let rhs =
                        parse_inner(rhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let mut terms = lhs.terms;
                    terms.extend(rhs.terms);
                    let offset = combine_invariant_terms(
                        lhs.offset.into_iter().chain(rhs.offset).collect::<Vec<_>>(),
                    );
                    Some(ParsedAdditiveSource { terms, offset })
                }
                _ => None,
            }
        }

        let parsed = parse_inner(expr, binding_name, carries, &expr_contains_loop_variant_ref)?;
        match (parsed.terms.is_empty(), parsed.offset.is_some()) {
            (false, true) | (false, false) => Some((parsed.terms, parsed.offset)),
            _ => None,
        }
    }

    fn parse_scaled_additive_source_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<(Vec<PreparedLoopStateRef>, NirExpr, Option<NirExpr>)> {
        let NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } = expr
        else {
            return None;
        };
        let invariant = |expr: &NirExpr| {
            is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
        };
        if let Some((factor_terms, factor_offset)) =
            parse_additive_source_for_factor(lhs, binding_name, carries)
        {
            if invariant(rhs) {
                return Some((factor_terms, (**rhs).clone(), factor_offset));
            }
        }
        if let Some((factor_terms, factor_offset)) =
            parse_additive_source_for_factor(rhs, binding_name, carries)
        {
            if invariant(lhs) {
                return Some((factor_terms, (**lhs).clone(), factor_offset));
            }
        }
        None
    }

    fn parse_factor_group_product_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<(
        Vec<PreparedLoopStateRef>,
        Option<NirExpr>,
        Vec<PreparedLoopStateRef>,
        Option<NirExpr>,
    )> {
        let NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } = expr
        else {
            return None;
        };
        let lhs_group = parse_additive_source_for_factor(lhs, binding_name, carries)?;
        let rhs_group = parse_additive_source_for_factor(rhs, binding_name, carries)?;
        Some((lhs_group.0, lhs_group.1, rhs_group.0, rhs_group.1))
    }

    fn parse_scaled_factor_group_product_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<(
        Vec<PreparedLoopStateRef>,
        Option<NirExpr>,
        Vec<PreparedLoopStateRef>,
        Option<NirExpr>,
        NirExpr,
    )> {
        let NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } = expr
        else {
            return None;
        };
        let invariant = |expr: &NirExpr| {
            is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
        };
        if let Some((lhs_terms, lhs_offset, rhs_terms, rhs_offset)) =
            parse_factor_group_product_for_factor(lhs, binding_name, carries)
        {
            if invariant(rhs) {
                return Some((
                    lhs_terms,
                    lhs_offset,
                    rhs_terms,
                    rhs_offset,
                    (**rhs).clone(),
                ));
            }
        }
        if let Some((lhs_terms, lhs_offset, rhs_terms, rhs_offset)) =
            parse_factor_group_product_for_factor(rhs, binding_name, carries)
        {
            if invariant(lhs) {
                return Some((
                    lhs_terms,
                    lhs_offset,
                    rhs_terms,
                    rhs_offset,
                    (**lhs).clone(),
                ));
            }
        }
        None
    }

    let normalized = inline_pure_helper_calls(expr, inlineable_pure_helpers);
    let parse_additive_source = |expr: &NirExpr| -> Option<PreparedCarrySource> {
        fn parse_inner(
            expr: &NirExpr,
            binding_name: &str,
            carries: &[PreparedCarryUpdate],
            expr_contains_loop_variant_ref: &impl Fn(&NirExpr, &str, &[PreparedCarryUpdate]) -> bool,
        ) -> Option<ParsedAdditiveSource> {
            if let Some(state_ref) = parse_prepared_loop_state_ref_expr(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
                    terms: vec![state_ref],
                    offset: None,
                });
            }
            if is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
                    terms: Vec::new(),
                    offset: Some(expr.clone()),
                });
            }
            match expr {
                NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs,
                    rhs,
                } => {
                    let lhs =
                        parse_inner(lhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let rhs =
                        parse_inner(rhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let mut terms = lhs.terms;
                    terms.extend(rhs.terms);
                    let offset = combine_invariant_terms(
                        lhs.offset.into_iter().chain(rhs.offset).collect::<Vec<_>>(),
                    );
                    Some(ParsedAdditiveSource { terms, offset })
                }
                NirExpr::Binary {
                    op: NirBinaryOp::Mul,
                    lhs,
                    rhs,
                } => {
                    let (base, factor) = match (lhs.as_ref(), rhs.as_ref()) {
                        (base, NirExpr::Int(factor)) if *factor >= 0 => (base, *factor),
                        (NirExpr::Int(factor), base) if *factor >= 0 => (base, *factor),
                        _ => return None,
                    };
                    let parsed =
                        parse_inner(base, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let terms = parsed
                        .terms
                        .iter()
                        .flat_map(|term| std::iter::repeat_n(*term, factor as usize))
                        .collect::<Vec<_>>();
                    let offset = parsed
                        .offset
                        .map(|offset| scale_invariant_expr(offset, factor));
                    Some(ParsedAdditiveSource { terms, offset })
                }
                _ => None,
            }
        }

        let parsed = parse_inner(expr, binding_name, carries, &expr_contains_loop_variant_ref)?;
        if parsed.terms.len() + usize::from(parsed.offset.is_some()) <= 1 {
            return None;
        }
        match (parsed.terms.len(), parsed.offset) {
            (0, Some(invariant)) => Some(PreparedCarrySource::InvariantExpr(invariant)),
            (1, Some(invariant)) => Some(PreparedCarrySource::AddInvariant {
                base: Box::new(loop_state_ref_into_carry_source(parsed.terms[0])),
                offset: invariant,
            }),
            (count, offset) if count >= 2 => Some(PreparedCarrySource::AddStateList {
                terms: parsed.terms,
                offset,
            }),
            _ => None,
        }
    };
    if matches!(op, PreparedCarryLinearOp::Add)
        && is_terminal_branch_pure_expr(&normalized, &BTreeSet::new())
        && !expr_contains_loop_variant_ref(&normalized, binding_name, carries)
    {
        return Some(PreparedCarryBranchSource::from_linear_source(
            op,
            PreparedCarrySource::InvariantExpr(normalized),
        ));
    }
    if matches!(op, PreparedCarryLinearOp::Add) && matches!(normalized, NirExpr::Int(0)) {
        return Some(PreparedCarryBranchSource::keep());
    }
    if matches!(op, PreparedCarryLinearOp::Add) {
        if let NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } = &normalized
        {
            let factor_supported = |expr: &NirExpr| {
                parse_prepared_loop_state_ref_expr(expr, binding_name, carries).is_some()
                    || (is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                        && !expr_contains_loop_variant_ref(expr, binding_name, carries))
                    || parse_additive_source_for_factor(expr, binding_name, carries).is_some()
                    || parse_scaled_additive_source_for_factor(expr, binding_name, carries)
                        .is_some()
                    || parse_factor_group_product_for_factor(expr, binding_name, carries).is_some()
                    || parse_scaled_factor_group_product_for_factor(expr, binding_name, carries)
                        .is_some()
            };
            let scaled = if let Some(base) = parse_additive_source(lhs) {
                if factor_supported(rhs) {
                    scale_additive_source(base, (**rhs).clone(), binding_name, carries)
                } else {
                    None
                }
            } else if let Some(base) = parse_additive_source(rhs) {
                if factor_supported(lhs) {
                    scale_additive_source(base, (**lhs).clone(), binding_name, carries)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(source) = scaled {
                return Some(PreparedCarryBranchSource::from_linear_source(op, source));
            }
        }
        if let Some(source) = parse_additive_source(&normalized) {
            return Some(PreparedCarryBranchSource::from_linear_source(op, source));
        }
        if let NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } = &normalized
        {
            if let Some(base_ref) = parse_prepared_loop_state_ref_expr(lhs, binding_name, carries) {
                if is_terminal_branch_pure_expr(rhs, &BTreeSet::new()) {
                    return Some(PreparedCarryBranchSource::from_linear_source(
                        op,
                        PreparedCarrySource::AddInvariant {
                            base: Box::new(loop_state_ref_into_carry_source(base_ref)),
                            offset: (**rhs).clone(),
                        },
                    ));
                }
            }
        }
    }
    if let Some(state_ref) = parse_prepared_loop_state_ref_expr(&normalized, binding_name, carries)
    {
        return Some(PreparedCarryBranchSource::from_linear_source(
            op,
            loop_state_ref_into_carry_source(state_ref),
        ));
    }
    parse_prepared_fixed_read_carry_source(&normalized, binding_name, carries)
        .map(PreparedCarrySource::FixedRead)
        .or_else(|| parse_prepared_dynamic_read_carry_source(&normalized, binding_name, carries))
        .map(|source| PreparedCarryBranchSource::from_linear_source(op, source))
}

fn parse_conditional_temp_driven_loop_carry_update(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    conditional_temps: &BTreeMap<String, PreparedConditionalTempBinding>,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryUpdate> {
    let (carry_name, carry_expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    let normalized = inline_pure_helper_calls(&carry_expr, inlineable_pure_helpers);
    let (op, rhs_name) = match &normalized {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Var(rhs_name)) if lhs_name == &carry_name => {
                (PreparedCarryLinearOp::Add, rhs_name)
            }
            _ => return None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Var(rhs_name)) if lhs_name == &carry_name => {
                (PreparedCarryLinearOp::Mul, rhs_name)
            }
            _ => return None,
        },
        _ => return None,
    };
    let temp = conditional_temps.get(rhs_name)?;
    Some(PreparedCarryUpdate {
        binding_name: carry_name,
        kind: PreparedCarryUpdateKind::Conditional {
            condition: temp.condition.clone(),
            then_source: parse_loop_carry_delta_branch_source(
                op,
                &temp.then_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )?,
            else_source: parse_loop_carry_delta_branch_source(
                op,
                &temp.else_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )?,
        },
    })
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

fn extract_loop_control_temp_binding(
    stmt: &NirStmt,
    consumer_stmts: &[&NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<(String, NirExpr)> {
    let (name, expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    if is_loop_match_scrutinee_temp_binding(&name) {
        return Some((name, expr));
    }
    let declares_bool_temp = match stmt {
        NirStmt::Let { ty, .. } => ty.as_ref().is_some_and(|ty| ty.is_bool_scalar()),
        NirStmt::Const { ty, .. } => ty.is_bool_scalar(),
        _ => false,
    };
    if !declares_bool_temp {
        return None;
    }
    if consumer_stmts
        .iter()
        .any(|consumer| stmt_references_any_name(consumer, &BTreeSet::from([name.clone()])))
    {
        Some((name, expr))
    } else {
        None
    }
}

fn normalize_loop_control_temp_bindings(
    bindings: Vec<(String, NirExpr)>,
) -> Vec<(String, NirExpr)> {
    let mut normalized = Vec::<(String, NirExpr)>::new();
    for (name, expr) in bindings {
        let normalized_expr =
            normalized
                .iter()
                .fold(expr, |current, (binding_name, binding_expr)| {
                    substitute_branch_binding(&current, binding_name, binding_expr)
                });
        normalized.push((name, normalized_expr));
    }
    normalized
}

fn prepare_loop_carry_sequence(
    stmts: &[NirStmt],
    binding_name: &str,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<Vec<PreparedCarryUpdate>> {
    let carry_names =
        collect_loop_carry_binding_names(stmts, pure_helpers, inlineable_pure_helpers)?;
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let mut conditional_temps = BTreeMap::<String, PreparedConditionalTempBinding>::new();
    let mut carry_index = 0usize;
    for stmt in stmts {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
        if let Some(current_carry_name) =
            extract_non_temp_loop_carry_name(stmt, pure_helpers, inlineable_pure_helpers)
        {
            if diagnose_future_carry_reference(
                &substituted,
                &current_carry_name,
                &carry_names[carry_index + 1..],
            )
            .is_some()
            {
                return None;
            }
            carry_index += 1;
        }
        if let Some(prepared) = parse_loop_carry_update(
            &substituted,
            binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(prepared) = parse_conditional_temp_driven_loop_carry_update(
            &substituted,
            binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(temp) = parse_conditional_temp_binding(
            &substituted,
            binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
            continue;
        }
        if let Some(temp) = parse_derived_conditional_temp_binding(
            &substituted,
            binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
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

pub(super) fn diagnose_unsupported_prepared_while_carry(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<String> {
    let (binding_name, _, _) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (step_temp_bindings, step_binding, carry_bindings) =
        split_temp_prefixed_loop_step_bindings(
            body,
            &binding_name,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
    if carry_bindings.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &step_temp_bindings);
    let sync_step = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    );
    let async_step = parse_prepared_async_loop_step(&substituted_step, &binding_name);
    if sync_step.is_none() && async_step.is_none() {
        return None;
    }
    let substituted_carry_bindings = carry_bindings
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &step_temp_bindings))
        .collect::<Vec<_>>();

    let carry_names = collect_loop_carry_binding_names(
        &substituted_carry_bindings,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let mut conditional_temps = BTreeMap::<String, PreparedConditionalTempBinding>::new();
    let mut carry_index = 0usize;
    for stmt in &substituted_carry_bindings {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
        if let Some(current_carry_name) =
            extract_non_temp_loop_carry_name(stmt, pure_helpers, inlineable_pure_helpers)
        {
            if let Some(diagnostic) = diagnose_future_carry_reference(
                &substituted,
                &current_carry_name,
                &carry_names[carry_index + 1..],
            ) {
                return Some(diagnostic);
            }
            carry_index += 1;
        }
        if let Some(diagnostic) = diagnose_unsupported_loop_carry_update(
            &substituted,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        ) {
            return Some(diagnostic);
        }
        if let Some(prepared) = parse_loop_carry_update(
            &substituted,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(prepared) = parse_conditional_temp_driven_loop_carry_update(
            &substituted,
            &binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(temp) = parse_conditional_temp_binding(
            &substituted,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
            continue;
        }
        if let Some(temp) = parse_derived_conditional_temp_binding(
            &substituted,
            &binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
            continue;
        }
        let (temp_name, temp_expr) = extract_pure_branch_binding(&substituted, pure_helpers)?;
        if !is_loop_match_scrutinee_temp_binding(&temp_name) {
            return None;
        }
        temp_bindings.push((temp_name, temp_expr));
    }
    None
}

pub(super) fn diagnose_unstructured_while_shape(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<String> {
    let (binding_name, _, _) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;

    let Some((step_temp_bindings, step_binding, rest)) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    ) else {
        let Some((first_stmt, _)) = body.split_first() else {
            return Some(
                "structured `while` lowering recognized the loop header, but the body is empty; expected a loop-state step, a guarded terminal body, or a structured carry/control sequence"
                    .to_owned(),
            );
        };
        let first_binding_name = match first_stmt {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => Some(name.as_str()),
            _ => None,
        };
        return Some(match first_binding_name {
            Some(name) => format!(
                "structured `while` lowering recognized loop state `{binding_name}`, but the first body binding `{name}` is not a supported temp/step prefix; expected pure temp bindings followed by `{binding_name}` updated via `{binding_name} +/- ...` or `await callee({binding_name})`"
            ),
            None => format!(
                "structured `while` lowering recognized loop state `{binding_name}`, but the body does not begin with a supported step prefix; expected pure temp bindings followed by `let {binding_name} = {binding_name} +/- ...`, `let {binding_name} = await callee({binding_name})`, or a guarded terminal body"
            ),
        });
    };
    let substituted_step = substitute_stmt_bindings(step_binding, &step_temp_bindings);
    let sync_step = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    );
    let async_step = parse_prepared_async_loop_step(&substituted_step, &binding_name);
    if sync_step.is_none() && async_step.is_none() {
        return Some(
            format!(
                "structured `while` lowering recognized loop state `{binding_name}`, but the step binding is not supported after the pure temp prefix; expected `{binding_name}` to be updated via `{binding_name} +/- ...` or `await callee({binding_name})`"
            ),
        );
    }

    if rest.is_empty() {
        if async_step.is_some() {
            return Some(
                "structured async `while` lowering recognized the loop header and awaited step, but the remaining body is empty; expected a structured carry/control sequence after the async step"
                    .to_owned(),
            );
        }
        return None;
    }

    let substituted_rest = rest
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &step_temp_bindings))
        .collect::<Vec<_>>();

    if substituted_rest.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Print(_)
                | NirStmt::Expr(_)
                | NirStmt::Await(_)
                | NirStmt::Return(_)
                | NirStmt::While { .. }
        )
    }) {
        return Some(format!(
            "structured `while` lowering recognized loop state `{binding_name}` and its step, but the remaining body still contains arbitrary executable statements; only pure temp prefixes before the step plus structured carry updates and flow/post-flow control after the step are lowered"
        ));
    }

    let carries = prepare_loop_carry_sequence(
        &substituted_rest,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        &BTreeMap::<String, PureHelperBlock>::new(),
    )
    .unwrap_or_default();
    if let Some(diagnostic) = substituted_rest.iter().find_map(|stmt| {
        diagnose_unstructured_loop_flow_control(
            stmt,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
        )
    }) {
        return Some(diagnostic);
    }

    Some(format!(
        "structured `while` lowering recognized loop state `{binding_name}` and its step, but the remaining body is not reducible to supported carry updates or flow/post-flow control"
    ))
}

fn split_temp_prefixed_loop_flow_control<'a>(
    stmts: &'a [NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<(Vec<(String, NirExpr)>, &'a NirStmt, &'a [NirStmt])> {
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    for (index, stmt) in stmts.iter().enumerate() {
        let remaining = &stmts[index + 1..];
        let consumer_stmts = remaining.iter().collect::<Vec<_>>();
        if let Some((temp_name, temp_expr)) =
            extract_loop_control_temp_binding(stmt, &consumer_stmts, pure_helpers)
        {
            temp_bindings.push((temp_name, temp_expr));
            continue;
        }
        return Some((
            normalize_loop_control_temp_bindings(temp_bindings),
            stmt,
            &stmts[index + 1..],
        ));
    }
    None
}

fn split_trailing_loop_control_temp_bindings<'a>(
    stmts: &'a [NirStmt],
    control_stmt: &'a NirStmt,
    pure_helpers: &BTreeSet<String>,
) -> Option<(&'a [NirStmt], Vec<(String, NirExpr)>)> {
    let mut accepted = Vec::<(String, NirExpr)>::new();
    let mut consumer_stmts = vec![control_stmt];
    let mut split_index = stmts.len();
    for stmt in stmts.iter().rev() {
        let Some((temp_name, temp_expr)) =
            extract_loop_control_temp_binding(stmt, &consumer_stmts, pure_helpers)
        else {
            break;
        };
        accepted.push((temp_name, temp_expr));
        consumer_stmts.push(stmt);
        split_index -= 1;
    }
    accepted.reverse();
    Some((
        &stmts[..split_index],
        normalize_loop_control_temp_bindings(accepted),
    ))
}

fn split_temp_prefixed_loop_step_bindings<'a>(
    body: &'a [NirStmt],
    binding_name: &str,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(Vec<(String, NirExpr)>, &'a NirStmt, &'a [NirStmt])> {
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let prev_current = NirExpr::Var(TAIL_RECURSIVE_PREV_CURRENT_BINDING.to_owned());
    for (index, stmt) in body.iter().enumerate() {
        let binding = match stmt {
            NirStmt::Let { .. } | NirStmt::Const { .. } => stmt,
            _ => return None,
        };
        let (name, expr) = extract_pure_branch_binding(binding, pure_helpers)?;
        if name == binding_name {
            return Some((
                normalize_loop_control_temp_bindings(temp_bindings),
                stmt,
                &body[index + 1..],
            ));
        }
        let normalized = inline_pure_helper_calls(&expr, inlineable_pure_helpers);
        let preserved = substitute_branch_binding(&normalized, binding_name, &prev_current);
        temp_bindings.push((name, preserved));
    }
    None
}

pub(super) fn prepare_counted_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    _pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedCountedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if !rest.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    Some(PreparedCountedWhile {
        binding_name,
        limit,
        step,
        compare,
        step_kind,
    })
}

pub(super) fn prepare_chained_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedChainedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, carry_bindings) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if carry_bindings.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let substituted_carry_bindings = carry_bindings
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();

    let carries = prepare_loop_carry_sequence(
        &substituted_carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
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

pub(super) fn prepare_async_chained_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedAsyncChainedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, carry_bindings) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if carry_bindings.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let step_callee = parse_prepared_async_loop_step(&substituted_step, &binding_name)?;
    let substituted_carry_bindings = carry_bindings
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();

    let carries = prepare_loop_carry_sequence(
        &substituted_carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
    if carries.is_empty() {
        return None;
    }

    Some(PreparedAsyncChainedWhile {
        binding_name,
        limit,
        compare,
        step_callee,
        carries,
    })
}

pub(super) fn prepare_async_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedAsyncFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if rest.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let step_callee = parse_prepared_async_loop_step(&substituted_step, &binding_name)?;
    let substituted_rest = rest
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let (control_temp_bindings, raw_control_stmt, carry_bindings) =
        split_temp_prefixed_loop_flow_control(&substituted_rest, pure_helpers)?;
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
                if let Some(prepared_carries) = prepare_loop_carry_sequence(
                    else_body,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                ) {
                    let control_condition = parse_loop_flow_condition(
                        condition,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (
                        PreparedLoopFlowControl::Terminal {
                            condition: control_condition,
                            action,
                        },
                        prepared_carries,
                    )
                } else {
                    let prepared_carries = prepare_loop_carry_sequence(
                        carry_bindings,
                        &binding_name,
                        pure_helpers,
                        inlineable_pure_helpers,
                        pure_helper_blocks,
                    )?;
                    let control = parse_loop_flow_control(
                        &substituted_control_stmt,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (control, prepared_carries)
                }
            } else {
                let prepared_carries = prepare_loop_carry_sequence(
                    carry_bindings,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                )?;
                let control = parse_loop_flow_control(
                    &substituted_control_stmt,
                    &binding_name,
                    &prepared_carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?;
                (control, prepared_carries)
            }
        } else {
            let prepared_carries = prepare_loop_carry_sequence(
                carry_bindings,
                &binding_name,
                pure_helpers,
                inlineable_pure_helpers,
                pure_helper_blocks,
            )?;
            let control = parse_loop_flow_control(
                &substituted_control_stmt,
                &binding_name,
                &prepared_carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?;
            (control, prepared_carries)
        }
    } else {
        let prepared_carries = prepare_loop_carry_sequence(
            carry_bindings,
            &binding_name,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )?;
        let control = parse_loop_flow_control(
            &substituted_control_stmt,
            &binding_name,
            &prepared_carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        (control, prepared_carries)
    };

    Some(PreparedAsyncFlowWhile {
        binding_name,
        limit,
        compare,
        step_callee,
        control,
        carries: prepared_carries,
    })
}

pub(super) fn prepare_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if rest.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let substituted_rest = rest
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let (control_temp_bindings, raw_control_stmt, carry_bindings) =
        split_temp_prefixed_loop_flow_control(&substituted_rest, pure_helpers)?;
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
                if let Some(prepared_carries) = prepare_loop_carry_sequence(
                    else_body,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                ) {
                    let control_condition = parse_loop_flow_condition(
                        condition,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (
                        PreparedLoopFlowControl::Terminal {
                            condition: control_condition,
                            action,
                        },
                        prepared_carries,
                    )
                } else {
                    let prepared_carries = prepare_loop_carry_sequence(
                        carry_bindings,
                        &binding_name,
                        pure_helpers,
                        inlineable_pure_helpers,
                        pure_helper_blocks,
                    )?;
                    let control = parse_loop_flow_control(
                        &substituted_control_stmt,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (control, prepared_carries)
                }
            } else {
                let prepared_carries = prepare_loop_carry_sequence(
                    carry_bindings,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                )?;
                let control = parse_loop_flow_control(
                    &substituted_control_stmt,
                    &binding_name,
                    &prepared_carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?;
                (control, prepared_carries)
            }
        } else {
            let prepared_carries = prepare_loop_carry_sequence(
                carry_bindings,
                &binding_name,
                pure_helpers,
                inlineable_pure_helpers,
                pure_helper_blocks,
            )?;
            let control = parse_loop_flow_control(
                &substituted_control_stmt,
                &binding_name,
                &prepared_carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?;
            (control, prepared_carries)
        }
    } else {
        let prepared_carries = prepare_loop_carry_sequence(
            carry_bindings,
            &binding_name,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )?;
        let control = parse_loop_flow_control(
            &substituted_control_stmt,
            &binding_name,
            &prepared_carries,
            pure_helpers,
            inlineable_pure_helpers,
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
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedPostFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let [middle @ .., control_stmt] = rest else {
        return None;
    };
    if middle.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let substituted_middle = middle
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let substituted_control_stmt = substitute_stmt_bindings(control_stmt, &temp_bindings);

    let (carry_bindings, control_temp_bindings) = split_trailing_loop_control_temp_bindings(
        &substituted_middle,
        &substituted_control_stmt,
        pure_helpers,
    )?;
    let prepared_carries = prepare_loop_carry_sequence(
        carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
    let final_control_stmt =
        substitute_stmt_bindings(&substituted_control_stmt, &control_temp_bindings);
    let control = parse_loop_flow_control(
        &final_control_stmt,
        &binding_name,
        &prepared_carries,
        pure_helpers,
        inlineable_pure_helpers,
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

pub(super) fn prepare_async_post_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedAsyncPostFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let [middle @ .., control_stmt] = rest else {
        return None;
    };
    if middle.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let step_callee = parse_prepared_async_loop_step(&substituted_step, &binding_name)?;
    let substituted_middle = middle
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let substituted_control_stmt = substitute_stmt_bindings(control_stmt, &temp_bindings);

    let (carry_bindings, control_temp_bindings) = split_trailing_loop_control_temp_bindings(
        &substituted_middle,
        &substituted_control_stmt,
        pure_helpers,
    )?;
    let prepared_carries = prepare_loop_carry_sequence(
        carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
    let final_control_stmt =
        substitute_stmt_bindings(&substituted_control_stmt, &control_temp_bindings);
    let control = parse_loop_flow_control(
        &final_control_stmt,
        &binding_name,
        &prepared_carries,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    Some(PreparedAsyncPostFlowWhile {
        binding_name,
        limit,
        compare,
        step_callee,
        carries: prepared_carries,
        control,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parse_nuis_module;
    use crate::lowering::loop_carries::tail_recursive_prev_carry_binding;

    #[test]
    fn diagnose_unsupported_stmt_carry_tree_allows_previous_value_keep_branch() {
        let stmt = NirStmt::If {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Gt,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            },
            then_body: vec![NirStmt::Let {
                name: "acc".to_owned(),
                ty: None,
                value: NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("acc".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                },
            }],
            else_body: vec![NirStmt::Let {
                name: "acc".to_owned(),
                ty: None,
                value: NirExpr::Var(tail_recursive_prev_carry_binding(0)),
            }],
        };

        let diagnostic = diagnose_unsupported_stmt_carry_tree(
            &stmt,
            "acc",
            "current",
            &[],
            &BTreeSet::new(),
            &BTreeMap::new(),
        );
        assert!(diagnostic.is_none());
    }

    #[test]
    fn extracts_loop_carry_name_from_if_expression_with_branch_local_temp_prefix() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn step(value: i64) -> i64 {
                return value + 1;
              }

              async fn main() -> i64 {
                let value: i64 = 0;
                let acc: i64 = 0;
                while value < 7 {
                  let value: i64 = await step(value);
                  let branch_value: i64 = if value > 2 {
                    let picked: i64 = value;
                    picked
                  } else {
                    let picked: i64 = 0;
                    picked
                  };
                  let acc: i64 = acc + branch_value;
                  if acc > 8 {
                    break;
                  }
                }
                return acc;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("expected main function");
        let NirStmt::While { body, .. } = function
            .body
            .iter()
            .find(|stmt| matches!(stmt, NirStmt::While { .. }))
            .expect("expected while body")
        else {
            unreachable!();
        };

        let branch_stmt = &body[1];
        assert_eq!(
            extract_non_temp_loop_carry_name(branch_stmt, &BTreeSet::new(), &BTreeMap::new())
                .as_deref(),
            Some("branch_value"),
            "loop body shape was: {body:#?}"
        );
    }

    #[test]
    fn prepares_async_post_flow_carry_sequence_with_shared_suffix_branch_value() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn step(value: i64) -> i64 {
                return value + 1;
              }

              async fn main() -> i64 {
                let value: i64 = 0;
                let acc: i64 = 0;
                while value < 7 {
                  let value: i64 = await step(value);
                  let branch_value: i64 = if value > 2 {
                    let picked: i64 = value;
                    picked
                  } else {
                    let picked: i64 = 0;
                    picked
                  };
                  let acc: i64 = acc + branch_value;
                  if acc > 8 {
                    break;
                  }
                }
                return acc;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("expected main function");
        let NirStmt::While { body, .. } = function
            .body
            .iter()
            .find(|stmt| matches!(stmt, NirStmt::While { .. }))
            .expect("expected while body")
        else {
            unreachable!();
        };

        let middle = &body[1..body.len() - 1];
        let prepared = prepare_loop_carry_sequence(
            middle,
            "value",
            &BTreeSet::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("expected carry sequence to prepare");
        assert_eq!(
            prepared
                .iter()
                .map(|carry| carry.binding_name.as_str())
                .collect::<Vec<_>>(),
            vec!["acc"]
        );
    }

    #[test]
    fn prepares_async_post_flow_carry_sequence_with_derived_conditional_temp_suffix() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn step(value: i64) -> i64 {
                return value + 1;
              }

              async fn main() -> i64 {
                let value: i64 = 0;
                let acc: i64 = 0;
                while value < 7 {
                  let value: i64 = await step(value);
                  let branch_value: i64 = if value > 2 {
                    let picked: i64 = value;
                    picked
                  } else {
                    let picked: i64 = 0;
                    picked
                  };
                  let widened: i64 = branch_value + 1;
                  let acc: i64 = acc + widened;
                  if acc > 8 {
                    break;
                  }
                }
                return acc;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("expected main function");
        let NirStmt::While { body, .. } = function
            .body
            .iter()
            .find(|stmt| matches!(stmt, NirStmt::While { .. }))
            .expect("expected while body")
        else {
            unreachable!();
        };

        let middle = &body[1..body.len() - 1];
        let prepared = prepare_loop_carry_sequence(
            middle,
            "value",
            &BTreeSet::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("expected derived conditional temp carry sequence to prepare");
        assert_eq!(
            prepared
                .iter()
                .map(|carry| carry.binding_name.as_str())
                .collect::<Vec<_>>(),
            vec!["acc"]
        );
    }

    #[test]
    fn prepares_async_post_flow_carry_sequence_with_remixed_loop_state_suffix() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn step(value: i64) -> i64 {
                return value + 1;
              }

              async fn main() -> i64 {
                let value: i64 = 0;
                let acc: i64 = 0;
                while value < 7 {
                  let value: i64 = await step(value);
                  let branch_value: i64 = if value > 2 {
                    let picked: i64 = value;
                    picked
                  } else {
                    let picked: i64 = 0;
                    picked
                  };
                  let widened: i64 = branch_value + 1;
                  let normalized: i64 = widened + value;
                  let acc: i64 = acc + normalized;
                  if acc > 8 {
                    break;
                  }
                }
                return acc;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("expected main function");
        let NirStmt::While { body, .. } = function
            .body
            .iter()
            .find(|stmt| matches!(stmt, NirStmt::While { .. }))
            .expect("expected while body")
        else {
            unreachable!();
        };

        let middle = &body[1..body.len() - 1];
        let prepared = prepare_loop_carry_sequence(
            middle,
            "value",
            &BTreeSet::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("expected remixed loop-state carry sequence to prepare");
        assert_eq!(
            prepared
                .iter()
                .map(|carry| carry.binding_name.as_str())
                .collect::<Vec<_>>(),
            vec!["acc"]
        );
    }

    #[test]
    fn parses_derived_conditional_temp_binding_with_loop_state_remix() {
        let stmt = NirStmt::Let {
            name: "normalized".to_owned(),
            ty: None,
            value: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("widened".to_owned())),
                rhs: Box::new(NirExpr::Var("value".to_owned())),
            },
        };
        let mut conditional_temps = BTreeMap::new();
        conditional_temps.insert(
            "widened".to_owned(),
            PreparedConditionalTempBinding {
                binding_name: "widened".to_owned(),
                condition: PreparedLoopFlowCondition::Simple(PreparedLoopCarryCondition {
                    lhs: PreparedCarryCondSource::Current,
                    compare: PreparedLoopCompare::Gt,
                    rhs: NirExpr::Int(2),
                }),
                then_expr: NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                },
                else_expr: NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Int(0)),
                    rhs: Box::new(NirExpr::Int(1)),
                },
            },
        );

        let prepared = parse_derived_conditional_temp_binding(
            &stmt,
            "value",
            &[],
            &conditional_temps,
            &BTreeSet::new(),
            &BTreeMap::new(),
        )
        .expect("expected remixed derived conditional temp binding");
        assert_eq!(prepared.binding_name, "normalized");
    }

    #[test]
    fn parses_conditional_temp_driven_carry_update_with_loop_state_remix() {
        let stmt = NirStmt::Let {
            name: "acc".to_owned(),
            ty: None,
            value: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("acc".to_owned())),
                rhs: Box::new(NirExpr::Var("normalized".to_owned())),
            },
        };
        let mut conditional_temps = BTreeMap::new();
        conditional_temps.insert(
            "normalized".to_owned(),
            PreparedConditionalTempBinding {
                binding_name: "normalized".to_owned(),
                condition: PreparedLoopFlowCondition::Simple(PreparedLoopCarryCondition {
                    lhs: PreparedCarryCondSource::Current,
                    compare: PreparedLoopCompare::Gt,
                    rhs: NirExpr::Int(2),
                }),
                then_expr: NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Var("value".to_owned())),
                        rhs: Box::new(NirExpr::Int(1)),
                    }),
                    rhs: Box::new(NirExpr::Var("value".to_owned())),
                },
                else_expr: NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Int(0)),
                        rhs: Box::new(NirExpr::Int(1)),
                    }),
                    rhs: Box::new(NirExpr::Var("value".to_owned())),
                },
            },
        );

        let prepared = parse_conditional_temp_driven_loop_carry_update(
            &stmt,
            "value",
            &[],
            &conditional_temps,
            &BTreeSet::new(),
            &BTreeMap::new(),
        )
        .expect("expected remixed conditional temp carry update");
        assert_eq!(prepared.binding_name, "acc");
    }
}
