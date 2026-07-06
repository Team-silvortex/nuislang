use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::generics::infer_alias_aware_ast_expr_type;
use super::super::{lower_type_ref, resolve_ast_type_ref_aliases, FunctionSignature};
use super::exprs_aliases::{
    method_call_receiver_expected_type, MethodCallReceiverExpectedTypeInput,
};
pub(super) use super::exprs_expected::{call_arg_expected_type, CallArgExpectedTypeInput};
use super::exprs_operators::{
    builtin_binary_supported_ast, builtin_unary_supported_ast, overloaded_binary_trait,
    overloaded_unary_trait,
};
use super::exprs_specialization::{
    ensure_generic_impl_method_specialization,
    ensure_generic_impl_method_specialization_from_receiver_expected,
};
use super::GenericImplMethodTemplate;

pub(super) struct GenericExprRewriteInput<'a> {
    pub(super) expr: &'a AstExpr,
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

pub(super) fn rewrite_generic_calls_in_expr(
    input: GenericExprRewriteInput<'_>,
) -> Result<AstExpr, String> {
    let GenericExprRewriteInput {
        expr,
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
    Ok(match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            AstExpr::If {
                condition: Box::new(rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                    expr: condition,
                    context,
                    expected: Some(&super::super::ast_named_type("bool")),
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
                then_body: super::blocks::rewrite_generic_calls_in_block(
                    then_body,
                    &format!("{context} if-then"),
                    expected,
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
                else_body: super::blocks::rewrite_generic_calls_in_block(
                    else_body,
                    &format!("{context} if-else"),
                    expected,
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
        AstExpr::Match { value, arms } => {
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
            let scrutinee_type = infer_alias_aware_ast_expr_type(
                &rewritten_value,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            );
            AstExpr::Match {
                value: Box::new(rewritten_value),
                arms: super::blocks::rewrite_generic_calls_in_match_arms(
                    arms,
                    context,
                    scrutinee_type.as_ref(),
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
                )?,
            }
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_generic_calls_in_expr(
            GenericExprRewriteInput {
                expr: value,
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
            },
        )?)),
        AstExpr::Unary { op, operand } => {
            let rewritten_operand = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: operand,
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
            })?;
            if let Some((trait_name, method_name)) = overloaded_unary_trait(*op) {
                if let Some(operand_ty) = infer_alias_aware_ast_expr_type(
                    &rewritten_operand,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok())
                {
                    if !builtin_unary_supported_ast(*op, &operand_ty) {
                        let call_args = vec![rewritten_operand.clone()];
                        if let Some(specialized_name) = ensure_generic_impl_method_specialization(
                            Some(trait_name),
                            method_name,
                            &call_args,
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
                                args: call_args,
                            });
                        }
                    }
                }
            }
            AstExpr::Unary {
                op: *op,
                operand: Box::new(rewritten_operand),
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => super::exprs_calls::rewrite_generic_call_expr(
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
        )?,
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            let explicit_receiver_expected =
                super::super::receiver_expected::explicit_receiver_expected_type(
                    receiver,
                    generic_args,
                    visible_type_aliases,
                );
            let rewritten_args = args
                .iter()
                .map(|arg| {
                    rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                        expr: arg,
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
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let receiver_expected =
                method_call_receiver_expected_type(MethodCallReceiverExpectedTypeInput {
                    receiver,
                    method,
                    generic_args,
                    args: &rewritten_args,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                });
            let rewritten_receiver = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: receiver,
                context,
                expected: receiver_expected.as_ref(),
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
            let rewritten_receiver =
                super::super::receiver_expected::specialize_receiver_constructor_from_expected(
                    &rewritten_receiver,
                    explicit_receiver_expected.as_ref(),
                    visible_type_aliases,
                    struct_table,
                );
            let mut call_args = vec![rewritten_receiver.clone()];
            call_args.extend(rewritten_args.clone());
            if let Some(specialized_name) = ensure_generic_impl_method_specialization(
                None,
                method,
                &call_args,
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
                AstExpr::Call {
                    callee: specialized_name,
                    generic_args: Vec::new(),
                    args: call_args,
                }
            } else {
                if let Some(explicit_receiver_expected) = receiver_expected.as_ref() {
                    if let Some(specialized_name) =
                        ensure_generic_impl_method_specialization_from_receiver_expected(
                            method,
                            explicit_receiver_expected,
                            &call_args,
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
                        )?
                    {
                        return Ok(AstExpr::Call {
                            callee: specialized_name,
                            generic_args: Vec::new(),
                            args: call_args,
                        });
                    }
                }
                AstExpr::MethodCall {
                    receiver: Box::new(rewritten_receiver),
                    method: method.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                }
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => super::exprs_structs::rewrite_generic_struct_literal_expr(
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
        )?,
        AstExpr::FieldAccess { base, field } => {
            super::exprs_structs::rewrite_generic_field_access_expr(
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
            )?
        }
        AstExpr::Binary { op, lhs, rhs } => {
            let rewritten_lhs = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: lhs,
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
            let rewritten_rhs = rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                expr: rhs,
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
            if let Some((trait_name, method_name)) = overloaded_binary_trait(*op) {
                let lhs_ty = infer_alias_aware_ast_expr_type(
                    &rewritten_lhs,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok());
                let rhs_ty = infer_alias_aware_ast_expr_type(
                    &rewritten_rhs,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok());
                if let (Some(lhs_ty), Some(rhs_ty)) = (lhs_ty, rhs_ty) {
                    if !builtin_binary_supported_ast(*op, &lhs_ty, &rhs_ty)
                        && lower_type_ref(&lhs_ty).render() == lower_type_ref(&rhs_ty).render()
                    {
                        let call_args = vec![rewritten_lhs.clone(), rewritten_rhs.clone()];
                        if let Some(specialized_name) = ensure_generic_impl_method_specialization(
                            Some(trait_name),
                            method_name,
                            &call_args,
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
                            let call = AstExpr::Call {
                                callee: specialized_name,
                                generic_args: Vec::new(),
                                args: call_args,
                            };
                            return Ok(match op {
                                AstBinaryOp::Ne => AstExpr::Binary {
                                    op: AstBinaryOp::Eq,
                                    lhs: Box::new(call),
                                    rhs: Box::new(AstExpr::Bool(false)),
                                },
                                _ => call,
                            });
                        }
                    }
                }
            }
            AstExpr::Binary {
                op: *op,
                lhs: Box::new(rewritten_lhs),
                rhs: Box::new(rewritten_rhs),
            }
        }
        other => other.clone(),
    })
}
