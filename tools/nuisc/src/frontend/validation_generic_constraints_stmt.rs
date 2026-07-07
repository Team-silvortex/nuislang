use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstMatchArm, AstStmt, AstTypeRef};

use super::super::infer_ast_expr_type;
use super::super::validation_binding_env::{
    bind_destructure_fields_for_type, bind_match_pattern_for_type, simple_match_value_type,
};
use super::super::validation_method_bounds::{
    validate_expr_generic_method_bounds, MethodBoundsContext, MethodBoundsExprInput,
};
use super::validation_generic_constraints_expr::{
    validate_expr_generic_constraints, ExprGenericConstraintInput,
};
use super::validation_generic_constraints_types::{
    validate_ast_type_ref_generic_constraints, AstTypeConstraintInput,
    GenericConstraintValidationContext,
};

pub(super) struct StmtGenericConstraintInput<'a> {
    pub(super) stmt: &'a AstStmt,
    pub(super) validation: GenericConstraintValidationContext<'a>,
    pub(super) visible_trait_methods: &'a BTreeMap<String, BTreeSet<String>>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) generic_param_names: &'a BTreeSet<String>,
    pub(super) generic_bounds: &'a BTreeMap<String, Vec<String>>,
    pub(super) local_type_env: &'a mut BTreeMap<String, AstTypeRef>,
    pub(super) context: &'a str,
}

pub(super) fn validate_stmt_generic_constraints(
    input: StmtGenericConstraintInput<'_>,
) -> Result<(), String> {
    let StmtGenericConstraintInput {
        stmt,
        validation,
        visible_trait_methods,
        function_return_types,
        generic_param_names,
        generic_bounds,
        local_type_env,
        context,
    } = input;
    macro_rules! validate_expr {
        ($expr:expr, $local_type_env:expr, $context:expr) => {
            validate_expr_generic_constraints(ExprGenericConstraintInput {
                expr: $expr,
                validation,
                visible_trait_methods,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env: $local_type_env,
                context: $context,
            })
        };
    }
    macro_rules! validate_stmt {
        ($stmt:expr, $local_type_env:expr, $context:expr) => {
            validate_stmt_generic_constraints(StmtGenericConstraintInput {
                stmt: $stmt,
                validation,
                visible_trait_methods,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env: $local_type_env,
                context: $context,
            })
        };
    }
    macro_rules! validate_method_bounds {
        ($expr:expr, $local_type_env:expr, $context:expr) => {
            validate_expr_generic_method_bounds(MethodBoundsExprInput {
                expr: $expr,
                bounds: MethodBoundsContext {
                    visible_type_aliases: validation.visible_type_aliases,
                    impl_lookup: validation.impl_lookup,
                    visible_structs: validation.visible_structs,
                    function_return_types,
                    trait_methods: visible_trait_methods,
                    generic_param_names,
                    generic_bounds,
                },
                local_type_env: $local_type_env,
                context: $context,
            })
        };
    }
    match stmt {
        AstStmt::Let { name, ty, .. } | AstStmt::Const { name, ty, .. } => {
            let value = match stmt {
                AstStmt::Let { value, .. } | AstStmt::Const { value, .. } => value,
                _ => unreachable!(),
            };
            validate_expr!(value, local_type_env, context)?;
            validate_method_bounds!(value, local_type_env, context)?;
            if let Some(ty) = ty {
                // Keep explicit annotation validation as its own pass after
                // walking the value expression. In deep expected-type chains,
                // this is where constrained aliases intentionally get a chance
                // to report their own bound failure context, even if an inner
                // generic call was also inferable from the same expected type.
                validate_ast_type_ref_generic_constraints(AstTypeConstraintInput {
                    ty,
                    validation,
                    generic_bounds,
                    context: &format!("{context} local `{name}`"),
                })?;
            }
            if let Some(inferred_ty) = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    validation.impl_lookup,
                    validation.visible_structs,
                    function_return_types,
                )
            }) {
                let name = match stmt {
                    AstStmt::Let { name, .. } | AstStmt::Const { name, .. } => name.clone(),
                    _ => unreachable!(),
                };
                local_type_env.insert(name, inferred_ty);
            }
        }
        AstStmt::AssignLocal { name, value } => {
            validate_expr!(value, local_type_env, context)?;
            validate_method_bounds!(value, local_type_env, context)?;
            if let Some(inferred_ty) = infer_ast_expr_type(
                value,
                local_type_env,
                validation.impl_lookup,
                validation.visible_structs,
                function_return_types,
            )
            .or_else(|| local_type_env.get(name).cloned())
            {
                local_type_env.insert(name.clone(), inferred_ty);
            }
        }
        AstStmt::DestructureLet { type_ref, .. } => {
            let value = match stmt {
                AstStmt::DestructureLet { value, .. } => value,
                _ => unreachable!(),
            };
            validate_expr!(value, local_type_env, context)?;
            validate_method_bounds!(value, local_type_env, context)?;
            if let Some(type_ref) = type_ref {
                validate_ast_type_ref_generic_constraints(AstTypeConstraintInput {
                    ty: type_ref,
                    validation,
                    generic_bounds,
                    context: &format!("{context} destructure type"),
                })?;
            }
            let fields = match stmt {
                AstStmt::DestructureLet { fields, .. } => fields,
                _ => unreachable!(),
            };
            let root_type = type_ref.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    validation.impl_lookup,
                    validation.visible_structs,
                    function_return_types,
                )
            });
            if let Some(root_type) = root_type.as_ref() {
                bind_destructure_fields_for_type(
                    root_type,
                    fields,
                    validation.visible_type_aliases,
                    validation.visible_structs,
                    local_type_env,
                )?;
            }
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            validate_expr!(condition, local_type_env, context)?;
            validate_method_bounds!(condition, local_type_env, context)?;
            let mut then_env = local_type_env.clone();
            for nested in then_body {
                validate_stmt!(nested, &mut then_env, context)?;
            }
            let mut else_env = local_type_env.clone();
            for nested in else_body {
                validate_stmt!(nested, &mut else_env, context)?;
            }
        }
        AstStmt::Match { value, arms } => {
            validate_expr!(value, local_type_env, context)?;
            validate_method_bounds!(value, local_type_env, context)?;
            let match_value_ty = simple_match_value_type(value, local_type_env);
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
                        validation.visible_type_aliases,
                        validation.visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = guard {
                    validate_expr!(guard, &arm_env, context)?;
                    validate_method_bounds!(guard, &arm_env, context)?;
                }
                for nested in body {
                    validate_stmt!(nested, &mut arm_env, context)?;
                }
            }
        }
        AstStmt::While { condition, body } => {
            validate_expr!(condition, local_type_env, context)?;
            validate_method_bounds!(condition, local_type_env, context)?;
            let mut loop_env = local_type_env.clone();
            for nested in body {
                validate_stmt!(nested, &mut loop_env, context)?;
            }
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
            validate_expr!(value, local_type_env, context)?;
            validate_method_bounds!(value, local_type_env, context)?;
        }
        AstStmt::Return(Some(value)) => {
            validate_expr!(value, local_type_env, context)?;
            validate_method_bounds!(value, local_type_env, context)?;
        }
        AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
    }
    Ok(())
}
