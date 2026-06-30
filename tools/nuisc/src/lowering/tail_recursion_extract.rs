use super::*;

pub(super) fn extract_self_tail_recursive_shape(
    function: &NirFunction,
    pure_helpers: &BTreeSet<String>,
) -> Option<(NirExpr, NirExpr, SelfTailRecursiveStep)> {
    match function.body.as_slice() {
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }, NirStmt::Return(Some(recursive_return))]
            if else_body.is_empty() =>
        {
            let base_return = extract_terminal_return_expr(then_body, pure_helpers)?;
            let recursive_args = extract_self_tail_recursive_call(function, recursive_return)?;
            Some((
                invert_self_tail_recursive_condition(condition, &function.params[0].name)?,
                base_return,
                SelfTailRecursiveStep::Linear(recursive_args),
            ))
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            if let Some(base_return) = extract_terminal_return_expr(then_body, pure_helpers) {
                let recursive_return = extract_terminal_return_expr(else_body, pure_helpers)?;
                let recursive_args = extract_self_tail_recursive_call(function, &recursive_return)?;
                return Some((
                    invert_self_tail_recursive_condition(condition, &function.params[0].name)?,
                    base_return,
                    SelfTailRecursiveStep::Linear(recursive_args),
                ));
            }
            let recursive_return = extract_terminal_return_expr(then_body, pure_helpers)?;
            let recursive_args = extract_self_tail_recursive_call(function, &recursive_return)?;
            let base_return = extract_terminal_return_expr(else_body, pure_helpers)?;
            Some((
                condition.clone(),
                base_return,
                SelfTailRecursiveStep::Linear(recursive_args),
            ))
        }
        [NirStmt::If {
            condition: base_condition,
            then_body: base_then,
            else_body: base_else,
        }, recursive_branch]
            if base_else.is_empty() =>
        {
            let base_return = extract_terminal_return_expr(base_then, pure_helpers)?;
            if let Some(flow_step) = extract_self_tail_recursive_flow_break_step(
                function,
                recursive_branch,
                pure_helpers,
                &base_return,
            ) {
                return Some((
                    invert_self_tail_recursive_condition(base_condition, &function.params[0].name)?,
                    base_return,
                    flow_step,
                ));
            }
            let recursive_step =
                extract_self_tail_recursive_branch_step(function, recursive_branch, pure_helpers)?;
            Some((
                invert_self_tail_recursive_condition(base_condition, &function.params[0].name)?,
                base_return,
                recursive_step,
            ))
        }
        [NirStmt::If {
            condition: base_condition,
            then_body: base_then,
            else_body: base_else,
        }, control_stmt, recursive_stmt]
            if base_else.is_empty() =>
        {
            let base_return = extract_terminal_return_expr(base_then, pure_helpers)?;
            if let Some(post_flow_step) = extract_self_tail_recursive_post_flow_break_step(
                function,
                control_stmt,
                recursive_stmt,
                pure_helpers,
                &base_return,
            ) {
                return Some((
                    invert_self_tail_recursive_condition(base_condition, &function.params[0].name)?,
                    base_return,
                    post_flow_step,
                ));
            }
            let recursive_step = extract_self_tail_recursive_branch_step_from_body(
                function,
                std::slice::from_ref(control_stmt),
                std::slice::from_ref(recursive_stmt),
                pure_helpers,
            )?;
            Some((
                invert_self_tail_recursive_condition(base_condition, &function.params[0].name)?,
                base_return,
                recursive_step,
            ))
        }
        _ => None,
    }
}

fn extract_self_tail_recursive_branch_step(
    function: &NirFunction,
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
) -> Option<SelfTailRecursiveStep> {
    let tree = extract_self_tail_recursive_decision_tree(function, stmt, pure_helpers)?;
    collapse_self_tail_recursive_decision_tree(&tree)
}

