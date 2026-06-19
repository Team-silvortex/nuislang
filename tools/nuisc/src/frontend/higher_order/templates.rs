use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstMatchArm, AstParam, AstStmt, AstTypeAlias, AstVisibility,
};

use super::callables::is_callable_type_with_aliases;
use super::expansion::{specialize_higher_order_call, BoundCallable};

pub(crate) fn specialize_higher_order_template(
    template: &AstFunction,
    specialized_name: &str,
    callable_bindings: &BTreeMap<String, BoundCallable>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstFunction, String> {
    let body = rewrite_higher_order_template_block(
        &template.body,
        callable_bindings,
        templates,
        function_table,
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    )?;
    let extra_capture_params = template
        .params
        .iter()
        .filter(|param| {
            is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
        })
        .flat_map(|param| {
            let Some(bound) = callable_bindings.get(&param.name) else {
                return Vec::<AstParam>::new();
            };
            bound
                .capture_params
                .iter()
                .zip(bound.capture_args.iter())
                .filter_map(|(capture_param, capture_arg)| match capture_arg {
                    AstExpr::Var(name) => Some(AstParam {
                        name: name.clone(),
                        ty: capture_param.ty.clone(),
                    }),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    Ok(AstFunction {
        name: specialized_name.to_owned(),
        visibility: AstVisibility::Private,
        attributes: Vec::new(),
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: template.is_async,
        generic_params: template.generic_params.clone(),
        where_bounds: template.where_bounds.clone(),
        params: template
            .params
            .iter()
            .filter(|param| {
                !is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            })
            .cloned()
            .chain(extra_capture_params)
            .collect(),
        return_type: template.return_type.clone(),
        body,
    })
}

pub(crate) fn rewrite_higher_order_template_block(
    body: &[AstStmt],
    callable_bindings: &BTreeMap<String, BoundCallable>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<Vec<AstStmt>, String> {
    body.iter()
        .map(|stmt| {
            rewrite_higher_order_template_stmt(
                stmt,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )
        })
        .collect()
}

pub(crate) fn rewrite_higher_order_template_stmt(
    stmt: &AstStmt,
    callable_bindings: &BTreeMap<String, BoundCallable>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstStmt, String> {
    Ok(match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => AstStmt::Let {
            mutable: *mutable,
            name: name.clone(),
            ty: ty.clone(),
            value: rewrite_higher_order_template_expr(
                value,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::AssignLocal { name, value } => AstStmt::AssignLocal {
            name: name.clone(),
            value: rewrite_higher_order_template_expr(
                value,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => AstStmt::DestructureLet {
            type_ref: type_ref.clone(),
            fields: fields.clone(),
            value: rewrite_higher_order_template_expr(
                value,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Const { name, ty, value } => AstStmt::Const {
            name: name.clone(),
            ty: ty.clone(),
            value: rewrite_higher_order_template_expr(
                value,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Print(value) => AstStmt::Print(rewrite_higher_order_template_expr(
            value,
            callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Await(value) => AstStmt::Await(rewrite_higher_order_template_expr(
            value,
            callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => AstStmt::If {
            condition: rewrite_higher_order_template_expr(
                condition,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
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
        AstStmt::Match { value, arms } => AstStmt::Match {
            value: rewrite_higher_order_template_expr(
                value,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: arm
                            .guard
                            .as_ref()
                            .map(|guard| {
                                rewrite_higher_order_template_expr(
                                    guard,
                                    callable_bindings,
                                    templates,
                                    function_table,
                                    visible_type_aliases,
                                    specialized_cache,
                                    specialized_functions,
                                )
                            })
                            .transpose()?,
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
        AstStmt::While { condition, body } => AstStmt::While {
            condition: rewrite_higher_order_template_expr(
                condition,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            body: rewrite_higher_order_template_block(
                body,
                callable_bindings,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_higher_order_template_expr(
            expr,
            callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Return(Some(value)) => AstStmt::Return(Some(rewrite_higher_order_template_expr(
            value,
            callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?)),
        AstStmt::Return(None) => AstStmt::Return(None),
        AstStmt::Break => AstStmt::Break,
        AstStmt::Continue => AstStmt::Continue,
    })
}

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
        } if templates.contains_key(callee) => specialize_higher_order_call(
            callee,
            args,
            generic_args,
            Some(callable_bindings),
            None,
            &BTreeMap::new(),
            templates,
            function_table,
            &[],
            &BTreeMap::new(),
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?,
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
