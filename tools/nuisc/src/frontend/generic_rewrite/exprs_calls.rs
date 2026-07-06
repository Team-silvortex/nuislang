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
    GenericSpecializationInput,
};
use super::GenericImplMethodTemplate;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_call_expr(
    callee: &str,
    generic_args: &[AstTypeRef],
    args: &[AstExpr],
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
        if let Some(specialized_name) = ensure_generic_impl_method_specialization(
            Some(trait_name),
            method_name,
            &rewritten_args,
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
        )? {
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