fn extract_self_tail_recursive_branch_step_from_body(
    function: &NirFunction,
    prefix: &[NirStmt],
    suffix: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<SelfTailRecursiveStep> {
    let mut combined = Vec::with_capacity(prefix.len() + suffix.len());
    combined.extend_from_slice(prefix);
    combined.extend_from_slice(suffix);
    let tree = extract_self_tail_recursive_tree_body(function, &combined, pure_helpers)?;
    collapse_self_tail_recursive_decision_tree(&tree)
}

fn extract_self_tail_recursive_decision_tree(
    function: &NirFunction,
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
) -> Option<SelfTailRecursiveDecisionTree> {
    match stmt {
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => Some(SelfTailRecursiveDecisionTree::Branch {
            condition: condition.clone(),
            then_tree: Box::new(extract_self_tail_recursive_tree_body(
                function,
                then_body,
                pure_helpers,
            )?),
            else_tree: Box::new(extract_self_tail_recursive_tree_body(
                function,
                else_body,
                pure_helpers,
            )?),
        }),
        NirStmt::Return(Some(expr)) => Some(SelfTailRecursiveDecisionTree::Leaf(
            extract_self_tail_recursive_call(function, expr)?,
        )),
        _ => None,
    }
}

fn extract_self_tail_recursive_tree_body(
    function: &NirFunction,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<SelfTailRecursiveDecisionTree> {
    if let Some(returned) = extract_terminal_return_expr(body, pure_helpers) {
        return Some(SelfTailRecursiveDecisionTree::Leaf(
            extract_self_tail_recursive_call(function, &returned)?,
        ));
    }
    match body {
        [stmt] => extract_self_tail_recursive_decision_tree(function, stmt, pure_helpers),
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }, tail @ NirStmt::Return(Some(_))]
            if else_body.is_empty() =>
        {
            Some(SelfTailRecursiveDecisionTree::Branch {
                condition: condition.clone(),
                then_tree: Box::new(extract_self_tail_recursive_tree_body(
                    function,
                    then_body,
                    pure_helpers,
                )?),
                else_tree: Box::new(extract_self_tail_recursive_tree_body(
                    function,
                    std::slice::from_ref(tail),
                    pure_helpers,
                )?),
            })
        }
        _ => None,
    }
}

fn collapse_self_tail_recursive_decision_tree(
    tree: &SelfTailRecursiveDecisionTree,
) -> Option<SelfTailRecursiveStep> {
    fn leaf_args(tree: &SelfTailRecursiveDecisionTree) -> Option<&Vec<NirExpr>> {
        match tree {
            SelfTailRecursiveDecisionTree::Leaf(args) => Some(args),
            SelfTailRecursiveDecisionTree::Branch { .. } => None,
        }
    }

    match tree {
        SelfTailRecursiveDecisionTree::Leaf(args) => {
            Some(SelfTailRecursiveStep::Linear(args.clone()))
        }
        SelfTailRecursiveDecisionTree::Branch {
            condition,
            then_tree,
            else_tree,
        } => {
            if let (Some(then_args), Some(else_args)) = (leaf_args(then_tree), leaf_args(else_tree))
            {
                return Some(SelfTailRecursiveStep::Branch {
                    condition: condition.clone(),
                    then_args: then_args.clone(),
                    else_args: else_args.clone(),
                });
            }

            if let Some(then_args) = leaf_args(then_tree) {
                if let Some(SelfTailRecursiveStep::Branch {
                    condition: nested_condition,
                    then_args: nested_then_args,
                    else_args: nested_else_args,
                }) = collapse_self_tail_recursive_decision_tree(else_tree)
                {
                    if &nested_then_args == then_args {
                        return Some(SelfTailRecursiveStep::Branch {
                            condition: NirExpr::Binary {
                                op: NirBinaryOp::Or,
                                lhs: Box::new(condition.clone()),
                                rhs: Box::new(nested_condition),
                            },
                            then_args: then_args.clone(),
                            else_args: nested_else_args,
                        });
                    }
                }
            }

            if let Some(else_args) = leaf_args(else_tree) {
                if let Some(SelfTailRecursiveStep::Branch {
                    condition: nested_condition,
                    then_args: nested_then_args,
                    else_args: nested_else_args,
                }) = collapse_self_tail_recursive_decision_tree(then_tree)
                {
                    if &nested_else_args == else_args {
                        return Some(SelfTailRecursiveStep::Branch {
                            condition: NirExpr::Binary {
                                op: NirBinaryOp::And,
                                lhs: Box::new(condition.clone()),
                                rhs: Box::new(nested_condition),
                            },
                            then_args: nested_then_args,
                            else_args: else_args.clone(),
                        });
                    }
                }
            }

            None
        }
    }
}

