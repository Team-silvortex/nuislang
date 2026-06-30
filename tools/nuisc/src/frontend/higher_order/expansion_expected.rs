use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstFunction, AstParam, AstTypeAlias, AstTypeRef};

use super::super::generics::{specialize_ast_type_ref, unify_generic_type_pattern};
use super::super::{lower_type_ref, resolve_ast_type_ref_aliases};

pub(super) fn find_generic_method_template_name(
    receiver_ty: &AstTypeRef,
    method_name: &str,
    templates: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<Option<String>, String> {
    let resolved_receiver = resolve_ast_type_ref_aliases(receiver_ty, visible_type_aliases)?;
    let mut matches = Vec::new();
    for (template_name, template) in templates {
        if !template_name.starts_with("impl.") || template.name != *template_name {
            continue;
        }
        if template.params.is_empty() || template.name.rsplit('.').next() != Some(method_name) {
            continue;
        }
        let receiver_pattern =
            resolve_ast_type_ref_aliases(&template.params[0].ty, visible_type_aliases)?;
        let generic_names = collect_generic_type_names(&receiver_pattern);
        if generic_names.is_empty() {
            continue;
        }
        let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
        if unify_generic_type_pattern(
            &receiver_pattern,
            &resolved_receiver,
            &generic_names,
            &mut substitutions,
            template_name,
        )
        .is_ok()
        {
            matches.push(template_name.clone());
        }
    }
    if matches.len() > 1 {
        return Err(format!(
            "generic higher-order impl method resolution for `{}` is ambiguous; matching templates: {}",
            method_name,
            matches.join(", ")
        ));
    }
    Ok(matches.into_iter().next())
}

pub(super) fn collect_generic_type_names(ty: &AstTypeRef) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_generic_type_names_into(ty, &mut names);
    names
}

fn collect_generic_type_names_into(ty: &AstTypeRef, names: &mut BTreeSet<String>) {
    if ty.generic_args.is_empty() {
        names.insert(ty.name.clone());
    }
    for arg in &ty.generic_args {
        collect_generic_type_names_into(arg, names);
    }
}

pub(super) fn explicit_higher_order_generic_substitutions(
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

pub(super) fn higher_order_param_expected_type(
    template: &AstFunction,
    param: &AstParam,
    explicit_substitutions: &BTreeMap<String, AstTypeRef>,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    if !explicit_substitutions.is_empty() {
        let lowered_substitutions = explicit_substitutions
            .iter()
            .map(|(name, ty)| (name.clone(), lower_type_ref(ty)))
            .collect::<BTreeMap<_, _>>();
        return specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok();
    }
    let expected = expected?;
    let return_pattern = template.return_type.as_ref()?;
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    if generic_names.is_empty() {
        return None;
    }
    let resolved_return_pattern =
        resolve_ast_type_ref_aliases(return_pattern, visible_type_aliases).ok()?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    unify_generic_type_pattern(
        &resolved_return_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        &template.name,
    )
    .ok()?;
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok()
}

pub(super) fn annotate_expr_head_with_expected_type(
    expr: AstExpr,
    expected: Option<&AstTypeRef>,
) -> AstExpr {
    let Some(expected) = expected else {
        return expr;
    };
    match expr {
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty()
            && callee == expected.name
            && !expected.generic_args.is_empty() =>
        {
            AstExpr::Call {
                callee,
                generic_args: expected.generic_args.clone(),
                args,
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } if type_args.is_empty()
            && type_name == expected.name
            && !expected.generic_args.is_empty() =>
        {
            AstExpr::StructLiteral {
                type_name,
                type_args: expected.generic_args.clone(),
                fields,
            }
        }
        other => other,
    }
}
