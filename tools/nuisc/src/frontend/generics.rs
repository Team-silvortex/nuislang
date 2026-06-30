use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef, NirTypeRef,
};

use super::types::{ast_type_from_nir, infer_ast_expr_type, infer_ast_expr_type_for_pattern};
use super::validation_binding_env::instantiate_ast_struct_field_type;
use super::validation_trait_bounds::validate_generic_parameter_use_site_bound_with_context;
use super::{lower_type_ref, resolve_ast_type_ref_aliases};

#[path = "generics_specialize.rs"]
mod generics_specialize;

pub(crate) use generics_specialize::{specialize_ast_type_ref, specialize_function_template};

fn is_builtin_concrete_type_name(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "String"
            | "str"
            | "char"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "Task"
            | "Option"
            | "Result"
            | "Pipe"
    )
}

fn type_ref_looks_unresolved_placeholder(ty: &AstTypeRef) -> bool {
    if ty.is_optional || ty.is_ref {
        return false;
    }
    if !ty.generic_args.is_empty() {
        return ty
            .generic_args
            .iter()
            .any(type_ref_looks_unresolved_placeholder);
    }
    !is_builtin_concrete_type_name(&ty.name)
        && !ty.name.contains('.')
        && ty.name.len() <= 2
        && ty.name.chars().all(|ch| ch.is_ascii_uppercase())
}

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
    context: Option<&str>,
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
    for (param, arg) in template.params.iter().zip(args) {
        let resolved_param_ty = resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)?;
        if !contains_unresolved_generic_placeholders(
            &resolved_param_ty,
            &generic_names,
            &substitutions,
        ) {
            continue;
        }
        let pattern_arg_ty = infer_ast_expr_type_for_pattern(
            arg,
            &resolved_param_ty,
            &generic_names,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
        );
        let arg_ty = pattern_arg_ty.or_else(|| {
            infer_alias_aware_ast_expr_type(
                arg,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            )
        });
        let Some(arg_ty) = arg_ty else {
            return Err(format!(
                "cannot infer concrete type for generic arg `{}` in call to `{}`",
                param.name, template.name
            ));
        };
        let arg_ty = if type_ref_looks_unresolved_placeholder(&arg_ty)
            && !contains_unresolved_generic_placeholders(
                &resolved_param_ty,
                &generic_names,
                &substitutions,
            ) {
            resolved_param_ty.clone()
        } else {
            arg_ty
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
        for bound in &generic.bounds {
            let concrete_ast = ast_type_from_nir(concrete);
            validate_generic_parameter_use_site_bound_with_context(
                &generic.name,
                &concrete_ast,
                &bound.name,
                visible_type_aliases,
                impl_lookup,
                context,
            )?;
        }
    }
    for predicate in &template.where_bounds {
        let Some(concrete) = lowered_substitutions.get(&predicate.param_name) else {
            return Err(format!(
                "generic function `{}` currently requires inferring concrete type for `{}` from direct parameter positions or explicit expected type",
                template.name, predicate.param_name
            ));
        };
        let concrete_ast = ast_type_from_nir(concrete);
        for bound in &predicate.bounds {
            validate_generic_parameter_use_site_bound_with_context(
                &predicate.param_name,
                &concrete_ast,
                &bound.name,
                visible_type_aliases,
                impl_lookup,
                context,
            )?;
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
        if generic_names.contains(&concrete.name)
            && concrete.generic_args.is_empty()
            && !concrete.is_optional
            && !concrete.is_ref
            && pattern.name == concrete.name
        {
            return Ok(());
        }
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
    let same_shape = pattern.name == concrete.name
        && pattern.generic_args.len() == concrete.generic_args.len()
        && pattern.is_optional == concrete.is_optional
        && pattern.is_ref == concrete.is_ref;
    let enum_parent_shape = concrete.name.rsplit_once('.').is_some_and(|(parent, _)| {
        pattern.name == parent
            && pattern.is_optional == concrete.is_optional
            && pattern.is_ref == concrete.is_ref
            && (pattern.generic_args.len() == concrete.generic_args.len()
                || concrete.generic_args.is_empty())
    });
    if !same_shape && !enum_parent_shape {
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
