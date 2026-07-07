use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::FunctionSignature;
use super::exprs::{rewrite_generic_calls_in_expr, GenericExprRewriteInput};
use super::exprs_aliases::{
    concrete_struct_literal_type, resolved_struct_literal_alias, StructLiteralAliasInput,
};
use super::exprs_expected::{field_access_base_expected_type, struct_field_expected_type};
use super::exprs_specialization::generic_arg_contains_definition_placeholder;
use super::GenericImplMethodTemplate;

pub(super) struct GenericStructLiteralRewriteInput<'a> {
    pub(super) type_name: &'a str,
    pub(super) type_args: &'a [AstTypeRef],
    pub(super) fields: &'a [(String, AstExpr)],
    pub(super) context: &'a str,
    pub(super) expected: Option<&'a AstTypeRef>,
    pub(super) env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) generic_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) generic_impl_method_templates: &'a [GenericImplMethodTemplate],
    pub(super) higher_order_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) function_table: &'a BTreeMap<String, AstFunction>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) specialization_cache: &'a mut BTreeSet<String>,
    pub(super) specialized_functions: &'a mut Vec<AstFunction>,
    pub(super) specialized_signatures: &'a mut Vec<(String, FunctionSignature)>,
}

pub(super) struct GenericFieldAccessRewriteInput<'a> {
    pub(super) base: &'a AstExpr,
    pub(super) field: &'a str,
    pub(super) context: &'a str,
    pub(super) expected: Option<&'a AstTypeRef>,
    pub(super) env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) generic_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) generic_impl_method_templates: &'a [GenericImplMethodTemplate],
    pub(super) higher_order_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) function_table: &'a BTreeMap<String, AstFunction>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) specialization_cache: &'a mut BTreeSet<String>,
    pub(super) specialized_functions: &'a mut Vec<AstFunction>,
    pub(super) specialized_signatures: &'a mut Vec<(String, FunctionSignature)>,
}

pub(super) fn rewrite_generic_struct_literal_expr(
    input: GenericStructLiteralRewriteInput<'_>,
) -> Result<AstExpr, String> {
    let GenericStructLiteralRewriteInput {
        type_name,
        type_args,
        fields,
        context,
        expected,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    } = input;
    let rewritten_head = resolved_struct_literal_alias(StructLiteralAliasInput {
        type_name,
        type_args,
        expected,
        fields,
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
    })?
    .unwrap_or_else(|| (type_name.to_owned(), type_args.to_vec()));
    let concrete_literal_ty = concrete_struct_literal_type(
        &rewritten_head.0,
        &rewritten_head.1,
        expected,
        visible_type_aliases,
    );
    let final_name = concrete_literal_ty
        .as_ref()
        .map(|ty| ty.name.clone())
        .unwrap_or_else(|| rewritten_head.0.clone());
    let mut final_args = concrete_literal_ty
        .as_ref()
        .map(|ty| ty.generic_args.clone())
        .unwrap_or_else(|| rewritten_head.1.clone());
    if let Some(definition) = struct_table.get(&final_name) {
        let placeholder_names = definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        if final_args
            .iter()
            .any(|arg| generic_arg_contains_definition_placeholder(arg, &placeholder_names))
        {
            final_args.clear();
        }
    }
    Ok(AstExpr::StructLiteral {
        type_name: final_name.clone(),
        type_args: final_args.clone(),
        fields: fields
            .iter()
            .map(|(name, value)| {
                let literal_ty = concrete_literal_ty.clone().unwrap_or_else(|| AstTypeRef {
                    name: final_name.clone(),
                    generic_args: final_args.clone(),
                    is_optional: false,
                    is_ref: false,
                });
                let field_expected = struct_field_expected_type(&literal_ty, name, struct_table);
                Ok((
                    name.clone(),
                    rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                        expr: value,
                        context,
                        expected: field_expected.as_ref(),
                        env,
                        visible_type_aliases,
                        generic_templates,
                        generic_impl_method_templates,
                        higher_order_templates,
                        function_table,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    })?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?,
    })
}

pub(super) fn rewrite_generic_field_access_expr(
    input: GenericFieldAccessRewriteInput<'_>,
) -> Result<AstExpr, String> {
    let GenericFieldAccessRewriteInput {
        base,
        field,
        context,
        expected,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    } = input;
    let base_expected =
        field_access_base_expected_type(expected, field, visible_type_aliases, struct_table);
    Ok(AstExpr::FieldAccess {
        base: Box::new(rewrite_generic_calls_in_expr(GenericExprRewriteInput {
            expr: base,
            context,
            expected: base_expected.as_ref(),
            env,
            visible_type_aliases,
            generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        })?),
        field: field.to_owned(),
    })
}
