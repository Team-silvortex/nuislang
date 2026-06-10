use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::generics::{
    infer_generic_substitutions, specialize_ast_type_ref, specialize_function_template,
    unify_generic_type_pattern,
};
use super::super::types::{ast_type_from_nir, infer_ast_expr_type};
use super::super::{
    lower_type_ref, lower_type_ref_with_aliases, resolve_ast_type_ref_aliases,
    substitute_ast_type_alias_target, FunctionSignature,
};
use crate::frontend::generic_rewrite::rewrite_generic_calls_in_function;

pub(super) fn rewrite_generic_calls_in_expr(
    expr: &AstExpr,
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_generic_calls_in_expr(
            value,
            expected,
            env,
            visible_type_aliases,
            generic_templates,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?)),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            let rewritten_args = args
                .iter()
                .enumerate()
                .map(|(index, arg)| {
                    let arg_expected = call_arg_expected_type(
                        callee,
                        generic_args,
                        index,
                        expected,
                        generic_templates,
                        signatures,
                        visible_type_aliases,
                        struct_table,
                    );
                    rewrite_generic_calls_in_expr(
                        arg,
                        arg_expected.as_ref(),
                        env,
                        visible_type_aliases,
                        generic_templates,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(template) = generic_templates.get(callee) {
                let specialized_name = ensure_generic_specialization(
                    template,
                    &rewritten_args,
                    expected,
                    env,
                    visible_type_aliases,
                    generic_templates,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?;
                AstExpr::Call {
                    callee: specialized_name,
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                }
            } else {
                let rewritten_callee = resolved_struct_constructor_alias(
                    callee,
                    generic_args,
                    expected,
                    &rewritten_args,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )?
                .unwrap_or_else(|| (callee.clone(), generic_args.clone()));
                AstExpr::Call {
                    callee: rewritten_callee.0,
                    generic_args: rewritten_callee.1,
                    args: rewritten_args,
                }
            }
        }
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(rewrite_generic_calls_in_expr(
                receiver,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_generic_calls_in_expr(
                        arg,
                        None,
                        env,
                        visible_type_aliases,
                        generic_templates,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let rewritten_head = resolved_struct_literal_alias(
                type_name,
                type_args,
                expected,
                fields,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            )?
            .unwrap_or_else(|| (type_name.clone(), type_args.clone()));
            AstExpr::StructLiteral {
                type_name: rewritten_head.0.clone(),
                type_args: rewritten_head.1.clone(),
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        let literal_ty = expected
                            .filter(|ty| ty.name == rewritten_head.0)
                            .cloned()
                            .unwrap_or_else(|| AstTypeRef {
                                name: rewritten_head.0.clone(),
                                generic_args: rewritten_head.1.clone(),
                                is_optional: false,
                                is_ref: false,
                            });
                        let field_expected =
                            struct_field_expected_type(&literal_ty, name, struct_table);
                        Ok((
                            name.clone(),
                            rewrite_generic_calls_in_expr(
                                value,
                                field_expected.as_ref(),
                                env,
                                visible_type_aliases,
                                generic_templates,
                                signatures,
                                impl_lookup,
                                struct_table,
                                function_return_types,
                                specialization_cache,
                                specialized_functions,
                                specialized_signatures,
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }
        }
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_generic_calls_in_expr(
                base,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?),
            field: field.clone(),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_generic_calls_in_expr(
                lhs,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?),
            rhs: Box::new(rewrite_generic_calls_in_expr(
                rhs,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?),
        },
        other => other.clone(),
    })
}

fn resolved_struct_constructor_alias(
    callee: &str,
    generic_args: &[AstTypeRef],
    expected: Option<&AstTypeRef>,
    args: &[AstExpr],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    if generic_args.is_empty() {
        if let Some(from_expected) = infer_alias_struct_target_from_expected(
            callee,
            expected,
            visible_type_aliases,
            struct_table,
        )? {
            return Ok(Some(from_expected));
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
            if generic_args.is_empty() {
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

fn resolved_struct_literal_alias(
    type_name: &str,
    type_args: &[AstTypeRef],
    expected: Option<&AstTypeRef>,
    fields: &[(String, AstExpr)],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    if type_args.is_empty() {
        if let Some(from_expected) = infer_alias_struct_target_from_expected(
            type_name,
            expected,
            visible_type_aliases,
            struct_table,
        )? {
            return Ok(Some(from_expected));
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
            if type_args.is_empty() {
                if let Some(inferred) = infer_alias_struct_literal_type_from_fields(
                    type_name,
                    fields,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )? {
                    return Ok(Some(inferred));
                }
            }
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
    let Some(arg_ty) = infer_ast_expr_type(
        &args[0],
        env,
        impl_lookup,
        struct_table,
        function_return_types,
    ) else {
        return Err(format!(
            "payload-style alias constructor `{alias_name}(...)` is not yet supported for generic alias target `{}`; current frontend could not infer the payload type strongly enough",
            lower_type_ref(&alias_definition.target).render()
        ));
    };
    infer_alias_struct_target_from_usage(
        alias_name,
        alias_definition,
        &[field_pattern],
        &[arg_ty],
        visible_type_aliases,
        struct_table,
    )
}

#[allow(clippy::too_many_arguments)]
fn infer_alias_struct_literal_type_from_fields(
    alias_name: &str,
    fields: &[(String, AstExpr)],
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
    let mut concrete_types = Vec::new();
    for (name, value) in fields {
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
        let Some(value_ty) =
            infer_ast_expr_type(value, env, impl_lookup, struct_table, function_return_types)
        else {
            return Err(format!(
                "struct literal alias `{alias_name}` is not yet supported for generic alias target `{}`; current frontend could not infer field `{name}` strongly enough",
                lower_type_ref(&alias_definition.target).render()
            ));
        };
        field_patterns.push(pattern);
        concrete_types.push(value_ty);
    }
    infer_alias_struct_target_from_usage(
        alias_name,
        alias_definition,
        &field_patterns,
        &concrete_types,
        visible_type_aliases,
        struct_table,
    )
}

fn infer_alias_struct_target_from_usage(
    alias_name: &str,
    alias_definition: &AstTypeAlias,
    patterns: &[AstTypeRef],
    concretes: &[AstTypeRef],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    for (pattern, concrete) in patterns.iter().zip(concretes) {
        if let Err(error) = unify_generic_type_pattern(
            pattern,
            concrete,
            &generic_names,
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
                lower_type_ref(concrete).render()
            ));
        }
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

fn infer_alias_struct_target_from_expected(
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
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            format!(
                "generic alias `{alias_name}` could not be fully determined from expected type `{}`",
                lower_type_ref(expected).render()
            )
        })?;
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

pub(super) fn ensure_generic_specialization(
    template: &AstFunction,
    args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<String, String> {
    let substitutions = infer_generic_substitutions(
        template,
        args,
        expected,
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
    )?;
    let specialized_name = format!(
        "{}__{}",
        template.name,
        template
            .generic_params
            .iter()
            .map(|param| substitutions[&param.name]
                .render()
                .replace(|ch: char| !ch.is_ascii_alphanumeric(), "_"))
            .collect::<Vec<_>>()
            .join("__")
    );
    if specialization_cache.insert(specialized_name.clone()) {
        let specialized =
            specialize_function_template(template, &specialized_name, &substitutions)?;
        let rewritten = rewrite_generic_calls_in_function(
            &specialized,
            &BTreeMap::new(),
            visible_type_aliases,
            generic_templates,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?;
        specialized_signatures.push((
            specialized_name.clone(),
            FunctionSignature {
                abi: "nuis".to_owned(),
                interface: None,
                symbol_name: specialized_name.clone(),
                params: rewritten
                    .params
                    .iter()
                    .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: rewritten
                    .return_type
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                    .transpose()?,
                is_extern: false,
                is_async: rewritten.is_async,
            },
        ));
        specialized_functions.push(rewritten);
    }
    Ok(specialized_name)
}

pub(super) fn call_arg_expected_type(
    callee: &str,
    generic_args: &[AstTypeRef],
    index: usize,
    expected: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    if let Some(from_template_expected) = generic_template_arg_expected_type_from_return(
        callee,
        index,
        expected,
        generic_templates,
        visible_type_aliases,
    ) {
        return Some(from_template_expected);
    }
    if let Some(from_signature_expected) = generic_signature_arg_expected_type_from_return(
        callee,
        index,
        expected,
        signatures,
        visible_type_aliases,
        struct_table,
    ) {
        return Some(from_signature_expected);
    }
    if let Some(from_signature) = signatures
        .get(callee)
        .and_then(|signature| signature.params.get(index))
        .map(ast_type_from_nir)
    {
        return Some(from_signature);
    }
    constructor_value_expected_type(
        callee,
        generic_args,
        index,
        expected,
        visible_type_aliases,
        struct_table,
    )
}

fn generic_template_arg_expected_type_from_return(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    let template = generic_templates.get(callee)?;
    let return_pattern = template.return_type.as_ref()?;
    let param = template.params.get(index)?;
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    if unify_generic_type_pattern(
        return_pattern,
        expected,
        &generic_names,
        &mut substitutions,
        &template.name,
    )
    .is_err()
    {
        let resolved_return_pattern =
            resolve_ast_type_ref_aliases(return_pattern, visible_type_aliases).ok()?;
        let resolved_expected =
            resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
        substitutions.clear();
        unify_generic_type_pattern(
            &resolved_return_pattern,
            &resolved_expected,
            &generic_names,
            &mut substitutions,
            &template.name,
        )
        .ok()?;
    }
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok()
}

fn generic_signature_arg_expected_type_from_return(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    let signature = signatures.get(callee)?;
    let return_ty = ast_type_from_nir(signature.return_type.as_ref()?);
    let param_ty = ast_type_from_nir(signature.params.get(index)?);
    let mut generic_names = BTreeSet::new();
    collect_signature_generic_placeholders(
        &return_ty,
        visible_type_aliases,
        struct_table,
        &mut generic_names,
    );
    collect_signature_generic_placeholders(
        &param_ty,
        visible_type_aliases,
        struct_table,
        &mut generic_names,
    );
    if generic_names.is_empty() {
        return None;
    }
    let resolved_return_pattern =
        resolve_ast_type_ref_aliases(&return_ty, visible_type_aliases).ok()?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    unify_generic_type_pattern(
        &resolved_return_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        callee,
    )
    .ok()?;
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    specialize_ast_type_ref(&param_ty, &lowered_substitutions).ok()
}

fn collect_signature_generic_placeholders(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
    out: &mut BTreeSet<String>,
) {
    if ty.generic_args.is_empty()
        && ty
            .name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        && !visible_type_aliases.contains_key(&ty.name)
        && !struct_table.contains_key(&ty.name)
        && ty.name != "String"
    {
        out.insert(ty.name.clone());
    }
    for arg in &ty.generic_args {
        collect_signature_generic_placeholders(arg, visible_type_aliases, struct_table, out);
    }
}

fn constructor_value_expected_type(
    callee: &str,
    generic_args: &[AstTypeRef],
    index: usize,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    if index != 0 {
        return None;
    }
    let concrete_head = if let Some(definition) = struct_table.get(callee) {
        if definition.fields.len() != 1 {
            return None;
        }
        if generic_args.is_empty() {
            expected.filter(|expected| expected.name == callee).cloned()
        } else {
            Some(AstTypeRef {
                name: callee.to_owned(),
                generic_args: generic_args.to_vec(),
                is_optional: false,
                is_ref: false,
            })
        }
    } else if generic_args.is_empty() {
        infer_alias_struct_target_from_expected(
            callee,
            expected,
            visible_type_aliases,
            struct_table,
        )
        .ok()
        .flatten()
        .map(|(name, generic_args)| AstTypeRef {
            name,
            generic_args,
            is_optional: false,
            is_ref: false,
        })
    } else {
        let alias_definition = visible_type_aliases.get(callee)?;
        let substituted = substitute_ast_type_alias_target(
            &alias_definition.target,
            &alias_definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .zip(generic_args.iter().cloned())
                .collect::<BTreeMap<_, _>>(),
        )
        .ok()?;
        let resolved = resolve_ast_type_ref_aliases(&substituted, visible_type_aliases).ok()?;
        Some(resolved)
    }?;
    let definition = struct_table.get(&concrete_head.name)?;
    if definition.fields.len() != 1 {
        return None;
    }
    Some(
        super::super::validation_binding_env::instantiate_ast_struct_field_type(
            &concrete_head,
            definition,
            &definition.fields[0].ty,
        ),
    )
}

fn struct_field_expected_type(
    literal_ty: &AstTypeRef,
    field_name: &str,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let definition = struct_table.get(&literal_ty.name)?;
    let field = definition
        .fields
        .iter()
        .find(|field| field.name == field_name)?;
    Some(
        super::super::validation_binding_env::instantiate_ast_struct_field_type(
            literal_ty, definition, &field.ty,
        ),
    )
}
