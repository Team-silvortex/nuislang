use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstEnumDef, AstExpr, AstImplDef, AstMatchArm, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::validation_binding_env::{bind_match_pattern_for_type, simple_match_value_type};
use super::validate_stmt_generic_constraints;
use super::validation_generic_constraints_types::validate_ast_type_ref_generic_constraints;

pub(super) fn validate_expr_generic_constraints(
    expr: &AstExpr,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_trait_names: &BTreeSet<String>,
    visible_trait_methods: &BTreeMap<String, BTreeSet<String>>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_enums: &BTreeMap<String, AstEnumDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    local_type_env: &BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
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
            validate_expr_generic_constraints(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut then_env = local_type_env.clone();
            for nested in then_body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut then_env,
                    &format!("{context} if-then"),
                )?;
            }
            let mut else_env = local_type_env.clone();
            for nested in else_body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut else_env,
                    &format!("{context} if-else"),
                )?;
            }
        }
        AstExpr::Match { value, arms } => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
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
                        &pattern,
                        visible_type_aliases,
                        visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = guard {
                    validate_expr_generic_constraints(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_trait_names,
                        visible_trait_methods,
                        visible_structs,
                        visible_enums,
                        function_return_types,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
                    )?;
                }
                for nested in body {
                    validate_stmt_generic_constraints(
                        &nested,
                        visible_type_aliases,
                        impl_lookup,
                        visible_trait_names,
                        visible_trait_methods,
                        visible_structs,
                        visible_enums,
                        function_return_types,
                        generic_param_names,
                        generic_bounds,
                        &mut arm_env,
                        &format!("{context} match-arm"),
                    )?;
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
                validate_ast_type_ref_generic_constraints(
                    &param.ty,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} lambda parameter `{}`", param.name),
                )?;
                lambda_env.insert(param.name.clone(), param.ty.clone());
            }
            if let Some(return_type) = return_type {
                validate_ast_type_ref_generic_constraints(
                    return_type,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} lambda return type"),
                )?;
            }
            for nested in body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut lambda_env,
                    &format!("{context} lambda body"),
                )?;
            }
        }
        AstExpr::Try(value) | AstExpr::Await(value) | AstExpr::FieldAccess { base: value, .. } => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            for (index, generic_arg) in generic_args.iter().enumerate() {
                validate_ast_type_ref_generic_constraints(
                    generic_arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} call `{callee}` generic argument #{}", index + 1),
                )?;
            }
            for arg in args {
                validate_expr_generic_constraints(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::Invoke { callee, args } => {
            validate_expr_generic_constraints(
                callee,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            for arg in args {
                validate_expr_generic_constraints(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::MethodCall {
            receiver,
            generic_args,
            args,
            ..
        } => {
            validate_expr_generic_constraints(
                receiver,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            for (index, generic_arg) in generic_args.iter().enumerate() {
                validate_ast_type_ref_generic_constraints(
                    generic_arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} method call generic argument #{}", index + 1),
                )?;
            }
            for arg in args {
                validate_expr_generic_constraints(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
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
                    .and_then(|(parent, _)| visible_enums.contains_key(parent).then_some(parent))
                    .unwrap_or(type_name)
                    .to_owned(),
                generic_args: type_args.clone(),
                is_optional: false,
                is_ref: false,
            };
            validate_ast_type_ref_generic_constraints(
                &literal_ty,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_structs,
                visible_enums,
                generic_bounds,
                &format!("{context} struct literal `{type_name}`"),
            )?;
            for (_, value) in fields {
                validate_expr_generic_constraints(
                    value,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::Unary { operand, .. } => {
            validate_expr_generic_constraints(
                operand,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_expr_generic_constraints(
                lhs,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_constraints(
                rhs,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
    }
    Ok(())
}
