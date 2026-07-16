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

pub(crate) struct HigherOrderRewriteContext<'a> {
    pub(crate) templates: &'a BTreeMap<String, AstFunction>,
    pub(crate) function_table: &'a BTreeMap<String, AstFunction>,
    pub(crate) module_impls: &'a [AstImplDef],
    pub(crate) visible_structs: &'a BTreeMap<String, AstStructDef>,
    pub(crate) method_template_lookup: &'a BTreeMap<(String, String), String>,
    pub(crate) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(crate) specialized_cache: &'a mut BTreeSet<String>,
    pub(crate) specialized_functions: &'a mut Vec<AstFunction>,
}

pub(crate) struct HigherOrderFunctionRewriteInput<'a> {
    pub(crate) function: &'a AstFunction,
    pub(crate) templates: &'a BTreeMap<String, AstFunction>,
    pub(crate) function_table: &'a BTreeMap<String, AstFunction>,
    pub(crate) module_impls: &'a [AstImplDef],
    pub(crate) visible_structs: &'a BTreeMap<String, AstStructDef>,
    pub(crate) method_template_lookup: &'a BTreeMap<(String, String), String>,
    pub(crate) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(crate) specialized_cache: &'a mut BTreeSet<String>,
    pub(crate) specialized_functions: &'a mut Vec<AstFunction>,
}

pub(crate) fn rewrite_higher_order_calls_in_function(
    input: HigherOrderFunctionRewriteInput<'_>,
) -> Result<AstFunction, String> {
    let HigherOrderFunctionRewriteInput {
        function,
        templates,
        function_table,
        module_impls,
        visible_structs,
        method_template_lookup,
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    } = input;
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
    let mut context = HigherOrderRewriteContext {
        templates,
        function_table,
        module_impls,
        visible_structs,
        method_template_lookup,
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    };
    let body = rewrite_higher_order_calls_in_block(
        &function.body,
        function.return_type.as_ref(),
        function.return_type.as_ref(),
        &local_types,
        &mut context,
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
    context: &mut HigherOrderRewriteContext<'_>,
) -> Result<Vec<AstStmt>, String> {
    let mut env = local_types.clone();
    let mut rewritten = Vec::with_capacity(body.len());
    for (index, stmt) in body.iter().enumerate() {
        let stmt_with_tail_expected =
            let_binding_with_following_return_expected(stmt, &body[index + 1..], tail_expected);
        let stmt = stmt_with_tail_expected.as_ref().unwrap_or(stmt);
        let rewritten_stmt = rewrite_higher_order_calls_in_stmt(
            stmt,
            current_return_type,
            tail_expected,
            &env,
            context,
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
                            context.visible_structs,
                            &mut env,
                        );
                    }
                } else if let Some(inferred_ty) = infer_local_binding_type(
                    value,
                    &env,
                    context.function_table,
                    context.module_impls,
                ) {
                    env.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    value,
                    &mut env,
                    context.function_table,
                    context.module_impls,
                );
            }
            AstStmt::AssignLocal { name, value } => {
                if let Some(inferred_ty) = infer_local_binding_type(
                    value,
                    &env,
                    context.function_table,
                    context.module_impls,
                ) {
                    env.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    value,
                    &mut env,
                    context.function_table,
                    context.module_impls,
                );
            }
            _ => {}
        }
        rewritten.push(rewritten_stmt);
    }
    Ok(rewritten)
}

fn let_binding_with_following_return_expected(
    stmt: &AstStmt,
    following: &[AstStmt],
    tail_expected: Option<&AstTypeRef>,
) -> Option<AstStmt> {
    let expected = tail_expected?;
    let AstStmt::Let {
        name,
        ty: None,
        value,
        mutable,
    } = stmt
    else {
        return None;
    };
    matches!(
        following.first(),
        Some(AstStmt::Return(Some(nuis_semantics::model::AstExpr::Var(returned))))
            if returned == name
    )
    .then(|| AstStmt::Let {
        mutable: *mutable,
        name: name.clone(),
        ty: Some(expected.clone()),
        value: value.clone(),
    })
}

pub(crate) fn rewrite_higher_order_calls_in_stmt(
    stmt: &AstStmt,
    current_return_type: Option<&AstTypeRef>,
    tail_expected: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    context: &mut HigherOrderRewriteContext<'_>,
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
                context,
            )?,
        },
        AstStmt::AssignLocal { name, value } => AstStmt::AssignLocal {
            name: name.clone(),
            value: rewrite_higher_order_calls_in_expr(
                value,
                current_return_type,
                current_return_type,
                local_types,
                context,
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
                context,
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
                context,
            )?,
        },
        AstStmt::Print(value) => AstStmt::Print(rewrite_higher_order_calls_in_expr(
            value,
            None,
            current_return_type,
            local_types,
            context,
        )?),
        AstStmt::Await(value) => AstStmt::Await(rewrite_higher_order_calls_in_expr(
            value,
            None,
            current_return_type,
            local_types,
            context,
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
                context,
            )?,
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                current_return_type,
                current_return_type,
                local_types,
                context,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                current_return_type,
                current_return_type,
                local_types,
                context,
            )?,
        },
        AstStmt::Match { value, arms } => {
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
                                context.visible_type_aliases,
                                context.visible_structs,
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
                                        context,
                                    )
                                })
                                .transpose()?,
                            body: rewrite_higher_order_calls_in_block(
                                &arm.body,
                                current_return_type,
                                current_return_type,
                                &arm_local_types,
                                context,
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
                context,
            )?,
            body: rewrite_higher_order_calls_in_block(
                body,
                current_return_type,
                current_return_type,
                local_types,
                context,
            )?,
        },
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_higher_order_calls_in_expr(
            expr,
            tail_expected,
            current_return_type,
            local_types,
            context,
        )?),
        AstStmt::Return(Some(value)) => AstStmt::Return(Some(rewrite_higher_order_calls_in_expr(
            value,
            tail_expected.or(current_return_type),
            current_return_type,
            local_types,
            context,
        )?)),
        AstStmt::Return(None) => AstStmt::Return(None),
        AstStmt::Break => AstStmt::Break,
        AstStmt::Continue => AstStmt::Continue,
    })
}
