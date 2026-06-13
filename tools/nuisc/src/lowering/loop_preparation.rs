use super::*;
use crate::lowering::loop_carries::{
    diagnose_unsupported_loop_carry_expr, loop_compare_from_binary_op,
    loop_state_ref_into_cond_source, parse_loop_carry_branch_source, parse_loop_carry_linear,
    parse_prepared_loop_state_ref_expr, unsupported_loop_carry_branch_source_message,
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
                    carries,
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
        carries,
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
        condition: PreparedLoopFlowCondition,
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
            let [then_stmt] = then_body.as_slice() else {
                return None;
            };
            let [else_stmt] = else_body.as_slice() else {
                return None;
            };
            let then_name = extract_single_stmt_carry_name(then_stmt, pure_helpers)?;
            let else_name = extract_single_stmt_carry_name(else_stmt, pure_helpers)?;
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
) -> Option<String> {
    let name = extract_single_stmt_carry_name(stmt, pure_helpers)?;
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
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
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
        | NirExpr::CpuSpawn { args, .. } => {
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
                || body.iter().any(|stmt| stmt_references_any_name(stmt, names))
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}

fn collect_loop_carry_binding_names(
    stmts: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for stmt in stmts {
        if extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers).is_some() {
            continue;
        }
        names.push(extract_non_temp_loop_carry_name(stmt, pure_helpers)?);
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
            let [then_stmt] = then_body.as_slice() else {
                return None;
            };
            let [else_stmt] = else_body.as_slice() else {
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
            let carry_name = extract_single_stmt_carry_name(stmt, pure_helpers)?;
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
    let carry_name = extract_single_stmt_carry_name(stmt, pure_helpers)?;
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
    let carry_names = collect_loop_carry_binding_names(stmts, pure_helpers)?;
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let mut carry_index = 0usize;
    for stmt in stmts {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
        if let Some(current_carry_name) = extract_non_temp_loop_carry_name(stmt, pure_helpers) {
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

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), carry_bindings @ ..] = body
    else {
        return None;
    };
    if carry_bindings.is_empty() {
        return None;
    }
    let sync_step = parse_prepared_loop_step(
        step_binding,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    );
    let async_step = parse_prepared_async_loop_step(step_binding, &binding_name);
    if sync_step.is_none() && async_step.is_none() {
        return None;
    }

    let carry_names = collect_loop_carry_binding_names(carry_bindings, pure_helpers)?;
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let mut carry_index = 0usize;
    for stmt in carry_bindings {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
        if let Some(current_carry_name) = extract_non_temp_loop_carry_name(stmt, pure_helpers) {
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
        let (temp_name, temp_expr) = extract_pure_branch_binding(&substituted, pure_helpers)?;
        if !is_loop_match_scrutinee_temp_binding(&temp_name) {
            return None;
        }
        temp_bindings.push((temp_name, temp_expr));
    }
    None
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

pub(super) fn prepare_async_chained_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedAsyncChainedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), carry_bindings @ ..] = body
    else {
        return None;
    };
    if carry_bindings.is_empty() {
        return None;
    }
    let step_callee = parse_prepared_async_loop_step(step_binding, &binding_name)?;

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

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), rest @ ..] = body else {
        return None;
    };
    if rest.is_empty() {
        return None;
    }
    let step_callee = parse_prepared_async_loop_step(step_binding, &binding_name)?;
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
                let control_condition = parse_loop_flow_condition(
                    condition,
                    &binding_name,
                    &prepared_carries,
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
                let control_condition = parse_loop_flow_condition(
                    condition,
                    &binding_name,
                    &prepared_carries,
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
    let control_temp_bindings = control_temp_stmts
        .iter()
        .map(|stmt| extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers))
        .collect::<Option<Vec<_>>>()?;
    let substituted_control_stmt = substitute_stmt_bindings(control_stmt, &control_temp_bindings);
    let control = parse_loop_flow_control(
        &substituted_control_stmt,
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

    let [step_binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), middle @ .., control_stmt] =
        body
    else {
        return None;
    };
    if middle.is_empty() {
        return None;
    }
    let step_callee = parse_prepared_async_loop_step(step_binding, &binding_name)?;

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
    let control_temp_bindings = control_temp_stmts
        .iter()
        .map(|stmt| extract_loop_match_scrutinee_temp_binding(stmt, pure_helpers))
        .collect::<Option<Vec<_>>>()?;
    let substituted_control_stmt = substitute_stmt_bindings(control_stmt, &control_temp_bindings);
    let control = parse_loop_flow_control(
        &substituted_control_stmt,
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
}
