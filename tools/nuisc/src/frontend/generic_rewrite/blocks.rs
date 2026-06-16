use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstMatchArm, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::validation_binding_env::bind_match_pattern_for_type;
use super::super::{ast_named_type, infer_ast_expr_type, FunctionSignature};
use super::exprs::rewrite_generic_calls_in_expr;
use super::hoists::hoist_direct_result_wrapper_args;
use super::GenericImplMethodTemplate;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_calls_in_block(
    body: &[AstStmt],
    context: &str,
    current_return_type: Option<&AstTypeRef>,
    env: &mut BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Vec<AstStmt>, String> {
    let mut rewritten = Vec::new();
    for (index, stmt) in body.iter().enumerate() {
        let let_fallback_expected = let_binding_expected_type_from_following_use(
            stmt,
            &body[index + 1..],
            current_return_type,
            generic_templates,
            signatures,
            visible_type_aliases,
            struct_table,
        );
        rewritten.extend(rewrite_generic_stmt_with_hoists(
            stmt,
            context,
            let_fallback_expected.as_ref(),
            current_return_type,
            env,
            visible_type_aliases,
            generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
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
    context: &str,
    let_fallback_expected: Option<&AstTypeRef>,
    current_return_type: Option<&AstTypeRef>,
    env: &mut BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Vec<AstStmt>, String> {
    match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => {
            let AstExpr::Call {
                callee,
                generic_args,
                args,
            } = value
            else {
                return Ok(vec![rewrite_generic_calls_in_stmt(
                    stmt,
                    context,
                    let_fallback_expected,
                    current_return_type,
                    env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
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
                    context,
                    let_fallback_expected,
                    current_return_type,
                    env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
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
                callee,
                generic_args,
                args,
                ty.as_ref(),
                name,
                &format!("{context} local `{name}`"),
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let rewritten_value = rewrite_generic_calls_in_expr(
                &AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                },
                &format!("{context} local `{name}`"),
                ty.as_ref().or(let_fallback_expected),
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let inferred = ty
                .clone()
                .or_else(|| {
                    infer_ast_expr_type(
                        &rewritten_value,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                    )
                })
                .or_else(|| let_fallback_expected.cloned());
            if let Some(inferred_ty) = &inferred {
                env.insert(name.clone(), inferred_ty.clone());
            }
            hoisted.push(AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: inferred.or_else(|| ty.clone()),
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
                callee,
                generic_args,
                args,
                current_return_type,
                "__nuis_generic_return_arg",
                context,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let rewritten_value = rewrite_generic_calls_in_expr(
                &AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                },
                context,
                current_return_type,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
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
            context,
            let_fallback_expected,
            current_return_type,
            env,
            visible_type_aliases,
            generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
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
    context: &str,
    let_fallback_expected: Option<&AstTypeRef>,
    current_return_type: Option<&AstTypeRef>,
    env: &mut BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<AstStmt, String> {
    Ok(match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => {
            let rewritten_value = rewrite_generic_calls_in_expr(
                value,
                &format!("{context} local `{name}`"),
                ty.as_ref().or(let_fallback_expected),
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let inferred = ty
                .clone()
                .or_else(|| {
                    infer_ast_expr_type(
                        &rewritten_value,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                    )
                })
                .or_else(|| let_fallback_expected.cloned());
            if let Some(inferred_ty) = &inferred {
                env.insert(name.clone(), inferred_ty.clone());
            }
            AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: inferred.or_else(|| ty.clone()),
                value: rewritten_value,
            }
        }
        AstStmt::AssignLocal { name, value } => AstStmt::AssignLocal {
            name: name.clone(),
            value: rewrite_generic_calls_in_expr(
                value,
                &format!("{context} local `{name}`"),
                env.get(name),
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?,
        },
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => AstStmt::DestructureLet {
            type_ref: type_ref.clone(),
            fields: fields.clone(),
            value: rewrite_generic_calls_in_expr(
                value,
                &format!("{context} destructure"),
                type_ref.as_ref(),
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
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
                &format!("{context} const `{name}`"),
                ty.as_ref(),
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
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
            context,
            None,
            env,
            visible_type_aliases,
            generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
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
            context,
            None,
            env,
            visible_type_aliases,
            generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
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
                context,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
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
                    &format!("{context} if-then"),
                    current_return_type,
                    &mut then_env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
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
                    &format!("{context} if-else"),
                    current_return_type,
                    &mut else_env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
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
        AstStmt::Match { value, arms } => {
            let rewritten_value = rewrite_generic_calls_in_expr(
                value,
                context,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let scrutinee_type = infer_ast_expr_type(
                &rewritten_value,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
            );
            AstStmt::Match {
                value: rewritten_value,
                arms: rewrite_generic_calls_in_match_arms(
                    arms,
                    context,
                    scrutinee_type.as_ref(),
                    current_return_type,
                    env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
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
        AstStmt::While { condition, body } => {
            let rewritten_condition = rewrite_generic_calls_in_expr(
                condition,
                context,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
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
                    &format!("{context} while-body"),
                    current_return_type,
                    &mut loop_env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
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
            context,
            None,
            env,
            visible_type_aliases,
            generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
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
                context,
                current_return_type,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
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

fn let_binding_expected_type_from_following_use(
    stmt: &AstStmt,
    following_stmts: &[AstStmt],
    current_return_type: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let AstStmt::Let { name, ty, .. } = stmt else {
        return None;
    };
    if ty.is_some() {
        return None;
    }
    expected_type_for_var_from_following_stmts(
        name,
        following_stmts,
        current_return_type,
        generic_templates,
        signatures,
        visible_type_aliases,
        struct_table,
    )
}

fn expected_type_for_var_from_following_stmts(
    current_name: &str,
    following_stmts: &[AstStmt],
    current_return_type: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let (stmt, rest) = following_stmts.split_first()?;
    match stmt {
        AstStmt::Let {
            name,
            value: AstExpr::Var(source_name),
            ..
        } if source_name == current_name => expected_type_for_var_from_following_stmts(
            name,
            rest,
            current_return_type,
            generic_templates,
            signatures,
            visible_type_aliases,
            struct_table,
        ),
        AstStmt::Let {
            name,
            ty,
            mutable: _,
            value:
                AstExpr::Call {
                    callee,
                    generic_args,
                    args,
                },
        } => {
            let index = args.iter().position(
                |arg| matches!(arg, AstExpr::Var(var_name) if var_name == current_name),
            )?;
            let call_expected = ty.clone().or_else(|| {
                expected_type_for_var_from_following_stmts(
                    name,
                    rest,
                    current_return_type,
                    generic_templates,
                    signatures,
                    visible_type_aliases,
                    struct_table,
                )
            });
            super::exprs::call_arg_expected_type(
                callee,
                generic_args,
                index,
                call_expected.as_ref().or(current_return_type),
                generic_templates,
                signatures,
                visible_type_aliases,
                struct_table,
            )
        }
        AstStmt::Return(Some(AstExpr::Call {
            callee,
            generic_args,
            args,
        })) => args.iter().enumerate().find_map(|(index, arg)| {
            matches!(arg, AstExpr::Var(var_name) if var_name == current_name).then(|| {
                super::exprs::call_arg_expected_type(
                    callee,
                    generic_args,
                    index,
                    current_return_type,
                    generic_templates,
                    signatures,
                    visible_type_aliases,
                    struct_table,
                )
            })?
        }),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn rewrite_generic_calls_in_match_arms(
    arms: &[AstMatchArm],
    context: &str,
    scrutinee_type: Option<&AstTypeRef>,
    current_return_type: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
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
        if let Some(scrutinee_type) = scrutinee_type {
            bind_match_pattern_for_type(
                scrutinee_type,
                &arm.pattern,
                visible_type_aliases,
                struct_table,
                &mut arm_env,
            )?;
        }
        rewritten.push(AstMatchArm {
            pattern: arm.pattern.clone(),
            guard: arm
                .guard
                .as_ref()
                .map(|guard| {
                    rewrite_generic_calls_in_expr(
                        guard,
                        context,
                        Some(&ast_named_type("bool")),
                        &mut arm_env,
                        visible_type_aliases,
                        generic_templates,
                        generic_impl_method_templates,
                        higher_order_templates,
                        function_table,
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
                &format!("{context} match-arm"),
                current_return_type,
                &mut arm_env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
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
