use super::tail_recursion_canonical::{
    canonicalize_tail_recursive_condition_expr, canonicalize_tail_recursive_loop_arg,
};
use super::*;

pub(super) fn rewrite_self_tail_recursive_loop_body(
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

pub(super) fn is_self_tail_recursive_loop_shape(
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
