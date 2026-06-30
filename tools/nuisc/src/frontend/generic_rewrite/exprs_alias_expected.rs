use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstStructDef, AstTypeAlias, AstTypeRef};

use super::super::generics::unify_generic_type_pattern;
use super::super::{resolve_ast_type_ref_aliases, substitute_ast_type_alias_target};

pub(super) fn infer_alias_struct_target_from_expected(
    alias_name: &str,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let Some(expected) = expected else {
        return Ok(None);
    };
    let Some(alias_definition) = visible_type_aliases.get(alias_name) else {
        return Ok(None);
    };
    if alias_definition.generic_params.is_empty() {
        return Ok(None);
    }
    let resolved_target_pattern =
        resolve_ast_type_ref_aliases(&alias_definition.target, visible_type_aliases)?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases)?;
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    if unify_generic_type_pattern(
        &resolved_target_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        alias_name,
    )
    .is_err()
    {
        return Ok(None);
    }
    let generic_args = alias_definition
        .generic_params
        .iter()
        .map(|param| substitutions.get(&param.name).cloned())
        .collect::<Option<Vec<_>>>();
    let Some(generic_args) = generic_args else {
        return Ok(None);
    };
    if generic_args
        .iter()
        .any(|arg| contains_ast_placeholder_generic_name(arg, &generic_names))
    {
        return Ok(None);
    }
    let generic_args = generic_args.into_iter().collect::<Vec<_>>();
    let substituted = substitute_ast_type_alias_target(
        &alias_definition.target,
        &alias_definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .zip(generic_args.clone())
            .collect::<BTreeMap<_, _>>(),
    )?;
    let resolved = resolve_ast_type_ref_aliases(&substituted, visible_type_aliases)?;
    if !struct_table.contains_key(&resolved.name) {
        return Ok(None);
    }
    Ok(Some((resolved.name, resolved.generic_args)))
}

pub(super) fn contains_ast_placeholder_generic_name(
    ty: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| contains_ast_placeholder_generic_name(arg, placeholder_names))
}

pub(super) fn ast_type_args_are_placeholder_generics(
    type_args: &[AstTypeRef],
    placeholder_names: &BTreeSet<String>,
) -> bool {
    type_args.is_empty()
        || type_args
            .iter()
            .all(|arg| contains_ast_placeholder_generic_name(arg, placeholder_names))
}

pub(super) fn seed_alias_generic_substitutions_from_expected_pattern(
    alias_name: &str,
    alias_definition: &AstTypeAlias,
    resolved_target_pattern: &AstTypeRef,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_names: &BTreeSet<String>,
) -> Result<BTreeMap<String, AstTypeRef>, String> {
    let Some(expected) = expected else {
        return Ok(BTreeMap::new());
    };
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases)?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    if unify_generic_type_pattern(
        resolved_target_pattern,
        &resolved_expected,
        generic_names,
        &mut substitutions,
        alias_name,
    )
    .is_err()
    {
        return Ok(BTreeMap::new());
    }
    Ok(alias_definition
        .generic_params
        .iter()
        .filter_map(|param| {
            substitutions.get(&param.name).and_then(|arg| {
                (!contains_ast_placeholder_generic_name(arg, generic_names))
                    .then_some((param.name.clone(), arg.clone()))
            })
        })
        .collect())
}
