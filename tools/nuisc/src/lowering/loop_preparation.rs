use super::*;
use crate::lowering::loop_carries::{
    loop_compare_from_binary_op, parse_loop_carry_branch_source, parse_loop_carry_condition,
    parse_loop_carry_linear,
};
use crate::lowering::loop_purity::{
    normalize_pure_bool_test_expr, substitute_branch_binding, substitute_stmt_bindings,
};

fn parse_loop_flow_condition_atom(
    expr: &NirExpr,
    binding_name: &str,
    carry_binding_names: &[String],
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
                carry_binding_names,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
            rhs: Box::new(parse_loop_flow_condition(
                rhs,
                binding_name,
                carry_binding_names,
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
                carry_binding_names,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
            rhs: Box::new(parse_loop_flow_condition(
                rhs,
                binding_name,
                carry_binding_names,
                pure_helpers,
                inlineable_pure_helpers,
            )?),
        }),
        _ => Some(PreparedLoopFlowCondition::Simple(
            parse_loop_flow_condition_atom(
                expr,
                binding_name,
                carry_binding_names,
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
    carry_binding_names: &[String],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedLoopFlowControl> {
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
        carry_binding_names,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
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
                    inlineable_pure_helpers,
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
        inlineable_pure_helpers,
    )?;
    if !matches!(
        (then_action, nested.action),
        (PreparedLoopFlowAction::Break, PreparedLoopFlowAction::Break)
            | (
                PreparedLoopFlowAction::Continue,
                PreparedLoopFlowAction::Continue
            )
    ) {
        return None;
    }
    let condition =
        combine_loop_flow_conditions(outer_condition, PreparedLoopLogicOp::Or, nested.condition)?;
    Some(PreparedLoopFlowControl {
        condition,
        action: then_action,
    })
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
        condition: PreparedLoopCarryCondition,
        then_tree: Box<PreparedCarryDecisionTree>,
        else_tree: Box<PreparedCarryDecisionTree>,
    },
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
            condition: parse_loop_carry_condition(
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
            PreparedCarryDecisionTree::Leaf(source) => Some(*source),
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
                return Some((
                    PreparedLoopFlowCondition::Simple(condition.clone()),
                    then_source,
                    else_source,
                ));
            }

            if let Some(then_source) = leaf_source(then_tree) {
                if let Some((nested_condition, nested_then, nested_else)) =
                    collapse_carry_decision_tree(else_tree)
                {
                    if nested_then == then_source {
                        return Some((
                            PreparedLoopFlowCondition::Compound {
                                op: PreparedLoopLogicOp::Or,
                                lhs: Box::new(PreparedLoopFlowCondition::Simple(condition.clone())),
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
                                lhs: Box::new(PreparedLoopFlowCondition::Simple(condition.clone())),
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

fn parse_loop_carry_update(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedCarryUpdate> {
    match stmt {
        carry_stmt @ (NirStmt::Let { .. } | NirStmt::Const { .. }) => {
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
            let condition = parse_loop_carry_condition(
                condition,
                binding_name,
                carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?;
            let then_source = parse_loop_carry_branch_source(
                &then_name,
                &then_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )?;
            let else_source = parse_loop_carry_branch_source(
                &else_name,
                &else_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )?;
            Some(PreparedCarryUpdate {
                binding_name: then_name,
                kind: PreparedCarryUpdateKind::Conditional {
                    condition: PreparedLoopFlowCondition::Simple(condition),
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
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<Vec<PreparedCarryUpdate>> {
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    for stmt in stmts {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
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
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    _pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedCountedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;

    match body {
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. })] => {
            let (step, step_kind) = parse_prepared_loop_step(
                binding,
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
        _ => None,
    }
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

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), carry_bindings @ ..] = body
    else {
        return None;
    };
    if carry_bindings.is_empty() {
        return None;
    }
    let (step, step_kind) = parse_prepared_loop_step(
        step_binding,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;

    let carries = prepare_loop_carry_sequence(
        carry_bindings,
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

pub(super) fn prepare_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), rest @ ..] = body else {
        return None;
    };
    if rest.is_empty() {
        return None;
    }
    let (step, step_kind) = parse_prepared_loop_step(
        step_binding,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
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
                let prepared_carries = prepare_loop_carry_sequence(
                    else_body,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                )?;
                let carry_binding_names = prepared_carries
                    .iter()
                    .map(|carry| carry.binding_name.clone())
                    .collect::<Vec<_>>();
                let control_condition = parse_loop_flow_condition(
                    condition,
                    &binding_name,
                    &carry_binding_names,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?;
                (
                    PreparedLoopFlowControl {
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
                let carry_binding_names = prepared_carries
                    .iter()
                    .map(|carry| carry.binding_name.clone())
                    .collect::<Vec<_>>();
                let control = parse_loop_flow_control(
                    &substituted_control_stmt,
                    &binding_name,
                    &carry_binding_names,
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
            let carry_binding_names = prepared_carries
                .iter()
                .map(|carry| carry.binding_name.clone())
                .collect::<Vec<_>>();
            let control = parse_loop_flow_control(
                &substituted_control_stmt,
                &binding_name,
                &carry_binding_names,
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
        let carry_binding_names = prepared_carries
            .iter()
            .map(|carry| carry.binding_name.clone())
            .collect::<Vec<_>>();
        let control = parse_loop_flow_control(
            &substituted_control_stmt,
            &binding_name,
            &carry_binding_names,
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

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), middle @ .., control_stmt] =
        body
    else {
        return None;
    };
    if middle.is_empty() {
        return None;
    }
    let (step, step_kind) = parse_prepared_loop_step(
        step_binding,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;

    let trailing_temp_count = middle
        .iter()
        .rev()
        .take_while(|stmt| extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers).is_some())
        .count();
    let split_index = middle.len().saturating_sub(trailing_temp_count);
    let (carry_bindings, control_temp_stmts) = middle.split_at(split_index);
    let prepared_carries = prepare_loop_carry_sequence(
        carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
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
