use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{nir_glm_profile, NirExpr, NirGlmUseMode};

use super::super::owned_address_error;
use super::super::task_result_facts::{expr_is_borrowed_pointer, BorrowBindings};
use super::super::uses::expr_resource_key;

pub(super) fn verify_glm_expr_access(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
) -> Result<(), String> {
    if let Some(profile) = nir_glm_profile(expr) {
        if let Some(first_access) = profile.accesses.first() {
            match expr {
                NirExpr::Move(inner)
                | NirExpr::Free(inner)
                | NirExpr::CpuJoin(inner)
                | NirExpr::CpuThreadJoin(inner)
                | NirExpr::CpuCancel(inner)
                | NirExpr::CpuJoinResult(inner)
                | NirExpr::CpuThreadJoinResult(inner)
                | NirExpr::CpuMutexLock(inner)
                | NirExpr::CpuMutexUnlock(inner) => {
                    if let Some(operation) = match expr {
                        NirExpr::Move(_) => Some("move(...)"),
                        NirExpr::Free(_) => Some("free(...)"),
                        _ => None,
                    } {
                        if expr_is_borrowed_pointer(inner, borrow_bindings) {
                            return Err(owned_address_error(operation, inner, borrow_bindings));
                        }
                    }
                    if matches!(first_access.mode, NirGlmUseMode::Own) {
                        if let Some(source) = expr_resource_key(inner) {
                            if borrows.get(&source).copied().unwrap_or(0) > 0 {
                                return Err(format!(
                                    "nir verify: cannot consume `{}` while borrow(s) are active",
                                    source
                                ));
                            }
                        }
                    }
                }
                NirExpr::CpuTimeout { task, .. } => {
                    if let Some(source) = expr_resource_key(task) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot consume `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::StoreValue { target, .. } | NirExpr::StoreNext { target, .. } => {
                    if let Some(source) = expr_resource_key(target) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot write `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::StoreAt { buffer, .. } => {
                    if let Some(source) = expr_resource_key(buffer) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot write `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::Borrow(inner) => {
                    if let Some(source) = expr_resource_key(inner) {
                        if moved.contains(&source) {
                            return Err(format!(
                                "nir verify: cannot borrow moved value `{}`",
                                source
                            ));
                        }
                    }
                }
                NirExpr::BorrowEnd(inner) => {
                    let source = expr_resource_key(inner).and_then(|name| {
                        borrow_bindings
                            .get(&name)
                            .map(|binding| binding.source.clone())
                            .or(Some(name))
                    });
                    if let Some(source) = source {
                        if borrows.get(&source).copied().unwrap_or(0) == 0 {
                            return Err(format!(
                                "nir verify: cannot end borrow for `{}` with no active borrow",
                                source
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
