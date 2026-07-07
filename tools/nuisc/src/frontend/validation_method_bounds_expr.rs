use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstImplDef, AstParam, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::validation_method_bounds_bounds::{
    binary_operator_trait_requirement, unary_operator_trait_requirement,
    validate_explicit_trait_call_bound, validate_generic_receiver_method_bound,
    validate_generic_receiver_operator_bound, ExplicitTraitCallBoundInput,
    GenericReceiverOperatorBoundInput,
};
use super::validation_method_bounds_stmt::{
    validate_stmt_generic_method_bounds_block, MethodBoundsBlockInput,
};
use super::{inferred_match_value_type, normalize_method_bound_context};
use crate::frontend::validation_binding_env::bind_match_pattern_for_type;
use crate::frontend::{infer_ast_expr_type, render_field_access_path};

#[derive(Clone, Copy)]
pub(in crate::frontend) struct MethodBoundsContext<'a> {
    pub(in crate::frontend) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(in crate::frontend) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(in crate::frontend) visible_structs: &'a BTreeMap<String, AstStructDef>,
    pub(in crate::frontend) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(in crate::frontend) trait_methods: &'a BTreeMap<String, BTreeSet<String>>,
    pub(in crate::frontend) generic_param_names: &'a BTreeSet<String>,
    pub(in crate::frontend) generic_bounds: &'a BTreeMap<String, Vec<String>>,
}

pub(in crate::frontend) struct MethodBoundsExprInput<'a> {
    pub(in crate::frontend) expr: &'a AstExpr,
    pub(in crate::frontend) bounds: MethodBoundsContext<'a>,
    pub(in crate::frontend) local_type_env: &'a BTreeMap<String, AstTypeRef>,
    pub(in crate::frontend) context: &'a str,
}

