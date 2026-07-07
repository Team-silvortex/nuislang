use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{infer_ast_expr_type, FunctionSignature};
use super::exprs::{
    call_arg_expected_type, rewrite_generic_calls_in_expr, CallArgExpectedTypeInput,
    GenericExprRewriteInput,
};
use super::GenericImplMethodTemplate;

pub(super) struct DirectResultWrapperHoistInput<'a> {
    pub(super) callee: &'a str,
    pub(super) generic_args: &'a [AstTypeRef],
    pub(super) args: &'a [AstExpr],
    pub(super) expected: Option<&'a AstTypeRef>,
    pub(super) temp_prefix: &'a str,
    pub(super) context: &'a str,
    pub(super) env: &'a mut BTreeMap<String, AstTypeRef>,
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

pub(super) fn hoist_direct_result_wrapper_args(
    input: DirectResultWrapperHoistInput<'_>,
) -> Result<(Vec<AstStmt>, Vec<AstExpr>), String> {
    let DirectResultWrapperHoistInput {
        callee,
        generic_args,
        args,
        expected,
        temp_prefix,
        context,
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
    let mut hoisted = Vec::new();
    let mut rewritten_args = Vec::new();
    for (index, arg) in args.iter().enumerate() {
        let arg_expected = call_arg_expected_type(CallArgExpectedTypeInput {
            callee,
            generic_args,
            index,
            expected,
            generic_templates,
            signatures,
            visible_type_aliases,
            struct_table,
        });
        let rewritten_arg = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
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
        })?;
        if is_direct_result_wrapper_expr(&rewritten_arg) {
            let Some(inferred_ty) = infer_ast_expr_type(
                &rewritten_arg,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
            ) else {
                return Err(format!(
                    "could not infer type for hoisted generic argument {} in call to `{}`",
                    index, callee
                ));
            };
            let temp_name = format!("{temp_prefix}_{index}");
            env.insert(temp_name.clone(), inferred_ty.clone());
            hoisted.push(AstStmt::Let {
                mutable: false,
                name: temp_name.clone(),
                ty: Some(inferred_ty),
                value: rewritten_arg,
            });
            rewritten_args.push(AstExpr::Var(temp_name));
        } else {
            rewritten_args.push(rewritten_arg);
        }
    }
    Ok((hoisted, rewritten_args))
}

fn is_direct_result_wrapper_expr(expr: &AstExpr) -> bool {
    matches!(
        expr,
        AstExpr::Call { callee, .. }
            if matches!(
                callee.as_str(),
                "data_result" | "join_result" | "shader_result" | "kernel_result" | "network_result"
            )
    )
}
