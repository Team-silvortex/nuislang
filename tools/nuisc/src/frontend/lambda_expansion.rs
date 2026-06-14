use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstDestructureBinding, AstDestructureField, AstExpr, AstFunction, AstGenericParam, AstMatchArm,
    AstModule, AstParam, AstStmt, AstTypeRef, AstVisibility,
};

use super::lambda_validation::validate_lambda_block_no_capture;

pub(super) fn expand_module_lambdas(module: &AstModule) -> Result<AstModule, String> {
    let module_const_names = module
        .consts
        .iter()
        .map(|constant| constant.name.clone())
        .collect::<BTreeSet<_>>();
    let mut expanded = module.clone();
    expanded.functions.clear();
    for function in &module.functions {
        let (rewritten, synthesized) = expand_function_lambdas(function, &module_const_names)?;
        expanded.functions.extend(synthesized);
        expanded.functions.push(rewritten);
    }
    Ok(expanded)
}

fn expand_function_lambdas(
    function: &AstFunction,
    module_const_names: &BTreeSet<String>,
) -> Result<(AstFunction, Vec<AstFunction>), String> {
    let mut counter = 0usize;
    let mut synthesized = Vec::new();
    let visible_locals = function
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let body = expand_lambda_block(
        &function.body,
        &function.generic_params,
        &BTreeMap::new(),
        &visible_locals,
        module_const_names,
        &function.name,
        &mut counter,
        &mut synthesized,
    )?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok((rewritten, synthesized))
}

