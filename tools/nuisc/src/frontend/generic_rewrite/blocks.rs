use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstMatchArm, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::{ast_named_type, infer_ast_expr_type, FunctionSignature};
use super::exprs::rewrite_generic_calls_in_expr;
use super::hoists::hoist_direct_result_wrapper_args;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_calls_in_block(
    body: &[AstStmt],
    current_return_type: Option<&AstTypeRef>,
    env: &mut BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Vec<AstStmt>, String> {
    let mut rewritten = Vec::new();
    for stmt in body {
        rewritten.extend(rewrite_generic_stmt_with_hoists(
            stmt,
            current_return_type,
            env,
            visible_type_aliases,
            generic_templates,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?);
    }
    Ok(rewritten)
}

#[allow(clippy::too_many_arguments)]
fn rewrite_generic_stmt_with_hoists(
    stmt: &AstStmt,
    current_return_type: Option<&AstTypeRef>,
    env: &mut BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Vec<AstStmt>, String> {
    match stmt {
        AstStmt::Let { name, ty, value } => {
            let AstExpr::Call {
                callee,
                generic_args,
                args,
            } = value
            else {
                return Ok(vec![rewrite_generic_calls_in_stmt(
                    stmt,
                    current_return_type,
                    env,
                    visible_type_aliases,
                    generic_templates,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?]);
            };
            if !generic_templates.contains_key(callee) {
                return Ok(vec![rewrite_generic_calls_in_stmt(
                    stmt,
                    current_return_type,
                    env,
                    visible_type_aliases,
                    generic_templates,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?]);
            }
            let (mut hoisted, rewritten_args) = hoist_direct_result_wrapper_args(
                args,
                name,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
                callee,
            )?;
            let rewritten_value = rewrite_generic_calls_in_expr(
                &AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                },
                ty.as_ref(),
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let inferred = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    &rewritten_value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
            });
            if let Some(inferred_ty) = &inferred {
                env.insert(name.clone(), inferred_ty.clone());
            }
            hoisted.push(AstStmt::Let {
                name: name.clone(),
                ty: ty.clone(),
                value: rewritten_value,
            });
            Ok(hoisted)
        }
        AstStmt::Return(Some(AstExpr::Call {
            callee,
            generic_args,
            args,
        })) if generic_templates.contains_key(callee) => {
            let (mut hoisted, rewritten_args) = hoist_direct_result_wrapper_args(
                args,
                "__nuis_generic_return_arg",
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
                callee,
            )?;
            let rewritten_value = rewrite_generic_calls_in_expr(
                &AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                },
                current_return_type,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            hoisted.push(AstStmt::Return(Some(rewritten_value)));
            Ok(hoisted)
        }
        _ => Ok(vec![rewrite_generic_calls_in_stmt(
            stmt,
            current_return_type,
            env,
            visible_type_aliases,
            generic_templates,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?]),
    }
}

#[allow(clippy::too_many_arguments)]
fn rewrite_generic_calls_in_stmt(
    stmt: &AstStmt,
    current_return_type: Option<&AstTypeRef>,
    env: &mut BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<AstStmt, String> {
    Ok(match stmt {
        AstStmt::Let { name, ty, value } => {
            let rewritten_value = rewrite_generic_calls_in_expr(
                value,
                ty.as_ref(),
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let inferred = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    &rewritten_value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
            });
            if let Some(inferred_ty) = &inferred {
                env.insert(name.clone(), inferred_ty.clone());
            }
            AstStmt::Let {
                name: name.clone(),
                ty: ty.clone(),
                value: rewritten_value,
            }
        }
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => AstStmt::DestructureLet {
            type_ref: type_ref.clone(),
            fields: fields.clone(),
            value: rewrite_generic_calls_in_expr(
                value,
                type_ref.as_ref(),
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?,
        },
        AstStmt::Const { name, ty, value } => {
            let rewritten_value = rewrite_generic_calls_in_expr(
                value,
                ty.as_ref(),
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let inferred = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    &rewritten_value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
            });
            if let Some(inferred_ty) = &inferred {
                env.insert(name.clone(), inferred_ty.clone());
            }
            AstStmt::Const {
                name: name.clone(),
                ty: inferred.or_else(|| ty.clone()),
                value: rewritten_value,
            }
        }
        AstStmt::Print(value) => AstStmt::Print(rewrite_generic_calls_in_expr(
            value,
            None,
            env,
            visible_type_aliases,
            generic_templates,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?),
        AstStmt::Await(value) => AstStmt::Await(rewrite_generic_calls_in_expr(
            value,
            None,
            env,
            visible_type_aliases,
            generic_templates,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?),
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let rewritten_condition = rewrite_generic_calls_in_expr(
                condition,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            AstStmt::If {
                condition: rewritten_condition,
                then_body: rewrite_generic_calls_in_block(
                    then_body,
                    current_return_type,
                    &mut then_env,
                    visible_type_aliases,
                    generic_templates,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?,
                else_body: rewrite_generic_calls_in_block(
                    else_body,
                    current_return_type,
                    &mut else_env,
                    visible_type_aliases,
                    generic_templates,
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
        AstStmt::Match { value, arms } => AstStmt::Match {
            value: rewrite_generic_calls_in_expr(
                value,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?,
            arms: rewrite_generic_calls_in_match_arms(
                arms,
                current_return_type,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?,
        },
        AstStmt::While { condition, body } => {
            let rewritten_condition = rewrite_generic_calls_in_expr(
                condition,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let mut loop_env = env.clone();
            AstStmt::While {
                condition: rewritten_condition,
                body: rewrite_generic_calls_in_block(
                    body,
                    current_return_type,
                    &mut loop_env,
                    visible_type_aliases,
                    generic_templates,
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
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_generic_calls_in_expr(
            expr,
            None,
            env,
            visible_type_aliases,
            generic_templates,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?),
        AstStmt::Return(value) => AstStmt::Return(match value {
            Some(value) => Some(rewrite_generic_calls_in_expr(
                value,
                current_return_type,
                env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?),
            None => None,
        }),
        AstStmt::Break => AstStmt::Break,
        AstStmt::Continue => AstStmt::Continue,
    })
}

#[allow(clippy::too_many_arguments)]
fn rewrite_generic_calls_in_match_arms(
    arms: &[AstMatchArm],
    current_return_type: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Vec<AstMatchArm>, String> {
    let mut rewritten = Vec::with_capacity(arms.len());
    for arm in arms {
        let mut arm_env = env.clone();
        rewritten.push(AstMatchArm {
            pattern: arm.pattern.clone(),
            guard: arm
                .guard
                .as_ref()
                .map(|guard| {
                    rewrite_generic_calls_in_expr(
                        guard,
                        Some(&ast_named_type("bool")),
                        &mut arm_env,
                        visible_type_aliases,
                        generic_templates,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    )
                })
                .transpose()?,
            body: rewrite_generic_calls_in_block(
                &arm.body,
                current_return_type,
                &mut arm_env,
                visible_type_aliases,
                generic_templates,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?,
        });
    }
    Ok(rewritten)
}
