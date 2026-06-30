use std::collections::BTreeMap;

use super::metadata::ModuleConstValue;
use super::stmt_lowering::lower_stmt_sequence_with_async;
use super::validation_helpers::validate_type_ref;
use super::{
    lower_type_ref_with_aliases, AstStmt, AstTypeAlias, AstTypeRef, FunctionSignature, NirStmt,
    NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_expanded_stmt_sequence_with_async(
    original_stmt: &AstStmt,
    expanded: Vec<AstStmt>,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirStmt>, String> {
    seed_expanded_stmt_bindings(original_stmt, bindings, type_aliases)?;
    let mut lowered = Vec::new();
    for stmt in expanded {
        lowered.extend(lower_stmt_sequence_with_async(
            &stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?);
    }
    Ok(lowered)
}

fn seed_expanded_stmt_bindings(
    stmt: &AstStmt,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<(), String> {
    match stmt {
        AstStmt::Let {
            name, ty: Some(ty), ..
        }
        | AstStmt::Const {
            name, ty: Some(ty), ..
        } => {
            let lowered_ty = lower_type_ref_with_aliases(ty, type_aliases)?;
            validate_type_ref(&lowered_ty)?;
            bindings.insert(name.clone(), lowered_ty);
        }
        _ => {}
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_stmt_block_with_async(
    stmts: &[AstStmt],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirStmt>, String> {
    let mut lowered = Vec::new();
    for stmt in stmts {
        lowered.extend(lower_stmt_sequence_with_async(
            stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?);
    }
    Ok(lowered)
}
