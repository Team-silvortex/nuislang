use std::collections::BTreeMap;

use super::match_lowering::lower_match_stmt_with_async;
use super::metadata::ModuleConstValue;
use super::stmt_lowering_control::{
    expand_nested_control_expr_stmt, lower_if_expr_stmt_with_async,
    lower_match_expr_stmt_with_async, ControlExprKind,
};
use super::stmt_lowering_destructure::lower_destructure_let_stmt_with_async;
use super::stmt_lowering_sequence::lower_expanded_stmt_sequence_with_async;
pub(super) use super::stmt_lowering_sequence::lower_stmt_block_with_async;
use super::stmt_lowering_try::expand_try_stmt;
use super::stmt_lowering_try_nested::expand_nested_try_stmt;
use super::validation_helpers::validate_type_ref;
use super::{
    bool_type, infer_nir_expr_type, lower_expr_with_async, lower_type_ref_with_aliases,
    resolve_declared_or_inferred, AstStmt, AstTypeAlias, AstTypeRef, FunctionSignature, NirStmt,
    NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_stmt_with_async(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirStmt, String> {
    Ok(match stmt {
        AstStmt::Let {
            name, ty, value, ..
        } => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                let expected = ty
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                    .transpose()?;
                if let Some(expected_ty) = expected.as_ref() {
                    validate_type_ref(expected_ty)?;
                }
                let final_type = expected.clone().ok_or_else(|| {
                    format!(
                        "`if` expression let binding `{name}` currently requires an explicit type annotation"
                    )
                })?;
                bindings.insert(name.clone(), final_type.clone());
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: Some(ty.clone().unwrap_or_else(|| AstTypeRef {
                            name: final_type.name.clone(),
                            generic_args: Vec::new(),
                            is_ref: final_type.is_ref,
                            is_optional: final_type.is_optional,
                        })),
                        value,
                    },
                );
            }
            if let super::AstExpr::Match { value, arms } = value {
                let expected = ty
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                    .transpose()?;
                if let Some(expected_ty) = expected.as_ref() {
                    validate_type_ref(expected_ty)?;
                }
                let final_type = expected.clone().ok_or_else(|| {
                    format!(
                        "`match` expression let binding `{name}` currently requires an explicit type annotation"
                    )
                })?;
                bindings.insert(name.clone(), final_type.clone());
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: Some(ty.clone().unwrap_or_else(|| AstTypeRef {
                            name: final_type.name.clone(),
                            generic_args: Vec::new(),
                            is_ref: final_type.is_ref,
                            is_optional: final_type.is_optional,
                        })),
                        value,
                    },
                );
            }
            let expected = ty
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            let lowered = lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected.as_ref(),
                false,
            )?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, expected, inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Let {
                name: name.clone(),
                ty: Some(final_type),
                value: lowered,
            }
        }
        AstStmt::AssignLocal { name, value } => {
            let current_ty = bindings
                .get(name)
                .cloned()
                .ok_or_else(|| format!("cannot assign to unknown local `{name}`"))?;
            let lowered = lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                Some(&current_ty),
                false,
            )?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type =
                resolve_declared_or_inferred(name, Some(current_ty.clone()), inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Let {
                name: name.clone(),
                ty: Some(final_type),
                value: lowered,
            }
        }
        AstStmt::Const { name, ty, value } => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                if ty.is_none() {
                    return Err(format!(
                        "`if` expression const binding `{name}` currently requires an explicit type annotation"
                    ));
                }
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Const {
                        name: name.clone(),
                        ty: ty.clone(),
                        value,
                    },
                );
            }
            if let super::AstExpr::Match { value, arms } = value {
                if ty.is_none() {
                    return Err(format!(
                        "`match` expression const binding `{name}` currently requires an explicit type annotation"
                    ));
                }
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Const {
                        name: name.clone(),
                        ty: ty.clone(),
                        value,
                    },
                );
            }
            let expected = ty
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            let lowered = lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected.as_ref(),
                false,
            )?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, expected, inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Const {
                name: name.clone(),
                ty: final_type,
                value: lowered,
            }
        }
        AstStmt::Print(value) => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &AstStmt::Print,
                );
            }
            if let super::AstExpr::Match { value, arms } = value {
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &AstStmt::Print,
                );
            }
            NirStmt::Print(lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
                false,
            )?)
        }
        AstStmt::Await(value) => {
            if !current_function_is_async {
                return Err("`await` is only allowed inside `async fn`".to_owned());
            }
            NirStmt::Await(lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
                true,
            )?)
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let mut then_bindings = bindings.clone();
            let mut else_bindings = bindings.clone();
            NirStmt::If {
                condition: lower_expr_with_async(
                    condition,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    Some(&bool_type()),
                    false,
                )?,
                then_body: lower_stmt_block_with_async(
                    then_body,
                    current_domain,
                    current_function_is_async,
                    &mut then_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )?,
                else_body: lower_stmt_block_with_async(
                    else_body,
                    current_domain,
                    current_function_is_async,
                    &mut else_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )?,
            }
        }
        AstStmt::Match { value, arms } => lower_match_stmt_with_async(
            value,
            arms,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?,
        AstStmt::While { condition, body } => {
            let mut loop_bindings = bindings.clone();
            NirStmt::While {
                condition: lower_expr_with_async(
                    condition,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    Some(&bool_type()),
                    false,
                )?,
                body: lower_stmt_block_with_async(
                    body,
                    current_domain,
                    current_function_is_async,
                    &mut loop_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )?,
            }
        }
        AstStmt::Break => NirStmt::Break,
        AstStmt::Continue => NirStmt::Continue,
        AstStmt::Expr(expr) => NirStmt::Expr(lower_expr_with_async(
            expr,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            None,
            false,
        )?),
        AstStmt::Return(value) => {
            if let Some(super::AstExpr::If {
                condition,
                then_body,
                else_body,
            }) = value
            {
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Return(Some(value)),
                );
            }
            if let Some(super::AstExpr::Match { value, arms }) = value {
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Return(Some(value)),
                );
            }
            let expected = return_type
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            NirStmt::Return(match value {
                Some(value) => Some(lower_expr_with_async(
                    value,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    expected.as_ref(),
                    false,
                )?),
                None => None,
            })
        }
        AstStmt::DestructureLet { .. } => return Err(
            "internal error: destructuring let must be lowered through statement-sequence lowering"
                .to_owned(),
        ),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_stmt_sequence_with_async(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirStmt>, String> {
    for kind in [ControlExprKind::If, ControlExprKind::Match] {
        if let Some(expanded) = expand_nested_control_expr_stmt(stmt, kind)? {
            return lower_expanded_stmt_sequence_with_async(
                stmt,
                expanded,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            );
        }
    }
    if let Some(expanded) = expand_try_stmt(
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )? {
        return lower_expanded_stmt_sequence_with_async(
            stmt,
            expanded,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        );
    }
    if let Some(expanded) = expand_nested_try_stmt(
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )? {
        return lower_expanded_stmt_sequence_with_async(
            stmt,
            expanded,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        );
    }
    if matches!(stmt, AstStmt::DestructureLet { .. }) {
        return lower_destructure_let_stmt_with_async(
            stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            type_aliases,
            signatures,
            struct_table,
        );
    }
    Ok(vec![lower_stmt_with_async(
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )?])
}
