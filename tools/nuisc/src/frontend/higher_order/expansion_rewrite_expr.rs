use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, AstMatchArm, AstTypeRef};

use super::super::validation_binding_env::bind_match_pattern_for_type;
use super::super::{ast_named_type, lower_type_ref};
use super::expansion::{specialize_higher_order_call, HigherOrderCallSpecializationInput};
use super::expansion_expected::find_generic_method_template_name;
use super::expansion_inference::{
    expected_await_operand_type, expected_try_operand_type, infer_local_binding_type,
};
use super::expansion_rewrite::{rewrite_higher_order_calls_in_block, HigherOrderRewriteContext};

pub(crate) fn rewrite_higher_order_calls_in_expr(
    expr: &AstExpr,
    expected: Option<&AstTypeRef>,
    current_return_type: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    context: &mut HigherOrderRewriteContext<'_>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(rewrite_higher_order_calls_in_expr(
                condition,
                Some(&ast_named_type("bool")),
                current_return_type,
                local_types,
                context,
            )?),
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                current_return_type,
                expected,
                local_types,
                context,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                current_return_type,
                expected,
                local_types,
                context,
            )?,
        },
        AstExpr::Match { value, arms } => {
            let rewritten_value = rewrite_higher_order_calls_in_expr(
                value,
                None,
                current_return_type,
                local_types,
                context,
            )?;
            let scrutinee_type = infer_local_binding_type(
                &rewritten_value,
                local_types,
                context.function_table,
                context.module_impls,
            );
            AstExpr::Match {
                value: Box::new(rewritten_value),
                arms: arms
                    .iter()
                    .map(|arm| {
                        let mut arm_local_types = local_types.clone();
                        if let Some(scrutinee_type) = scrutinee_type.as_ref() {
                            bind_match_pattern_for_type(
                                scrutinee_type,
                                &arm.pattern,
                                context.visible_type_aliases,
                                context.visible_structs,
                                &mut arm_local_types,
                            )?;
                        }
                        Ok(AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: match &arm.guard {
                                Some(guard) => Some(rewrite_higher_order_calls_in_expr(
                                    guard,
                                    Some(&ast_named_type("bool")),
                                    current_return_type,
                                    &arm_local_types,
                                    context,
                                )?),
                                None => None,
                            },
                            body: rewrite_higher_order_calls_in_block(
                                &arm.body,
                                current_return_type,
                                expected,
                                &arm_local_types,
                                context,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if context.templates.contains_key(callee) => {
            specialize_higher_order_call(HigherOrderCallSpecializationInput {
                callee,
                args,
                explicit_generic_args: generic_args,
                template_callable_bindings: None,
                expected,
                local_types,
                templates: context.templates,
                function_table: context.function_table,
                module_impls: context.module_impls,
                visible_structs: context.visible_structs,
                visible_type_aliases: context.visible_type_aliases,
                specialized_cache: context.specialized_cache,
                specialized_functions: context.specialized_functions,
            })?
        }
        AstExpr::Await(value) => {
            let await_expected = expected_await_operand_type(expected);
            AstExpr::Await(Box::new(rewrite_higher_order_calls_in_expr(
                value,
                await_expected.as_ref(),
                current_return_type,
                local_types,
                context,
            )?))
        }
        AstExpr::Unary { op, operand } => AstExpr::Unary {
            op: *op,
            operand: Box::new(rewrite_higher_order_calls_in_expr(
                operand,
                expected,
                current_return_type,
                local_types,
                context,
            )?),
        },
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => AstExpr::Call {
            callee: callee.clone(),
            generic_args: generic_args.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_calls_in_expr(
                        arg,
                        None,
                        current_return_type,
                        local_types,
                        context,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::Try(value) => {
            let try_expected = expected_try_operand_type(expected, current_return_type);
            AstExpr::Try(Box::new(rewrite_higher_order_calls_in_expr(
                value,
                try_expected.as_ref(),
                current_return_type,
                local_types,
                context,
            )?))
        }
        AstExpr::Invoke { callee, args } => AstExpr::Invoke {
            callee: Box::new(rewrite_higher_order_calls_in_expr(
                callee,
                None,
                current_return_type,
                local_types,
                context,
            )?),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_calls_in_expr(
                        arg,
                        None,
                        current_return_type,
                        local_types,
                        context,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            if let Some(receiver_ty) = infer_local_binding_type(
                receiver,
                local_types,
                context.function_table,
                context.module_impls,
            ) {
                if let Some(template_name) = context
                    .method_template_lookup
                    .get(&(lower_type_ref(&receiver_ty).render(), method.clone()))
                {
                    let mut full_args = Vec::with_capacity(args.len() + 1);
                    full_args.push(receiver.as_ref().clone());
                    full_args.extend(args.iter().cloned());
                    return specialize_higher_order_call(HigherOrderCallSpecializationInput {
                        callee: template_name,
                        args: &full_args,
                        explicit_generic_args: &[],
                        template_callable_bindings: None,
                        expected,
                        local_types,
                        templates: context.templates,
                        function_table: context.function_table,
                        module_impls: context.module_impls,
                        visible_structs: context.visible_structs,
                        visible_type_aliases: context.visible_type_aliases,
                        specialized_cache: context.specialized_cache,
                        specialized_functions: context.specialized_functions,
                    });
                }
                if let Some(template_name) = find_generic_method_template_name(
                    &receiver_ty,
                    method,
                    context.templates,
                    context.visible_type_aliases,
                )? {
                    let mut full_args = Vec::with_capacity(args.len() + 1);
                    full_args.push(receiver.as_ref().clone());
                    full_args.extend(args.iter().cloned());
                    return specialize_higher_order_call(HigherOrderCallSpecializationInput {
                        callee: &template_name,
                        args: &full_args,
                        explicit_generic_args: &[],
                        template_callable_bindings: None,
                        expected,
                        local_types,
                        templates: context.templates,
                        function_table: context.function_table,
                        module_impls: context.module_impls,
                        visible_structs: context.visible_structs,
                        visible_type_aliases: context.visible_type_aliases,
                        specialized_cache: context.specialized_cache,
                        specialized_functions: context.specialized_functions,
                    });
                }
            }
            AstExpr::MethodCall {
                receiver: Box::new(rewrite_higher_order_calls_in_expr(
                    receiver,
                    None,
                    current_return_type,
                    local_types,
                    context,
                )?),
                method: method.clone(),
                generic_args: generic_args.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        rewrite_higher_order_calls_in_expr(
                            arg,
                            None,
                            current_return_type,
                            local_types,
                            context,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        rewrite_higher_order_calls_in_expr(
                            value,
                            None,
                            current_return_type,
                            local_types,
                            context,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_higher_order_calls_in_expr(
                base,
                None,
                current_return_type,
                local_types,
                context,
            )?),
            field: field.clone(),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_higher_order_calls_in_expr(
                lhs,
                None,
                current_return_type,
                local_types,
                context,
            )?),
            rhs: Box::new(rewrite_higher_order_calls_in_expr(
                rhs,
                None,
                current_return_type,
                local_types,
                context,
            )?),
        },
        _ => expr.clone(),
    })
}
