use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstDestructureBinding, AstDestructureField, AstExpr, AstFunction, AstGenericParam,
    AstImplDef, AstMatchArm, AstModule, AstParam, AstStmt, AstStructDef, AstTypeRef, AstUnaryOp,
    AstVisibility,
};

use super::lambda_validation::collect_lambda_block_captures;
use super::validation_binding_env::instantiate_ast_struct_field_type;

const LAMBDA_BIND_PREFIX: &str = "__lambda_bind.";

fn render_local_access_path(expr: &AstExpr) -> Option<String> {
    match expr {
        AstExpr::Var(name) => Some(name.clone()),
        AstExpr::FieldAccess { base, field } => {
            Some(format!("{}.{}", render_local_access_path(base)?, field))
        }
        _ => None,
    }
}

fn extend_local_field_bindings_from_expr(
    binding_path: &str,
    expr: &AstExpr,
    local_types: &mut BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) {
    let AstExpr::StructLiteral { fields, .. } = expr else {
        return;
    };
    for (field_name, value) in fields {
        let Some(field_ty) =
            infer_local_binding_type(value, local_types, module_function_table, module_impls)
        else {
            continue;
        };
        let field_path = format!("{binding_path}.{}", field_name);
        local_types.insert(field_path.clone(), field_ty);
        extend_local_field_bindings_from_expr(
            &field_path,
            value,
            local_types,
            module_function_table,
            module_impls,
        );
    }
}

fn extend_local_field_bindings_from_type(
    binding_path: &str,
    ty: &AstTypeRef,
    visible_structs: &BTreeMap<String, AstStructDef>,
    local_types: &mut BTreeMap<String, AstTypeRef>,
) {
    let Some(definition) = visible_structs.get(&ty.name) else {
        return;
    };
    for field in &definition.fields {
        let field_ty = instantiate_ast_struct_field_type(ty, definition, &field.ty);
        let field_path = format!("{}.{}", binding_path, field.name);
        local_types.insert(field_path.clone(), field_ty.clone());
        extend_local_field_bindings_from_type(&field_path, &field_ty, visible_structs, local_types);
    }
}

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
    let visible_structs = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut expanded = module.clone();
    expanded.functions.clear();
    for function in &module.functions {
        let (rewritten, synthesized) = expand_function_lambdas(
            function,
            &module.impls,
            &visible_structs,
            &module_const_names,
            &module_function_table,
        )?;
        expanded.functions.extend(synthesized);
        expanded.functions.push(rewritten);
    }
    Ok(expanded)
}

