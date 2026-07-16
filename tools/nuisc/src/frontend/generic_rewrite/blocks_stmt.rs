use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{infer_ast_expr_type, FunctionSignature};
use super::blocks::{
    rewrite_generic_calls_in_block, rewrite_generic_calls_in_match_arms, GenericBlockRewriteInput,
    GenericMatchArmsRewriteInput,
};
use super::blocks_expected::{
    contains_unresolved_generic_placeholder, contains_unresolved_struct_placeholder,
};
use super::exprs::{rewrite_generic_calls_in_expr, GenericExprRewriteInput};
use super::GenericImplMethodTemplate;

pub(super) struct GenericStmtRewriteInput<'a> {
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

pub(super) fn rewrite_generic_calls_in_stmt(
    input: GenericStmtRewriteInput<'_>,
) -> Result<AstStmt, String> {
    let GenericStmtRewriteInput {
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
    Ok(match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => {
            let value_context = format!("{context} local `{name}`");
            let rewritten_value = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: value,
                context: &value_context,
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
            if ty.is_none()
                && let_fallback_expected.is_some()
                && inferred
                    .as_ref()
                    .is_some_and(contains_unresolved_generic_placeholder)
            {
                inferred = let_fallback_expected.cloned();
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
            value: {
                let value_context = format!("{context} local `{name}`");
                rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                    expr: value,
                    context: &value_context,
                    expected: env.get(name),
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
            },
        },
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => AstStmt::DestructureLet {
            type_ref: type_ref.clone(),
            fields: fields.clone(),
            value: {
                let value_context = format!("{context} destructure");
                rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                    expr: value,
                    context: &value_context,
                    expected: type_ref.as_ref(),
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
            },
        },
        AstStmt::Const { name, ty, value } => {
            let value_context = format!("{context} const `{name}`");
            let rewritten_value = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: value,
                context: &value_context,
                expected: ty.as_ref(),
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
        AstStmt::Print(value) => {
            AstStmt::Print(rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: value,
                context,
                expected: None,
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
            })?)
        }
        AstStmt::Await(value) => {
            AstStmt::Await(rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: value,
                context,
                expected: None,
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
            })?)
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let rewritten_condition = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: condition,
                context,
                expected: None,
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
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            AstStmt::If {
                condition: rewritten_condition,
                then_body: rewrite_generic_calls_in_block(GenericBlockRewriteInput {
                    body: then_body,
                    context: &format!("{context} if-then"),
                    current_return_type,
                    env: &mut then_env,
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
                else_body: rewrite_generic_calls_in_block(GenericBlockRewriteInput {
                    body: else_body,
                    context: &format!("{context} if-else"),
                    current_return_type,
                    env: &mut else_env,
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
            }
        }
        AstStmt::Match { value, arms } => {
            let rewritten_value = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: value,
                context,
                expected: None,
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
            let scrutinee_type = infer_ast_expr_type(
                &rewritten_value,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
            );
            AstStmt::Match {
                value: rewritten_value,
                arms: rewrite_generic_calls_in_match_arms(GenericMatchArmsRewriteInput {
                    arms,
                    context,
                    scrutinee_type: scrutinee_type.as_ref(),
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
                })?,
            }
        }
        AstStmt::While { condition, body } => {
            let rewritten_condition = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: condition,
                context,
                expected: None,
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
            let mut loop_env = env.clone();
            AstStmt::While {
                condition: rewritten_condition,
                body: rewrite_generic_calls_in_block(GenericBlockRewriteInput {
                    body,
                    context: &format!("{context} while-body"),
                    current_return_type,
                    env: &mut loop_env,
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
            }
        }
        AstStmt::Expr(expr) => {
            AstStmt::Expr(rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr,
                context,
                expected: None,
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
            })?)
        }
        AstStmt::Return(value) => AstStmt::Return(match value {
            Some(value) => Some(rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: value,
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
            })?),
            None => None,
        }),
        AstStmt::Break => AstStmt::Break,
        AstStmt::Continue => AstStmt::Continue,
    })
}
