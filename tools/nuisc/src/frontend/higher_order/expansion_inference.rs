use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::generics::unify_generic_type_pattern;
use super::super::types::infer_ast_expr_type_for_pattern;
use super::super::validation_binding_env::instantiate_ast_struct_field_type;
use super::super::{lower_type_ref, resolve_ast_type_ref_aliases};
use super::callables::is_callable_type_with_aliases;
use super::expansion::parse_bound_callable_expr;
use super::expansion_callable_inference::{
    contains_unresolved_template_generic, infer_callable_binding_substitutions,
    type_ref_looks_unresolved_placeholder,
};
use super::expansion_expected::collect_generic_type_names;

fn render_local_access_path(expr: &AstExpr) -> Option<String> {
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
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) {
    let AstExpr::StructLiteral { fields, .. } = expr else {
        return;
    };
    for (field_name, value) in fields {
        let Some(field_ty) =
            infer_local_binding_type(value, local_types, function_table, module_impls)
        else {
            continue;
        };
        let field_path = format!("{binding_path}.{}", field_name);
        local_types.insert(field_path.clone(), field_ty);
        extend_local_field_bindings_from_expr(
            &field_path,
            value,
            local_types,
            function_table,
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

struct ImplMethodSubstitutionInput<'a> {
    definition: &'a AstImplDef,
    method_index: usize,
    receiver_ty: &'a AstTypeRef,
    args: &'a [AstExpr],
    expected_result_type: Option<&'a AstTypeRef>,
    local_types: &'a BTreeMap<String, AstTypeRef>,
    function_table: &'a BTreeMap<String, AstFunction>,
    module_impls: &'a [AstImplDef],
}

fn infer_impl_method_substitutions(
    input: ImplMethodSubstitutionInput<'_>,
) -> BTreeMap<String, AstTypeRef> {
    let ImplMethodSubstitutionInput {
        definition,
        method_index,
        receiver_ty,
        args,
        expected_result_type,
        local_types,
        function_table,
        module_impls,
    } = input;
    let generic_names = collect_generic_type_names(&definition.for_type);
    let mut substitutions = BTreeMap::new();
    unify_generic_receiver_pattern(
        &definition.for_type,
        receiver_ty,
        &generic_names,
        &mut substitutions,
    );
    let Some(method_def) = definition.methods.get(method_index) else {
        return substitutions;
    };
    for (param, arg) in method_def.params.iter().skip(1).zip(args.iter()) {
        let Some(arg_ty) = infer_local_binding_type(arg, local_types, function_table, module_impls)
        else {
            continue;
        };
        unify_generic_receiver_pattern(&param.ty, &arg_ty, &generic_names, &mut substitutions);
    }
    let method_return_ty = method_def
        .return_type
        .as_ref()
        .unwrap_or(&definition.for_type);
    if let Some(expected_result_type) = expected_result_type {
        unify_generic_receiver_pattern(
            method_return_ty,
            expected_result_type,
            &generic_names,
            &mut substitutions,
        );
    }
    substitutions
}

fn unify_generic_receiver_pattern(
    pattern: &AstTypeRef,
    actual: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    substitutions: &mut BTreeMap<String, AstTypeRef>,
) {
    if pattern.is_optional != actual.is_optional || pattern.is_ref != actual.is_ref {
        return;
    }
    if pattern.generic_args.is_empty() && generic_names.contains(&pattern.name) {
        if !substitutions.contains_key(&pattern.name) {
            substitutions.insert(pattern.name.clone(), actual.clone());
        }
        return;
    }
    if pattern.name != actual.name || pattern.generic_args.len() != actual.generic_args.len() {
        return;
    }
    for (pattern_arg, actual_arg) in pattern.generic_args.iter().zip(actual.generic_args.iter()) {
        unify_generic_receiver_pattern(pattern_arg, actual_arg, generic_names, substitutions);
    }
}

pub(super) fn infer_local_binding_type(
    value: &AstExpr,
    local_types: &BTreeMap<String, AstTypeRef>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    match value {
        AstExpr::Bool(_) => Some(AstTypeRef {
            name: "bool".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Text(_) => Some(AstTypeRef {
            name: "String".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Int(_) => Some(AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Float(_) => Some(AstTypeRef {
            name: "f64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Var(name) => local_types.get(name).cloned(),
        AstExpr::FieldAccess { .. } => {
            render_local_access_path(value).and_then(|path| local_types.get(&path).cloned())
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
            args,
        } => infer_call_result_type(
            callee,
            generic_args,
            args,
            local_types,
            function_table,
            module_impls,
        ),
        AstExpr::If {
            then_body,
            else_body,
            ..
        } => {
            let then_ty =
                infer_block_result_type(then_body, local_types, function_table, module_impls)?;
            let else_ty =
                infer_block_result_type(else_body, local_types, function_table, module_impls)?;
            if lower_type_ref(&then_ty).render() == lower_type_ref(&else_ty).render() {
                Some(then_ty)
            } else {
                None
            }
        }
        AstExpr::Match { arms, .. } => {
            let mut arm_types = arms.iter().filter_map(|arm| {
                infer_block_result_type(&arm.body, local_types, function_table, module_impls)
            });
            let first = arm_types.next()?;
            if arm_types.all(|ty| lower_type_ref(&first).render() == lower_type_ref(&ty).render()) {
                Some(first)
            } else {
                None
            }
        }
        AstExpr::Try(value) => {
            let result_ty =
                infer_local_binding_type(value, local_types, function_table, module_impls)?;
            result_payload_type(&result_ty)
        }
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args: _,
            args,
        } => {
            let receiver_ty =
                infer_local_binding_type(receiver, local_types, function_table, module_impls)?;
            for definition in module_impls {
                let Some(method_index) = definition
                    .methods
                    .iter()
                    .position(|item| item.name == *method)
                else {
                    continue;
                };
                let substitutions = infer_impl_method_substitutions(ImplMethodSubstitutionInput {
                    definition,
                    method_index,
                    receiver_ty: &receiver_ty,
                    args,
                    expected_result_type: None,
                    local_types,
                    function_table,
                    module_impls,
                });
                let specialized_for_type =
                    specialize_type_with_substitutions(&definition.for_type, &substitutions);
                if lower_type_ref(&specialized_for_type).render()
                    != lower_type_ref(&receiver_ty).render()
                {
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

fn infer_call_result_type(
    callee: &str,
    explicit_generic_args: &[AstTypeRef],
    args: &[AstExpr],
    local_types: &BTreeMap<String, AstTypeRef>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    let function = function_table.get(callee)?;
    let return_ty = function.return_type.as_ref()?.clone();
    let mut generic_names = if function.generic_params.is_empty() {
        if return_ty.generic_args.is_empty() {
            return Some(return_ty);
        }
        collect_generic_type_names(&return_ty)
    } else {
        function
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>()
    };
    if generic_names.is_empty() {
        return Some(return_ty);
    }
    let mut substitutions = BTreeMap::new();

    for (param, arg) in function.params.iter().zip(args.iter()) {
        let arg_ty = infer_local_binding_type(arg, local_types, function_table, module_impls)?;
        unify_generic_type_pattern(
            &param.ty,
            &arg_ty,
            &generic_names,
            &mut substitutions,
            &function.name,
        )
        .ok()?;
    }

    for (name, explicit) in function
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .zip(explicit_generic_args.iter().cloned())
    {
        substitutions.insert(name, explicit);
    }

    generic_names.retain(|name| substitutions.contains_key(name));
    if generic_names.is_empty() && substitutions.is_empty() {
        return Some(return_ty);
    }
    Some(specialize_type_with_substitutions(
        &return_ty,
        &substitutions,
    ))
}

fn infer_block_result_type(
    body: &[AstStmt],
    local_types: &BTreeMap<String, AstTypeRef>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    match body.last() {
        Some(AstStmt::Return(Some(expr))) | Some(AstStmt::Expr(expr)) => {
            infer_local_binding_type(expr, local_types, function_table, module_impls)
        }
        _ => None,
    }
}

fn result_payload_type(ty: &AstTypeRef) -> Option<AstTypeRef> {
    if ty.is_optional || ty.is_ref || ty.name != "Result" || ty.generic_args.len() != 2 {
        return None;
    }
    Some(ty.generic_args[0].clone())
}

pub(super) fn expected_try_operand_type(
    expected_payload: Option<&AstTypeRef>,
    current_return_type: Option<&AstTypeRef>,
) -> Option<AstTypeRef> {
    let payload = expected_payload?;
    let function_result = current_return_type?;
    if function_result.is_optional || function_result.is_ref {
        return None;
    }
    if function_result.name != "Result" || function_result.generic_args.len() != 2 {
        return None;
    }
    Some(AstTypeRef {
        name: "Result".to_owned(),
        generic_args: vec![payload.clone(), function_result.generic_args[1].clone()],
        is_optional: false,
        is_ref: false,
    })
}

pub(super) fn expected_await_operand_type(
    expected_payload: Option<&AstTypeRef>,
) -> Option<AstTypeRef> {
    let payload = expected_payload?;
    Some(AstTypeRef {
        name: "Task".to_owned(),
        generic_args: vec![payload.clone()],
        is_optional: false,
        is_ref: false,
    })
}

pub(super) struct HigherOrderSubstitutionInferenceInput<'a> {
    pub(super) template: &'a AstFunction,
    pub(super) explicit_substitutions: &'a BTreeMap<String, AstTypeRef>,
    pub(super) args: &'a [AstExpr],
    pub(super) expected: Option<&'a AstTypeRef>,
    pub(super) local_types: &'a BTreeMap<String, AstTypeRef>,
    pub(super) function_table: &'a BTreeMap<String, AstFunction>,
    pub(super) module_impls: &'a [AstImplDef],
    pub(super) visible_structs: &'a BTreeMap<String, AstStructDef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
}

pub(super) fn infer_higher_order_substitutions(
    input: HigherOrderSubstitutionInferenceInput<'_>,
) -> Result<BTreeMap<String, nuis_semantics::model::NirTypeRef>, String> {
    let HigherOrderSubstitutionInferenceInput {
        template,
        explicit_substitutions,
        args,
        expected,
        local_types,
        function_table,
        module_impls,
        visible_structs,
        visible_type_aliases,
    } = input;
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    if generic_names.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut substitutions = explicit_substitutions.clone();
    if let (Some(return_pattern), Some(expected_ty)) = (template.return_type.as_ref(), expected) {
        let resolved_return_pattern =
            resolve_ast_type_ref_aliases(return_pattern, visible_type_aliases)?;
        let resolved_expected_ty = resolve_ast_type_ref_aliases(expected_ty, visible_type_aliases)?;
        unify_generic_type_pattern(
            &resolved_return_pattern,
            &resolved_expected_ty,
            &generic_names,
            &mut substitutions,
            &template.name,
        )?;
    }
    let function_return_types = function_table
        .iter()
        .map(|(name, function)| (name.clone(), function.return_type.clone()))
        .collect::<BTreeMap<_, _>>();
    for (param, arg) in template.params.iter().zip(args) {
        let resolved_param_ty = resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)?;
        let specialized_param_ty =
            specialize_type_with_substitutions(&resolved_param_ty, &substitutions);
        if is_callable_type_with_aliases(&param.ty, visible_type_aliases)? {
            let Some(bound_callable) = parse_bound_callable_expr(arg, None) else {
                continue;
            };
            let Some(callable) = function_table.get(&bound_callable.symbol) else {
                continue;
            };
            for (name, ty) in infer_callable_binding_substitutions(
                &specialized_param_ty,
                callable,
                &generic_names,
                visible_type_aliases,
            )? {
                substitutions.entry(name).or_insert(ty);
            }
            continue;
        }
        let arg_ty = infer_ast_expr_type_for_pattern(
            arg,
            &specialized_param_ty,
            &generic_names,
            local_types,
            &BTreeMap::new(),
            visible_structs,
            &function_return_types,
        )
        .or_else(|| infer_local_binding_type(arg, local_types, function_table, module_impls));
        let arg_ty = match arg_ty {
            Some(arg_ty)
                if type_ref_looks_unresolved_placeholder(&arg_ty)
                    && !contains_unresolved_template_generic(
                        &specialized_param_ty,
                        &generic_names,
                    ) =>
            {
                Some(specialized_param_ty.clone())
            }
            other => other,
        };
        let Some(arg_ty) = arg_ty else {
            continue;
        };
        let resolved_arg_ty = resolve_ast_type_ref_aliases(&arg_ty, visible_type_aliases)?;
        unify_generic_type_pattern(
            &resolved_param_ty,
            &resolved_arg_ty,
            &generic_names,
            &mut substitutions,
            &template.name,
        )?;
    }
    Ok(substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect())
}
