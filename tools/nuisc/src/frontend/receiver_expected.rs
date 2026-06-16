use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, AstStructDef, AstTypeAlias, AstTypeRef};

use super::{lower_type_ref, resolve_ast_type_ref_aliases};

pub(super) fn constructor_like_receiver_path(path: &str) -> bool {
    path.split('.')
        .next()
        .and_then(|head| head.chars().next())
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

pub(super) fn render_receiver_path(expr: &AstExpr) -> Option<String> {
    match expr {
        AstExpr::Var(name) => Some(name.clone()),
        AstExpr::FieldAccess { base, field } => {
            Some(format!("{}.{}", render_receiver_path(base)?, field))
        }
        _ => None,
    }
}

pub(super) fn explicit_receiver_expected_type(
    receiver: &AstExpr,
    generic_args: &[AstTypeRef],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    if generic_args.is_empty() {
        return None;
    }
    let name = receiver_expected_head_name(receiver)?;
    let expected = AstTypeRef {
        name,
        generic_args: generic_args.to_vec(),
        is_optional: false,
        is_ref: false,
    };
    resolve_ast_type_ref_aliases(&expected, visible_type_aliases)
        .ok()
        .or(Some(expected))
}

pub(super) fn receiver_expected_head_name(receiver: &AstExpr) -> Option<String> {
    match receiver {
        AstExpr::StructLiteral { type_name, .. } => Some(type_name.clone()),
        AstExpr::Call { callee, .. } if constructor_like_receiver_path(callee) => {
            Some(callee.clone())
        }
        _ => {
            let path = render_receiver_path(receiver)?;
            Some(
                path.rsplit_once('.')
                    .map(|(parent, _)| parent)
                    .unwrap_or(&path)
                    .to_owned(),
            )
        }
    }
}

pub(super) fn specialize_receiver_constructor_from_expected(
    receiver: &AstExpr,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> AstExpr {
    let Some(expected) = expected else {
        return receiver.clone();
    };
    match receiver {
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty() => {
            let Some((name, type_args)) = concrete_constructor_head_from_expected(
                callee,
                expected,
                visible_type_aliases,
                struct_table,
            ) else {
                return receiver.clone();
            };
            AstExpr::Call {
                callee: name,
                generic_args: type_args,
                args: args.clone(),
            }
        }
        _ => receiver.clone(),
    }
}

fn concrete_constructor_head_from_expected(
    callee: &str,
    expected: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<(String, Vec<AstTypeRef>)> {
    if let Some(definition) = struct_table.get(callee) {
        if definition.fields.len() != 1 {
            return None;
        }
        if expected.name == callee {
            return Some((callee.to_owned(), expected.generic_args.clone()));
        }
        let resolved_expected =
            resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
        if resolved_expected.name == callee {
            return Some((callee.to_owned(), resolved_expected.generic_args));
        }
        return None;
    }
    infer_alias_struct_target_from_expected(
        callee,
        Some(expected),
        visible_type_aliases,
        struct_table,
    )
    .ok()
    .flatten()
}

fn infer_alias_struct_target_from_expected(
    alias_name: &str,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let expected = match expected {
        Some(expected) => resolve_ast_type_ref_aliases(expected, visible_type_aliases)?,
        None => return Ok(None),
    };
    let Some(alias_definition) = visible_type_aliases.get(alias_name) else {
        return Ok(None);
    };
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<Vec<_>>();
    let mut substitutions = BTreeMap::new();
    if alias_definition.target.name == expected.name
        && alias_definition.target.generic_args.len() == expected.generic_args.len()
    {
        for (pattern, concrete) in alias_definition
            .target
            .generic_args
            .iter()
            .zip(expected.generic_args.iter())
        {
            if generic_names.contains(&pattern.name) && pattern.generic_args.is_empty() {
                substitutions.insert(pattern.name.clone(), concrete.clone());
            }
        }
    }
    if substitutions.len() != alias_definition.generic_params.len() {
        return Ok(None);
    }
    let resolved_target = resolve_ast_type_ref_aliases(
        &substitute_alias_target(&alias_definition.target, &substitutions),
        visible_type_aliases,
    )?;
    if !struct_table.contains_key(&resolved_target.name) {
        return Ok(None);
    }
    Ok(Some((resolved_target.name, resolved_target.generic_args)))
}

fn substitute_alias_target(
    target: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if target.generic_args.is_empty() && substitutions.contains_key(&target.name) {
        return substitutions
            .get(&target.name)
            .cloned()
            .unwrap_or_else(|| target.clone());
    }
    AstTypeRef {
        name: target.name.clone(),
        generic_args: target
            .generic_args
            .iter()
            .map(|arg| substitute_alias_target(arg, substitutions))
            .collect(),
        is_optional: target.is_optional,
        is_ref: target.is_ref,
    }
}

pub(super) fn explicit_receiver_expected_nir_type(
    receiver: &AstExpr,
    generic_args: &[AstTypeRef],
) -> Option<nuis_semantics::model::NirTypeRef> {
    let expected = explicit_receiver_expected_type(receiver, generic_args, &BTreeMap::new())?;
    Some(lower_type_ref(&expected))
}
