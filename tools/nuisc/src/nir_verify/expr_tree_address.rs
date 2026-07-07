use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::super::super::data::NirDataKind;
use super::super::super::task_result_facts::{
    borrowed_address_alias_source, BorrowBindings, TaskResultStateFact,
};
use super::super::super::{ensure_owned_address_target, owned_structural_address_error};
use super::super::expr_effects::apply_guaranteed_expr_effects;
use super::verify_expr;

pub(super) fn verify_address_expr_tree(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<bool, String> {
    match expr {
        NirExpr::AllocNode { value, next } => {
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                next,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if borrowed_address_alias_source(next, borrow_bindings).is_some() {
                return Err(owned_structural_address_error(
                    "alloc_node(..., next)",
                    next,
                    borrow_bindings,
                ));
            }
        }
        NirExpr::AllocBuffer { len, fill } => {
            verify_expr(
                len,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                fill,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::LoadAt { buffer, index } => {
            verify_expr(
                buffer,
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
        NirExpr::StoreValue { target, value } => {
            verify_expr(
                target,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut next_moved = moved.clone();
            let mut next_borrows = borrows.clone();
            let mut next_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                target,
                &mut next_moved,
                &mut next_borrows,
                &mut next_borrow_bindings,
                true,
            );
            verify_expr(
                value,
                &next_moved,
                &next_borrows,
                &next_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            ensure_owned_address_target("store_value(..., target)", target, &next_borrow_bindings)?;
        }
        NirExpr::StoreNext { target, next } => {
            verify_expr(
                target,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut next_moved = moved.clone();
            let mut next_borrows = borrows.clone();
            let mut next_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                target,
                &mut next_moved,
                &mut next_borrows,
                &mut next_borrow_bindings,
                true,
            );
            verify_expr(
                next,
                &next_moved,
                &next_borrows,
                &next_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            ensure_owned_address_target("store_next(..., target)", target, &next_borrow_bindings)?;
            if borrowed_address_alias_source(next, &next_borrow_bindings).is_some() {
                return Err(owned_structural_address_error(
                    "store_next(..., next)",
                    next,
                    &next_borrow_bindings,
                ));
            }
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            verify_expr(
                buffer,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut current_moved = moved.clone();
            let mut current_borrows = borrows.clone();
            let mut current_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                buffer,
                &mut current_moved,
                &mut current_borrows,
                &mut current_borrow_bindings,
                true,
            );
            verify_expr(
                index,
                &current_moved,
                &current_borrows,
                &current_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            apply_guaranteed_expr_effects(
                index,
                &mut current_moved,
                &mut current_borrows,
                &mut current_borrow_bindings,
                true,
            );
            verify_expr(
                value,
                &current_moved,
                &current_borrows,
                &current_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            ensure_owned_address_target("store_at(..., buffer)", buffer, &current_borrow_bindings)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