fn expand_function_lambdas(
    function: &AstFunction,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
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
    let mut visible_local_types = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    for param in &function.params {
        extend_local_field_bindings_from_type(
            &param.name,
            &param.ty,
            visible_structs,
            &mut visible_local_types,
        );
    }
    let body = expand_lambda_block(
        &function.body,
        function.return_type.as_ref(),
        &function.generic_params,
        &BTreeMap::new(),
        &visible_locals,
        &visible_local_types,
        module_impls,
        visible_structs,
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

fn callable_type_from_signature(
    params: &[AstParam],
    return_type: &AstTypeRef,
) -> Option<AstTypeRef> {
    let name = match params.len() {
        1 => "Fn1",
        2 => "Fn2",
        3 => "Fn3",
        _ => return None,
    };
    let mut generic_args = params
        .iter()
        .map(|param| param.ty.clone())
        .collect::<Vec<_>>();
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

fn specialize_type_with_substitutions(
    ty: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if ty.generic_args.is_empty() {
        if let Some(substitution) = substitutions.get(&ty.name) {
            return substitution.clone();
        }
    }
    AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| specialize_type_with_substitutions(arg, substitutions))
            .collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}

fn unify_generic_type_pattern(
    pattern: &AstTypeRef,
    actual: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    substitutions: &mut BTreeMap<String, AstTypeRef>,
) {
    if pattern.is_optional != actual.is_optional || pattern.is_ref != actual.is_ref {
        return;
    }
    if pattern.generic_args.is_empty() && generic_names.contains(&pattern.name) {
        match substitutions.get(&pattern.name) {
            Some(existing) if existing != actual => {}
            Some(_) => {}
            None => {
                substitutions.insert(pattern.name.clone(), actual.clone());
            }
        }
        return;
    }
    if pattern.name != actual.name || pattern.generic_args.len() != actual.generic_args.len() {
        return;
    }
    for (pattern_arg, actual_arg) in pattern.generic_args.iter().zip(actual.generic_args.iter()) {
        unify_generic_type_pattern(pattern_arg, actual_arg, generic_names, substitutions);
    }
}

fn infer_generic_call_substitutions(
    function: &AstFunction,
    explicit_generic_args: &[AstTypeRef],
    args: &[AstExpr],
    expected_result_type: Option<&AstTypeRef>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> BTreeMap<String, AstTypeRef> {
    let generic_names = function
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::new();
    if explicit_generic_args.len() == function.generic_params.len() {
        for (param, arg) in function.generic_params.iter().zip(explicit_generic_args.iter()) {
            substitutions.insert(param.name.clone(), arg.clone());
        }
    }
    for (param, arg) in function.params.iter().zip(args.iter()) {
        if matches!(arg, AstExpr::Lambda { .. }) {
            continue;
        }
        let Some(arg_ty) = infer_local_binding_type(
            arg,
            visible_local_types,
            module_function_table,
            module_impls,
        )
        else {
            continue;
        };
        unify_generic_type_pattern(&param.ty, &arg_ty, &generic_names, &mut substitutions);
    }
    if let (Some(return_pattern), Some(expected_result_type)) =
        (function.return_type.as_ref(), expected_result_type)
    {
        unify_generic_type_pattern(
            return_pattern,
            expected_result_type,
            &generic_names,
            &mut substitutions,
        );
    }
    substitutions
}

fn infer_impl_method_substitutions(
    definition: &AstImplDef,
    method_index: usize,
    receiver_ty: &AstTypeRef,
    args: &[AstExpr],
    expected_result_type: Option<&AstTypeRef>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> BTreeMap<String, AstTypeRef> {
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::new();
    unify_generic_type_pattern(
        &definition.for_type,
        receiver_ty,
        &generic_names,
        &mut substitutions,
    );
    let Some(method_def) = definition.methods.get(method_index) else {
        return substitutions;
    };
    for (param, arg) in method_def.params.iter().skip(1).zip(args.iter()) {
        if matches!(arg, AstExpr::Lambda { .. }) {
            continue;
        }
        let Some(arg_ty) = infer_local_binding_type(
            arg,
            visible_local_types,
            module_function_table,
            module_impls,
        )
        else {
            continue;
        };
        unify_generic_type_pattern(&param.ty, &arg_ty, &generic_names, &mut substitutions);
    }
    let method_return_ty = method_def
        .return_type
        .as_ref()
        .unwrap_or(&definition.for_type);
    if let Some(expected_result_type) = expected_result_type {
        unify_generic_type_pattern(
            method_return_ty,
            expected_result_type,
            &generic_names,
            &mut substitutions,
        );
    }
    substitutions
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
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    match value {
        AstExpr::Bool(_) => Some(named_type("bool")),
        AstExpr::Text(_) => Some(named_type("String")),
        AstExpr::Int(_) => Some(named_type("i64")),
        AstExpr::Float(_) => Some(named_type("f64")),
        AstExpr::Var(name) => visible_local_types.get(name).cloned().or_else(|| {
            module_function_table
                .get(name)
                .and_then(callable_type_from_function)
        }),
        AstExpr::FieldAccess { .. } => render_local_access_path(value)
            .and_then(|path| visible_local_types.get(&path).cloned()),
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
            AstUnaryOp::Neg => {
                infer_local_binding_type(
                    operand,
                    visible_local_types,
                    module_function_table,
                    module_impls,
                )
            }
            AstUnaryOp::Deref => None,
        },
        AstExpr::Binary { op, lhs, rhs } => {
            let lhs_ty = infer_local_binding_type(
                lhs,
                visible_local_types,
                module_function_table,
                module_impls,
            )?;
            let rhs_ty = infer_local_binding_type(
                rhs,
                visible_local_types,
                module_function_table,
                module_impls,
            )?;
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
                | AstBinaryOp::Rem
                    if lhs_ty == rhs_ty =>
                {
                    Some(lhs_ty)
                }
                _ => None,
            }
        }
        AstExpr::If {
            then_body,
            else_body,
            ..
        } => {
            let then_ty = infer_block_result_type(
                then_body,
                visible_local_types,
                module_function_table,
                module_impls,
            )?;
            let else_ty = infer_block_result_type(
                else_body,
                visible_local_types,
                module_function_table,
                module_impls,
            )?;
            if then_ty == else_ty {
                Some(then_ty)
            } else {
                None
            }
        }
        AstExpr::Match { arms, .. } => {
            let mut arm_types = arms.iter().filter_map(|arm| {
                infer_block_result_type(
                    &arm.body,
                    visible_local_types,
                    module_function_table,
                    module_impls,
                )
            });
            let first = arm_types.next()?;
            if arm_types.all(|ty| ty == first) {
                Some(first)
            } else {
                None
            }
        }
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let receiver_ty = infer_local_binding_type(
                receiver,
                visible_local_types,
                module_function_table,
                module_impls,
            )?;
            for definition in module_impls {
                let Some(method_index) =
                    definition.methods.iter().position(|item| item.name == *method)
                else {
                    continue;
                };
                let substitutions = infer_impl_method_substitutions(
                    definition,
                    method_index,
                    &receiver_ty,
                    args,
                    None,
                    visible_local_types,
                    module_function_table,
                    module_impls,
                );
                let specialized_for_type =
                    specialize_type_with_substitutions(&definition.for_type, &substitutions);
                if specialized_for_type != receiver_ty {
                    continue;
                }
                let method_def = &definition.methods[method_index];
                let method_return_ty =
                    method_def.return_type.as_ref().unwrap_or(&definition.for_type);
                return Some(specialize_type_with_substitutions(
                    method_return_ty,
                    &substitutions,
                ));
            }
            None
        }
        _ => None,
    }
}

fn infer_block_result_type(
    body: &[AstStmt],
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    match body.last() {
        Some(AstStmt::Return(Some(expr))) | Some(AstStmt::Expr(expr)) => {
            infer_local_binding_type(
                expr,
                visible_local_types,
                module_function_table,
                module_impls,
            )
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
    final_args.extend(binding.captured_locals.iter().cloned().map(AstExpr::Var));
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
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<LambdaBinding, String> {
    let Some(lambda_return_type) = return_type.clone() else {
        return Err("inline lambda currently requires an explicit return type".to_owned());
    };
    synthesize_lambda_function_with_known_return_type(
        params,
        lambda_return_type,
        body,
        inherited_generic_params,
        lambda_aliases,
        outer_locals,
        outer_local_types,
        module_impls,
        visible_structs,
        module_const_names,
        module_function_table,
        owning_function_name,
        counter,
        synthesized,
    )
}

#[allow(clippy::too_many_arguments)]
fn synthesize_lambda_function_with_known_return_type(
    params: &[AstParam],
    lambda_return_type: AstTypeRef,
    body: &[AstStmt],
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    outer_locals: &BTreeSet<String>,
    outer_local_types: &BTreeMap<String, AstTypeRef>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<LambdaBinding, String> {
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
        Some(&lambda_return_type),
        inherited_generic_params,
        lambda_aliases,
        &lambda_visible_locals,
        &lambda_visible_local_types,
        module_impls,
        visible_structs,
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
        where_bounds: Vec::new(),
        params: synthesized_params,
        return_type: Some(lambda_return_type),
        body: lambda_body,
    });
    Ok(LambdaBinding {
        symbol: synthesized_name,
        captured_locals: capture_params.into_iter().map(|param| param.name).collect(),
    })
}

fn inline_lambda_return_type_from_callable(
    params: &[AstParam],
    explicit_return_type: &Option<AstTypeRef>,
    expected_callable_type: Option<&AstTypeRef>,
) -> Result<Option<AstTypeRef>, String> {
    let Some(expected_callable_type) = expected_callable_type else {
        return Ok(explicit_return_type.clone());
    };
    let Some(arity) = callable_type_arity(expected_callable_type) else {
        return Ok(explicit_return_type.clone());
    };
    if params.len() != arity || expected_callable_type.generic_args.len() != arity + 1 {
        return Ok(explicit_return_type.clone());
    }
    for (param, expected) in params.iter().zip(expected_callable_type.generic_args[..arity].iter()) {
        if param.ty != *expected {
            return Ok(explicit_return_type.clone());
        }
    }
    let inferred_return_type = expected_callable_type.generic_args[arity].clone();
    if let Some(explicit_return_type) = explicit_return_type {
        if *explicit_return_type != inferred_return_type {
            return Err(format!(
                "inline lambda return type `{}` does not match expected callable return type `{}`",
                explicit_return_type.name, inferred_return_type.name
            ));
        }
    }
    Ok(Some(inferred_return_type))
}

fn expected_callable_type_for_call_arg(
    callee: &str,
    index: usize,
    generic_args: &[AstTypeRef],
    args: &[AstExpr],
    expected_result_type: Option<&AstTypeRef>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    let function = module_function_table.get(callee)?;
    let param = function.params.get(index)?;
    let specialized = if function.generic_params.is_empty() && generic_args.is_empty() {
        param.ty.clone()
    } else {
        let substitutions = infer_generic_call_substitutions(
            function,
            generic_args,
            args,
            expected_result_type,
            visible_local_types,
            module_function_table,
            module_impls,
        );
        specialize_type_with_substitutions(&param.ty, &substitutions)
    };
    callable_type_arity(&specialized).map(|_| specialized)
}

fn expected_callable_type_for_method_arg(
    receiver: &AstExpr,
    method: &str,
    index: usize,
    args: &[AstExpr],
    expected_result_type: Option<&AstTypeRef>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    let receiver_ty =
        infer_local_binding_type(
            receiver,
            visible_local_types,
            module_function_table,
            module_impls,
        )?;
    for definition in module_impls {
        let Some(method_index) = definition.methods.iter().position(|item| item.name == method) else {
            continue;
        };
        let substitutions = infer_impl_method_substitutions(
            definition,
            method_index,
            &receiver_ty,
            args,
            expected_result_type,
            visible_local_types,
            module_function_table,
            module_impls,
        );
        let specialized_for_type = specialize_type_with_substitutions(&definition.for_type, &substitutions);
        if specialized_for_type != receiver_ty {
            continue;
        }
        let method_def = &definition.methods[method_index];
        let Some(param) = method_def.params.get(index + 1) else {
            continue;
        };
        let specialized = specialize_type_with_substitutions(&param.ty, &substitutions);
        if callable_type_arity(&specialized).is_some() {
            return Some(specialized);
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn expand_lambda_block(
    body: &[AstStmt],
    current_return_type: Option<&AstTypeRef>,
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    visible_locals: &BTreeSet<String>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
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
                    module_impls,
                    visible_structs,
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
                    ty.as_ref(),
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
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
                    if let Some(bound_ty) = local_types.get(name).cloned() {
                        extend_local_field_bindings_from_type(
                            name,
                            &bound_ty,
                            visible_structs,
                            &mut local_types,
                        );
                    }
                } else if let Some(inferred_ty) =
                    infer_local_binding_type(
                        &rewritten_value,
                        &local_types,
                        module_function_table,
                        module_impls,
                    )
                {
                    local_types.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    &rewritten_value,
                    &mut local_types,
                    module_function_table,
                    module_impls,
                );
            }
            AstStmt::AssignLocal { name, value } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    local_types.get(name),
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
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
                if let Some(inferred_ty) =
                    infer_local_binding_type(
                        &rewritten_value,
                        &local_types,
                        module_function_table,
                        module_impls,
                    )
                {
                    local_types.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    &rewritten_value,
                    &mut local_types,
                    module_function_table,
                    module_impls,
                );
            }
            AstStmt::DestructureLet {
                type_ref,
                fields,
                value,
            } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    type_ref.as_ref(),
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
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
                    ty.as_ref(),
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
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
                    if let Some(bound_ty) = local_types.get(name).cloned() {
                        extend_local_field_bindings_from_type(
                            name,
                            &bound_ty,
                            visible_structs,
                            &mut local_types,
                        );
                    }
                } else if let Some(inferred_ty) =
                    infer_local_binding_type(
                        &rewritten_value,
                        &local_types,
                        module_function_table,
                        module_impls,
                    )
                {
                    local_types.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    &rewritten_value,
                    &mut local_types,
                    module_function_table,
                    module_impls,
                );
            }
            AstStmt::Print(value) => rewritten.push(AstStmt::Print(rewrite_lambda_expr(
                value,
                None,
                inherited_generic_params,
                &aliases,
                &locals,
                &local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::Await(value) => rewritten.push(AstStmt::Await(rewrite_lambda_expr(
                value,
                None,
                inherited_generic_params,
                &aliases,
                &locals,
                &local_types,
                module_impls,
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
                    None,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                then_body: expand_lambda_block(
                    then_body,
                    current_return_type,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
                    visible_structs,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                else_body: expand_lambda_block(
                    else_body,
                    current_return_type,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
                    visible_structs,
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
                    None,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
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
                                        None,
                                        inherited_generic_params,
                                        &aliases,
                                        &locals,
                                        &local_types,
                                        module_impls,
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
                                current_return_type,
                                inherited_generic_params,
                                &aliases,
                                &locals,
                                &local_types,
                                module_impls,
                                visible_structs,
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
                    None,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                body: expand_lambda_block(
                    body,
                    current_return_type,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
                    visible_structs,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
            }),
            AstStmt::Expr(expr) => rewritten.push(AstStmt::Expr(rewrite_lambda_expr(
                expr,
                None,
                inherited_generic_params,
                &aliases,
                &locals,
                &local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::Return(value) => rewritten.push(AstStmt::Return(match value {
                Some(value) => Some(rewrite_lambda_expr(
                    value,
                    current_return_type,
                    inherited_generic_params,
                    &aliases,
                    &locals,
                    &local_types,
                    module_impls,
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
    expected_expr_type: Option<&AstTypeRef>,
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    visible_locals: &BTreeSet<String>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_impls: &[AstImplDef],
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
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            then_body: expand_lambda_block(
                then_body,
                expected_expr_type,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                &BTreeMap::new(),
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?,
            else_body: expand_lambda_block(
                else_body,
                expected_expr_type,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                &BTreeMap::new(),
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
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
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
                                None,
                                inherited_generic_params,
                                lambda_aliases,
                                visible_locals,
                                visible_local_types,
                                module_impls,
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
                            expected_expr_type,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            &BTreeMap::new(),
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
                module_impls,
                &BTreeMap::new(),
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
            None,
            inherited_generic_params,
            lambda_aliases,
            visible_locals,
            visible_local_types,
            module_impls,
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
                        None,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        visible_local_types,
        module_impls,
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
                        module_impls,
                        &BTreeMap::new(),
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
                .enumerate()
                .map(|(index, arg)| {
                    if let AstExpr::Lambda {
                        params,
                        return_type,
                        body,
                    } = arg
                    {
                        let inferred_return_type = inline_lambda_return_type_from_callable(
                            params,
                            return_type,
                            expected_callable_type_for_call_arg(
                                callee,
                                index,
                                generic_args,
                                args,
                                expected_expr_type,
                                visible_local_types,
                                module_function_table,
                                module_impls,
                            )
                            .as_ref(),
                        )?;
                        let binding = synthesize_lambda_function_with_known_return_type(
                            params,
                            inferred_return_type.ok_or_else(|| {
                                "inline lambda currently requires an explicit return type"
                                    .to_owned()
                            })?,
                            body,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            &BTreeMap::new(),
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )?;
                        Ok(build_lambda_binding_value(&binding))
                    } else {
                        rewrite_lambda_expr(
                            arg,
                            None,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                    module_impls,
                    module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )
                    }
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
        } => {
            let rewritten_receiver = Box::new(rewrite_lambda_expr(
                receiver,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?);
            let rewritten_args = args
                .iter()
                .enumerate()
                .map(|(index, arg)| {
                    if let AstExpr::Lambda {
                        params,
                        return_type,
                        body,
                    } = arg
                    {
                        let inferred_return_type = inline_lambda_return_type_from_callable(
                            params,
                            return_type,
                            expected_callable_type_for_method_arg(
                                receiver,
                                method,
                                index,
                                args,
                                expected_expr_type,
                                visible_local_types,
                                module_function_table,
                                module_impls,
                            )
                            .as_ref(),
                        )?;
                        let binding = synthesize_lambda_function_with_known_return_type(
                            params,
                            inferred_return_type.ok_or_else(|| {
                                "inline lambda currently requires an explicit return type"
                                    .to_owned()
                            })?,
                            body,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            &BTreeMap::new(),
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )?;
                        Ok(build_lambda_binding_value(&binding))
                    } else {
                        rewrite_lambda_expr(
                            arg,
                            None,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            AstExpr::MethodCall {
                receiver: rewritten_receiver,
                method: method.clone(),
                args: rewritten_args,
            }
        }
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
                            None,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
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
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
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
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                    module_impls,
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
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                    module_impls,
                    module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            rhs: Box::new(rewrite_lambda_expr(
                rhs,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
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
