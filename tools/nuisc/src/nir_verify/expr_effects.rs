use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::super::effects::note_binding_effects;
use super::super::task_result_facts::BorrowBindings;

pub(in crate::nir_verify) fn apply_guaranteed_expr_effects(
    expr: &NirExpr,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    borrow_bindings: &mut BorrowBindings,
    include_temporary_borrows: bool,
) {
    match expr {
        NirExpr::Binary { op, lhs, rhs } => match op {
            nuis_semantics::model::NirBinaryOp::And | nuis_semantics::model::NirBinaryOp::Or => {
                apply_guaranteed_expr_effects(
                    lhs,
                    moved,
                    borrows,
                    borrow_bindings,
                    include_temporary_borrows,
                );
            }
            _ => {
                apply_guaranteed_expr_effects(
                    lhs,
                    moved,
                    borrows,
                    borrow_bindings,
                    include_temporary_borrows,
                );
                apply_guaranteed_expr_effects(
                    rhs,
                    moved,
                    borrows,
                    borrow_bindings,
                    include_temporary_borrows,
                );
            }
        },
        NirExpr::SelectOwnedPointer {
            condition,
            then_owner,
            else_owner,
        } => {
            apply_guaranteed_expr_effects(
                condition,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                then_owner,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                else_owner,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
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
        | NirExpr::CopyBufferOwned(inner)
        | NirExpr::BytesLen(inner)
        | NirExpr::DropBytes(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => apply_guaranteed_expr_effects(
            inner,
            moved,
            borrows,
            borrow_bindings,
            include_temporary_borrows,
        ),
        NirExpr::AllocNode { value, next } => {
            apply_guaranteed_expr_effects(
                value,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                next,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
        }
        NirExpr::AllocBuffer { len, fill } => {
            apply_guaranteed_expr_effects(
                len,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                fill,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
        }
        NirExpr::StoreValue { target, value } => {
            apply_guaranteed_expr_effects(
                target,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                value,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
        }
        NirExpr::StoreNext { target, next } => {
            apply_guaranteed_expr_effects(
                target,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                next,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
        }
        NirExpr::LoadAt { buffer, index } => {
            apply_guaranteed_expr_effects(
                buffer,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                index,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            apply_guaranteed_expr_effects(
                buffer,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                index,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
            apply_guaranteed_expr_effects(
                value,
                moved,
                borrows,
                borrow_bindings,
                include_temporary_borrows,
            );
        }
        NirExpr::Call { args, .. } | NirExpr::MethodCall { args, .. } => {
            for arg in args {
                apply_guaranteed_expr_effects(
                    arg,
                    moved,
                    borrows,
                    borrow_bindings,
                    include_temporary_borrows,
                );
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                apply_guaranteed_expr_effects(
                    value,
                    moved,
                    borrows,
                    borrow_bindings,
                    include_temporary_borrows,
                );
            }
        }
        NirExpr::FieldAccess { base, .. }
        | NirExpr::VariantIs { base, .. }
        | NirExpr::VariantFieldAccess { base, .. } => apply_guaranteed_expr_effects(
            base,
            moved,
            borrows,
            borrow_bindings,
            include_temporary_borrows,
        ),
        _ => {}
    }

    match expr {
        NirExpr::Move(_)
        | NirExpr::SelectOwnedPointer { .. }
        | NirExpr::Free(_)
        | NirExpr::DropBytes(_)
        | NirExpr::CpuJoin(_)
        | NirExpr::CpuThreadJoin(_)
        | NirExpr::CpuCancel(_)
        | NirExpr::CpuJoinResult(_)
        | NirExpr::CpuThreadJoinResult(_)
        | NirExpr::CpuTimeout { .. }
        | NirExpr::CpuReadyAfter { .. }
        | NirExpr::CpuMutexLock(_)
        | NirExpr::CpuMutexUnlock(_)
        | NirExpr::BorrowEnd(_) => note_binding_effects(expr, "_", moved, borrows, borrow_bindings),
        NirExpr::Borrow(_) | NirExpr::LoadNext(_) if include_temporary_borrows => {
            note_binding_effects(expr, "_", moved, borrows, borrow_bindings)
        }
        _ => {}
    }
}
