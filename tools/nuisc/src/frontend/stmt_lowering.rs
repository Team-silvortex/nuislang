use std::collections::BTreeMap;

use super::match_lowering::lower_match_stmt_with_async;
use super::metadata::ModuleConstValue;
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
        AstStmt::Let { name, ty, value } => {
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
        AstStmt::Const { name, ty, value } => {
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
        AstStmt::Print(value) => NirStmt::Print(lower_expr_with_async(
            value,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            None,
            false,
        )?),
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
                then_body: then_body
                    .iter()
                    .map(|stmt| {
                        lower_stmt_with_async(
                            stmt,
                            current_domain,
                            current_function_is_async,
                            &mut then_bindings,
                            module_consts,
                            return_type,
                            type_aliases,
                            signatures,
                            struct_table,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                else_body: else_body
                    .iter()
                    .map(|stmt| {
                        lower_stmt_with_async(
                            stmt,
                            current_domain,
                            current_function_is_async,
                            &mut else_bindings,
                            module_consts,
                            return_type,
                            type_aliases,
                            signatures,
                            struct_table,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
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
                body: body
                    .iter()
                    .map(|stmt| {
                        lower_stmt_with_async(
                            stmt,
                            current_domain,
                            current_function_is_async,
                            &mut loop_bindings,
                            module_consts,
                            return_type,
                            type_aliases,
                            signatures,
                            struct_table,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
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
    })
}
