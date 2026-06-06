use super::*;
use crate::lowering::loop_carries::tail_recursive_prev_carry_binding;

pub(super) fn rewrite_self_tail_recursive_functions(module: &NirModule) -> NirModule {
    let pure_helpers = collect_pure_helper_functions(module);
    let mut rewritten = module.clone();
    for function in &mut rewritten.functions {
        if let Some(body) = rewrite_self_tail_recursive_function(function, &pure_helpers) {
            function.body = body;
        }
    }
    rewritten
}

fn rewrite_self_tail_recursive_function(
    function: &NirFunction,
    pure_helpers: &BTreeSet<String>,
) -> Option<Vec<NirStmt>> {
    if function.is_async || function.params.is_empty() {
        return None;
    }

    let (recurse_condition, base_return, recursive_step) =
        extract_self_tail_recursive_shape(function)?;
    let loop_body = rewrite_self_tail_recursive_loop_body(function, recursive_step)?;

    if !is_self_tail_recursive_loop_shape(&recurse_condition, &loop_body, pure_helpers) {
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
    Branch {
        condition: NirExpr,
        then_args: Vec<NirExpr>,
        else_args: Vec<NirExpr>,
    },
}

fn extract_self_tail_recursive_shape(
    function: &NirFunction,
) -> Option<(NirExpr, NirExpr, SelfTailRecursiveStep)> {
    match function.body.as_slice() {
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }, NirStmt::Return(Some(recursive_return))]
            if else_body.is_empty() =>
        {
            let base_return = extract_terminal_return_expr(then_body)?;
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
            if let Some(base_return) = extract_terminal_return_expr(then_body) {
                let recursive_return = extract_terminal_return_expr(else_body)?;
                let recursive_args = extract_self_tail_recursive_call(function, &recursive_return)?;
                return Some((
                    invert_self_tail_recursive_condition(condition, &function.params[0].name)?,
                    base_return,
                    SelfTailRecursiveStep::Linear(recursive_args),
                ));
            }
            let recursive_return = extract_terminal_return_expr(then_body)?;
            let recursive_args = extract_self_tail_recursive_call(function, &recursive_return)?;
            let base_return = extract_terminal_return_expr(else_body)?;
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
            let base_return = extract_terminal_return_expr(base_then)?;
            let recursive_step =
                extract_self_tail_recursive_branch_step(function, recursive_branch)?;
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
) -> Option<SelfTailRecursiveStep> {
    match stmt {
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let then_return = extract_terminal_return_expr(then_body)?;
            let then_args = extract_self_tail_recursive_call(function, &then_return)?;
            let else_return = extract_terminal_return_expr(else_body)?;
            let else_args = extract_self_tail_recursive_call(function, &else_return)?;
            Some(SelfTailRecursiveStep::Branch {
                condition: condition.clone(),
                then_args,
                else_args,
            })
        }
        NirStmt::Return(Some(expr)) => Some(SelfTailRecursiveStep::Linear(
            extract_self_tail_recursive_call(function, expr)?,
        )),
        _ => None,
    }
}

fn rewrite_self_tail_recursive_loop_body(
    function: &NirFunction,
    recursive_step: SelfTailRecursiveStep,
) -> Option<Vec<NirStmt>> {
    let non_current_param_names = function
        .params
        .iter()
        .skip(1)
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
                    .map(|(index, (param, arg))| {
                        let value = if index == 0 {
                            arg.clone()
                        } else {
                            canonicalize_tail_recursive_loop_arg(
                                arg,
                                &function.params[0].name,
                                &non_current_param_names,
                                Some(&param.name),
                                &next_current_expr,
                            )
                        };
                        NirStmt::Let {
                            name: param.name.clone(),
                            ty: Some(param.ty.clone()),
                            value,
                        }
                    })
                    .collect(),
            )
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
                &non_current_param_names,
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
                    &non_current_param_names,
                    Some(&param.name),
                    &next_current_expr,
                );
                let else_value = canonicalize_tail_recursive_loop_arg(
                    &else_args[index],
                    &function.params[0].name,
                    &non_current_param_names,
                    Some(&param.name),
                    &next_current_expr,
                );
                body.push(NirStmt::If {
                    condition: branch_condition.clone(),
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
                });
            }
            Some(body)
        }
    }
}

fn extract_terminal_return_expr(stmts: &[NirStmt]) -> Option<NirExpr> {
    match stmts {
        [NirStmt::Return(Some(expr))] => Some(expr.clone()),
        _ => None,
    }
}

fn extract_self_tail_recursive_call(
    function: &NirFunction,
    expr: &NirExpr,
) -> Option<Vec<NirExpr>> {
    match expr {
        NirExpr::Call { callee, args } if callee == &function.name => Some(args.clone()),
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
) -> bool {
    prepare_post_flow_while(condition, body, pure_helpers).is_some()
        || prepare_flow_while(condition, body, pure_helpers).is_some()
        || prepare_chained_while(condition, body, pure_helpers).is_some()
        || prepare_counted_while(condition, body, pure_helpers).is_some()
}

fn canonicalize_tail_recursive_loop_arg(
    expr: &NirExpr,
    current_name: &str,
    non_current_param_names: &[String],
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
            } else {
                non_current_param_names
                    .iter()
                    .position(|param_name| param_name == name)
                    .map(tail_recursive_prev_carry_binding)
                    .map(NirExpr::Var)
                    .unwrap_or_else(|| expr.clone())
            }
        }
        NirExpr::Bool(_) | NirExpr::Text(_) | NirExpr::Int(_) | NirExpr::Null => expr.clone(),
        NirExpr::CastI64ToI32(inner) => {
            NirExpr::CastI64ToI32(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::Await(inner) => NirExpr::Await(Box::new(canonicalize_tail_recursive_loop_arg(
            inner,
            current_name,
            non_current_param_names,
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
                target_carry_name,
                next_current_expr,
            )),
            field: field.clone(),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(canonicalize_tail_recursive_loop_arg(
                lhs,
                current_name,
                non_current_param_names,
                target_carry_name,
                next_current_expr,
            )),
            rhs: Box::new(canonicalize_tail_recursive_loop_arg(
                rhs,
                current_name,
                non_current_param_names,
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
) -> NirExpr {
    match expr {
        NirExpr::Var(name) if name == current_name => {
            NirExpr::Var(TAIL_RECURSIVE_PREV_CURRENT_BINDING.to_owned())
        }
        NirExpr::Var(name) => non_current_param_names
            .iter()
            .position(|param_name| param_name == name)
            .map(tail_recursive_prev_carry_binding)
            .map(NirExpr::Var)
            .unwrap_or_else(|| expr.clone()),
        NirExpr::Bool(_) | NirExpr::Text(_) | NirExpr::Int(_) | NirExpr::Null => expr.clone(),
        NirExpr::CastI64ToI32(inner) => {
            NirExpr::CastI64ToI32(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
            )))
        }
        NirExpr::Await(inner) => {
            NirExpr::Await(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
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
            )),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_condition_expr(
                        arg,
                        current_name,
                        non_current_param_names,
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
            )),
            field: field.clone(),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(canonicalize_tail_recursive_condition_expr(
                lhs,
                current_name,
                non_current_param_names,
            )),
            rhs: Box::new(canonicalize_tail_recursive_condition_expr(
                rhs,
                current_name,
                non_current_param_names,
            )),
        },
        _ => expr.clone(),
    }
}
