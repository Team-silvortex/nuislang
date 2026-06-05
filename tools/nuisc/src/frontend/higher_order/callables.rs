use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstFunction, AstTypeAlias, AstTypeRef};

use super::super::types::ast_type_from_nir;
use super::super::{lower_type_ref_with_aliases, resolve_ast_type_ref_aliases};
use crate::frontend::generics::unify_generic_type_pattern;

pub(crate) fn callable_type_arity(ty: &AstTypeRef) -> Option<usize> {
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

pub(crate) fn is_callable_type(ty: &AstTypeRef) -> bool {
    callable_type_arity(ty).is_some()
}

pub(crate) fn is_callable_type_with_aliases(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<bool, String> {
    let resolved = resolve_ast_type_ref_aliases(ty, visible_type_aliases)?;
    Ok(is_callable_type(&resolved))
}

pub(crate) fn sanitize_symbol_fragment(name: &str) -> String {
    name.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '_',
        })
        .collect()
}

pub(crate) fn function_type_matches_callable(
    callable: &AstFunction,
    expected: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<bool, String> {
    let expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases)?;
    let Some(arity) = callable_type_arity(&expected) else {
        return Ok(false);
    };
    if callable.params.len() != arity {
        return Ok(false);
    }
    let Some(callable_return_type) = &callable.return_type else {
        return Ok(false);
    };
    let expected_parts = expected.generic_args[..arity]
        .iter()
        .chain(std::iter::once(&expected.generic_args[arity]))
        .map(|arg| {
            lower_type_ref_with_aliases(arg, visible_type_aliases).map(|ty| ast_type_from_nir(&ty))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let callable_parts = callable
        .params
        .iter()
        .map(|param| {
            lower_type_ref_with_aliases(&param.ty, visible_type_aliases)
                .map(|ty| ast_type_from_nir(&ty))
        })
        .chain(std::iter::once(
            lower_type_ref_with_aliases(callable_return_type, visible_type_aliases)
                .map(|ty| ast_type_from_nir(&ty)),
        ))
        .collect::<Result<Vec<_>, _>>()?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    for (expected_part, callable_part) in expected_parts.iter().zip(callable_parts.iter()) {
        unify_generic_type_pattern(
            expected_part,
            callable_part,
            generic_names,
            &mut substitutions,
            &callable.name,
        )?;
    }
    Ok(true)
}
