use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstMatchArm, AstMatchPattern, AstParam, AstStmt,
    AstStructDef, AstTypeAlias, AstTypeRef, NirTypeRef,
};

use super::types::{ast_type_from_nir, infer_ast_expr_type};
use super::validation_binding_env::instantiate_ast_struct_field_type;
use super::{lower_type_ref, resolve_ast_type_ref_aliases};

pub(crate) fn infer_generic_substitutions(
    template: &AstFunction,
    explicit_generic_args: &[AstTypeRef],
    args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<BTreeMap<String, NirTypeRef>, String> {
    if template.params.len() != args.len() {
        return Err(format!(
            "generic function `{}` expects {} args, found {}",
            template.name,
            template.params.len(),
            args.len()
        ));
    }
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions =
        explicit_generic_substitutions(template, explicit_generic_args, visible_type_aliases)?;
    for (param, arg) in template.params.iter().zip(args) {
        let resolved_param_ty = resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)?;
        if !contains_unresolved_generic_placeholders(
            &resolved_param_ty,
            &generic_names,
            &substitutions,
        ) {
            continue;
        }
        let Some(arg_ty) = infer_alias_aware_ast_expr_type(
            arg,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
        ) else {
            return Err(format!(
                "cannot infer concrete type for generic arg `{}` in call to `{}`",
                param.name, template.name
            ));
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
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    for generic in &template.generic_params {
        let Some(concrete) = lowered_substitutions.get(&generic.name) else {
            return Err(format!(
                "generic function `{}` currently requires inferring concrete type for `{}` from direct parameter positions or explicit expected type",
                template.name, generic.name
            ));
        };
        if let Some(bound) = &generic.bound {
            let bound_key = (bound.name.clone(), concrete.render());
            if !impl_lookup.contains_key(&bound_key) {
                return Err(format!(
                    "type `{}` does not satisfy bound `{}` for generic parameter `{}`",
                    concrete.render(),
                    bound.name,
                    generic.name
                ));
            }
        }
    }
    Ok(lowered_substitutions)
}

pub(crate) fn infer_alias_aware_ast_expr_type(
    expr: &AstExpr,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    infer_ast_expr_type(expr, env, impl_lookup, struct_table, function_return_types).or_else(|| {
        match expr {
            AstExpr::FieldAccess { base, field } => {
                let base_ty = infer_alias_aware_ast_expr_type(
                    base,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )?;
                let resolved_base_ty =
                    resolve_ast_type_ref_aliases(&base_ty, visible_type_aliases).ok()?;
                let definition = struct_table.get(&resolved_base_ty.name)?;
                definition
                    .fields
                    .iter()
                    .find(|item| item.name == *field)
                    .map(|field_def| {
                        instantiate_ast_struct_field_type(
                            &resolved_base_ty,
                            definition,
                            &field_def.ty,
                        )
                    })
            }
            _ => None,
        }
    })
}

fn explicit_generic_substitutions(
    template: &AstFunction,
    explicit_generic_args: &[AstTypeRef],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<BTreeMap<String, AstTypeRef>, String> {
    if explicit_generic_args.is_empty() {
        return Ok(BTreeMap::new());
    }
    if explicit_generic_args.len() != template.generic_params.len() {
        return Err(format!(
            "generic function `{}` expects {} explicit generic argument(s), found {}",
            template.name,
            template.generic_params.len(),
            explicit_generic_args.len()
        ));
    }
    template
        .generic_params
        .iter()
        .zip(explicit_generic_args.iter())
        .map(|(param, arg)| {
            Ok((
                param.name.clone(),
                resolve_ast_type_ref_aliases(arg, visible_type_aliases)?,
            ))
        })
        .collect()
}

fn contains_unresolved_generic_placeholders(
    ty: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> bool {
    if generic_names.contains(&ty.name)
        && ty.generic_args.is_empty()
        && !substitutions.contains_key(&ty.name)
    {
        return true;
    }
    ty.generic_args
        .iter()
        .any(|arg| contains_unresolved_generic_placeholders(arg, generic_names, substitutions))
}

pub(crate) fn unify_generic_type_pattern(
    pattern: &AstTypeRef,
    concrete: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    substitutions: &mut BTreeMap<String, AstTypeRef>,
    function_name: &str,
) -> Result<(), String> {
    if generic_names.contains(&pattern.name) && pattern.generic_args.is_empty() {
        if let Some(existing) = substitutions.get(&pattern.name) {
            if lower_type_ref(existing).render() != lower_type_ref(concrete).render() {
                return Err(format!(
                    "generic parameter `{}` in `{}` resolved to conflicting types `{}` and `{}`",
                    pattern.name,
                    function_name,
                    lower_type_ref(existing).render(),
                    lower_type_ref(concrete).render()
                ));
            }
        } else {
            substitutions.insert(pattern.name.clone(), concrete.clone());
        }
        return Ok(());
    }
    if pattern.name != concrete.name
        || pattern.generic_args.len() != concrete.generic_args.len()
        || pattern.is_optional != concrete.is_optional
        || pattern.is_ref != concrete.is_ref
    {
        return Err(format!(
            "generic function `{}` could not match expected type pattern `{}` with concrete type `{}`",
            function_name,
            lower_type_ref(pattern).render(),
            lower_type_ref(concrete).render()
        ));
    }
    for (pattern_arg, concrete_arg) in pattern.generic_args.iter().zip(&concrete.generic_args) {
        unify_generic_type_pattern(
            pattern_arg,
            concrete_arg,
            generic_names,
            substitutions,
            function_name,
        )?;
    }
    Ok(())
}

pub(crate) fn specialize_function_template(
    template: &AstFunction,
    specialized_name: &str,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstFunction, String> {
    Ok(AstFunction {
        name: specialized_name.to_owned(),
        visibility: template.visibility,
        attributes: template.attributes.clone(),
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        is_async: template.is_async,
        generic_params: vec![],
        params: template
            .params
            .iter()
            .map(|param| {
                Ok(AstParam {
                    name: param.name.clone(),
                    ty: specialize_ast_type_ref(&param.ty, substitutions)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        return_type: template
            .return_type
            .as_ref()
            .map(|ty| specialize_ast_type_ref(ty, substitutions))
            .transpose()?,
        body: specialize_stmt_types(&template.body, substitutions)?,
    })
}

pub(crate) fn specialize_stmt_types(
    body: &[AstStmt],
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<Vec<AstStmt>, String> {
    body.iter()
        .map(|stmt| {
            Ok(match stmt {
                AstStmt::Let { name, ty, value } => AstStmt::Let {
                    name: name.clone(),
                    ty: ty
                        .as_ref()
                        .map(|ty| specialize_ast_type_ref(ty, substitutions))
                        .transpose()?,
                    value: specialize_expr_types(value, substitutions)?,
                },
                AstStmt::DestructureLet {
                    type_ref,
                    fields,
                    value,
                } => AstStmt::DestructureLet {
                    type_ref: type_ref
                        .as_ref()
                        .map(|type_ref| specialize_ast_type_ref(type_ref, substitutions))
                        .transpose()?,
                    fields: fields.clone(),
                    value: specialize_expr_types(value, substitutions)?,
                },
                AstStmt::Const { name, ty, value } => AstStmt::Const {
                    name: name.clone(),
                    ty: ty
                        .as_ref()
                        .map(|ty| specialize_ast_type_ref(ty, substitutions))
                        .transpose()?,
                    value: specialize_expr_types(value, substitutions)?,
                },
                AstStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => AstStmt::If {
                    condition: specialize_expr_types(condition, substitutions)?,
                    then_body: specialize_stmt_types(then_body, substitutions)?,
                    else_body: specialize_stmt_types(else_body, substitutions)?,
                },
                AstStmt::Match { value, arms } => AstStmt::Match {
                    value: specialize_expr_types(value, substitutions)?,
                    arms: arms
                        .iter()
                        .map(|arm| {
                            Ok(AstMatchArm {
                                pattern: specialize_match_pattern(&arm.pattern, substitutions)?,
                                guard: arm
                                    .guard
                                    .as_ref()
                                    .map(|guard| specialize_expr_types(guard, substitutions))
                                    .transpose()?,
                                body: specialize_stmt_types(&arm.body, substitutions)?,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                },
                AstStmt::While { condition, body } => AstStmt::While {
                    condition: specialize_expr_types(condition, substitutions)?,
                    body: specialize_stmt_types(body, substitutions)?,
                },
                AstStmt::Print(value) => {
                    AstStmt::Print(specialize_expr_types(value, substitutions)?)
                }
                AstStmt::Await(value) => {
                    AstStmt::Await(specialize_expr_types(value, substitutions)?)
                }
                AstStmt::Expr(value) => AstStmt::Expr(specialize_expr_types(value, substitutions)?),
                AstStmt::Return(value) => AstStmt::Return(
                    value
                        .as_ref()
                        .map(|value| specialize_expr_types(value, substitutions))
                        .transpose()?,
                ),
                AstStmt::Break => AstStmt::Break,
                AstStmt::Continue => AstStmt::Continue,
            })
        })
        .collect()
}

fn specialize_expr_types(
    expr: &AstExpr,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(specialize_expr_types(condition, substitutions)?),
            then_body: specialize_stmt_types(then_body, substitutions)?,
            else_body: specialize_stmt_types(else_body, substitutions)?,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(specialize_expr_types(value, substitutions)?),
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: specialize_match_pattern(&arm.pattern, substitutions)?,
                        guard: arm
                            .guard
                            .as_ref()
                            .map(|guard| specialize_expr_types(guard, substitutions))
                            .transpose()?,
                        body: specialize_stmt_types(&arm.body, substitutions)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Await(value) => {
            AstExpr::Await(Box::new(specialize_expr_types(value, substitutions)?))
        }
        AstExpr::Instantiate { domain, unit } => AstExpr::Instantiate {
            domain: domain.clone(),
            unit: unit.clone(),
        },
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => AstExpr::Call {
            callee: callee.clone(),
            generic_args: generic_args
                .iter()
                .map(|arg| specialize_ast_type_ref(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
            args: args
                .iter()
                .map(|arg| specialize_expr_types(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::Invoke { callee, args } => AstExpr::Invoke {
            callee: Box::new(specialize_expr_types(callee, substitutions)?),
            args: args
                .iter()
                .map(|arg| specialize_expr_types(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(specialize_expr_types(receiver, substitutions)?),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| specialize_expr_types(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| specialize_ast_type_ref(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
            fields: fields
                .iter()
                .map(|(field, value)| {
                    Ok((field.clone(), specialize_expr_types(value, substitutions)?))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(specialize_expr_types(base, substitutions)?),
            field: field.clone(),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(specialize_expr_types(lhs, substitutions)?),
            rhs: Box::new(specialize_expr_types(rhs, substitutions)?),
        },
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => AstExpr::Lambda {
            params: params
                .iter()
                .map(|param| {
                    Ok(AstParam {
                        name: param.name.clone(),
                        ty: specialize_ast_type_ref(&param.ty, substitutions)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            return_type: return_type
                .as_ref()
                .map(|ty| specialize_ast_type_ref(ty, substitutions))
                .transpose()?,
            body: specialize_stmt_types(body, substitutions)?,
        },
        other => other.clone(),
    })
}

pub(crate) fn specialize_ast_type_ref(
    ty: &AstTypeRef,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstTypeRef, String> {
    if ty.generic_args.is_empty() {
        if let Some(substitution) = substitutions.get(&ty.name) {
            return Ok(ast_type_from_nir(substitution));
        }
    }
    Ok(AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| specialize_ast_type_ref(arg, substitutions))
            .collect::<Result<Vec<_>, _>>()?,
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    })
}

fn specialize_match_pattern(
    pattern: &AstMatchPattern,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstMatchPattern, String> {
    Ok(match pattern {
        AstMatchPattern::Or(patterns) => AstMatchPattern::Or(
            patterns
                .iter()
                .map(|pattern| specialize_match_pattern(pattern, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        AstMatchPattern::PayloadStruct { type_ref, payload } => AstMatchPattern::PayloadStruct {
            type_ref: specialize_ast_type_ref(type_ref, substitutions)?,
            payload: Box::new(specialize_match_pattern(payload, substitutions)?),
        },
        AstMatchPattern::StructFields { type_ref, fields } => AstMatchPattern::StructFields {
            type_ref: type_ref
                .as_ref()
                .map(|type_ref| specialize_ast_type_ref(type_ref, substitutions))
                .transpose()?,
            fields: fields
                .iter()
                .map(|(field, pattern)| {
                    Ok((
                        field.clone(),
                        specialize_match_pattern(pattern, substitutions)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        other => other.clone(),
    })
}
