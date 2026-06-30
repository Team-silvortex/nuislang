use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstMatchArm, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::validation_binding_env::bind_match_pattern_for_type;
use super::expansion_inference::{
    extend_local_field_bindings_from_expr, extend_local_field_bindings_from_type,
    infer_local_binding_type,
};
use super::expansion_rewrite_expr::rewrite_higher_order_calls_in_expr;

pub(crate) fn rewrite_higher_order_calls_in_function(
    function: &AstFunction,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    method_template_lookup: &BTreeMap<(String, String), String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstFunction, String> {
    let mut local_types = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    for param in &function.params {
        extend_local_field_bindings_from_type(
            &param.name,
            &param.ty,
            visible_structs,
            &mut local_types,
        );
    }
    let body = rewrite_higher_order_calls_in_block(
        &function.body,
        function.return_type.as_ref(),
        function.return_type.as_ref(),
        &local_types,
        templates,
        function_table,
        module_impls,
        visible_structs,
        method_template_lookup,
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
    tail_expected: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    method_template_lookup: &BTreeMap<(String, String), String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<Vec<AstStmt>, String> {
    let mut env = local_types.clone();
    let mut rewritten = Vec::with_capacity(body.len());
    for stmt in body {
        let rewritten_stmt = rewrite_higher_order_calls_in_stmt(
            stmt,
            current_return_type,
            tail_expected,
            &env,
            templates,
            function_table,
            module_impls,
            visible_structs,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?;
        match &rewritten_stmt {
            AstStmt::Let {
                name, ty, value, ..
            }
            | AstStmt::Const { name, ty, value } => {
                if let Some(ty) = ty.clone() {
                    env.insert(name.clone(), ty);
                    if let Some(bound_ty) = env.get(name).cloned() {
                        extend_local_field_bindings_from_type(
                            name,
                            &bound_ty,
                            visible_structs,
                            &mut env,
                        );
                    }
                } else if let Some(inferred_ty) =
                    infer_local_binding_type(value, &env, function_table, module_impls)
                {
                    env.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    value,
                    &mut env,
                    function_table,
                    module_impls,
                );
            }
            AstStmt::AssignLocal { name, value } => {
                if let Some(inferred_ty) =
                    infer_local_binding_type(value, &env, function_table, module_impls)
                {
                    env.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    value,
                    &mut env,
                    function_table,
                    module_impls,
                );
            }
            _ => {}
        }
        rewritten.push(rewritten_stmt);
    }
    Ok(rewritten)
}

pub(crate) fn rewrite_higher_order_calls_in_stmt(
    stmt: &AstStmt,
    current_return_type: Option<&AstTypeRef>,
    tail_expected: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    method_template_lookup: &BTreeMap<(String, String), String>,
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
            value: rewrite_higher_order_calls_in_expr(
                value,
                ty.as_ref(),
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::AssignLocal { name, value } => AstStmt::AssignLocal {
            name: name.clone(),
            value: rewrite_higher_order_calls_in_expr(
                value,
                current_return_type,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
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
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
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
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Print(value) => AstStmt::Print(rewrite_higher_order_calls_in_expr(
            value,
            None,
            current_return_type,
            local_types,
            templates,
            function_table,
            module_impls,
            visible_structs,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Await(value) => AstStmt::Await(rewrite_higher_order_calls_in_expr(
            value,
            None,
            current_return_type,
            local_types,
            templates,
            function_table,
            module_impls,
            visible_structs,
            method_template_lookup,
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
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                current_return_type,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                current_return_type,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Match { value, arms } => {
            let rewritten_value = rewrite_higher_order_calls_in_expr(
                value,
                None,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?;
            let scrutinee_type = infer_local_binding_type(
                &rewritten_value,
                local_types,
                function_table,
                module_impls,
            );
            AstStmt::Match {
                value: rewritten_value,
                arms: arms
                    .iter()
                    .map(|arm| {
                        let mut arm_local_types = local_types.clone();
                        if let Some(scrutinee_type) = scrutinee_type.as_ref() {
                            bind_match_pattern_for_type(
                                scrutinee_type,
                                &arm.pattern,
                                visible_type_aliases,
                                visible_structs,
                                &mut arm_local_types,
                            )?;
                        }
                        Ok(AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm
                                .guard
                                .as_ref()
                                .map(|guard| {
                                    rewrite_higher_order_calls_in_expr(
                                        guard,
                                        None,
                                        current_return_type,
                                        &arm_local_types,
                                        templates,
                                        function_table,
                                        module_impls,
                                        visible_structs,
                                        method_template_lookup,
                                        visible_type_aliases,
                                        specialized_cache,
                                        specialized_functions,
                                    )
                                })
                                .transpose()?,
                            body: rewrite_higher_order_calls_in_block(
                                &arm.body,
                                current_return_type,
                                current_return_type,
                                &arm_local_types,
                                templates,
                                function_table,
                                module_impls,
                                visible_structs,
                                method_template_lookup,
                                visible_type_aliases,
                                specialized_cache,
                                specialized_functions,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }
        }
        AstStmt::While { condition, body } => AstStmt::While {
            condition: rewrite_higher_order_calls_in_expr(
                condition,
                None,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            body: rewrite_higher_order_calls_in_block(
                body,
                current_return_type,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_higher_order_calls_in_expr(
            expr,
            tail_expected,
            current_return_type,
            local_types,
            templates,
            function_table,
            module_impls,
            visible_structs,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Return(Some(value)) => AstStmt::Return(Some(rewrite_higher_order_calls_in_expr(
            value,
            tail_expected.or(current_return_type),
            current_return_type,
            local_types,
            templates,
            function_table,
            module_impls,
            visible_structs,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?)),
        AstStmt::Return(None) => AstStmt::Return(None),
        AstStmt::Break => AstStmt::Break,
        AstStmt::Continue => AstStmt::Continue,
    })
}
