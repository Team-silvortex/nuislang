use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::FunctionSignature;
use super::exprs::rewrite_generic_calls_in_expr;
use super::exprs_aliases::{concrete_struct_literal_type, resolved_struct_literal_alias};
use super::exprs_expected::{field_access_base_expected_type, struct_field_expected_type};
use super::exprs_specialization::generic_arg_contains_definition_placeholder;
use super::GenericImplMethodTemplate;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_struct_literal_expr(
    type_name: &str,
    type_args: &[AstTypeRef],
    fields: &[(String, AstExpr)],
    context: &str,
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<AstExpr, String> {
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
                    rewrite_generic_calls_in_expr(
                        value,
                        context,
                        field_expected.as_ref(),
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
                    )?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_field_access_expr(
    base: &AstExpr,
    field: &str,
    context: &str,
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<AstExpr, String> {
    let base_expected =
        field_access_base_expected_type(expected, field, visible_type_aliases, struct_table);
    Ok(AstExpr::FieldAccess {
        base: Box::new(rewrite_generic_calls_in_expr(
            base,
            context,
            base_expected.as_ref(),
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
        )?),
        field: field.to_owned(),
    })
}
