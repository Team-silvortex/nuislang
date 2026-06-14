use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstDestructureBinding, AstDestructureField, AstExpr, AstFunction,
    AstGenericParam, AstMatchArm, AstModule, AstParam, AstStmt, AstTypeRef, AstUnaryOp,
    AstVisibility,
};

use super::lambda_validation::collect_lambda_block_captures;

const LAMBDA_BIND_PREFIX: &str = "__lambda_bind.";

#[derive(Debug, Clone, PartialEq, Eq)]
struct LambdaBinding {
    symbol: String,
    captured_locals: Vec<String>,
}

pub(super) fn expand_module_lambdas(module: &AstModule) -> Result<AstModule, String> {
    let module_const_names = module
        .consts
        .iter()
        .map(|constant| constant.name.clone())
        .collect::<BTreeSet<_>>();
    let module_function_table = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut expanded = module.clone();
    expanded.functions.clear();
    for function in &module.functions {
        let (rewritten, synthesized) =
            expand_function_lambdas(function, &module_const_names, &module_function_table)?;
        expanded.functions.extend(synthesized);
        expanded.functions.push(rewritten);
    }
    Ok(expanded)
}

fn expand_function_lambdas(
    function: &AstFunction,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
) -> Result<(AstFunction, Vec<AstFunction>), String> {
    let mut counter = 0usize;
    let mut synthesized = Vec::new();
    let visible_locals = function
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let visible_local_types = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    let body = expand_lambda_block(
        &function.body,
        &function.generic_params,
        &BTreeMap::new(),
        &visible_locals,
        &visible_local_types,
        module_const_names,
        module_function_table,
        &function.name,
        &mut counter,
        &mut synthesized,
    )?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok((rewritten, synthesized))
}

fn callable_type_arity(ty: &AstTypeRef) -> Option<usize> {
    if ty.is_optional || ty.is_ref {
        return None;
    }
    match ty.name.as_str() {
        "Fn1" if ty.generic_args.len() == 2 => Some(1),
        "Fn2" if ty.generic_args.len() == 3 => Some(2),
        "Fn3" if ty.generic_args.len() == 4 => Some(3),
        _ => None,
    }
}

fn callable_type_from_signature(params: &[AstParam], return_type: &AstTypeRef) -> Option<AstTypeRef> {
    let name = match params.len() {
        1 => "Fn1",
        2 => "Fn2",
        3 => "Fn3",
        _ => return None,
    };
    let mut generic_args = params.iter().map(|param| param.ty.clone()).collect::<Vec<_>>();
    generic_args.push(return_type.clone());
    Some(AstTypeRef {
        name: name.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    })
}

fn callable_type_from_function(function: &AstFunction) -> Option<AstTypeRef> {
    let return_type = function.return_type.as_ref()?;
    callable_type_from_signature(&function.params, return_type)
}

fn callable_type_matches_signature(
    params: &[AstParam],
    return_type: &AstTypeRef,
    declared_type: &AstTypeRef,
) -> bool {
    let Some(arity) = callable_type_arity(declared_type) else {
        return false;
    };
    if params.len() != arity {
        return false;
    }
    params
        .iter()
        .zip(declared_type.generic_args[..arity].iter())
        .all(|(param, expected)| param.ty == *expected)
        && declared_type.generic_args[arity] == *return_type
}

fn callable_binding_return_type(
    binding_name: &str,
    params: &[AstParam],
    declared_type: Option<&AstTypeRef>,
    explicit_return_type: &Option<AstTypeRef>,
) -> Result<Option<AstTypeRef>, String> {
    match declared_type {
        Some(ty) => {
            let Some(arity) = callable_type_arity(ty) else {
                return Err(format!(
                    "lambda binding `{binding_name}` expects a callable type annotation like `Fn1<...>`/`Fn2<...>`/`Fn3<...>`"
                ));
            };
            if params.len() != arity {
                return Err(format!(
                    "lambda binding `{binding_name}` parameter list does not match declared callable type `{}`",
                    ty.name
                ));
            }
            for (param, expected) in params.iter().zip(ty.generic_args[..arity].iter()) {
                if param.ty != *expected {
                    return Err(format!(
                        "lambda binding `{binding_name}` parameter `{}` type `{}` does not match declared callable parameter type `{}`",
                        param.name, param.ty.name, expected.name
                    ));
                }
            }
            let inferred_return_type = ty.generic_args[arity].clone();
            if let Some(explicit_return_type) = explicit_return_type {
                if *explicit_return_type != inferred_return_type {
                    return Err(format!(
                        "lambda binding `{binding_name}` return type `{}` does not match declared callable return type `{}`",
                        explicit_return_type.name, inferred_return_type.name
                    ));
                }
            }
            Ok(Some(inferred_return_type))
        }
        None => Ok(explicit_return_type.clone()),
    }
}

