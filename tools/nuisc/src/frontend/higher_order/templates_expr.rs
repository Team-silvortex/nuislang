use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstFunction, AstMatchArm, AstTypeAlias};

use super::super::expansion::{
    specialize_higher_order_call, BoundCallable, HigherOrderCallSpecializationInput,
};
use super::rewrite_higher_order_template_block;

pub(crate) fn rewrite_higher_order_template_expr(
    expr: &AstExpr,
    callable_bindings: &BTreeMap<String, BoundCallable>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::Var(name) if callable_bindings.contains_key(name) => {
            let bound = callable_bindings
                .get(name)
                .cloned()
                .unwrap_or_else(|| BoundCallable {
                    symbol: name.clone(),
                    capture_args: Vec::new(),
                    capture_params: Vec::new(),
                });
            if bound.capture_args.is_empty() {
                AstExpr::Var(bound.symbol)
            } else {
                AstExpr::Call {
                    callee: bound.symbol,
                    generic_args: Vec::new(),
                    args: bound.capture_args,
                }
            }
        }
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(rewrite_higher_order_template_expr(
                condition,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            then_body: rewrite_higher_order_template_block(
                then_body,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            else_body: rewrite_higher_order_template_block(
                else_body,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(rewrite_higher_order_template_expr(
                value,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: match &arm.guard {
                            Some(guard) => Some(rewrite_higher_order_template_expr(
                                guard,
                                callable_bindings,
                                templates,
                                function_table,
                                visible_type_aliases,
                                specialized_cache,
                                specialized_functions,
                            )?),
                            None => None,
                        },
                        body: rewrite_higher_order_template_block(
                            &arm.body,
                            callable_bindings,
                            templates,
                            function_table,
                            visible_type_aliases,
                            specialized_cache,
                            specialized_functions,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if callable_bindings.contains_key(callee) => {
            let mut rewritten_args = args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_template_expr(
                        arg,
                        callable_bindings,
                        templates,
                        function_table,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let bound = callable_bindings
                .get(callee)
                .cloned()
                .unwrap_or_else(|| BoundCallable {
                    symbol: callee.clone(),
                    capture_args: Vec::new(),
                    capture_params: Vec::new(),
                });
            rewritten_args.extend(bound.capture_args);
            AstExpr::Call {
                callee: bound.symbol,
                generic_args: generic_args.clone(),
                args: rewritten_args,
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if templates.contains_key(callee) => {
            specialize_higher_order_call(HigherOrderCallSpecializationInput {
                callee,
                args,
                explicit_generic_args: generic_args,
                template_callable_bindings: Some(callable_bindings),
                expected: None,
                local_types: &BTreeMap::new(),
                templates,
                function_table,
                module_impls: &[],
                visible_structs: &BTreeMap::new(),
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            })?
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_higher_order_template_expr(
            value,
            callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?)),
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
                    rewrite_higher_order_template_expr(
                        arg,
                        callable_bindings,
                        templates,
                        function_table,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::Invoke { callee, args } => AstExpr::Invoke {
            callee: Box::new(rewrite_higher_order_template_expr(
                callee,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_template_expr(
                        arg,
                        callable_bindings,
                        templates,
                        function_table,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(rewrite_higher_order_template_expr(
                receiver,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            method: method.clone(),
            generic_args: generic_args.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_template_expr(
                        arg,
                        callable_bindings,
                        templates,
                        function_table,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
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
                        rewrite_higher_order_template_expr(
                            value,
                            callable_bindings,
                            templates,
                            function_table,
                            visible_type_aliases,
                            specialized_cache,
                            specialized_functions,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_higher_order_template_expr(
                base,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            field: field.clone(),
        },
        AstExpr::Unary { op, operand } => AstExpr::Unary {
            op: *op,
            operand: Box::new(rewrite_higher_order_template_expr(
                operand,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_higher_order_template_expr(
                lhs,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            rhs: Box::new(rewrite_higher_order_template_expr(
                rhs,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
        },
        _ => expr.clone(),
    })
}
