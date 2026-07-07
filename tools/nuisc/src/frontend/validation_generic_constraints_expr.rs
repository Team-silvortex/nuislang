use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstMatchArm, AstTypeRef};

use super::super::validation_binding_env::{bind_match_pattern_for_type, simple_match_value_type};
use super::validation_generic_constraints_types::{
    validate_ast_type_ref_generic_constraints, AstTypeConstraintInput,
    GenericConstraintValidationContext,
};
use super::{validate_stmt_generic_constraints, StmtGenericConstraintInput};

pub(super) struct ExprGenericConstraintInput<'a> {
    pub(super) expr: &'a AstExpr,
    pub(super) validation: GenericConstraintValidationContext<'a>,
    pub(super) visible_trait_methods: &'a BTreeMap<String, BTreeSet<String>>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) generic_param_names: &'a BTreeSet<String>,
    pub(super) generic_bounds: &'a BTreeMap<String, Vec<String>>,
    pub(super) local_type_env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) context: &'a str,
}

pub(super) fn validate_expr_generic_constraints(
    input: ExprGenericConstraintInput<'_>,
) -> Result<(), String> {
    let ExprGenericConstraintInput {
        expr,
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
    match expr {
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => {}
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr!(condition, local_type_env, context)?;
            let mut then_env = local_type_env.clone();
            for nested in then_body {
                validate_stmt!(nested, &mut then_env, &format!("{context} if-then"))?;
            }
            let mut else_env = local_type_env.clone();
            for nested in else_body {
                validate_stmt!(nested, &mut else_env, &format!("{context} if-else"))?;
            }
        }
        AstExpr::Match { value, arms } => {
            validate_expr!(value, local_type_env, context)?;
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
                }
                for nested in body {
                    validate_stmt!(nested, &mut arm_env, &format!("{context} match-arm"))?;
                }
            }
        }
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => {
            let mut lambda_env = local_type_env.clone();
            for param in params {
                validate_ast_type_ref_generic_constraints(AstTypeConstraintInput {
                    ty: &param.ty,
                    validation,
                    generic_bounds,
                    context: &format!("{context} lambda parameter `{}`", param.name),
                })?;
                lambda_env.insert(param.name.clone(), param.ty.clone());
            }
            if let Some(return_type) = return_type {
                validate_ast_type_ref_generic_constraints(AstTypeConstraintInput {
                    ty: return_type,
                    validation,
                    generic_bounds,
                    context: &format!("{context} lambda return type"),
                })?;
            }
            for nested in body {
                validate_stmt!(nested, &mut lambda_env, &format!("{context} lambda body"))?;
            }
        }
        AstExpr::Try(value) | AstExpr::Await(value) | AstExpr::FieldAccess { base: value, .. } => {
            validate_expr!(value, local_type_env, context)?;
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            for (index, generic_arg) in generic_args.iter().enumerate() {
                validate_ast_type_ref_generic_constraints(AstTypeConstraintInput {
                    ty: generic_arg,
                    validation,
                    generic_bounds,
                    context: &format!("{context} call `{callee}` generic argument #{}", index + 1),
                })?;
            }
            for arg in args {
                validate_expr!(arg, local_type_env, context)?;
            }
        }
        AstExpr::Invoke { callee, args } => {
            validate_expr!(callee, local_type_env, context)?;
            for arg in args {
                validate_expr!(arg, local_type_env, context)?;
            }
        }
        AstExpr::MethodCall {
            receiver,
            generic_args,
            args,
            ..
        } => {
            validate_expr!(receiver, local_type_env, context)?;
            for (index, generic_arg) in generic_args.iter().enumerate() {
                validate_ast_type_ref_generic_constraints(AstTypeConstraintInput {
                    ty: generic_arg,
                    validation,
                    generic_bounds,
                    context: &format!("{context} method call generic argument #{}", index + 1),
                })?;
            }
            for arg in args {
                validate_expr!(arg, local_type_env, context)?;
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let literal_ty = AstTypeRef {
                name: type_name
                    .rsplit_once('.')
                    .and_then(|(parent, _)| {
                        validation
                            .visible_enums
                            .contains_key(parent)
                            .then_some(parent)
                    })
                    .unwrap_or(type_name)
                    .to_owned(),
                generic_args: type_args.clone(),
                is_optional: false,
                is_ref: false,
            };
            validate_ast_type_ref_generic_constraints(AstTypeConstraintInput {
                ty: &literal_ty,
                validation,
                generic_bounds,
                context: &format!("{context} struct literal `{type_name}`"),
            })?;
            for (_, value) in fields {
                validate_expr!(value, local_type_env, context)?;
            }
        }
        AstExpr::Unary { operand, .. } => {
            validate_expr!(operand, local_type_env, context)?;
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_expr!(lhs, local_type_env, context)?;
            validate_expr!(rhs, local_type_env, context)?;
        }
    }
    Ok(())
}
