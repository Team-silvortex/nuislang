use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef};

use super::super::generics::{infer_alias_aware_ast_expr_type, unify_generic_type_pattern};
use super::super::types::infer_ast_expr_type_for_pattern;
use super::super::{
    lower_type_ref, resolve_ast_type_ref_aliases, substitute_ast_type_alias_target,
};
use super::exprs_alias_expected::{
    ast_type_args_are_placeholder_generics, infer_alias_struct_target_from_expected,
    seed_alias_generic_substitutions_from_expected_pattern,
};
pub(super) use super::exprs_alias_inputs::{
    MethodCallReceiverExpectedTypeInput, StructConstructorAliasInput, StructLiteralAliasInput,
};

pub(super) fn concrete_struct_literal_type(
    rewritten_name: &str,
    rewritten_args: &[AstTypeRef],
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    if expected.name == rewritten_name {
        return Some(expected.clone());
    }
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    if resolved_expected.name == rewritten_name {
        return Some(resolved_expected);
    }
    if !rewritten_args.is_empty() {
        return Some(AstTypeRef {
            name: rewritten_name.to_owned(),
            generic_args: rewritten_args.to_vec(),
            is_optional: false,
            is_ref: false,
        });
    }
    None
}

pub(super) fn resolved_struct_constructor_alias(
    input: StructConstructorAliasInput<'_>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let StructConstructorAliasInput {
        callee,
        generic_args,
        expected,
        args,
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
    } = input;
    let alias_placeholder_names = visible_type_aliases
        .get(callee)
        .map(|alias| {
            alias
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let has_placeholder_generic_args =
        ast_type_args_are_placeholder_generics(generic_args, &alias_placeholder_names);
    if has_placeholder_generic_args {
        if let Some(from_expected) = infer_alias_struct_target_from_expected(
            callee,
            expected,
            visible_type_aliases,
            struct_table,
        )? {
            return Ok(Some(from_expected));
        }
        if let Some(inferred) = infer_alias_struct_constructor_type_from_args(
            callee,
            args,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
        )? {
            return Ok(Some(inferred));
        }
    }
    let type_ref = AstTypeRef {
        name: callee.to_owned(),
        generic_args: generic_args.to_vec(),
        is_optional: false,
        is_ref: false,
    };
    let resolved = match resolve_ast_type_ref_aliases(&type_ref, visible_type_aliases) {
        Ok(resolved) => resolved,
        Err(error) => {
            if has_placeholder_generic_args {
                if let Some(inferred) = infer_alias_struct_constructor_type_from_args(
                    callee,
                    args,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )? {
                    return Ok(Some(inferred));
                }
            }
            if visible_type_aliases.contains_key(callee) {
                return Err(error);
            }
            return Ok(None);
        }
    };
    Ok(struct_table
        .contains_key(&resolved.name)
        .then_some((resolved.name, resolved.generic_args)))
}

pub(super) fn resolved_struct_literal_alias(
    input: StructLiteralAliasInput<'_>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let StructLiteralAliasInput {
        type_name,
        type_args,
        expected,
        fields,
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
    } = input;
    let alias_placeholder_names = visible_type_aliases
        .get(type_name)
        .map(|alias| {
            alias
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let has_placeholder_type_args =
        ast_type_args_are_placeholder_generics(type_args, &alias_placeholder_names);
    if has_placeholder_type_args {
        if let Some(from_expected) = infer_alias_struct_target_from_expected(
            type_name,
            expected,
            visible_type_aliases,
            struct_table,
        )? {
            return Ok(Some(from_expected));
        }
        if let Some(inferred) = infer_alias_struct_literal_type_from_fields(
            type_name,
            fields,
            expected,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
        )? {
            return Ok(Some(inferred));
        }
    }
    let type_ref = AstTypeRef {
        name: type_name.to_owned(),
        generic_args: type_args.to_vec(),
        is_optional: false,
        is_ref: false,
    };
    let resolved = match resolve_ast_type_ref_aliases(&type_ref, visible_type_aliases) {
        Ok(resolved) => resolved,
        Err(error) => {
            if visible_type_aliases.contains_key(type_name) {
                return Err(error);
            }
            return Ok(None);
        }
    };
    Ok(struct_table
        .contains_key(&resolved.name)
        .then_some((resolved.name, resolved.generic_args)))
}

pub(super) fn method_call_receiver_expected_type(
    input: MethodCallReceiverExpectedTypeInput<'_>,
) -> Option<AstTypeRef> {
    let MethodCallReceiverExpectedTypeInput {
        receiver,
        method,
        generic_args,
        args,
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
    } = input;
    if let Some(explicit) = super::super::receiver_expected::explicit_receiver_expected_type(
        receiver,
        generic_args,
        visible_type_aliases,
    ) {
        return Some(explicit);
    }

    let arg_types = args
        .iter()
        .map(|arg| {
            infer_alias_aware_ast_expr_type(
                arg,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            )
            .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok())
        })
        .collect::<Option<Vec<_>>>()?;

    let mut candidates =
        impl_lookup
            .values()
            .filter_map(|definition| {
                let method_def = definition
                    .methods
                    .iter()
                    .find(|item| item.name == *method)?;
                if method_def.params.len() != arg_types.len() + 1 {
                    return None;
                }
                let params_match = method_def.params.iter().skip(1).zip(arg_types.iter()).all(
                    |(param, arg_ty)| {
                        resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)
                            .ok()
                            .is_some_and(|resolved| {
                                lower_type_ref(&resolved).render()
                                    == lower_type_ref(arg_ty).render()
                            })
                    },
                );
                params_match.then(|| definition.for_type.clone())
            })
            .collect::<Vec<_>>();
    if candidates.len() == 1 {
        let candidate = candidates.pop()?;
        if !generic_args.is_empty() && candidate.generic_args.len() == generic_args.len() {
            return Some(AstTypeRef {
                name: candidate.name,
                generic_args: generic_args.to_vec(),
                is_optional: candidate.is_optional,
                is_ref: candidate.is_ref,
            });
        }
        return Some(candidate);
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn infer_alias_struct_constructor_type_from_args(
    alias_name: &str,
    args: &[AstExpr],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let Some(alias_definition) = visible_type_aliases.get(alias_name) else {
        return Ok(None);
    };
    if alias_definition.generic_params.is_empty() {
        return Ok(None);
    }
    let resolved_target_pattern =
        resolve_ast_type_ref_aliases(&alias_definition.target, visible_type_aliases)?;
    let Some(target_definition) = struct_table.get(&resolved_target_pattern.name) else {
        return Err(format!(
            "payload-style alias constructor `{alias_name}(...)` is not yet supported for generic alias target `{}`; current frontend only supports aliases that resolve to struct targets",
            lower_type_ref(&alias_definition.target).render()
        ));
    };
    if target_definition.fields.len() != 1 || args.len() != 1 {
        return Ok(None);
    }
    let field_pattern = super::super::validation_binding_env::instantiate_ast_struct_field_type(
        &resolved_target_pattern,
        target_definition,
        &target_definition.fields[0].ty,
    );
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    infer_alias_struct_target_from_usage_seeded(
        alias_name,
        alias_definition,
        &[field_pattern],
        &[args[0].clone()],
        visible_type_aliases,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        &generic_names,
        BTreeMap::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn infer_alias_struct_literal_type_from_fields(
    alias_name: &str,
    fields: &[(String, AstExpr)],
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let Some(alias_definition) = visible_type_aliases.get(alias_name) else {
        return Ok(None);
    };
    if alias_definition.generic_params.is_empty() {
        return Ok(None);
    }
    let resolved_target_pattern =
        resolve_ast_type_ref_aliases(&alias_definition.target, visible_type_aliases)?;
    let Some(target_definition) = struct_table.get(&resolved_target_pattern.name) else {
        return Err(format!(
            "struct literal alias `{alias_name}` is not yet supported for generic alias target `{}`; current frontend only supports aliases that resolve to struct targets",
            lower_type_ref(&alias_definition.target).render()
        ));
    };
    let mut field_patterns = Vec::new();
    for (name, _value) in fields {
        let Some(field) = target_definition
            .fields
            .iter()
            .find(|field| field.name == *name)
        else {
            return Ok(None);
        };
        let pattern = super::super::validation_binding_env::instantiate_ast_struct_field_type(
            &resolved_target_pattern,
            target_definition,
            &field.ty,
        );
        field_patterns.push(pattern);
    }
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let seed = seed_alias_generic_substitutions_from_expected_pattern(
        alias_name,
        alias_definition,
        &resolved_target_pattern,
        expected,
        visible_type_aliases,
        &generic_names,
    )?;
    infer_alias_struct_target_from_usage_seeded(
        alias_name,
        alias_definition,
        &field_patterns,
        &fields
            .iter()
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>(),
        visible_type_aliases,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        &generic_names,
        seed,
    )
}

#[allow(clippy::too_many_arguments)]
fn infer_alias_struct_target_from_usage_seeded(
    alias_name: &str,
    alias_definition: &AstTypeAlias,
    patterns: &[AstTypeRef],
    concrete_exprs: &[AstExpr],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    generic_names: &BTreeSet<String>,
    mut substitutions: BTreeMap<String, AstTypeRef>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let mut pending = patterns
        .iter()
        .zip(concrete_exprs.iter())
        .collect::<Vec<_>>();
    while !pending.is_empty() {
        let mut progress = false;
        let mut next_pending = Vec::new();
        for (pattern, concrete_expr) in pending {
            let expected_pattern =
                specialize_ast_type_pattern_with_known_substitutions(pattern, &substitutions);
            let concrete = infer_alias_aware_ast_expr_type_for_pattern(
                concrete_expr,
                &expected_pattern,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            )
            .or_else(|| {
                infer_ast_expr_type_for_pattern(
                    concrete_expr,
                    &expected_pattern,
                    generic_names,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
            })
            .or_else(|| {
                infer_alias_aware_ast_expr_type(
                    concrete_expr,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
            });
            let Some(concrete) = concrete else {
                next_pending.push((pattern, concrete_expr));
                continue;
            };
            if let Err(error) = unify_generic_type_pattern(
                pattern,
                &concrete,
                generic_names,
                &mut substitutions,
                alias_name,
            ) {
                if error.contains("resolved to conflicting types") {
                    return Err(format!(
                        "generic alias constructor `{alias_name}` inferred conflicting types while matching target `{}`",
                        lower_type_ref(&alias_definition.target).render()
                    ));
                }
                return Err(format!(
                    "generic alias constructor `{alias_name}` could not match target field shape `{}` with concrete type `{}`",
                    lower_type_ref(pattern).render(),
                    lower_type_ref(&concrete).render()
                ));
            }
            progress = true;
        }
        if !progress {
            return Err(format!(
                "generic alias constructor `{alias_name}` could not be inferred from current field/payload usage for target `{}`; add explicit type arguments or a stronger expected type",
                lower_type_ref(&alias_definition.target).render()
            ));
        }
        pending = next_pending;
    }
    let mut generic_args = Vec::new();
    for param in &alias_definition.generic_params {
        let Some(argument) = substitutions.get(&param.name).cloned() else {
            return Err(format!(
                "generic alias constructor `{alias_name}` could not infer generic parameter `{}` for target `{}`; add explicit type arguments or a stronger expected type",
                param.name,
                lower_type_ref(&alias_definition.target).render()
            ));
        };
        generic_args.push(argument);
    }
    let substituted = substitute_ast_type_alias_target(
        &alias_definition.target,
        &alias_definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .zip(generic_args)
            .collect::<BTreeMap<_, _>>(),
    )?;
    let resolved = resolve_ast_type_ref_aliases(&substituted, visible_type_aliases)?;
    Ok(struct_table
        .contains_key(&resolved.name)
        .then_some((resolved.name, resolved.generic_args)))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn infer_alias_aware_ast_expr_type_for_pattern(
    expr: &AstExpr,
    expected_pattern: &AstTypeRef,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    match expr {
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            if visible_type_aliases.contains_key(type_name) {
                if let Ok(Some((resolved_name, resolved_args))) =
                    resolved_struct_literal_alias(StructLiteralAliasInput {
                        type_name,
                        type_args,
                        expected: Some(expected_pattern),
                        fields,
                        env,
                        visible_type_aliases,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                    })
                {
                    return Some(AstTypeRef {
                        name: resolved_name,
                        generic_args: resolved_args,
                        is_optional: false,
                        is_ref: false,
                    });
                }
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            if visible_type_aliases.contains_key(callee) {
                if let Ok(Some((resolved_name, resolved_args))) =
                    resolved_struct_constructor_alias(StructConstructorAliasInput {
                        callee,
                        generic_args,
                        expected: Some(expected_pattern),
                        args,
                        env,
                        visible_type_aliases,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                    })
                {
                    return Some(AstTypeRef {
                        name: resolved_name,
                        generic_args: resolved_args,
                        is_optional: false,
                        is_ref: false,
                    });
                }
            }
        }
        _ => {}
    }
    None
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
