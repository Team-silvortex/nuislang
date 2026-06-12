use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::expr_resource_key;
use super::task_result_facts::{borrowed_address_binding, BorrowBindings, BorrowedAddressBinding};

pub(super) fn merge_branch_state(
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    then_moved: &BTreeSet<String>,
    then_borrows: &BTreeMap<String, usize>,
    else_moved: &BTreeSet<String>,
    else_borrows: &BTreeMap<String, usize>,
) {
    moved.extend(then_moved.iter().cloned());
    moved.extend(else_moved.iter().cloned());

    let mut merged_borrows = BTreeMap::<String, usize>::new();
    for name in then_borrows.keys().chain(else_borrows.keys()) {
        let then_count = then_borrows.get(name).copied().unwrap_or(0);
        let else_count = else_borrows.get(name).copied().unwrap_or(0);
        let merged = then_count.max(else_count);
        if merged > 0 {
            merged_borrows.insert(name.clone(), merged);
        }
    }

    *borrows = merged_borrows;
}

pub(super) fn ensure_binding_can_be_rebound(
    name: &str,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
) -> Result<(), String> {
    if borrows.get(name).copied().unwrap_or(0) > 0 {
        return Err(format!(
            "nir verify: cannot rebind `{}` while borrow(s) are active",
            name
        ));
    }
    if let Some(binding) = borrow_bindings.get(name) {
        if borrows.get(&binding.source).copied().unwrap_or(0) > 0 {
            return Err(format!(
                "nir verify: cannot rebind borrow alias `{}` while borrow of `{}` is active",
                name, binding.source
            ));
        }
    }
    Ok(())
}

pub(super) fn note_binding_effects(
    expr: &NirExpr,
    binding_name: &str,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    borrow_bindings: &mut BorrowBindings,
) {
    match expr {
        NirExpr::Move(inner)
        | NirExpr::Free(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner) => {
            if let Some(source) = expr_resource_key(inner) {
                moved.insert(source.clone());
                borrows.remove(&source);
            }
        }
        NirExpr::CpuTimeout { task, .. } => {
            if let Some(source) = expr_resource_key(task) {
                moved.insert(source.clone());
                borrows.remove(&source);
            }
        }
        NirExpr::Borrow(inner) => {
            if let Some(source) = expr_resource_key(inner) {
                *borrows.entry(source.clone()).or_insert(0) += 1;
                if binding_name != "_" {
                    borrow_bindings.insert(
                        binding_name.to_owned(),
                        BorrowedAddressBinding::direct(source),
                    );
                }
            }
        }
        NirExpr::LoadNext(inner) => {
            if let Some(binding) = borrowed_address_binding(inner, borrow_bindings) {
                if binding_name != "_" {
                    borrow_bindings.insert(
                        binding_name.to_owned(),
                        BorrowedAddressBinding::traversed(binding.source),
                    );
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
                let next = borrows.get(&source).copied().unwrap_or(0).saturating_sub(1);
                if next == 0 {
                    borrows.remove(&source);
                } else {
                    borrows.insert(source.clone(), next);
                }
                if let NirExpr::Var(alias_name) = inner.as_ref() {
                    borrow_bindings.remove(alias_name);
                }
                if binding_name != "_" && binding_name != source {
                    borrow_bindings.remove(binding_name);
                }
            }
        }
        _ => {}
    }
}
