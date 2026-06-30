use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstEnumDef, AstImplDef, AstMatchArm, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::infer_ast_expr_type;
use super::super::validation_binding_env::{
    bind_destructure_fields_for_type, bind_match_pattern_for_type, simple_match_value_type,
};
use super::super::validation_method_bounds::validate_expr_generic_method_bounds;
use super::validation_generic_constraints_expr::validate_expr_generic_constraints;
use super::validation_generic_constraints_types::validate_ast_type_ref_generic_constraints;

pub(super) fn validate_stmt_generic_constraints(
    stmt: &AstStmt,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_trait_names: &BTreeSet<String>,
    visible_trait_methods: &BTreeMap<String, BTreeSet<String>>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_enums: &BTreeMap<String, AstEnumDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    local_type_env: &mut BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    match stmt {
        AstStmt::Let { name, ty, .. } | AstStmt::Const { name, ty, .. } => {
            let value = match stmt {
                AstStmt::Let { value, .. } | AstStmt::Const { value, .. } => value,
                _ => unreachable!(),
            };
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
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(ty) = ty {
                // Keep explicit annotation validation as its own pass after
                // walking the value expression. In deep expected-type chains,
                // this is where constrained aliases intentionally get a chance
                // to report their own bound failure context, even if an inner
                // generic call was also inferable from the same expected type.
                validate_ast_type_ref_generic_constraints(
                    ty,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} local `{name}`"),
                )?;
            }
            if let Some(inferred_ty) = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
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
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(inferred_ty) = infer_ast_expr_type(
                value,
                local_type_env,
                impl_lookup,
                visible_structs,
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
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(type_ref) = type_ref {
                validate_ast_type_ref_generic_constraints(
                    type_ref,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} destructure type"),
                )?;
            }
            let fields = match stmt {
                AstStmt::DestructureLet { fields, .. } => fields,
                _ => unreachable!(),
            };
            let root_type = type_ref.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                )
            });
            if let Some(root_type) = root_type.as_ref() {
                bind_destructure_fields_for_type(
                    root_type,
                    fields,
                    visible_type_aliases,
                    visible_structs,
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
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
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
                    context,
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
                    context,
                )?;
            }
        }
        AstStmt::Match { value, arms } => {
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
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
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
                        pattern,
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
                    validate_expr_generic_method_bounds(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        visible_trait_methods,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
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
                        &mut arm_env,
                        context,
                    )?;
                }
            }
        }
        AstStmt::While { condition, body } => {
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
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut loop_env = local_type_env.clone();
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
                    &mut loop_env,
                    context,
                )?;
            }
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
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
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstStmt::Return(Some(value)) => {
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
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
    }
    Ok(())
}