fn synthesize_lambda_function(
    params: &[AstParam],
    return_type: &Option<AstTypeRef>,
    body: &[AstStmt],
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, String>,
    outer_locals: &BTreeSet<String>,
    module_const_names: &BTreeSet<String>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<String, String> {
    let Some(lambda_return_type) = return_type.clone() else {
        return Err("inline lambda currently requires an explicit return type".to_owned());
    };
    let mut lambda_locals = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    validate_lambda_block_no_capture(body, &mut lambda_locals, outer_locals)?;
    let synthesized_name = format!("__lambda_{}_{}", owning_function_name, *counter);
    *counter += 1;
    let lambda_body = expand_lambda_block(
        body,
        inherited_generic_params,
        lambda_aliases,
        &params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>(),
        module_const_names,
        owning_function_name,
        counter,
        synthesized,
    )?;
    synthesized.push(AstFunction {
        visibility: AstVisibility::Private,
        name: synthesized_name.clone(),
        attributes: Vec::new(),
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        is_async: false,
        generic_params: inherited_generic_params.to_vec(),
        params: params.to_vec(),
        return_type: Some(lambda_return_type),
        body: lambda_body,
    });
    Ok(synthesized_name)
}

#[allow(clippy::too_many_arguments)]
fn expand_lambda_block(
    body: &[AstStmt],
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, String>,
    visible_locals: &BTreeSet<String>,
    module_const_names: &BTreeSet<String>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<Vec<AstStmt>, String> {
    let mut aliases = lambda_aliases.clone();
    let mut locals = visible_locals.clone();
    let mut rewritten = Vec::new();
    for stmt in body {
        match stmt {
            AstStmt::Let {
                name,
                ty,
                mutable: _,
                value:
                    AstExpr::Lambda {
                        params,
                        return_type,
                        body,
                    },
            } => {
                if ty.is_some() {
                    return Err(format!(
                        "lambda binding `{name}` currently does not support an explicit type annotation"
                    ));
                }
                let synthesized_name = synthesize_lambda_function(
                    params,
                    return_type,
                    body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )
                .map_err(|error| {
                    if error == "inline lambda currently requires an explicit return type" {
                        format!(
                            "lambda binding `{name}` currently requires an explicit return type"
                        )
                    } else {
                        error
                    }
                })?;
                aliases.insert(name.clone(), synthesized_name);
                locals.insert(name.clone());
            }
            AstStmt::Let {
                name,
                ty,
                value,
                mutable,
            } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::Let {
                    mutable: *mutable,
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value,
                });
                aliases.remove(name);
                locals.insert(name.clone());
            }
            AstStmt::AssignLocal { name, value } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::AssignLocal {
                    name: name.clone(),
                    value: rewritten_value,
                });
            }
            AstStmt::DestructureLet {
                type_ref,
                fields,
                value,
            } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::DestructureLet {
                    type_ref: type_ref.clone(),
                    fields: fields.clone(),
                    value: rewritten_value,
                });
                let mut names = Vec::new();
                collect_destructure_binding_names(fields, &mut names);
                for name in names {
                    aliases.remove(&name);
                    locals.insert(name);
                }
            }
            AstStmt::Const { name, ty, value } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value,
                });
                aliases.remove(name);
                locals.insert(name.clone());
            }
            AstStmt::Print(value) => rewritten.push(AstStmt::Print(rewrite_lambda_expr(
                value,
                inherited_generic_params,
                &aliases,
                &locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::Await(value) => rewritten.push(AstStmt::Await(rewrite_lambda_expr(
                value,
                inherited_generic_params,
                &aliases,
                &locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => rewritten.push(AstStmt::If {
                condition: rewrite_lambda_expr(
                    condition,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                then_body: expand_lambda_block(
                    then_body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                else_body: expand_lambda_block(
                    else_body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
            }),
            AstStmt::Match { value, arms } => rewritten.push(AstStmt::Match {
                value: rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                arms: arms
                    .iter()
                    .map(|arm| {
                        Ok(AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm
                                .guard
                                .clone()
                                .map(|guard| {
                                    rewrite_lambda_expr(
                                        &guard,
                                        inherited_generic_params,
                                        &aliases,
                                        &locals,
                                        module_const_names,
                                        owning_function_name,
                                        counter,
                                        synthesized,
                                    )
                                })
                                .transpose()?,
                            body: expand_lambda_block(
                                &arm.body,
                                inherited_generic_params,
                                &aliases,
                                &locals,
                                module_const_names,
                                owning_function_name,
                                counter,
                                synthesized,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }),
            AstStmt::While { condition, body } => rewritten.push(AstStmt::While {
                condition: rewrite_lambda_expr(
                    condition,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                body: expand_lambda_block(
                    body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
            }),
            AstStmt::Expr(expr) => rewritten.push(AstStmt::Expr(rewrite_lambda_expr(
                expr,
                inherited_generic_params,
                &aliases,
                &locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::Return(value) => rewritten.push(AstStmt::Return(match value {
                Some(value) => Some(rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?),
                None => None,
            })),
            AstStmt::Break => rewritten.push(AstStmt::Break),
            AstStmt::Continue => rewritten.push(AstStmt::Continue),
        }
    }
    Ok(rewritten)
}

fn collect_destructure_binding_names(fields: &[AstDestructureField], names: &mut Vec<String>) {
    for field in fields {
        match &field.binding {
            AstDestructureBinding::Bind(name) => names.push(name.clone()),
            AstDestructureBinding::Ignore => {}
            AstDestructureBinding::Nested { fields, .. } => {
                collect_destructure_binding_names(fields, names)
            }
        }
    }
}

fn rewrite_lambda_expr(
    expr: &AstExpr,
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, String>,
    visible_locals: &BTreeSet<String>,
    module_const_names: &BTreeSet<String>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::Var(name)
            if lambda_aliases.contains_key(name) && !module_const_names.contains(name) =>
        {
            AstExpr::Var(
                lambda_aliases
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone()),
            )
        }
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(rewrite_lambda_expr(
                condition,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            then_body: expand_lambda_block(
                then_body,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?,
            else_body: expand_lambda_block(
                else_body,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(rewrite_lambda_expr(
                value,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: match &arm.guard {
                            Some(guard) => Some(rewrite_lambda_expr(
                                guard,
                                inherited_generic_params,
                                lambda_aliases,
                                visible_locals,
                                module_const_names,
                                owning_function_name,
                                counter,
                                synthesized,
                            )?),
                            None => None,
                        },
                        body: expand_lambda_block(
                            &arm.body,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            module_const_names,
                            owning_function_name,
                            counter,
                            synthesized,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Lambda { .. } => {
            let AstExpr::Lambda {
                params,
                return_type,
                body,
            } = expr
            else {
                unreachable!();
            };
            let synthesized_name = synthesize_lambda_function(
                params,
                return_type,
                body,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?;
            AstExpr::Var(synthesized_name)
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_lambda_expr(
            value,
            inherited_generic_params,
            lambda_aliases,
            visible_locals,
            module_const_names,
            owning_function_name,
            counter,
            synthesized,
        )?)),
        AstExpr::Invoke { callee, args } => {
            let rewritten_args = args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            match callee.as_ref() {
                AstExpr::Lambda {
                    params,
                    return_type,
                    body,
                } => {
                    let synthesized_name = synthesize_lambda_function(
                        params,
                        return_type,
                        body,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )?;
                    AstExpr::Call {
                        callee: synthesized_name,
                        generic_args: Vec::new(),
                        args: rewritten_args,
                    }
                }
                AstExpr::Var(name) => AstExpr::Call {
                    callee: lambda_aliases
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone()),
                    generic_args: Vec::new(),
                    args: rewritten_args,
                },
                _ => {
                    return Err(
                        "only immediate no-capture lambda invocation and named function invocation are supported in the current MVP"
                            .to_owned(),
                    )
                }
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => AstExpr::Call {
            callee: lambda_aliases
                .get(callee)
                .cloned()
                .unwrap_or_else(|| callee.clone()),
            generic_args: generic_args.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(rewrite_lambda_expr(
                receiver,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        rewrite_lambda_expr(
                            value,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            module_const_names,
                            owning_function_name,
                            counter,
                            synthesized,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_lambda_expr(
                base,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            field: field.clone(),
        },
        AstExpr::Unary { op, operand } => AstExpr::Unary {
            op: *op,
            operand: Box::new(rewrite_lambda_expr(
                operand,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_lambda_expr(
                lhs,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            rhs: Box::new(rewrite_lambda_expr(
                rhs,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
        },
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => expr.clone(),
    })
}
