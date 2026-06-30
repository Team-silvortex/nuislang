use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstFunction, AstTypeAlias, AstTypeRef};

use super::super::generics::unify_generic_type_pattern;
use super::super::resolve_ast_type_ref_aliases;
use super::expansion_inference::specialize_type_with_substitutions;

pub(super) fn infer_callable_binding_substitutions(
    expected_callable_ty: &AstTypeRef,
    callable: &AstFunction,
    generic_names: &BTreeSet<String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<BTreeMap<String, AstTypeRef>, String> {
    let Some(arity) = super::callables::callable_type_arity(expected_callable_ty) else {
        return Ok(BTreeMap::new());
    };
    if callable.params.len() < arity {
        return Ok(BTreeMap::new());
    }
    let Some(callable_return_ty) = callable.return_type.as_ref() else {
        return Ok(BTreeMap::new());
    };
    let callable_generic_names = callable
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let expected_parts = expected_callable_ty.generic_args[..arity]
        .iter()
        .chain(std::iter::once(&expected_callable_ty.generic_args[arity]))
        .collect::<Vec<_>>();
    let callable_parts = callable
        .params
        .iter()
        .take(arity)
        .map(|param| resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases))
        .chain(std::iter::once(resolve_ast_type_ref_aliases(
            callable_return_ty,
            visible_type_aliases,
        )))
        .collect::<Result<Vec<_>, _>>()?;
    let callable_part_refs = callable_parts.iter().collect::<Vec<_>>();

    let callable_substitutions = infer_callable_generic_substitutions_from_expected_parts(
        &expected_parts,
        &callable_part_refs,
        &callable_generic_names,
        generic_names,
        &callable.name,
    )?;

    let specialized_callable_parts = callable_part_refs
        .iter()
        .map(|part| specialize_type_with_substitutions(part, &callable_substitutions))
        .collect::<Vec<_>>();

    let mut substitutions = BTreeMap::new();
    for (expected_part, callable_part) in expected_parts.iter().zip(specialized_callable_parts) {
        if contains_unresolved_template_generic(&callable_part, &callable_generic_names) {
            continue;
        }
        if contains_unresolved_template_generic(expected_part, generic_names) {
            unify_generic_type_pattern(
                expected_part,
                &callable_part,
                generic_names,
                &mut substitutions,
                &callable.name,
            )?;
        }
    }

    Ok(substitutions)
}

pub(super) fn infer_callable_generic_substitutions(
    expected_callable_ty: &AstTypeRef,
    callable: &AstFunction,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<BTreeMap<String, AstTypeRef>, String> {
    let Some(arity) = super::callables::callable_type_arity(expected_callable_ty) else {
        return Ok(BTreeMap::new());
    };
    if callable.params.len() < arity {
        return Ok(BTreeMap::new());
    }
    let Some(callable_return_ty) = callable.return_type.as_ref() else {
        return Ok(BTreeMap::new());
    };
    let callable_generic_names = callable
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let expected_parts = expected_callable_ty.generic_args[..arity]
        .iter()
        .chain(std::iter::once(&expected_callable_ty.generic_args[arity]))
        .collect::<Vec<_>>();
    let callable_parts = callable
        .params
        .iter()
        .take(arity)
        .map(|param| resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases))
        .chain(std::iter::once(resolve_ast_type_ref_aliases(
            callable_return_ty,
            visible_type_aliases,
        )))
        .collect::<Result<Vec<_>, _>>()?;
    let callable_part_refs = callable_parts.iter().collect::<Vec<_>>();
    infer_callable_generic_substitutions_from_expected_parts(
        &expected_parts,
        &callable_part_refs,
        &callable_generic_names,
        &BTreeSet::new(),
        &callable.name,
    )
}

fn infer_callable_generic_substitutions_from_expected_parts(
    expected_parts: &[&AstTypeRef],
    callable_parts: &[&AstTypeRef],
    callable_generic_names: &BTreeSet<String>,
    unresolved_expected_generic_names: &BTreeSet<String>,
    callable_name: &str,
) -> Result<BTreeMap<String, AstTypeRef>, String> {
    let mut callable_substitutions = BTreeMap::new();
    for (expected_part, callable_part) in expected_parts.iter().zip(callable_parts.iter()) {
        if contains_unresolved_template_generic(expected_part, unresolved_expected_generic_names) {
            continue;
        }
        if contains_unresolved_template_generic(callable_part, callable_generic_names) {
            unify_generic_type_pattern(
                callable_part,
                expected_part,
                callable_generic_names,
                &mut callable_substitutions,
                callable_name,
            )?;
        }
    }
    Ok(callable_substitutions)
}

pub(super) fn contains_unresolved_template_generic(
    ty: &AstTypeRef,
    generic_names: &BTreeSet<String>,
) -> bool {
    (ty.generic_args.is_empty() && generic_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| contains_unresolved_template_generic(arg, generic_names))
}

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
    )
}

pub(super) fn type_ref_looks_unresolved_placeholder(ty: &AstTypeRef) -> bool {
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

pub(super) fn type_ref_contains_unresolved_placeholder(ty: &AstTypeRef) -> bool {
    type_ref_looks_unresolved_placeholder(ty)
        || ty
            .generic_args
            .iter()
            .any(type_ref_contains_unresolved_placeholder)
}
