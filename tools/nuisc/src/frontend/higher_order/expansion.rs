use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstMatchArm, AstModule, AstParam, AstStmt, AstTypeAlias, AstTypeRef,
};

use super::super::generics::{specialize_ast_type_ref, unify_generic_type_pattern};
use super::super::{lower_type_ref, resolve_ast_type_ref_aliases};
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
        function.return_type.as_ref(),
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
    current_return_type: Option<&AstTypeRef>,
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
                current_return_type,
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
    current_return_type: Option<&AstTypeRef>,
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
                ty.as_ref(),
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
            value: rewrite_higher_order_calls_in_expr(
                value,
                type_ref.as_ref(),
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
                ty.as_ref(),
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Print(value) => AstStmt::Print(rewrite_higher_order_calls_in_expr(
            value,
            None,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Await(value) => AstStmt::Await(rewrite_higher_order_calls_in_expr(
            value,
            None,
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
                None,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                current_return_type,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                current_return_type,
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
                None,
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
                                    None,
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
                            current_return_type,
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
                None,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            body: rewrite_higher_order_calls_in_block(
                body,
                current_return_type,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_higher_order_calls_in_expr(
            expr,
            None,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Return(Some(value)) => AstStmt::Return(Some(rewrite_higher_order_calls_in_expr(
            value,
            current_return_type,
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
    expected: Option<&AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(rewrite_higher_order_calls_in_expr(
                condition,
                Some(&super::super::ast_named_type("bool")),
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                expected,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                expected,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(rewrite_higher_order_calls_in_expr(
                value,
                None,
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
                            Some(guard) => Some(rewrite_higher_order_calls_in_expr(
                                guard,
                                Some(&super::super::ast_named_type("bool")),
                                templates,
                                function_table,
                                visible_type_aliases,
                                specialized_cache,
                                specialized_functions,
                            )?),
                            None => None,
                        },
                        body: rewrite_higher_order_calls_in_block(
                            &arm.body,
                            expected,
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
        } if templates.contains_key(callee) => {
            if !generic_args.is_empty() {
                return Err(format!(
                    "explicit generic arguments are not yet supported for higher-order template call `{callee}<...>(...)`"
                ));
            }
            specialize_higher_order_call(
                callee,
                args,
                expected,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_higher_order_calls_in_expr(
            value,
            expected,
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
                    rewrite_higher_order_calls_in_expr(
                        arg,
                        None,
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
                None,
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
                        None,
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
                None,
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
                        None,
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
                        rewrite_higher_order_calls_in_expr(
                            value,
                            None,
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
                None,
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
                None,
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            rhs: Box::new(rewrite_higher_order_calls_in_expr(
                rhs,
                None,
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
    expected: Option<&AstTypeRef>,
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
        let ordinary_expected = higher_order_param_expected_type_from_call_expected(
            template,
            param,
            expected,
            visible_type_aliases,
        );
        let rewritten_arg = rewrite_higher_order_calls_in_expr(
            arg,
            ordinary_expected.as_ref(),
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
            ordinary_args.push(annotate_expr_head_with_expected_type(
                rewritten_arg,
                ordinary_expected.as_ref(),
            ));
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
        generic_args: Vec::new(),
        args: ordinary_args,
    })
}

fn higher_order_param_expected_type_from_call_expected(
    template: &AstFunction,
    param: &AstParam,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    let return_pattern = template.return_type.as_ref()?;
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    if generic_names.is_empty() {
        return None;
    }
    let resolved_return_pattern =
        resolve_ast_type_ref_aliases(return_pattern, visible_type_aliases).ok()?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    unify_generic_type_pattern(
        &resolved_return_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        &template.name,
    )
    .ok()?;
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok()
}

fn annotate_expr_head_with_expected_type(expr: AstExpr, expected: Option<&AstTypeRef>) -> AstExpr {
    let Some(expected) = expected else {
        return expr;
    };
    match expr {
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty()
            && callee == expected.name
            && !expected.generic_args.is_empty() =>
        {
            AstExpr::Call {
                callee,
                generic_args: expected.generic_args.clone(),
                args,
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } if type_args.is_empty()
            && type_name == expected.name
            && !expected.generic_args.is_empty() =>
        {
            AstExpr::StructLiteral {
                type_name,
                type_args: expected.generic_args.clone(),
                fields,
            }
        }
        other => other,
    }
}
