use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::super::super::data::NirDataKind;
use super::super::super::task_result_facts::{BorrowBindings, TaskResultStateFact};
use super::verify_expr;

pub(super) fn verify_data_expr_tree(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<bool, String> {
    match expr {
        NirExpr::DataFreezeWindow(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::DataReadWindow { window, index } => {
            verify_expr(
                window,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                index,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            verify_expr(
                window,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                index,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => verify_expr(
            input,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                offset,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                len,
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
