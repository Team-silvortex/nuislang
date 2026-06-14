use super::*;
use crate::lowering::loop_carries::tail_recursive_prev_carry_binding;
use crate::lowering::loop_purity::{
    collect_inlineable_pure_helper_exprs, extract_pure_branch_binding, substitute_branch_binding,
};
use nuis_semantics::model::NirParam;

pub(super) fn rewrite_self_tail_recursive_functions(module: &NirModule) -> NirModule {
    let pure_helpers = collect_pure_helper_functions(module);
    let inlineable_pure_helpers = collect_inlineable_pure_helper_exprs(module);
    let pure_helper_blocks = collect_pure_helper_blocks(module);
    let mut rewritten = module.clone();
    for function in &mut rewritten.functions {
        if let Some(body) = rewrite_self_tail_recursive_function(
            function,
            &pure_helpers,
            &inlineable_pure_helpers,
            &pure_helper_blocks,
        ) {
            function.body = body;
        }
    }
    rewritten
}

fn rewrite_self_tail_recursive_function(
    function: &NirFunction,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<Vec<NirStmt>> {
    if function.params.is_empty() {
        return None;
    }

    let (recurse_condition, base_return, recursive_step) =
        extract_self_tail_recursive_shape(function, pure_helpers)?;
    let loop_body = rewrite_self_tail_recursive_loop_body(function, recursive_step)?;

    if !is_self_tail_recursive_loop_shape(
        &recurse_condition,
        &loop_body,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    ) {
        return None;
    }

    Some(vec![
        NirStmt::While {
            condition: recurse_condition,
            body: loop_body,
        },
        NirStmt::Return(Some(base_return)),
    ])
}

enum SelfTailRecursiveStep {
    Linear(Vec<NirExpr>),
    FlowBreak {
        condition: NirExpr,
        recursive_step: Box<SelfTailRecursiveStep>,
    },
    PostFlowBreak {
        condition: NirExpr,
        recursive_step: Box<SelfTailRecursiveStep>,
        control_carry_index: usize,
    },
    Branch {
        condition: NirExpr,
        then_args: Vec<NirExpr>,
        else_args: Vec<NirExpr>,
    },
}

enum SelfTailRecursiveDecisionTree {
    Leaf(Vec<NirExpr>),
    Branch {
        condition: NirExpr,
        then_tree: Box<SelfTailRecursiveDecisionTree>,
        else_tree: Box<SelfTailRecursiveDecisionTree>,
    },
}

fn extract_self_tail_recursive_shape(
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

fn rewrite_self_tail_recursive_loop_body(
    function: &NirFunction,
    recursive_step: SelfTailRecursiveStep,
) -> Option<Vec<NirStmt>> {
    let invariant_param_names = tail_recursive_invariant_param_names(function, &recursive_step)?;
    let carry_param_names = function
        .params
        .iter()
        .skip(1)
        .filter(|param| !invariant_param_names.contains(&param.name))
        .map(|param| param.name.clone())
        .collect::<Vec<_>>();
    match recursive_step {
        SelfTailRecursiveStep::Linear(recursive_args) => {
            if recursive_args.len() != function.params.len() {
                return None;
            }

            let next_current_expr = recursive_args[0].clone();

            Some(
                function
                    .params
                    .iter()
                    .zip(recursive_args.iter())
                    .enumerate()
                    .filter_map(|(index, (param, arg))| {
                        let value = if index == 0 {
                            arg.clone()
                        } else {
                            canonicalize_tail_recursive_loop_arg(
                                arg,
                                &function.params[0].name,
                                &carry_param_names,
                                &invariant_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            )
                        };
                        tail_recursive_param_update_stmt(index, param, value)
                    })
                    .collect(),
            )
        }
        SelfTailRecursiveStep::FlowBreak {
            condition,
            recursive_step,
        } => {
            let (next_current_expr, tail_stmts) = match recursive_step.as_ref() {
                SelfTailRecursiveStep::Linear(recursive_args) => {
                    if recursive_args.len() != function.params.len() {
                        return None;
                    }
                    let next_current_expr = recursive_args[0].clone();
                    let tail_stmts = function
                        .params
                        .iter()
                        .enumerate()
                        .skip(1)
                        .filter_map(|(index, param)| {
                            let value = canonicalize_tail_recursive_loop_arg(
                                &recursive_args[index],
                                &function.params[0].name,
                                &carry_param_names,
                                &invariant_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            );
                            tail_recursive_param_update_stmt(index, param, value)
                        })
                        .collect::<Vec<_>>();
                    (next_current_expr, tail_stmts)
                }
                SelfTailRecursiveStep::Branch {
                    condition: branch_condition,
                    then_args,
                    else_args,
                } => {
                    if then_args.len() != function.params.len()
                        || else_args.len() != function.params.len()
                    {
                        return None;
                    }
                    if then_args[0] != else_args[0] {
                        return None;
                    }
                    let next_current_expr = then_args[0].clone();
                    let branch_condition = canonicalize_tail_recursive_condition_expr(
                        branch_condition,
                        &function.params[0].name,
                        &carry_param_names,
                        &invariant_param_names,
                    );
                    let tail_stmts = function
                        .params
                        .iter()
                        .enumerate()
                        .skip(1)
                        .filter_map(|(index, param)| {
                            let then_value = canonicalize_tail_recursive_loop_arg(
                                &then_args[index],
                                &function.params[0].name,
                                &carry_param_names,
                                &invariant_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            );
                            let else_value = canonicalize_tail_recursive_loop_arg(
                                &else_args[index],
                                &function.params[0].name,
                                &carry_param_names,
                                &invariant_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            );
                            tail_recursive_param_branch_update_stmt(
                                index,
                                param,
                                branch_condition.clone(),
                                then_value,
                                else_value,
                            )
                        })
                        .collect::<Vec<_>>();
                    (next_current_expr, tail_stmts)
                }
                SelfTailRecursiveStep::FlowBreak { .. } => return None,
                SelfTailRecursiveStep::PostFlowBreak { .. } => return None,
            };
            let mut body = vec![NirStmt::Let {
                name: function.params[0].name.clone(),
                ty: Some(function.params[0].ty.clone()),
                value: next_current_expr.clone(),
            }];
            body.push(NirStmt::If {
                condition: canonicalize_tail_recursive_condition_expr(
                    &condition,
                    &function.params[0].name,
                    &carry_param_names,
                    &invariant_param_names,
                ),
                then_body: vec![NirStmt::Break],
                else_body: vec![],
            });
            body.extend(tail_stmts);
            Some(body)
        }
        SelfTailRecursiveStep::PostFlowBreak {
            condition,
            recursive_step,
            control_carry_index,
        } => {
            let branch_condition;
            let (next_current_expr, control_carry_expr, tail_stmts) = match recursive_step.as_ref()
            {
                SelfTailRecursiveStep::Linear(recursive_args) => {
                    if recursive_args.len() != function.params.len() {
                        return None;
                    }
                    let next_current_expr = recursive_args[0].clone();
                    let tail_stmts = function
                        .params
                        .iter()
                        .enumerate()
                        .skip(1)
                        .filter(|(index, _)| *index != control_carry_index)
                        .filter_map(|(index, param)| {
                            let value = canonicalize_tail_recursive_loop_arg(
                                &recursive_args[index],
                                &function.params[0].name,
                                &carry_param_names,
                                &invariant_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            );
                            tail_recursive_param_update_stmt(index, param, value)
                        })
                        .collect::<Vec<_>>();
                    (
                        next_current_expr,
                        recursive_args[control_carry_index].clone(),
                        tail_stmts,
                    )
                }
                SelfTailRecursiveStep::Branch {
                    condition: inner_condition,
                    then_args,
                    else_args,
                } => {
                    if then_args.len() != function.params.len()
                        || else_args.len() != function.params.len()
                    {
                        return None;
                    }
                    if then_args[0] != else_args[0] {
                        return None;
                    }
                    if then_args[control_carry_index] != else_args[control_carry_index] {
                        return None;
                    }
                    let next_current_expr = then_args[0].clone();
                    branch_condition = canonicalize_tail_recursive_condition_expr(
                        inner_condition,
                        &function.params[0].name,
                        &carry_param_names,
                        &invariant_param_names,
                    );
                    let tail_stmts = function
                        .params
                        .iter()
                        .enumerate()
                        .skip(1)
                        .filter(|(index, _)| *index != control_carry_index)
                        .filter_map(|(index, param)| {
                            let then_value = canonicalize_tail_recursive_loop_arg(
                                &then_args[index],
                                &function.params[0].name,
                                &carry_param_names,
                                &invariant_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            );
                            let else_value = canonicalize_tail_recursive_loop_arg(
                                &else_args[index],
                                &function.params[0].name,
                                &carry_param_names,
                                &invariant_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            );
                            tail_recursive_param_branch_update_stmt(
                                index,
                                param,
                                branch_condition.clone(),
                                then_value,
                                else_value,
                            )
                        })
                        .collect::<Vec<_>>();
                    (
                        next_current_expr,
                        then_args[control_carry_index].clone(),
                        tail_stmts,
                    )
                }
                SelfTailRecursiveStep::FlowBreak { .. } => return None,
                SelfTailRecursiveStep::PostFlowBreak { .. } => return None,
            };
            let mut body = vec![NirStmt::Let {
                name: function.params[0].name.clone(),
                ty: Some(function.params[0].ty.clone()),
                value: next_current_expr.clone(),
            }];
            let control_param = &function.params[control_carry_index];
            let control_value = canonicalize_tail_recursive_loop_arg(
                &control_carry_expr,
                &function.params[0].name,
                &carry_param_names,
                &invariant_param_names,
                Some(&control_param.name),
                &next_current_expr,
            );
            if let Some(stmt) =
                tail_recursive_param_update_stmt(control_carry_index, control_param, control_value)
            {
                body.push(stmt);
            }
            body.extend(tail_stmts);
            body.push(NirStmt::If {
                condition: rewrite_post_flow_break_condition(
                    &condition,
                    &control_carry_expr,
                    &control_param.name,
                    &function.params[0].name,
                    &carry_param_names,
                    &invariant_param_names,
                )?,
                then_body: vec![NirStmt::Break],
                else_body: vec![],
            });
            Some(body)
        }
        SelfTailRecursiveStep::Branch {
            condition,
            then_args,
            else_args,
        } => {
            if then_args.len() != function.params.len() || else_args.len() != function.params.len()
            {
                return None;
            }
            if then_args[0] != else_args[0] {
                return None;
            }
            let branch_condition = canonicalize_tail_recursive_condition_expr(
                &condition,
                &function.params[0].name,
                &carry_param_names,
                &invariant_param_names,
            );
            let next_current_expr = then_args[0].clone();
            let mut body = vec![NirStmt::Let {
                name: function.params[0].name.clone(),
                ty: Some(function.params[0].ty.clone()),
                value: next_current_expr.clone(),
            }];
            for (index, param) in function.params.iter().enumerate().skip(1) {
                let then_value = canonicalize_tail_recursive_loop_arg(
                    &then_args[index],
                    &function.params[0].name,
                    &carry_param_names,
                    &invariant_param_names,
                    Some(&param.name),
                    &next_current_expr,
                );
                let else_value = canonicalize_tail_recursive_loop_arg(
                    &else_args[index],
                    &function.params[0].name,
                    &carry_param_names,
                    &invariant_param_names,
                    Some(&param.name),
                    &next_current_expr,
                );
                if let Some(stmt) = tail_recursive_param_branch_update_stmt(
                    index,
                    param,
                    branch_condition.clone(),
                    then_value,
                    else_value,
                ) {
                    body.push(stmt);
                }
            }
            Some(body)
        }
    }
}

fn tail_recursive_invariant_param_names(
    function: &NirFunction,
    recursive_step: &SelfTailRecursiveStep,
) -> Option<BTreeSet<String>> {
    fn step_args<'a>(
        step: &'a SelfTailRecursiveStep,
    ) -> Option<(&'a [NirExpr], Option<&'a [NirExpr]>)> {
        match step {
            SelfTailRecursiveStep::Linear(args) => Some((args.as_slice(), None)),
            SelfTailRecursiveStep::Branch {
                then_args,
                else_args,
                ..
            } => Some((then_args.as_slice(), Some(else_args.as_slice()))),
            SelfTailRecursiveStep::FlowBreak { recursive_step, .. }
            | SelfTailRecursiveStep::PostFlowBreak { recursive_step, .. } => {
                step_args(recursive_step)
            }
        }
    }

    let (primary_args, alternate_args) = step_args(recursive_step)?;
    if primary_args.len() != function.params.len() {
        return None;
    }
    if let Some(alternate_args) = alternate_args {
        if alternate_args.len() != function.params.len() {
            return None;
        }
    }

    let mut invariants = BTreeSet::new();
    for (index, param) in function.params.iter().enumerate().skip(1) {
        let primary_matches =
            matches!(&primary_args[index], NirExpr::Var(name) if name == &param.name);
        let alternate_matches = alternate_args
            .map(|args| matches!(&args[index], NirExpr::Var(name) if name == &param.name))
            .unwrap_or(true);
        if primary_matches && alternate_matches {
            invariants.insert(param.name.clone());
        }
    }
    Some(invariants)
}

fn tail_recursive_param_update_stmt(
    index: usize,
    param: &NirParam,
    value: NirExpr,
) -> Option<NirStmt> {
    if index != 0 && value == NirExpr::Var(param.name.clone()) {
        return None;
    }
    Some(NirStmt::Let {
        name: param.name.clone(),
        ty: Some(param.ty.clone()),
        value,
    })
}

fn tail_recursive_param_branch_update_stmt(
    index: usize,
    param: &NirParam,
    condition: NirExpr,
    then_value: NirExpr,
    else_value: NirExpr,
) -> Option<NirStmt> {
    fn branch_keep_previous_placeholder(index: usize, param: &NirParam, value: NirExpr) -> NirExpr {
        if index != 0 && value == NirExpr::Var(param.name.clone()) {
            NirExpr::Var(tail_recursive_prev_carry_binding(index - 1))
        } else {
            value
        }
    }

    if index != 0
        && then_value == NirExpr::Var(param.name.clone())
        && else_value == NirExpr::Var(param.name.clone())
    {
        return None;
    }
    let then_value = branch_keep_previous_placeholder(index, param, then_value);
    let else_value = branch_keep_previous_placeholder(index, param, else_value);
    Some(NirStmt::If {
        condition,
        then_body: vec![NirStmt::Let {
            name: param.name.clone(),
            ty: Some(param.ty.clone()),
            value: then_value,
        }],
        else_body: vec![NirStmt::Let {
            name: param.name.clone(),
            ty: Some(param.ty.clone()),
            value: else_value,
        }],
    })
}

fn rewrite_post_flow_break_condition(
    condition: &NirExpr,
    updated_expr: &NirExpr,
    updated_binding: &str,
    current_name: &str,
    non_current_param_names: &[String],
    invariant_param_names: &BTreeSet<String>,
) -> Option<NirExpr> {
    let NirExpr::Binary { op, lhs, rhs } = condition else {
        return None;
    };
    if lhs.as_ref() != updated_expr {
        return None;
    }
    Some(NirExpr::Binary {
        op: *op,
        lhs: Box::new(NirExpr::Var(updated_binding.to_owned())),
        rhs: Box::new(canonicalize_tail_recursive_condition_expr(
            rhs,
            current_name,
            non_current_param_names,
            invariant_param_names,
        )),
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

fn is_self_tail_recursive_loop_shape(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> bool {
    prepare_post_flow_while(
        condition,
        body,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )
    .is_some()
        || prepare_async_post_flow_while(
            condition,
            body,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )
        .is_some()
        || prepare_flow_while(
            condition,
            body,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )
        .is_some()
        || prepare_async_flow_while(
            condition,
            body,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )
        .is_some()
        || prepare_chained_while(
            condition,
            body,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )
        .is_some()
        || prepare_async_chained_while(
            condition,
            body,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )
        .is_some()
        || prepare_counted_while(
            condition,
            body,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )
        .is_some()
}

fn canonicalize_tail_recursive_loop_arg(
    expr: &NirExpr,
    current_name: &str,
    non_current_param_names: &[String],
    invariant_param_names: &BTreeSet<String>,
    target_carry_name: Option<&str>,
    next_current_expr: &NirExpr,
) -> NirExpr {
    if expr == next_current_expr {
        return NirExpr::Var(current_name.to_owned());
    }
    match expr {
        NirExpr::Var(name) if name == current_name => {
            NirExpr::Var(TAIL_RECURSIVE_PREV_CURRENT_BINDING.to_owned())
        }
        NirExpr::Var(name) => {
            if target_carry_name.is_some() && Some(name.as_str()) == target_carry_name {
                expr.clone()
            } else if invariant_param_names.contains(name) {
                expr.clone()
            } else {
                non_current_param_names
                    .iter()
                    .position(|param_name| param_name == name)
                    .map(tail_recursive_prev_carry_binding)
                    .map(NirExpr::Var)
                    .unwrap_or_else(|| expr.clone())
            }
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null => expr.clone(),
        NirExpr::CastI64ToI32(inner) => {
            NirExpr::CastI64ToI32(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI32ToI64(inner) => {
            NirExpr::CastI32ToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI64ToBool(inner) => {
            NirExpr::CastI64ToBool(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastBoolToI64(inner) => {
            NirExpr::CastBoolToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI64ToF32(inner) => {
            NirExpr::CastI64ToF32(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastF32ToI64(inner) => {
            NirExpr::CastF32ToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI64ToF64(inner) => {
            NirExpr::CastI64ToF64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastF64ToI64(inner) => {
            NirExpr::CastF64ToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::Await(inner) => NirExpr::Await(Box::new(canonicalize_tail_recursive_loop_arg(
            inner,
            current_name,
            non_current_param_names,
            invariant_param_names,
            target_carry_name,
            next_current_expr,
        ))),
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_loop_arg(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                        target_carry_name,
                        next_current_expr,
                    )
                })
                .collect(),
        },
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(canonicalize_tail_recursive_loop_arg(
                receiver,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_loop_arg(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                        target_carry_name,
                        next_current_expr,
                    )
                })
                .collect(),
        },
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| {
                    (
                        field.clone(),
                        canonicalize_tail_recursive_loop_arg(
                            value,
                            current_name,
                            non_current_param_names,
                            invariant_param_names,
                            target_carry_name,
                            next_current_expr,
                        ),
                    )
                })
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(canonicalize_tail_recursive_loop_arg(
                base,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            field: field.clone(),
        },
        NirExpr::IsNull(inner) => NirExpr::IsNull(Box::new(canonicalize_tail_recursive_loop_arg(
            inner,
            current_name,
            non_current_param_names,
            invariant_param_names,
            target_carry_name,
            next_current_expr,
        ))),
        NirExpr::LoadValue(inner) => {
            NirExpr::LoadValue(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::LoadNext(inner) => {
            NirExpr::LoadNext(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::BufferLen(inner) => {
            NirExpr::BufferLen(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::LoadAt { buffer, index } => NirExpr::LoadAt {
            buffer: Box::new(canonicalize_tail_recursive_loop_arg(
                buffer,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            index: Box::new(canonicalize_tail_recursive_loop_arg(
                index,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(canonicalize_tail_recursive_loop_arg(
                lhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            rhs: Box::new(canonicalize_tail_recursive_loop_arg(
                rhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
        },
        _ => expr.clone(),
    }
}

fn canonicalize_tail_recursive_condition_expr(
    expr: &NirExpr,
    current_name: &str,
    non_current_param_names: &[String],
    invariant_param_names: &BTreeSet<String>,
) -> NirExpr {
    match expr {
        NirExpr::Var(name) if name == current_name => {
            NirExpr::Var(TAIL_RECURSIVE_PREV_CURRENT_BINDING.to_owned())
        }
        NirExpr::Var(name) => {
            if invariant_param_names.contains(name) {
                expr.clone()
            } else {
                non_current_param_names
                    .iter()
                    .position(|param_name| param_name == name)
                    .map(tail_recursive_prev_carry_binding)
                    .map(NirExpr::Var)
                    .unwrap_or_else(|| expr.clone())
            }
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null => expr.clone(),
        NirExpr::CastI64ToI32(inner) => {
            NirExpr::CastI64ToI32(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI32ToI64(inner) => {
            NirExpr::CastI32ToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI64ToBool(inner) => {
            NirExpr::CastI64ToBool(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastBoolToI64(inner) => {
            NirExpr::CastBoolToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI64ToF32(inner) => {
            NirExpr::CastI64ToF32(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastF32ToI64(inner) => {
            NirExpr::CastF32ToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI64ToF64(inner) => {
            NirExpr::CastI64ToF64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastF64ToI64(inner) => {
            NirExpr::CastF64ToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::Await(inner) => {
            NirExpr::Await(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_condition_expr(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                    )
                })
                .collect(),
        },
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(canonicalize_tail_recursive_condition_expr(
                receiver,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_condition_expr(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                    )
                })
                .collect(),
        },
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| {
                    (
                        field.clone(),
                        canonicalize_tail_recursive_condition_expr(
                            value,
                            current_name,
                            non_current_param_names,
                            invariant_param_names,
                        ),
                    )
                })
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(canonicalize_tail_recursive_condition_expr(
                base,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            field: field.clone(),
        },
        NirExpr::IsNull(inner) => {
            NirExpr::IsNull(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::LoadValue(inner) => {
            NirExpr::LoadValue(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::LoadNext(inner) => {
            NirExpr::LoadNext(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::BufferLen(inner) => {
            NirExpr::BufferLen(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::LoadAt { buffer, index } => NirExpr::LoadAt {
            buffer: Box::new(canonicalize_tail_recursive_condition_expr(
                buffer,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            index: Box::new(canonicalize_tail_recursive_condition_expr(
                index,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(canonicalize_tail_recursive_condition_expr(
                lhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            rhs: Box::new(canonicalize_tail_recursive_condition_expr(
                rhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
        },
        _ => expr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parse_nuis_module;

    #[test]
    fn rewrites_async_nested_post_flow_branching_tail_recursion_into_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
                if current == 0 {
                  return acc;
                }
                if acc + current > 6 {
                  return acc + current;
                }
                if current > 3 {
                  return await sum_until(current - 1, acc + current, flag + current);
                } else {
                  if current > 1 {
                    return await sum_until(current - 1, acc + current, flag + current);
                  } else {
                    return await sum_until(current - 1, acc + current, flag + 0);
                  }
                }
              }

              async fn main() -> i64 {
                return await sum_until(5, 0, 0);
              }
            }
            "#,
        )
        .unwrap();

        let pure_helpers = collect_pure_helper_functions(&module);
        let inlineable_pure_helpers = collect_inlineable_pure_helper_exprs(&module);
        let pure_helper_blocks = collect_pure_helper_blocks(&module);
        let original = module
            .functions
            .iter()
            .find(|function| function.name == "sum_until")
            .expect("expected sum_until");
        let (recurse_condition, base_return, recursive_step) =
            extract_self_tail_recursive_shape(original, &pure_helpers)
                .expect("expected recursive shape to be recognized");
        let loop_body = rewrite_self_tail_recursive_loop_body(original, recursive_step)
            .expect("expected recursive loop body rewrite");
        assert!(
            is_self_tail_recursive_loop_shape(
                &recurse_condition,
                &loop_body,
                &pure_helpers,
                &inlineable_pure_helpers,
                &pure_helper_blocks,
            ),
            "expected rewritten loop body to satisfy self-tail-recursive loop shape; base_return={base_return:?}, body={loop_body:?}"
        );

        let rewritten = rewrite_self_tail_recursive_functions(&module);
        let sum_until = rewritten
            .functions
            .iter()
            .find(|function| function.name == "sum_until")
            .expect("expected sum_until");
        assert!(
            matches!(sum_until.body.first(), Some(NirStmt::While { .. })),
            "expected self tail recursion rewrite to produce a while loop, got {:?}",
            sum_until.body
        );
    }
}