fn extract_self_tail_recursive_flow_break_step(
    function: &NirFunction,
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
    base_return: &NirExpr,
) -> Option<SelfTailRecursiveStep> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    if else_body.is_empty() {
        return None;
    }
    let early_return = extract_terminal_return_expr(then_body, pure_helpers)?;
    if &early_return != base_return {
        return None;
    }
    let recursive_step =
        if let Some(recursive_return) = extract_terminal_return_expr(else_body, pure_helpers) {
            SelfTailRecursiveStep::Linear(extract_self_tail_recursive_call(
                function,
                &recursive_return,
            )?)
        } else if else_body.len() == 1 {
            extract_self_tail_recursive_branch_step(function, &else_body[0], pure_helpers)?
        } else {
            return None;
        };
    Some(SelfTailRecursiveStep::FlowBreak {
        condition: condition.clone(),
        recursive_step: Box::new(recursive_step),
    })
}

fn extract_self_tail_recursive_post_flow_break_step(
    function: &NirFunction,
    control_stmt: &NirStmt,
    recursive_stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
    base_return: &NirExpr,
) -> Option<SelfTailRecursiveStep> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = control_stmt
    else {
        return None;
    };
    if !else_body.is_empty() {
        return None;
    }
    let early_return = extract_terminal_return_expr(then_body, pure_helpers)?;
    let recursive_step =
        extract_self_tail_recursive_branch_step(function, recursive_stmt, pure_helpers)?;
    let control_carry_index = match &recursive_step {
        SelfTailRecursiveStep::Linear(args) => {
            args.iter().enumerate().skip(1).find_map(|(index, arg)| {
                if *arg == early_return {
                    Some(index)
                } else {
                    None
                }
            })?
        }
        SelfTailRecursiveStep::Branch {
            then_args,
            else_args,
            ..
        } => then_args
            .iter()
            .zip(else_args.iter())
            .enumerate()
            .skip(1)
            .find_map(|(index, (then_arg, else_arg))| {
                if *then_arg == early_return && *else_arg == early_return {
                    Some(index)
                } else {
                    None
                }
            })?,
        _ => return None,
    };
    if base_return == &early_return {
        return None;
    }
    Some(SelfTailRecursiveStep::PostFlowBreak {
        condition: condition.clone(),
        recursive_step: Box::new(recursive_step),
        control_carry_index,
    })
}

fn extract_terminal_return_expr(
    stmts: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<NirExpr> {
    let (NirStmt::Return(Some(expr)), prefix) = stmts.split_last()? else {
        return None;
    };
    let mut substituted = expr.clone();
    for stmt in prefix.iter().rev() {
        let (binding_name, binding_value) = extract_pure_branch_binding(stmt, pure_helpers)?;
        substituted = substitute_branch_binding(&substituted, &binding_name, &binding_value);
    }
    Some(substituted)
}

fn extract_self_tail_recursive_call(
    function: &NirFunction,
    expr: &NirExpr,
) -> Option<Vec<NirExpr>> {
    match expr {
        NirExpr::Call { callee, args } if callee == &function.name => Some(args.clone()),
        NirExpr::Await(inner) if function.is_async => match inner.as_ref() {
            NirExpr::Call { callee, args } if callee == &function.name => Some(args.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn invert_self_tail_recursive_condition(
    condition: &NirExpr,
    current_name: &str,
) -> Option<NirExpr> {
    let NirExpr::Binary { op, lhs, rhs } = condition else {
        return None;
    };
    let NirExpr::Var(name) = lhs.as_ref() else {
        return None;
    };
    if name != current_name {
        return None;
    }
    let inverted = match op {
        NirBinaryOp::Eq => NirBinaryOp::Ne,
        NirBinaryOp::Ne => NirBinaryOp::Eq,
        NirBinaryOp::Lt => NirBinaryOp::Ge,
        NirBinaryOp::Le => NirBinaryOp::Gt,
        NirBinaryOp::Gt => NirBinaryOp::Le,
        NirBinaryOp::Ge => NirBinaryOp::Lt,
        _ => return None,
    };
    Some(NirExpr::Binary {
        op: inverted,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    })
}
