use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{resolve_ast_type_ref_aliases, FunctionSignature};
use super::exprs::{rewrite_generic_calls_in_expr, GenericExprRewriteInput};
use super::exprs_aliases::{resolved_struct_constructor_alias, StructConstructorAliasInput};
use super::exprs_expected::{call_arg_expected_type, CallArgExpectedTypeInput};
use super::exprs_specialization::{
    ensure_generic_impl_method_specialization, ensure_generic_specialization,
    GenericImplMethodSpecializationInput, GenericSpecializationInput,
};
use super::GenericImplMethodTemplate;

pub(super) struct GenericCallExprRewriteInput<'a> {
    pub(super) callee: &'a str,
    pub(super) generic_args: &'a [AstTypeRef],
    pub(super) args: &'a [AstExpr],
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

pub(super) fn rewrite_generic_call_expr(
    input: GenericCallExprRewriteInput<'_>,
) -> Result<AstExpr, String> {
    let GenericCallExprRewriteInput {
        callee,
        generic_args,
        args,
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
    let rewritten_generic_args = generic_args
        .iter()
        .map(|arg| resolve_ast_type_ref_aliases(arg, visible_type_aliases))
        .collect::<Result<Vec<_>, _>>()?;
    let rewritten_args = args
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            let arg_expected = call_arg_expected_type(CallArgExpectedTypeInput {
                callee,
                generic_args: &rewritten_generic_args,
                index,
                expected,
                generic_templates,
                signatures,
                visible_type_aliases,
                struct_table,
            });
            rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: arg,
                context,
                expected: arg_expected.as_ref(),
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
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some((trait_name, method_name)) = callee.rsplit_once('.') {
        if let Some(specialized_name) =
            ensure_generic_impl_method_specialization(GenericImplMethodSpecializationInput {
                trait_name: Some(trait_name),
                method_name,
                args: &rewritten_args,
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
            })?
        {
            return Ok(AstExpr::Call {
                callee: specialized_name,
                generic_args: Vec::new(),
                args: rewritten_args,
            });
        }
    }
    if let Some(template) = generic_templates.get(callee) {
        let specialization_context = format!("{context} call `{callee}`");
        let specialized_name = ensure_generic_specialization(GenericSpecializationInput {
            template,
            explicit_generic_args: &rewritten_generic_args,
            args: &rewritten_args,
            expected,
            context: &specialization_context,
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
        })?;
        Ok(AstExpr::Call {
            callee: specialized_name,
            generic_args: Vec::new(),
            args: rewritten_args,
        })
    } else {
        let rewritten_callee = resolved_struct_constructor_alias(StructConstructorAliasInput {
            callee,
            generic_args: &rewritten_generic_args,
            expected,
            args: &rewritten_args,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
        })?
        .unwrap_or_else(|| (callee.to_owned(), rewritten_generic_args.clone()));
        Ok(AstExpr::Call {
            callee: rewritten_callee.0,
            generic_args: rewritten_callee.1,
            args: rewritten_args,
        })
    }
}
