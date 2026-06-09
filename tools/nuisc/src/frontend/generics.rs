use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstMatchArm, AstMatchPattern, AstParam, AstStmt,
    AstStructDef, AstTypeAlias, AstTypeRef, NirTypeRef,
};

use super::types::{ast_type_from_nir, infer_ast_expr_type};
use super::{lower_type_ref, resolve_ast_type_ref_aliases};

pub(crate) fn infer_generic_substitutions(
    template: &AstFunction,
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
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    for (param, arg) in template.params.iter().zip(args) {
        let Some(arg_ty) =
            infer_ast_expr_type(arg, env, impl_lookup, struct_table, function_return_types)
        else {
            return Err(format!(
                "cannot infer concrete type for generic arg `{}` in call to `{}`",
                param.name, template.name
            ));
        };
        let resolved_param_ty = resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)?;
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
                    value: value.clone(),
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
                    value: value.clone(),
                },
                AstStmt::Const { name, ty, value } => AstStmt::Const {
                    name: name.clone(),
                    ty: ty
                        .as_ref()
                        .map(|ty| specialize_ast_type_ref(ty, substitutions))
                        .transpose()?,
                    value: value.clone(),
                },
                AstStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => AstStmt::If {
                    condition: condition.clone(),
                    then_body: specialize_stmt_types(then_body, substitutions)?,
                    else_body: specialize_stmt_types(else_body, substitutions)?,
                },
                AstStmt::Match { value, arms } => AstStmt::Match {
                    value: value.clone(),
                    arms: arms
                        .iter()
                        .map(|arm| {
                            Ok(AstMatchArm {
                                pattern: specialize_match_pattern(&arm.pattern, substitutions)?,
                                guard: arm.guard.clone(),
                                body: specialize_stmt_types(&arm.body, substitutions)?,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                },
                AstStmt::While { condition, body } => AstStmt::While {
                    condition: condition.clone(),
                    body: specialize_stmt_types(body, substitutions)?,
                },
                other => other.clone(),
            })
        })
        .collect()
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
