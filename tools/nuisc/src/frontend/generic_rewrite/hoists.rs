use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{infer_ast_expr_type, FunctionSignature};
use super::GenericImplMethodTemplate;
use super::exprs::{call_arg_expected_type, rewrite_generic_calls_in_expr};

#[allow(clippy::too_many_arguments)]
pub(super) fn hoist_direct_result_wrapper_args(
    callee: &str,
    generic_args: &[AstTypeRef],
    args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    temp_prefix: &str,
    context: &str,
    env: &mut BTreeMap<String, AstTypeRef>,
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
) -> Result<(Vec<AstStmt>, Vec<AstExpr>), String> {
    let mut hoisted = Vec::new();
    let mut rewritten_args = Vec::new();
    for (index, arg) in args.iter().enumerate() {
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
        let rewritten_arg = rewrite_generic_calls_in_expr(
            arg,
            context,
            arg_expected.as_ref(),
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
        )?;
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
