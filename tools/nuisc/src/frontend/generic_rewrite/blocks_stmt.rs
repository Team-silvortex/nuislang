use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{infer_ast_expr_type, FunctionSignature};
use super::blocks::{rewrite_generic_calls_in_block, rewrite_generic_calls_in_match_arms};
use super::blocks_expected::contains_unresolved_struct_placeholder;
use super::exprs::rewrite_generic_calls_in_expr;
use super::GenericImplMethodTemplate;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_calls_in_stmt(
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
) -> Result<AstStmt, String> {
    Ok(match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => {
            let rewritten_value = rewrite_generic_calls_in_expr(
                value,
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
            AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: inferred.or_else(|| ty.clone()),
                value: rewritten_value,
            }
        }
        AstStmt::AssignLocal { name, value } => AstStmt::AssignLocal {
            name: name.clone(),
            value: rewrite_generic_calls_in_expr(
                value,
                &format!("{context} local `{name}`"),
                env.get(name),
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
        },
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => AstStmt::DestructureLet {
            type_ref: type_ref.clone(),
            fields: fields.clone(),
            value: rewrite_generic_calls_in_expr(
                value,
                &format!("{context} destructure"),
                type_ref.as_ref(),
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
        },
        AstStmt::Const { name, ty, value } => {
            let rewritten_value = rewrite_generic_calls_in_expr(
                value,
                &format!("{context} const `{name}`"),
                ty.as_ref(),
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
            let mut inferred = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    &rewritten_value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
            });
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
            AstStmt::Const {
                name: name.clone(),
                ty: inferred.or_else(|| ty.clone()),
                value: rewritten_value,
            }
        }
        AstStmt::Print(value) => AstStmt::Print(rewrite_generic_calls_in_expr(
            value,
            context,
            None,
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
        AstStmt::Await(value) => AstStmt::Await(rewrite_generic_calls_in_expr(
            value,
            context,
            None,
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
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let rewritten_condition = rewrite_generic_calls_in_expr(
                condition,
                context,
                None,
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
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            AstStmt::If {
                condition: rewritten_condition,
                then_body: rewrite_generic_calls_in_block(
                    then_body,
                    &format!("{context} if-then"),
                    current_return_type,
                    &mut then_env,
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
                else_body: rewrite_generic_calls_in_block(
                    else_body,
                    &format!("{context} if-else"),
                    current_return_type,
                    &mut else_env,
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
            }
        }
        AstStmt::Match { value, arms } => {
            let rewritten_value = rewrite_generic_calls_in_expr(
                value,
                context,
                None,
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
            let scrutinee_type = infer_ast_expr_type(
                &rewritten_value,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
            );
            AstStmt::Match {
                value: rewritten_value,
                arms: rewrite_generic_calls_in_match_arms(
                    arms,
                    context,
                    scrutinee_type.as_ref(),
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
                )?,
            }
        }
        AstStmt::While { condition, body } => {
            let rewritten_condition = rewrite_generic_calls_in_expr(
                condition,
                context,
                None,
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
            let mut loop_env = env.clone();
            AstStmt::While {
                condition: rewritten_condition,
                body: rewrite_generic_calls_in_block(
                    body,
                    &format!("{context} while-body"),
                    current_return_type,
                    &mut loop_env,
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
            }
        }
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_generic_calls_in_expr(
            expr,
            context,
            None,
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
        AstStmt::Return(value) => AstStmt::Return(match value {
            Some(value) => Some(rewrite_generic_calls_in_expr(
                value,
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
            )?),
            None => None,
        }),
        AstStmt::Break => AstStmt::Break,
        AstStmt::Continue => AstStmt::Continue,
    })
}
