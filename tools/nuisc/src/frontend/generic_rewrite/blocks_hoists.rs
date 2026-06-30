use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{infer_ast_expr_type, FunctionSignature};
use super::blocks_expected::contains_unresolved_struct_placeholder;
use super::blocks_stmt::rewrite_generic_calls_in_stmt;
use super::exprs::rewrite_generic_calls_in_expr;
use super::hoists::hoist_direct_result_wrapper_args;
use super::GenericImplMethodTemplate;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_stmt_with_hoists(
    stmt: &AstStmt,
    context: &str,
    let_fallback_expected: Option<&AstTypeRef>,
    current_return_type: Option<&AstTypeRef>,
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
) -> Result<Vec<AstStmt>, String> {
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
                )?]);
            };
            if !generic_templates.contains_key(callee) {
                return Ok(vec![rewrite_generic_calls_in_stmt(
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
                )?]);
            }
            let (mut hoisted, rewritten_args) = hoist_direct_result_wrapper_args(
                callee,
                generic_args,
                args,
                ty.as_ref(),
                name,
                &format!("{context} local `{name}`"),
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
            let rewritten_value = rewrite_generic_calls_in_expr(
                &AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                },
                &format!("{context} local `{name}`"),
                ty.as_ref().or(let_fallback_expected),
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
            let (mut hoisted, rewritten_args) = hoist_direct_result_wrapper_args(
                callee,
                generic_args,
                args,
                current_return_type,
                "__nuis_generic_return_arg",
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
            )?;
            let rewritten_value = rewrite_generic_calls_in_expr(
                &AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                },
                context,
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
            )?;
            hoisted.push(AstStmt::Return(Some(rewritten_value)));
            Ok(hoisted)
        }
        _ => Ok(vec![rewrite_generic_calls_in_stmt(
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
        )?]),
    }
}
