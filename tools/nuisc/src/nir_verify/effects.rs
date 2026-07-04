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

pub(super) fn merge_control_flow_borrow_bindings(
    borrow_bindings: &mut BorrowBindings,
    then_bindings: &BorrowBindings,
    else_bindings: &BorrowBindings,
) {
    let mut merged = BorrowBindings::new();
    for (name, then_binding) in then_bindings {
        if let Some(else_binding) = else_bindings.get(name) {
            if then_binding == else_binding {
                merged.insert(name.clone(), then_binding.clone());
            }
        }
    }
    *borrow_bindings = merged;
}

pub(super) fn merge_control_flow_data_bindings(
    data_bindings: &mut BTreeMap<String, super::NirDataKind>,
    then_bindings: &BTreeMap<String, super::NirDataKind>,
    else_bindings: &BTreeMap<String, super::NirDataKind>,
) {
    let mut merged = BTreeMap::new();
    for (name, then_kind) in then_bindings {
        if let Some(else_kind) = else_bindings.get(name) {
            let merged_kind = if then_kind == else_kind {
                *then_kind
            } else {
                super::NirDataKind::Other
            };
            merged.insert(name.clone(), merged_kind);
        }
    }
    *data_bindings = merged;
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
    note_nested_expr_effects(expr, moved, borrows, borrow_bindings);
    match expr {
        NirExpr::Move(inner)
        | NirExpr::Free(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuThreadJoin(inner)
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
        NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner) => {
            if let Some(source) = expr_resource_key(inner) {
                moved.insert(source.clone());
                borrows.remove(&source);
            }
        }
        NirExpr::Borrow(inner) => {
            if let Some(binding) = borrowed_address_binding(inner, borrow_bindings)
                .or_else(|| expr_resource_key(inner).map(BorrowedAddressBinding::direct))
            {
                *borrows.entry(binding.source.clone()).or_insert(0) += 1;
                if binding_name != "_" {
                    borrow_bindings.insert(binding_name.to_owned(), binding);
                }
            }
        }
        NirExpr::LoadNext(inner) => {
            if let Some(binding) = borrowed_address_binding(inner, borrow_bindings) {
                *borrows.entry(binding.source.clone()).or_insert(0) += 1;
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

fn note_nested_expr_effects(
    expr: &NirExpr,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    borrow_bindings: &mut BorrowBindings,
) {
    match expr {
        NirExpr::Binary { lhs, rhs, .. } => {
            note_binding_effects(lhs, "_", moved, borrows, borrow_bindings);
            note_binding_effects(rhs, "_", moved, borrows, borrow_bindings);
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::Move(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => {
            note_binding_effects(inner, "_", moved, borrows, borrow_bindings)
        }
        NirExpr::AllocNode { value, next } => {
            note_binding_effects(value, "_", moved, borrows, borrow_bindings);
            note_binding_effects(next, "_", moved, borrows, borrow_bindings);
        }
        NirExpr::AllocBuffer { len, fill } => {
            note_binding_effects(len, "_", moved, borrows, borrow_bindings);
            note_binding_effects(fill, "_", moved, borrows, borrow_bindings);
        }
        NirExpr::LoadAt { buffer, index } => {
            note_binding_effects(buffer, "_", moved, borrows, borrow_bindings);
            note_binding_effects(index, "_", moved, borrows, borrow_bindings);
        }
        NirExpr::StoreValue { target, value } => {
            note_binding_effects(target, "_", moved, borrows, borrow_bindings);
            note_binding_effects(value, "_", moved, borrows, borrow_bindings);
        }
        NirExpr::StoreNext { target, next } => {
            note_binding_effects(target, "_", moved, borrows, borrow_bindings);
            note_binding_effects(next, "_", moved, borrows, borrow_bindings);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            note_binding_effects(buffer, "_", moved, borrows, borrow_bindings);
            note_binding_effects(index, "_", moved, borrows, borrow_bindings);
            note_binding_effects(value, "_", moved, borrows, borrow_bindings);
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                note_binding_effects(arg, "_", moved, borrows, borrow_bindings);
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            note_binding_effects(receiver, "_", moved, borrows, borrow_bindings);
            for arg in args {
                note_binding_effects(arg, "_", moved, borrows, borrow_bindings);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                note_binding_effects(value, "_", moved, borrows, borrow_bindings);
            }
        }
        NirExpr::FieldAccess { base, .. }
        | NirExpr::VariantIs { base, .. }
        | NirExpr::VariantFieldAccess { base, .. } => {
            note_binding_effects(base, "_", moved, borrows, borrow_bindings);
        }
        _ => {}
    }
}
