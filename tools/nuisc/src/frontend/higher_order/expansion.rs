use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstFunction, AstMatchArm, AstModule, AstStmt, AstTypeAlias};

use super::callables::{
    function_type_matches_callable, is_callable_type_with_aliases, sanitize_symbol_fragment,
};
use super::templates::specialize_higher_order_template;

pub(crate) fn expand_higher_order_functions(
    module: &AstModule,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<AstModule, String> {
    let templates = module
        .functions
        .iter()
        .filter(|function| {
            function.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            })
        })
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    if templates.is_empty() {
        return Ok(module.clone());
    }

    let function_table = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut expanded = module.clone();
    expanded.functions.clear();
    let mut specialized_cache = BTreeSet::new();
    let mut specialized_functions = Vec::new();

    for function in &module.functions {
        if templates.contains_key(&function.name) {
            continue;
        }
        expanded
            .functions
            .push(rewrite_higher_order_calls_in_function(
                function,
                &templates,
                &function_table,
                visible_type_aliases,
                &mut specialized_cache,
                &mut specialized_functions,
            )?);
    }
    expanded.functions.extend(specialized_functions);
    Ok(expanded)
}

pub(crate) fn rewrite_higher_order_calls_in_function(
    function: &AstFunction,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstFunction, String> {
    let body = rewrite_higher_order_calls_in_block(
        &function.body,
        templates,
        function_table,
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    )?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok(rewritten)
}

pub(crate) fn rewrite_higher_order_calls_in_block(
    body: &[AstStmt],
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<Vec<AstStmt>, String> {
    body.iter()
        .map(|stmt| {
            rewrite_higher_order_calls_in_stmt(
                stmt,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )
        })
        .collect()
}

pub(crate) fn rewrite_higher_order_calls_in_stmt(
    stmt: &AstStmt,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstStmt, String> {
    Ok(match stmt {
        AstStmt::Let { name, ty, value } => AstStmt::Let {
            name: name.clone(),
            ty: ty.clone(),
            value: rewrite_higher_order_calls_in_expr(
                value,
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
            value: rewrite_higher_order_calls_in_expr(
                value,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Print(value) => AstStmt::Print(rewrite_higher_order_calls_in_expr(
            value,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Await(value) => AstStmt::Await(rewrite_higher_order_calls_in_expr(
            value,
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
            condition: rewrite_higher_order_calls_in_expr(
                condition,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Match { value, arms } => AstStmt::Match {
            value: rewrite_higher_order_calls_in_expr(
                value,
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
                                rewrite_higher_order_calls_in_expr(
                                    guard,
                                    templates,
                                    function_table,
                                    visible_type_aliases,
                                    specialized_cache,
                                    specialized_functions,
                                )
                            })
                            .transpose()?,
                        body: rewrite_higher_order_calls_in_block(
                            &arm.body,
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
            condition: rewrite_higher_order_calls_in_expr(
                condition,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            body: rewrite_higher_order_calls_in_block(
                body,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_higher_order_calls_in_expr(
            expr,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Return(Some(value)) => AstStmt::Return(Some(rewrite_higher_order_calls_in_expr(
            value,
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

pub(crate) fn rewrite_higher_order_calls_in_expr(
    expr: &AstExpr,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::Call { callee, args } if templates.contains_key(callee) => {
            specialize_higher_order_call(
                callee,
                args,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_higher_order_calls_in_expr(
            value,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?)),
        AstExpr::Call { callee, args } => AstExpr::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_calls_in_expr(
                        arg,
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
            callee: Box::new(rewrite_higher_order_calls_in_expr(
                callee,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_calls_in_expr(
                        arg,
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
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(rewrite_higher_order_calls_in_expr(
                receiver,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_calls_in_expr(
                        arg,
                        templates,
                        function_table,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::StructLiteral { type_name, fields } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        rewrite_higher_order_calls_in_expr(
                            value,
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
            base: Box::new(rewrite_higher_order_calls_in_expr(
                base,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            field: field.clone(),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_higher_order_calls_in_expr(
                lhs,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            rhs: Box::new(rewrite_higher_order_calls_in_expr(
                rhs,
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

pub(crate) fn specialize_higher_order_call(
    callee: &str,
    args: &[AstExpr],
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    let template = templates
        .get(callee)
        .ok_or_else(|| format!("unknown higher-order template `{callee}`"))?;
    if template.params.len() != args.len() {
        return Err(format!(
            "function `{}` expects {} args, found {}",
            callee,
            template.params.len(),
            args.len()
        ));
    }

    let mut callable_bindings = BTreeMap::<String, String>::new();
    let mut ordinary_args = Vec::new();
    let mut callable_fragments = Vec::new();
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();

    for (param, arg) in template.params.iter().zip(args) {
        let rewritten_arg = rewrite_higher_order_calls_in_expr(
            arg,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?;
        if is_callable_type_with_aliases(&param.ty, visible_type_aliases)? {
            let AstExpr::Var(callable_name) = &rewritten_arg else {
                return Err(format!(
                    "higher-order parameter `{}` currently expects a no-capture lambda or named function symbol",
                    param.name
                ));
            };
            let callable = function_table.get(callable_name).ok_or_else(|| {
                format!(
                    "higher-order parameter `{}` references unknown callable `{}`",
                    param.name, callable_name
                )
            })?;
            if !function_type_matches_callable(
                callable,
                &param.ty,
                &generic_names,
                visible_type_aliases,
            )? {
                return Err(format!(
                    "callable `{}` does not match higher-order parameter `{}` of type `{}`",
                    callable_name, param.name, param.ty.name
                ));
            }
            callable_bindings.insert(param.name.clone(), callable_name.clone());
            callable_fragments.push(sanitize_symbol_fragment(callable_name));
        } else {
            ordinary_args.push(rewritten_arg);
        }
    }

    let specialized_name = format!("__hof_{}_{}", callee, callable_fragments.join("__"));
    if specialized_cache.insert(specialized_name.clone()) {
        let specialized = specialize_higher_order_template(
            template,
            &specialized_name,
            &callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?;
        specialized_functions.push(specialized.clone());
    }

    Ok(AstExpr::Call {
        callee: specialized_name,
        args: ordinary_args,
    })
}
