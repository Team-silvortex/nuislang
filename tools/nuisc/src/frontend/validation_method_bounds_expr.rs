use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstImplDef, AstParam, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::validation_method_bounds_bounds::{
    binary_operator_trait_requirement, unary_operator_trait_requirement,
    validate_explicit_trait_call_bound, validate_generic_receiver_method_bound,
    validate_generic_receiver_operator_bound,
};
use super::validation_method_bounds_stmt::validate_stmt_generic_method_bounds_block;
use super::{inferred_match_value_type, normalize_method_bound_context};
use crate::frontend::validation_binding_env::bind_match_pattern_for_type;
use crate::frontend::{infer_ast_expr_type, render_field_access_path};

pub(in crate::frontend) fn validate_expr_generic_method_bounds(
    expr: &AstExpr,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    local_type_env: &BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    let normalized_context = normalize_method_bound_context(context);
    let context = normalized_context.as_str();
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
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut then_env = local_type_env.clone();
            let mut else_env = local_type_env.clone();
            validate_stmt_generic_method_bounds_block(
                then_body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut then_env,
                &format!("{context} if-then"),
            )?;
            validate_stmt_generic_method_bounds_block(
                else_body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut else_env,
                &format!("{context} if-else"),
            )?;
        }
        AstExpr::Match { value, arms } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let match_value_ty = inferred_match_value_type(
                value,
                local_type_env,
                impl_lookup,
                visible_structs,
                function_return_types,
            );
            for arm in arms {
                let mut arm_env = local_type_env.clone();
                if let Some(match_value_ty) = match_value_ty.as_ref() {
                    bind_match_pattern_for_type(
                        match_value_ty,
                        &arm.pattern,
                        visible_type_aliases,
                        visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = &arm.guard {
                    validate_expr_generic_method_bounds(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
                    )?;
                }
                validate_stmt_generic_method_bounds_block(
                    &arm.body,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    &mut arm_env,
                    context,
                )?;
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
            validate_stmt_generic_method_bounds_block(
                body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut lambda_env,
                &format!("{context} lambda body"),
            )?;
        }
        AstExpr::Instantiate { .. } => {}
        AstExpr::Try(value) | AstExpr::Await(value) | AstExpr::FieldAccess { base: value, .. } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstExpr::Unary { op, operand } => {
            validate_expr_generic_method_bounds(
                operand,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some((operator, method, required_bound)) = unary_operator_trait_requirement(*op)
            {
                if let Some(operand_ty) = infer_ast_expr_type(
                    operand,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                ) {
                    validate_generic_receiver_operator_bound(
                        &operand_ty,
                        operator,
                        method,
                        required_bound,
                        visible_type_aliases,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        context,
                    )?;
                }
            }
        }
        AstExpr::Call { callee, args, .. } => {
            for arg in args {
                validate_expr_generic_method_bounds(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
            if let Some((trait_name, method)) = callee.rsplit_once('.') {
                if trait_methods.contains_key(trait_name) {
                    validate_explicit_trait_call_bound(
                        trait_name,
                        method,
                        args,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        local_type_env,
                        context,
                    )?;
                }
            }
        }
        AstExpr::Invoke { args, .. } => {
            for arg in args {
                validate_expr_generic_method_bounds(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
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
                if !is_shadowed_simple_local && trait_methods.contains_key(&receiver_name) {
                    for arg in args {
                        validate_expr_generic_method_bounds(
                            arg,
                            visible_type_aliases,
                            impl_lookup,
                            visible_structs,
                            function_return_types,
                            trait_methods,
                            generic_param_names,
                            generic_bounds,
                            local_type_env,
                            context,
                        )?;
                    }
                    validate_explicit_trait_call_bound(
                        &receiver_name,
                        method,
                        args,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        local_type_env,
                        context,
                    )?;
                    return Ok(());
                }
            }
            validate_expr_generic_method_bounds(
                receiver,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            for arg in args {
                validate_expr_generic_method_bounds(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
            if let Some(receiver_ty) = infer_ast_expr_type(
                receiver,
                local_type_env,
                impl_lookup,
                visible_structs,
                function_return_types,
            ) {
                validate_generic_receiver_method_bound(
                    &receiver_ty,
                    method,
                    visible_type_aliases,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    context,
                )?;
            }
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_expr_generic_method_bounds(
                    value,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::Binary { op, lhs, rhs } => {
            validate_expr_generic_method_bounds(
                lhs,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                rhs,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some((operator, method, required_bound)) = binary_operator_trait_requirement(*op)
            {
                if let Some(lhs_ty) = infer_ast_expr_type(
                    lhs,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                ) {
                    validate_generic_receiver_operator_bound(
                        &lhs_ty,
                        operator,
                        method,
                        required_bound,
                        visible_type_aliases,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        context,
                    )?;
                }
            }
        }
    }
    Ok(())
}
