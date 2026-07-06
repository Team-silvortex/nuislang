use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstImplDef, AstStructDef, AstTypeRef};

use super::ast_infer::infer_ast_expr_type_inner;
use super::{ast_generic_named_type, ast_named_type};
use crate::frontend::lower_type_ref;

pub(crate) fn infer_ast_expr_type_for_pattern(
    expr: &AstExpr,
    expected_pattern: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    infer_ast_expr_type_for_pattern_inner(PatternExprInferenceInput {
        expr,
        expected_pattern,
        placeholder_names,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs: &mut BTreeSet::new(),
    })
}

pub(super) fn type_args_are_pattern_placeholders(
    type_args: &[AstTypeRef],
    placeholder_names: &BTreeSet<String>,
) -> bool {
    type_args.is_empty()
        || type_args
            .iter()
            .all(|arg| contains_ast_placeholder_generic_name(arg, placeholder_names))
}

struct PatternExprInferenceInput<'a> {
    expr: &'a AstExpr,
    expected_pattern: &'a AstTypeRef,
    placeholder_names: &'a BTreeSet<String>,
    env: &'a BTreeMap<String, AstTypeRef>,
    impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    struct_table: &'a BTreeMap<String, AstStructDef>,
    function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &'a mut BTreeSet<usize>,
}

fn infer_ast_expr_type_for_pattern_inner(
    input: PatternExprInferenceInput<'_>,
) -> Option<AstTypeRef> {
    let PatternExprInferenceInput {
        expr,
        expected_pattern,
        placeholder_names,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    } = input;
    match expr {
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } if expected_pattern.name == *type_name => {
            let definition = struct_table.get(type_name)?;
            if !type_args_are_pattern_placeholders(type_args, placeholder_names) {
                return infer_ast_expr_type_inner(
                    expr,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                );
            }
            let generic_names = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            let seed = seed_ast_generic_substitutions_from_expected(
                definition,
                expected_pattern,
                placeholder_names,
            );
            infer_struct_literal_ast_type_seeded(
                type_name,
                definition,
                fields,
                &generic_names,
                seed,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if expected_pattern.name == *callee => {
            let definition = struct_table.get(callee)?;
            if !type_args_are_pattern_placeholders(generic_args, placeholder_names) {
                return infer_ast_expr_type_inner(
                    expr,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                );
            }
            if definition.fields.len() != 1 || args.len() != 1 {
                return None;
            }
            let generic_names = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            let seed = seed_ast_generic_substitutions_from_expected(
                definition,
                expected_pattern,
                placeholder_names,
            );
            infer_payload_constructor_ast_type_seeded(
                callee,
                definition,
                &args[0],
                &generic_names,
                seed,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )
        }
        _ => infer_ast_expr_type_inner(
            expr,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn infer_struct_literal_ast_type_seeded(
    type_name: &str,
    definition: &AstStructDef,
    fields: &[(String, AstExpr)],
    generic_names: &BTreeSet<String>,
    mut substitutions: BTreeMap<String, AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let mut pending = fields
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .collect::<Vec<_>>();
    while !pending.is_empty() {
        let mut progress = false;
        let mut next_pending = Vec::new();
        for (name, value) in pending {
            let field = definition.fields.iter().find(|field| field.name == name)?;
            let field_pattern =
                specialize_ast_type_pattern_with_known_substitutions(&field.ty, &substitutions);
            let value_ty = infer_ast_expr_type_for_pattern_inner(PatternExprInferenceInput {
                expr: value,
                expected_pattern: &field_pattern,
                placeholder_names: generic_names,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            });
            let Some(value_ty) = value_ty else {
                next_pending.push((name, value));
                continue;
            };
            unify_ast_generic_type_pattern(&field.ty, &value_ty, generic_names, &mut substitutions)
                .ok()?;
            progress = true;
        }
        if !progress {
            return None;
        }
        pending = next_pending;
    }
    let generic_args = definition
        .generic_params
        .iter()
        .map(|param| {
            substitutions
                .get(&param.name)
                .cloned()
                .unwrap_or_else(|| ast_named_type(&param.name))
        })
        .collect::<Vec<_>>();
    Some(ast_generic_named_type(type_name, generic_args))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn infer_payload_constructor_ast_type_seeded(
    callee: &str,
    definition: &AstStructDef,
    arg: &AstExpr,
    generic_names: &BTreeSet<String>,
    mut substitutions: BTreeMap<String, AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let field_pattern = specialize_ast_type_pattern_with_known_substitutions(
        &definition.fields[0].ty,
        &substitutions,
    );
    let arg_ty = infer_ast_expr_type_for_pattern_inner(PatternExprInferenceInput {
        expr: arg,
        expected_pattern: &field_pattern,
        placeholder_names: generic_names,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    })?;
    unify_ast_generic_type_pattern(
        &definition.fields[0].ty,
        &arg_ty,
        generic_names,
        &mut substitutions,
    )
    .ok()?;
    let generic_args = definition
        .generic_params
        .iter()
        .map(|param| {
            substitutions
                .get(&param.name)
                .cloned()
                .unwrap_or_else(|| ast_named_type(&param.name))
        })
        .collect::<Vec<_>>();
    Some(ast_generic_named_type(callee, generic_args))
}

fn specialize_ast_type_pattern_with_known_substitutions(
    pattern: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if pattern.generic_args.is_empty()
        && !pattern.is_optional
        && !pattern.is_ref
        && substitutions.contains_key(&pattern.name)
    {
        return substitutions
            .get(&pattern.name)
            .cloned()
            .unwrap_or_else(|| pattern.clone());
    }
    AstTypeRef {
        name: pattern.name.clone(),
        generic_args: pattern
            .generic_args
            .iter()
            .map(|arg| specialize_ast_type_pattern_with_known_substitutions(arg, substitutions))
            .collect(),
        is_optional: pattern.is_optional,
        is_ref: pattern.is_ref,
    }
}

fn seed_ast_generic_substitutions_from_expected(
    definition: &AstStructDef,
    expected_pattern: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> BTreeMap<String, AstTypeRef> {
    if expected_pattern.name != definition.name
        || expected_pattern.generic_args.len() != definition.generic_params.len()
    {
        return BTreeMap::new();
    }
    definition
        .generic_params
        .iter()
        .zip(&expected_pattern.generic_args)
        .filter_map(|(param, arg)| {
            (!contains_ast_placeholder_generic_name(arg, placeholder_names))
                .then_some((param.name.clone(), arg.clone()))
        })
        .collect()
}

fn contains_ast_placeholder_generic_name(
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

fn unify_ast_generic_type_pattern(
    pattern: &AstTypeRef,
    concrete: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    substitutions: &mut BTreeMap<String, AstTypeRef>,
) -> Result<(), ()> {
    if generic_names.contains(&pattern.name) && pattern.generic_args.is_empty() {
        if generic_names.contains(&concrete.name)
            && concrete.generic_args.is_empty()
            && !concrete.is_optional
            && !concrete.is_ref
        {
            return Ok(());
        }
        if let Some(existing) = substitutions.get(&pattern.name) {
            if lower_type_ref(existing).render() != lower_type_ref(concrete).render() {
                return Err(());
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
        return Err(());
    }
    for (pattern_arg, concrete_arg) in pattern.generic_args.iter().zip(&concrete.generic_args) {
        unify_ast_generic_type_pattern(pattern_arg, concrete_arg, generic_names, substitutions)?;
    }
    Ok(())
}