fn infer_local_binding_type(
    value: &AstExpr,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
) -> Option<AstTypeRef> {
    match value {
        AstExpr::Bool(_) => Some(named_type("bool")),
        AstExpr::Text(_) => Some(named_type("String")),
        AstExpr::Int(_) => Some(named_type("i64")),
        AstExpr::Float(_) => Some(named_type("f64")),
        AstExpr::Var(name) => visible_local_types
            .get(name)
            .cloned()
            .or_else(|| module_function_table.get(name).and_then(callable_type_from_function)),
        AstExpr::StructLiteral {
            type_name,
            type_args,
            ..
        } => Some(AstTypeRef {
            name: type_name.clone(),
            generic_args: type_args.clone(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Call {
            callee,
            generic_args,
            ..
        } => module_function_table.get(callee).and_then(|function| {
            function.return_type.as_ref().map(|return_type| {
                if generic_args.is_empty() {
                    return_type.clone()
                } else {
                    AstTypeRef {
                        name: return_type.name.clone(),
                        generic_args: return_type.generic_args.clone(),
                        is_optional: return_type.is_optional,
                        is_ref: return_type.is_ref,
                    }
                }
            })
        }),
        AstExpr::Unary { op, operand } => match op {
            AstUnaryOp::Not => Some(named_type("bool")),
            AstUnaryOp::Neg => infer_local_binding_type(
                operand,
                visible_local_types,
                module_function_table,
            ),
            AstUnaryOp::Deref => None,
        },
        AstExpr::Binary { op, lhs, rhs } => {
            let lhs_ty = infer_local_binding_type(lhs, visible_local_types, module_function_table)?;
            let rhs_ty = infer_local_binding_type(rhs, visible_local_types, module_function_table)?;
            match op {
                AstBinaryOp::And
                | AstBinaryOp::Or
                | AstBinaryOp::Eq
                | AstBinaryOp::Ne
                | AstBinaryOp::Lt
                | AstBinaryOp::Le
                | AstBinaryOp::Gt
                | AstBinaryOp::Ge => Some(named_type("bool")),
                AstBinaryOp::Add
                | AstBinaryOp::Sub
                | AstBinaryOp::Mul
                | AstBinaryOp::Div
                | AstBinaryOp::Rem if lhs_ty == rhs_ty => Some(lhs_ty),
                _ => None,
            }
        }
        _ => None,
    }
}

fn named_type(name: &str) -> AstTypeRef {
    AstTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: false,
    }
}

fn build_lambda_call(binding: &LambdaBinding, args: Vec<AstExpr>) -> AstExpr {
    let mut final_args = args;
    final_args.extend(
        binding
            .captured_locals
            .iter()
            .cloned()
            .map(AstExpr::Var),
    );
    AstExpr::Call {
        callee: binding.symbol.clone(),
        generic_args: Vec::new(),
        args: final_args,
    }
}

fn build_lambda_binding_value(binding: &LambdaBinding) -> AstExpr {
    if binding.captured_locals.is_empty() {
        AstExpr::Var(binding.symbol.clone())
    } else {
        AstExpr::Call {
            callee: format!("{LAMBDA_BIND_PREFIX}{}", binding.symbol),
            generic_args: Vec::new(),
            args: binding
                .captured_locals
                .iter()
                .cloned()
                .map(AstExpr::Var)
                .collect(),
        }
    }
}

fn synthesize_lambda_function(
    params: &[AstParam],
    return_type: &Option<AstTypeRef>,
    body: &[AstStmt],
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    outer_locals: &BTreeSet<String>,
    outer_local_types: &BTreeMap<String, AstTypeRef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<LambdaBinding, String> {
    let Some(lambda_return_type) = return_type.clone() else {
        return Err("inline lambda currently requires an explicit return type".to_owned());
    };
    let mut lambda_locals = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut captures = BTreeSet::new();
    collect_lambda_block_captures(body, &mut lambda_locals, outer_locals, &mut captures)?;
    let capture_params = captures
        .iter()
        .map(|capture| {
            let capture_ty = outer_local_types.get(capture).cloned().ok_or_else(|| {
                format!(
                    "captured local `{capture}` currently requires an explicit type annotation before it can be used in a lambda"
                )
            })?;
            Ok(AstParam {
                name: capture.clone(),
                ty: capture_ty,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let synthesized_name = format!("__lambda_{}_{}", owning_function_name, *counter);
    *counter += 1;

    let mut lambda_visible_locals = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut lambda_visible_local_types = params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    for capture in &capture_params {
        lambda_visible_locals.insert(capture.name.clone());
        lambda_visible_local_types.insert(capture.name.clone(), capture.ty.clone());
    }

    let lambda_body = expand_lambda_block(
        body,
        inherited_generic_params,
        lambda_aliases,
        &lambda_visible_locals,
        &lambda_visible_local_types,
        module_const_names,
        module_function_table,
        owning_function_name,
        counter,
        synthesized,
    )?;
    let mut synthesized_params = params.to_vec();
    synthesized_params.extend(capture_params.clone());
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
        params: synthesized_params,
        return_type: Some(lambda_return_type),
        body: lambda_body,
    });
    Ok(LambdaBinding {
        symbol: synthesized_name,
        captured_locals: capture_params.into_iter().map(|param| param.name).collect(),
    })
}

#[allow(clippy::too_many_arguments)]
fn expand_lambda_block(
    body: &[AstStmt],
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    visible_locals: &BTreeSet<String>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<Vec<AstStmt>, String> {
    let mut aliases = lambda_aliases.clone();
    let mut locals = visible_locals.clone();
    let mut local_types = visible_local_types.clone();
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
                let effective_return_type =
                    callable_binding_return_type(name, params, ty.as_ref(), return_type)?;
                let binding = synthesize_lambda_function(
                    params,
                    &effective_return_type,
                    body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_const_names,
                    module_function_table,
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
                aliases.insert(name.clone(), binding);
                locals.insert(name.clone());
                if let Some(return_type) = effective_return_type.as_ref() {
                    if let Some(binding_ty) = callable_type_from_signature(params, return_type) {
                        local_types.insert(name.clone(), binding_ty);
                    }
                }
            }
            AstStmt::Let {
                name,
                ty: Some(ty),
                value: AstExpr::Var(value_name),
                mutable: _,
            } if !module_const_names.contains(value_name)
                && module_function_table.contains_key(value_name) =>
            {
                let function = module_function_table
                    .get(value_name)
                    .expect("checked function table presence");
                let Some(return_type) = function.return_type.as_ref() else {
                    return Err(format!(
                        "callable binding `{name}` target `{value_name}` requires an explicit return type"
                    ));
                };
                if !callable_type_matches_signature(&function.params, return_type, ty) {
                    return Err(format!(
                        "callable binding `{name}` target `{value_name}` does not match declared callable type `{}`",
                        ty.name
                    ));
                }
                aliases.insert(
                    name.clone(),
                    LambdaBinding {
                        symbol: value_name.clone(),
                        captured_locals: Vec::new(),
                    },
                );
                locals.insert(name.clone());
                local_types.insert(name.clone(), ty.clone());
            }
            AstStmt::Let {
                name,
                ty: None,
                value: AstExpr::Var(value_name),
                mutable: _,
            } if !module_const_names.contains(value_name)
                && module_function_table.contains_key(value_name) =>
            {
                aliases.insert(
                    name.clone(),
                    LambdaBinding {
                        symbol: value_name.clone(),
                        captured_locals: Vec::new(),
                    },
                );
                locals.insert(name.clone());
                if let Some(binding_ty) = module_function_table
                    .get(value_name)
                    .and_then(callable_type_from_function)
                {
                    local_types.insert(name.clone(), binding_ty);
                }
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
                    &local_types,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::Let {
                    mutable: *mutable,
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value.clone(),
                });
                aliases.remove(name);
                locals.insert(name.clone());
                if let Some(ty) = ty.clone() {
                    local_types.insert(name.clone(), ty);
                } else if let Some(inferred_ty) = infer_local_binding_type(
                    &rewritten_value,
                    &local_types,
                    module_function_table,
                ) {
                    local_types.insert(name.clone(), inferred_ty);
                }
            }
            AstStmt::AssignLocal { name, value } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::AssignLocal {
                    name: name.clone(),
                    value: rewritten_value.clone(),
                });
                if let Some(inferred_ty) = infer_local_binding_type(
                    &rewritten_value,
                    &local_types,
                    module_function_table,
                ) {
                    local_types.insert(name.clone(), inferred_ty);
                }
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
                    &local_types,
                    module_const_names,
                    module_function_table,
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
                    locals.insert(name.clone());
                    local_types.remove(&name);
                }
            }
            AstStmt::Const { name, ty, value } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value.clone(),
                });
                aliases.remove(name);
                locals.insert(name.clone());
                if let Some(ty) = ty.clone() {
                    local_types.insert(name.clone(), ty);
                } else if let Some(inferred_ty) = infer_local_binding_type(
                    &rewritten_value,
                    &local_types,
                    module_function_table,
                ) {
                    local_types.insert(name.clone(), inferred_ty);
                }
            }
            AstStmt::Print(value) => rewritten.push(AstStmt::Print(rewrite_lambda_expr(
                value,
                inherited_generic_params,
                &aliases,
                &locals,
                &local_types,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::Await(value) => rewritten.push(AstStmt::Await(rewrite_lambda_expr(
                value,
                inherited_generic_params,
                &aliases,
                &locals,
                &local_types,
                module_const_names,
                module_function_table,
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
                    &local_types,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                then_body: expand_lambda_block(
                    then_body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                else_body: expand_lambda_block(
                    else_body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_const_names,
                    module_function_table,
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
                    &local_types,
                    module_const_names,
                    module_function_table,
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
                                        &local_types,
                                        module_const_names,
                                        module_function_table,
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
                                &local_types,
                                module_const_names,
                                module_function_table,
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
                    &local_types,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                body: expand_lambda_block(
                    body,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_const_names,
                    module_function_table,
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
                &local_types,
                module_const_names,
                module_function_table,
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
                    &local_types,
                    module_const_names,
                    module_function_table,
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

#[allow(clippy::too_many_arguments)]
fn rewrite_lambda_expr(
    expr: &AstExpr,
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    visible_locals: &BTreeSet<String>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::Var(name)
            if lambda_aliases.contains_key(name) && !module_const_names.contains(name) =>
        {
            let binding = lambda_aliases
                .get(name)
                .cloned()
                .expect("checked lambda alias presence");
            build_lambda_binding_value(&binding)
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
                visible_local_types,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            then_body: expand_lambda_block(
                then_body,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?,
            else_body: expand_lambda_block(
                else_body,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_const_names,
                module_function_table,
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
                visible_local_types,
                module_const_names,
                module_function_table,
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
                                visible_local_types,
                                module_const_names,
                                module_function_table,
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
                            visible_local_types,
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => {
            let binding = synthesize_lambda_function(
                params,
                return_type,
                body,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?;
            build_lambda_binding_value(&binding)
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_lambda_expr(
            value,
            inherited_generic_params,
            lambda_aliases,
            visible_locals,
            visible_local_types,
            module_const_names,
            module_function_table,
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
                        visible_local_types,
                        module_const_names,
                        module_function_table,
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
                    let binding = synthesize_lambda_function(
                        params,
                        return_type,
                        body,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        visible_local_types,
                        module_const_names,
                        module_function_table,
                        owning_function_name,
                        counter,
                        synthesized,
                    )?;
                    build_lambda_call(&binding, rewritten_args)
                }
                AstExpr::Var(name) => {
                    if let Some(binding) = lambda_aliases.get(name) {
                        build_lambda_call(binding, rewritten_args)
                    } else {
                        AstExpr::Call {
                            callee: name.clone(),
                            generic_args: Vec::new(),
                            args: rewritten_args,
                        }
                    }
                }
                _ => {
                    return Err(
                        "only immediate lambda invocation and named function or lambda binding invocation are supported in the current MVP"
                            .to_owned(),
                    )
                }
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            let rewritten_args = args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        visible_local_types,
                        module_const_names,
                        module_function_table,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(binding) = lambda_aliases.get(callee) {
                build_lambda_call(binding, rewritten_args)
            } else {
                AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                }
            }
        }
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
                visible_local_types,
                module_const_names,
                module_function_table,
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
                        visible_local_types,
                        module_const_names,
                        module_function_table,
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
                            visible_local_types,
                            module_const_names,
                            module_function_table,
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
                visible_local_types,
                module_const_names,
                module_function_table,
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
                visible_local_types,
                module_const_names,
                module_function_table,
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
                visible_local_types,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            rhs: Box::new(rewrite_lambda_expr(
                rhs,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_const_names,
                module_function_table,
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
