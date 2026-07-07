use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{infer_ast_expr_type, FunctionSignature};
use super::blocks_expected::contains_unresolved_struct_placeholder;
use super::blocks_stmt::{rewrite_generic_calls_in_stmt, GenericStmtRewriteInput};
use super::exprs::{rewrite_generic_calls_in_expr, GenericExprRewriteInput};
use super::hoists::{hoist_direct_result_wrapper_args, DirectResultWrapperHoistInput};
use super::GenericImplMethodTemplate;

pub(super) struct GenericStmtHoistRewriteInput<'a> {
    pub(super) stmt: &'a AstStmt,
    pub(super) context: &'a str,
    pub(super) let_fallback_expected: Option<&'a AstTypeRef>,
    pub(super) current_return_type: Option<&'a AstTypeRef>,
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

pub(super) fn rewrite_generic_stmt_with_hoists(
    input: GenericStmtHoistRewriteInput<'_>,
) -> Result<Vec<AstStmt>, String> {
    let GenericStmtHoistRewriteInput {
        stmt,
        context,
        let_fallback_expected,
        current_return_type,
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
    match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => {
            let AstExpr::Call {
                callee,
                generic_args,
                args,
            } = value
            else {
                return Ok(vec![rewrite_generic_calls_in_stmt(
                    GenericStmtRewriteInput {
                        stmt,
                        context,
                        let_fallback_expected,
                        current_return_type,
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
                    },
                )?]);
            };
            if !generic_templates.contains_key(callee) {
                return Ok(vec![rewrite_generic_calls_in_stmt(
                    GenericStmtRewriteInput {
                        stmt,
                        context,
                        let_fallback_expected,
                        current_return_type,
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
                    },
                )?]);
            }
            let (mut hoisted, rewritten_args) =
                hoist_direct_result_wrapper_args(DirectResultWrapperHoistInput {
                    callee,
                    generic_args,
                    args,
                    expected: ty.as_ref(),
                    temp_prefix: name,
                    context: &format!("{context} local `{name}`"),
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
            let rewritten_call_expr = AstExpr::Call {
                callee: callee.clone(),
                generic_args: generic_args.clone(),
                args: rewritten_args,
            };
            let rewritten_context = format!("{context} local `{name}`");
            let rewritten_value = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: &rewritten_call_expr,
                context: &rewritten_context,
                expected: ty.as_ref().or(let_fallback_expected),
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
            let mut inferred = ty
                .clone()
                .or_else(|| {
                    infer_ast_expr_type(
                        &rewritten_value,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                    )
                })
                .or_else(|| let_fallback_expected.cloned());
            if ty.is_none()
                && inferred
                    .as_ref()
                    .is_some_and(|ty| contains_unresolved_struct_placeholder(ty, struct_table))
            {
                inferred = None;
            }
            if let Some(inferred_ty) = &inferred {
                env.insert(name.clone(), inferred_ty.clone());
            }
            hoisted.push(AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: inferred.or_else(|| ty.clone()),
                value: rewritten_value,
            });
            Ok(hoisted)
        }
        AstStmt::Return(Some(AstExpr::Call {
            callee,
            generic_args,
            args,
        })) if generic_templates.contains_key(callee) => {
            let (mut hoisted, rewritten_args) =
                hoist_direct_result_wrapper_args(DirectResultWrapperHoistInput {
                    callee,
                    generic_args,
                    args,
                    expected: current_return_type,
                    temp_prefix: "__nuis_generic_return_arg",
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
                })?;
            let rewritten_call_expr = AstExpr::Call {
                callee: callee.clone(),
                generic_args: generic_args.clone(),
                args: rewritten_args,
            };
            let rewritten_value = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: &rewritten_call_expr,
                context,
                expected: current_return_type,
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
            hoisted.push(AstStmt::Return(Some(rewritten_value)));
            Ok(hoisted)
        }
        _ => Ok(vec![rewrite_generic_calls_in_stmt(
            GenericStmtRewriteInput {
                stmt,
                context,
                let_fallback_expected,
                current_return_type,
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
            },
        )?]),
    }
}
