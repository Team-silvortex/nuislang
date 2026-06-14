use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, AstFunction, AstImplDef, AstStmt, AstStructDef, AstTypeRef};

use super::validation_binding_env::instantiate_ast_struct_field_type;
use super::{ast_named_type, infer_ast_expr_type, lower_type_ref, resolve_ast_type_ref_aliases};

pub(super) fn infer_missing_function_return_type(
    function: &AstFunction,
    module_const_env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<AstTypeRef>, String> {
    if function.return_type.is_some() {
        return Ok(function.return_type.clone());
    }
    let mut env = module_const_env.clone();
    for param in &function.params {
        env.insert(param.name.clone(), param.ty.clone());
    }
    let mut returns = Vec::<AstTypeRef>::new();
    let guaranteed_return = collect_inferred_return_types_from_block(
        &function.body,
        &function.name,
        &mut env,
        impl_lookup,
        struct_table,
        function_return_types,
        &mut returns,
    )?;
    if returns.is_empty() {
        return Ok(None);
    }
    if !guaranteed_return {
        return Err(format!(
            "function `{}` currently needs an explicit return type or total terminal return branches to infer its return type",
            function.name
        ));
    }
    let resolved = returns
        .iter()
        .map(lower_type_ref)
        .map(|ty| ty.render())
        .collect::<Vec<_>>();
    let mut deduped = resolved.clone();
    deduped.dedup();
    if deduped.len() > 1 {
        return Err(format!(
            "function `{}` has inconsistent inferred return types: {}",
            function.name,
            deduped.join(", ")
        ));
    }
    Ok(returns.into_iter().next())
}

fn collect_inferred_return_types_from_block(
    body: &[AstStmt],
    function_name: &str,
    env: &mut BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    returns: &mut Vec<AstTypeRef>,
) -> Result<bool, String> {
    for stmt in body {
        match stmt {
            AstStmt::Let { name, ty, value, .. } => {
                let inferred = ty.clone().or_else(|| {
                    infer_ast_expr_type(
                        value,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                    )
                });
                if let Some(inferred_ty) = inferred {
                    env.insert(name.clone(), inferred_ty);
                }
            }
            AstStmt::AssignLocal { name, value } => {
                let inferred = infer_ast_expr_type(
                    value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .or_else(|| env.get(name).cloned());
                if let Some(inferred_ty) = inferred {
                    env.insert(name.clone(), inferred_ty);
                }
            }
            AstStmt::DestructureLet {
                type_ref,
                fields,
                value,
            } => {
                let root_type = type_ref.clone().or_else(|| {
                    infer_ast_expr_type(
                        value,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                    )
                });
                if let Some(root_type) = root_type.as_ref() {
                    bind_destructure_fields(root_type, fields, env, struct_table)?;
                }
            }
            AstStmt::Const { name, ty, .. } => {
                if let Some(ty) = ty.clone() {
                    env.insert(name.clone(), ty);
                }
            }
            AstStmt::If {
                then_body,
                else_body,
                ..
            } => {
                let mut then_env = env.clone();
                let then_returns = collect_inferred_return_types_from_block(
                    then_body,
                    function_name,
                    &mut then_env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    returns,
                )?;
                let mut else_env = env.clone();
                let else_returns = collect_inferred_return_types_from_block(
                    else_body,
                    function_name,
                    &mut else_env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    returns,
                )?;
                if then_returns && else_returns {
                    return Ok(true);
                }
            }
            AstStmt::Match { arms, .. } => {
                let mut all_arms_return = !arms.is_empty();
                for arm in arms {
                    let mut arm_env = env.clone();
                    let arm_returns = collect_inferred_return_types_from_block(
                        &arm.body,
                        function_name,
                        &mut arm_env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        returns,
                    )?;
                    all_arms_return &= arm_returns;
                }
                if all_arms_return {
                    return Ok(true);
                }
            }
            AstStmt::While { body, .. } => {
                let mut loop_env = env.clone();
                collect_inferred_return_types_from_block(
                    body,
                    function_name,
                    &mut loop_env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    returns,
                )?;
            }
            AstStmt::Return(Some(value)) => {
                let inferred = infer_ast_expr_type(
                    value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .ok_or_else(|| {
                    format!(
                        "could not infer return type for function `{}` from return expression",
                        function_name
                    )
                })?;
                returns.push(inferred);
                return Ok(true);
            }
            AstStmt::Return(None) => {
                returns.push(ast_named_type("unit"));
                return Ok(true);
            }
            AstStmt::Expr(AstExpr::Call { .. })
            | AstStmt::Expr(_)
            | AstStmt::Print(_)
            | AstStmt::Await(_)
            | AstStmt::Break
            | AstStmt::Continue => {}
        }
    }
    Ok(false)
}

fn bind_destructure_fields(
    type_ref: &AstTypeRef,
    fields: &[nuis_semantics::model::AstDestructureField],
    env: &mut BTreeMap<String, AstTypeRef>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Result<(), String> {
    bind_destructure_fields_for_type(type_ref, fields, env, struct_table)
}

fn bind_destructure_fields_for_type(
    type_ref: &AstTypeRef,
    fields: &[nuis_semantics::model::AstDestructureField],
    env: &mut BTreeMap<String, AstTypeRef>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Result<(), String> {
    let resolved = resolve_ast_type_ref_aliases(type_ref, &BTreeMap::new())?;
    let Some(struct_def) = struct_table.get(&resolved.name) else {
        return Ok(());
    };
    for field in fields {
        let Some(struct_field) = struct_def
            .fields
            .iter()
            .find(|candidate| candidate.name == field.field)
        else {
            return Err(format!(
                "type `{}` has no field `{}` for destructuring let",
                resolved.name, field.field
            ));
        };
        let field_ty = instantiate_ast_struct_field_type(&resolved, struct_def, &struct_field.ty);
        match &field.binding {
            nuis_semantics::model::AstDestructureBinding::Bind(name) => {
                env.insert(name.clone(), field_ty.clone());
            }
            nuis_semantics::model::AstDestructureBinding::Ignore => {}
            nuis_semantics::model::AstDestructureBinding::Nested {
                type_ref,
                fields: nested_fields,
            } => {
                let nested_type = type_ref.as_ref().unwrap_or(&field_ty);
                bind_destructure_fields_for_type(nested_type, nested_fields, env, struct_table)?;
            }
        }
    }
    Ok(())
}
