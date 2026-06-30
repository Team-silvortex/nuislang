use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::super::super::data::NirDataKind;
use super::super::super::task_result_facts::{BorrowBindings, TaskResultStateFact};
use super::verify_expr;

pub(super) fn verify_kernel_expr_tree(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<bool, String> {
    match expr {
        NirExpr::KernelMatmul { lhs, rhs } => {
            verify_expr(
                lhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                rhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelElementAt { input, row, col } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                row,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                col,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelReshape { input, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelBroadcast { input, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelMap { input, scalar, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let Some(scalar) = scalar {
                verify_expr(
                    scalar,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::KernelMapAxis { input, scalar, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let Some(scalar) = scalar {
                verify_expr(
                    scalar,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::KernelZip { lhs, rhs, .. } => {
            verify_expr(
                lhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                rhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelAddBias { input, bias } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                bias,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
