use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstTypeAlias, AstTypeRef};

use super::super::generics::infer_alias_aware_ast_expr_type;
use super::super::{lower_type_ref, resolve_ast_type_ref_aliases};
use super::exprs_alias_expected::{
    ast_type_args_are_placeholder_generics, infer_alias_struct_target_from_expected,
};
pub(super) use super::exprs_alias_inputs::{
    AliasInferenceContext, MethodCallReceiverExpectedTypeInput, StructConstructorAliasInput,
    StructLiteralAliasInput,
};
use super::exprs_alias_usage::{
    infer_alias_struct_constructor_type_from_args, infer_alias_struct_literal_type_from_fields,
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
    let context = AliasInferenceContext {
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
    };
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
        if let Some(inferred) =
            infer_alias_struct_constructor_type_from_args(callee, args, context)?
        {
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
                if let Some(inferred) =
                    infer_alias_struct_constructor_type_from_args(callee, args, context)?
                {
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
    let context = AliasInferenceContext {
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
    };
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
        if let Some(inferred) =
            infer_alias_struct_literal_type_from_fields(type_name, fields, expected, context)?
        {
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
