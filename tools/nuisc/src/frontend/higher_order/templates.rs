use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstMatchArm, AstParam, AstStmt, AstTypeAlias, AstVisibility,
};

use super::callables::is_callable_type_with_aliases;
use super::expansion::BoundCallable;

#[path = "templates_expr.rs"]
mod templates_expr;

pub(crate) use templates_expr::rewrite_higher_order_template_expr;

pub(crate) struct HigherOrderTemplateSpecializationInput<'a> {
    pub(crate) template: &'a AstFunction,
    pub(crate) specialized_name: &'a str,
    pub(crate) callable_bindings: &'a BTreeMap<String, BoundCallable>,
    pub(crate) templates: &'a BTreeMap<String, AstFunction>,
    pub(crate) function_table: &'a BTreeMap<String, AstFunction>,
    pub(crate) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(crate) specialized_cache: &'a mut BTreeSet<String>,
    pub(crate) specialized_functions: &'a mut Vec<AstFunction>,
}

pub(crate) fn specialize_higher_order_template(
    input: HigherOrderTemplateSpecializationInput<'_>,
) -> Result<AstFunction, String> {
    let HigherOrderTemplateSpecializationInput {
        template,
        specialized_name,
        callable_bindings,
        templates,
        function_table,
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    } = input;
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
