use std::collections::BTreeMap;

use super::metadata::ModuleConstValue;
use super::stmt_lowering::{lower_stmt_sequence_with_async, StmtSequenceLoweringInput};
use super::validation_helpers::validate_type_ref;
use super::{
    lower_type_ref_with_aliases, AstStmt, AstTypeAlias, AstTypeRef, FunctionSignature, NirStmt,
    NirStructDef, NirTypeRef,
};

pub(super) struct ExpandedStmtSequenceLoweringInput<'a> {
    pub(super) original_stmt: &'a AstStmt,
    pub(super) expanded: Vec<AstStmt>,
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a mut BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) return_type: Option<&'a AstTypeRef>,
    pub(super) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) struct StmtBlockLoweringInput<'a> {
    pub(super) stmts: &'a [AstStmt],
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a mut BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) return_type: Option<&'a AstTypeRef>,
    pub(super) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_expanded_stmt_sequence_with_async(
    input: ExpandedStmtSequenceLoweringInput<'_>,
) -> Result<Vec<NirStmt>, String> {
    let ExpandedStmtSequenceLoweringInput {
        original_stmt,
        expanded,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    } = input;
    seed_expanded_stmt_bindings(original_stmt, bindings, type_aliases)?;
    let mut lowered = Vec::new();
    for stmt in expanded {
        lowered.extend(lower_stmt_sequence_with_async(StmtSequenceLoweringInput {
            stmt: &stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        })?);
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

pub(super) fn lower_stmt_block_with_async(
    input: StmtBlockLoweringInput<'_>,
) -> Result<Vec<NirStmt>, String> {
    let StmtBlockLoweringInput {
        stmts,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    } = input;
    let mut lowered = Vec::new();
    for stmt in stmts {
        lowered.extend(lower_stmt_sequence_with_async(StmtSequenceLoweringInput {
            stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        })?);
    }
    Ok(lowered)
}
