use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::generics::{infer_generic_substitutions, specialize_function_template};
use super::super::types::ast_type_from_nir;
use super::super::{lower_type_ref_with_aliases, FunctionSignature};
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
        AstExpr::Call { callee, args } => {
            let rewritten_args = args
                .iter()
                .enumerate()
                .map(|(index, arg)| {
                    let arg_expected = call_arg_expected_type(callee, index, signatures);
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
                    args: rewritten_args,
                }
            } else {
                AstExpr::Call {
                    callee: callee.clone(),
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
        AstExpr::StructLiteral { type_name, fields } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| {
                    let field_expected = struct_field_expected_type(type_name, name, struct_table);
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
        },
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

fn call_arg_expected_type<'a>(
    callee: &str,
    index: usize,
    signatures: &BTreeMap<String, FunctionSignature>,
) -> Option<AstTypeRef> {
    signatures
        .get(callee)
        .and_then(|signature| signature.params.get(index))
        .map(ast_type_from_nir)
}

fn struct_field_expected_type(
    type_name: &str,
    field_name: &str,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    struct_table
        .get(type_name)
        .and_then(|definition| {
            definition
                .fields
                .iter()
                .find(|field| field.name == field_name)
        })
        .map(|field| field.ty.clone())
}
