use std::collections::BTreeMap;

use nuis_semantics::model::{AstMatchArm, AstStmt, AstTypeRef};

use super::validation_method_bounds_expr::{
    validate_expr_generic_method_bounds, MethodBoundsContext, MethodBoundsExprInput,
};
use super::{inferred_match_value_type, normalize_method_bound_context};
use crate::frontend::infer_ast_expr_type;
use crate::frontend::validation_binding_env::{
    bind_destructure_fields_for_type, bind_match_pattern_for_type,
};

pub(super) fn validate_stmt_generic_method_bounds_block(
    input: MethodBoundsBlockInput<'_>,
) -> Result<(), String> {
    let MethodBoundsBlockInput {
        body,
        bounds,
        local_type_env,
        context,
    } = input;
    for stmt in body {
        validate_stmt_generic_method_bounds(MethodBoundsStmtInput {
            stmt,
            bounds,
            local_type_env,
            context,
        })?;
    }
    Ok(())
}

pub(super) struct MethodBoundsBlockInput<'a> {
    pub(super) body: &'a [AstStmt],
    pub(super) bounds: MethodBoundsContext<'a>,
    pub(super) local_type_env: &'a mut BTreeMap<String, AstTypeRef>,
    pub(super) context: &'a str,
}

struct MethodBoundsStmtInput<'a> {
    stmt: &'a AstStmt,
    bounds: MethodBoundsContext<'a>,
    local_type_env: &'a mut BTreeMap<String, AstTypeRef>,
    context: &'a str,
}

fn validate_stmt_generic_method_bounds(input: MethodBoundsStmtInput<'_>) -> Result<(), String> {
    let MethodBoundsStmtInput {
        stmt,
        bounds,
        local_type_env,
        context,
    } = input;
    let normalized_context = normalize_method_bound_context(context);
    let context = normalized_context.as_str();
    macro_rules! validate_expr {
        ($expr:expr, $local_type_env:expr, $context:expr) => {
            validate_expr_generic_method_bounds(MethodBoundsExprInput {
                expr: $expr,
                bounds,
                local_type_env: $local_type_env,
                context: $context,
            })
        };
    }
    macro_rules! validate_block {
        ($body:expr, $local_type_env:expr, $context:expr) => {
            validate_stmt_generic_method_bounds_block(MethodBoundsBlockInput {
                body: $body,
                bounds,
                local_type_env: $local_type_env,
                context: $context,
            })
        };
    }
    match stmt {
        AstStmt::Let {
            name, ty, value, ..
        }
        | AstStmt::Const { name, ty, value } => {
            validate_expr!(value, local_type_env, context)?;
            if let Some(ty) = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    bounds.impl_lookup,
                    bounds.visible_structs,
                    bounds.function_return_types,
                )
            }) {
                local_type_env.insert(name.clone(), ty);
            }
        }
        AstStmt::AssignLocal { name, value } => {
            validate_expr!(value, local_type_env, context)?;
            if let Some(ty) = local_type_env.get(name).cloned() {
                local_type_env.insert(name.clone(), ty);
            }
        }
        AstStmt::DestructureLet { value, .. } => {
            validate_expr!(value, local_type_env, context)?;
            let AstStmt::DestructureLet {
                type_ref, fields, ..
            } = stmt
            else {
                unreachable!();
            };
            let root_type = type_ref.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    bounds.impl_lookup,
                    bounds.visible_structs,
                    bounds.function_return_types,
                )
            });
            if let Some(root_type) = root_type.as_ref() {
                bind_destructure_fields_for_type(
                    root_type,
                    fields,
                    bounds.visible_type_aliases,
                    bounds.visible_structs,
                    local_type_env,
                )?;
            }
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr!(condition, local_type_env, context)?;
            let mut then_env = local_type_env.clone();
            validate_block!(then_body, &mut then_env, context)?;
            let mut else_env = local_type_env.clone();
            validate_block!(else_body, &mut else_env, context)?;
        }
        AstStmt::Match { value, arms } => {
            validate_expr!(value, local_type_env, context)?;
            let match_value_ty = inferred_match_value_type(
                value,
                local_type_env,
                bounds.impl_lookup,
                bounds.visible_structs,
                bounds.function_return_types,
            );
            for AstMatchArm {
                pattern,
                guard,
                body,
            } in arms
            {
                let mut arm_env = local_type_env.clone();
                if let Some(match_value_ty) = match_value_ty.as_ref() {
                    bind_match_pattern_for_type(
                        match_value_ty,
                        pattern,
                        bounds.visible_type_aliases,
                        bounds.visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = guard {
                    validate_expr!(guard, &arm_env, context)?;
                }
                validate_block!(body, &mut arm_env, context)?;
            }
        }
        AstStmt::While { condition, body } => {
            validate_expr!(condition, local_type_env, context)?;
            let mut loop_env = local_type_env.clone();
            validate_block!(body, &mut loop_env, context)?;
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
            validate_expr!(value, local_type_env, context)?;
        }
        AstStmt::Return(Some(value)) => {
            validate_expr!(value, local_type_env, context)?;
        }
        AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
    }
    Ok(())
}