pub(in crate::frontend) fn validate_expr_generic_method_bounds(
    input: MethodBoundsExprInput<'_>,
) -> Result<(), String> {
    let MethodBoundsExprInput {
        expr,
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
    match expr {
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_) => {}
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr!(condition, local_type_env, context)?;
            let mut then_env = local_type_env.clone();
            let mut else_env = local_type_env.clone();
            validate_block!(then_body, &mut then_env, &format!("{context} if-then"))?;
            validate_block!(else_body, &mut else_env, &format!("{context} if-else"))?;
        }
        AstExpr::Match { value, arms } => {
            validate_expr!(value, local_type_env, context)?;
            let match_value_ty = inferred_match_value_type(
                value,
                local_type_env,
                bounds.impl_lookup,
                bounds.visible_structs,
                bounds.function_return_types,
            );
            for arm in arms {
                let mut arm_env = local_type_env.clone();
                if let Some(match_value_ty) = match_value_ty.as_ref() {
                    bind_match_pattern_for_type(
                        match_value_ty,
                        &arm.pattern,
                        bounds.visible_type_aliases,
                        bounds.visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = &arm.guard {
                    validate_expr!(guard, &arm_env, context)?;
                }
                validate_block!(&arm.body, &mut arm_env, context)?;
            }
        }
        AstExpr::Lambda {
            params,
            body,
            return_type: _,
        } => {
            let mut lambda_env = local_type_env.clone();
            for AstParam { name, ty } in params {
                lambda_env.insert(name.clone(), ty.clone());
            }
            validate_block!(body, &mut lambda_env, &format!("{context} lambda body"))?;
        }
        AstExpr::Instantiate { .. } => {}
        AstExpr::Try(value) | AstExpr::Await(value) | AstExpr::FieldAccess { base: value, .. } => {
            validate_expr!(value, local_type_env, context)?;
        }
        AstExpr::Unary { op, operand } => {
            validate_expr!(operand, local_type_env, context)?;
            if let Some((operator, method, required_bound)) = unary_operator_trait_requirement(*op)
            {
                if let Some(operand_ty) = infer_ast_expr_type(
                    operand,
                    local_type_env,
                    bounds.impl_lookup,
                    bounds.visible_structs,
                    bounds.function_return_types,
                ) {
                    validate_generic_receiver_operator_bound(GenericReceiverOperatorBoundInput {
                        receiver_ty: &operand_ty,
                        operator,
                        method,
                        required_bound,
                        visible_type_aliases: bounds.visible_type_aliases,
                        trait_methods: bounds.trait_methods,
                        generic_param_names: bounds.generic_param_names,
                        generic_bounds: bounds.generic_bounds,
                        context,
                    })?;
                }
            }
        }
        AstExpr::Call { callee, args, .. } => {
            for arg in args {
                validate_expr!(arg, local_type_env, context)?;
            }
            if let Some((trait_name, method)) = callee.rsplit_once('.') {
                if bounds.trait_methods.contains_key(trait_name) {
                    validate_explicit_trait_call_bound(ExplicitTraitCallBoundInput {
                        trait_name,
                        method,
                        args,
                        visible_type_aliases: bounds.visible_type_aliases,
                        impl_lookup: bounds.impl_lookup,
                        visible_structs: bounds.visible_structs,
                        function_return_types: bounds.function_return_types,
                        trait_methods: bounds.trait_methods,
                        generic_param_names: bounds.generic_param_names,
                        generic_bounds: bounds.generic_bounds,
                        local_type_env,
                        context,
                    })?;
                }
            }
        }
        AstExpr::Invoke { args, .. } => {
            for arg in args {
                validate_expr!(arg, local_type_env, context)?;
            }
        }
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args: _,
            args,
        } => {
            if let Some(receiver_name) = render_field_access_path(receiver) {
                let is_shadowed_simple_local = matches!(
                    receiver.as_ref(),
                    AstExpr::Var(name) if local_type_env.contains_key(name)
                );
                if !is_shadowed_simple_local && bounds.trait_methods.contains_key(&receiver_name) {
                    for arg in args {
                        validate_expr!(arg, local_type_env, context)?;
                    }
                    validate_explicit_trait_call_bound(ExplicitTraitCallBoundInput {
                        trait_name: &receiver_name,
                        method,
                        args,
                        visible_type_aliases: bounds.visible_type_aliases,
                        impl_lookup: bounds.impl_lookup,
                        visible_structs: bounds.visible_structs,
                        function_return_types: bounds.function_return_types,
                        trait_methods: bounds.trait_methods,
                        generic_param_names: bounds.generic_param_names,
                        generic_bounds: bounds.generic_bounds,
                        local_type_env,
                        context,
                    })?;
                    return Ok(());
                }
            }
            validate_expr!(receiver, local_type_env, context)?;
            for arg in args {
                validate_expr!(arg, local_type_env, context)?;
            }
            if let Some(receiver_ty) = infer_ast_expr_type(
                receiver,
                local_type_env,
                bounds.impl_lookup,
                bounds.visible_structs,
                bounds.function_return_types,
            ) {
                validate_generic_receiver_method_bound(
                    &receiver_ty,
                    method,
                    bounds.visible_type_aliases,
                    bounds.trait_methods,
                    bounds.generic_param_names,
                    bounds.generic_bounds,
                    context,
                )?;
            }
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_expr!(value, local_type_env, context)?;
            }
        }
        AstExpr::Binary { op, lhs, rhs } => {
            validate_expr!(lhs, local_type_env, context)?;
            validate_expr!(rhs, local_type_env, context)?;
            if let Some((operator, method, required_bound)) = binary_operator_trait_requirement(*op)
            {
                if let Some(lhs_ty) = infer_ast_expr_type(
                    lhs,
                    local_type_env,
                    bounds.impl_lookup,
                    bounds.visible_structs,
                    bounds.function_return_types,
                ) {
                    validate_generic_receiver_operator_bound(GenericReceiverOperatorBoundInput {
                        receiver_ty: &lhs_ty,
                        operator,
                        method,
                        required_bound,
                        visible_type_aliases: bounds.visible_type_aliases,
                        trait_methods: bounds.trait_methods,
                        generic_param_names: bounds.generic_param_names,
                        generic_bounds: bounds.generic_bounds,
                        context,
                    })?;
                }
            }
        }
    }
    Ok(())
}
