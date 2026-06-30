use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstImplDef, AstParam, AstStmt, AstStructDef, AstTypeRef,
    AstUnaryOp,
};

use super::super::validation_binding_env::instantiate_ast_struct_field_type;

const LAMBDA_BIND_PREFIX: &str = "__lambda_bind.";

pub(super) fn render_local_access_path(expr: &AstExpr) -> Option<String> {
    match expr {
        AstExpr::Var(name) => Some(name.clone()),
        AstExpr::FieldAccess { base, field } => {
            Some(format!("{}.{}", render_local_access_path(base)?, field))
        }
        _ => None,
    }
}

pub(super) fn extend_local_field_bindings_from_expr(
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

pub(super) fn extend_local_field_bindings_from_type(
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
pub(super) struct LambdaBinding {
    pub(super) symbol: String,
    pub(super) captured_locals: Vec<String>,
}

pub(super) fn callable_type_arity(ty: &AstTypeRef) -> Option<usize> {
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

pub(super) fn callable_type_from_signature(
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

pub(super) fn callable_type_from_function(function: &AstFunction) -> Option<AstTypeRef> {
    let return_type = function.return_type.as_ref()?;
    callable_type_from_signature(&function.params, return_type)
}

pub(super) fn specialize_type_with_substitutions(
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

pub(super) fn unify_generic_type_pattern(
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

pub(super) fn infer_generic_call_substitutions(
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
        for (param, arg) in function
            .generic_params
            .iter()
            .zip(explicit_generic_args.iter())
        {
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
        ) else {
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

pub(super) fn infer_impl_method_substitutions(
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
        ) else {
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

pub(super) fn callable_type_matches_signature(
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

pub(super) fn callable_binding_return_type(
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

pub(super) fn infer_local_binding_type(
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
        AstExpr::FieldAccess { .. } => {
            render_local_access_path(value).and_then(|path| visible_local_types.get(&path).cloned())
        }
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
                module_impls,
            ),
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
            generic_args: _,
            args,
        } => {
            let receiver_ty = infer_local_binding_type(
                receiver,
                visible_local_types,
                module_function_table,
                module_impls,
            )?;
            for definition in module_impls {
                let Some(method_index) = definition
                    .methods
                    .iter()
                    .position(|item| item.name == *method)
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
                let method_return_ty = method_def
                    .return_type
                    .as_ref()
                    .unwrap_or(&definition.for_type);
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

pub(super) fn infer_block_result_type(
    body: &[AstStmt],
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    match body.last() {
        Some(AstStmt::Return(Some(expr))) | Some(AstStmt::Expr(expr)) => infer_local_binding_type(
            expr,
            visible_local_types,
            module_function_table,
            module_impls,
        ),
        _ => None,
    }
}

pub(super) fn named_type(name: &str) -> AstTypeRef {
    AstTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: false,
    }
}

pub(super) fn build_lambda_call(binding: &LambdaBinding, args: Vec<AstExpr>) -> AstExpr {
    let mut final_args = args;
    final_args.extend(binding.captured_locals.iter().cloned().map(AstExpr::Var));
    AstExpr::Call {
        callee: binding.symbol.clone(),
        generic_args: Vec::new(),
        args: final_args,
    }
}

pub(super) fn build_lambda_binding_value(binding: &LambdaBinding) -> AstExpr {
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
