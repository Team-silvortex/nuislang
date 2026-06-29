use super::*;

enum PreparedReturnDecisionTree {
    Return(NirExpr),
    Branch {
        condition: NirExpr,
        then_tree: Box<PreparedReturnDecisionTree>,
        else_tree: Box<PreparedReturnDecisionTree>,
    },
}

pub(super) enum PreparedCarryDecisionTree {
    Leaf(PreparedCarryBranchSource),
    Branch {
        condition: PreparedLoopFlowCondition,
        then_tree: Box<PreparedCarryDecisionTree>,
        else_tree: Box<PreparedCarryDecisionTree>,
    },
}

#[derive(Clone)]
pub(super) struct PreparedConditionalTempBinding {
    pub(super) binding_name: String,
    pub(super) condition: PreparedLoopFlowCondition,
    pub(super) then_expr: NirExpr,
    pub(super) else_expr: NirExpr,
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

pub(super) fn normalize_pure_stmt_prefix_body(
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

pub(super) fn collapse_carry_decision_tree(
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

pub(super) fn parse_helper_conditional_carry_update(
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

pub(super) fn extract_single_stmt_carry_name(
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

pub(super) fn extract_non_temp_loop_carry_name(
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

pub(super) fn collect_loop_carry_binding_names(
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

pub(super) fn diagnose_future_carry_reference(
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

pub(super) fn parse_stmt_carry_decision_tree(
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

pub(super) fn diagnose_unsupported_stmt_carry_tree(
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
