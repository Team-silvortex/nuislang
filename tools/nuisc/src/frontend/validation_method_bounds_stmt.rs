use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstImplDef, AstMatchArm, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::validation_method_bounds_expr::validate_expr_generic_method_bounds;
use super::{inferred_match_value_type, normalize_method_bound_context};
use crate::frontend::infer_ast_expr_type;
use crate::frontend::validation_binding_env::{
    bind_destructure_fields_for_type, bind_match_pattern_for_type,
};

pub(super) fn validate_stmt_generic_method_bounds_block(
    body: &[AstStmt],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    local_type_env: &mut BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    for stmt in body {
        validate_stmt_generic_method_bounds(
            stmt,
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
    Ok(())
}

fn validate_stmt_generic_method_bounds(
    stmt: &AstStmt,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    local_type_env: &mut BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    let normalized_context = normalize_method_bound_context(context);
    let context = normalized_context.as_str();
    match stmt {
        AstStmt::Let {
            name, ty, value, ..
        }
        | AstStmt::Const { name, ty, value } => {
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
            if let Some(ty) = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                )
            }) {
                local_type_env.insert(name.clone(), ty);
            }
        }
        AstStmt::AssignLocal { name, value } => {
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
            if let Some(ty) = local_type_env.get(name).cloned() {
                local_type_env.insert(name.clone(), ty);
            }
        }
        AstStmt::DestructureLet { value, .. } => {
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
                context,
            )?;
            let mut else_env = local_type_env.clone();
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
                context,
            )?;
        }
        AstStmt::Match { value, arms } => {
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
                    body,
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
        AstStmt::While { condition, body } => {
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
            let mut loop_env = local_type_env.clone();
            validate_stmt_generic_method_bounds_block(
                body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut loop_env,
                context,
            )?;
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
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
        AstStmt::Return(Some(value)) => {
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
        AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
    }
    Ok(())
}
