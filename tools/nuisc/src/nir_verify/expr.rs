use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

#[path = "expr_data.rs"]
mod expr_data;
#[path = "expr_effects.rs"]
mod expr_effects;
#[path = "expr_glm.rs"]
mod expr_glm;
#[path = "expr_tree.rs"]
mod expr_tree;

use super::data::NirDataKind;
use super::task_result_facts::{
    task_result_facts_for_short_circuit_rhs, BorrowBindings, TaskResultStateFact,
};
use super::uses::{expr_resource_key, verify_expr_uses};
use super::{expr_is_fixed_readable_carry_source, verify_fixed_readable_carry_source_expr};
use expr_data::verify_data_expr_shape;
pub(super) use expr_effects::apply_guaranteed_expr_effects;
use expr_glm::verify_glm_expr_access;
use expr_tree::verify_expr_tree;

pub(super) fn verify_condition_expr(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String> {
    match expr {
        NirExpr::Binary { op, lhs, rhs } => match op {
            nuis_semantics::model::NirBinaryOp::And => {
                verify_condition_expr(
                    lhs,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
                let mut rhs_moved = moved.clone();
                let mut rhs_borrows = borrows.clone();
                let mut rhs_borrow_bindings = borrow_bindings.clone();
                apply_guaranteed_expr_effects(
                    lhs,
                    &mut rhs_moved,
                    &mut rhs_borrows,
                    &mut rhs_borrow_bindings,
                    true,
                );
                let rhs_facts =
                    task_result_facts_for_short_circuit_rhs(lhs, true, task_result_facts);
                verify_condition_expr(
                    rhs,
                    &rhs_moved,
                    &rhs_borrows,
                    &rhs_borrow_bindings,
                    data_bindings,
                    &rhs_facts,
                )
            }
            nuis_semantics::model::NirBinaryOp::Or => {
                verify_condition_expr(
                    lhs,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
                let mut rhs_moved = moved.clone();
                let mut rhs_borrows = borrows.clone();
                let mut rhs_borrow_bindings = borrow_bindings.clone();
                apply_guaranteed_expr_effects(
                    lhs,
                    &mut rhs_moved,
                    &mut rhs_borrows,
                    &mut rhs_borrow_bindings,
                    true,
                );
                let rhs_facts =
                    task_result_facts_for_short_circuit_rhs(lhs, false, task_result_facts);
                verify_condition_expr(
                    rhs,
                    &rhs_moved,
                    &rhs_borrows,
                    &rhs_borrow_bindings,
                    data_bindings,
                    &rhs_facts,
                )
            }
            _ => verify_expr(
                expr,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            ),
        },
        _ => verify_expr(
            expr,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        ),
    }
}

fn verify_expr_sequence<'a, I>(
    exprs: I,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String>
where
    I: IntoIterator<Item = &'a NirExpr>,
{
    let mut current_moved = moved.clone();
    let mut current_borrows = borrows.clone();
    let mut current_borrow_bindings = borrow_bindings.clone();
    for expr in exprs {
        verify_expr(
            expr,
            &current_moved,
            &current_borrows,
            &current_borrow_bindings,
            data_bindings,
            task_result_facts,
        )?;
        apply_guaranteed_expr_effects(
            expr,
            &mut current_moved,
            &mut current_borrows,
            &mut current_borrow_bindings,
            true,
        );
    }
    Ok(())
}

pub(super) fn verify_expr(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String> {
    verify_expr_uses(expr, moved)?;

    if expr_is_fixed_readable_carry_source(expr) {
        return verify_fixed_readable_carry_source_expr(
            expr,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        );
    }

    if let NirExpr::CpuTaskValue(inner) = expr {
        if let Some(source) = expr_resource_key(inner) {
            if matches!(
                task_result_facts.get(&source),
                Some(
                    TaskResultStateFact::TimedOut
                        | TaskResultStateFact::Cancelled
                        | TaskResultStateFact::NotCompleted
                )
            ) {
                return Err(format!(
                    "nir verify: cannot extract task_value from `{}` on a non-completed lifecycle path",
                    source
                ));
            }
        }
    }

    verify_data_expr_shape(expr, data_bindings)?;

    verify_glm_expr_access(expr, moved, borrows, borrow_bindings)?;

    verify_expr_tree(
        expr,
        moved,
        borrows,
        borrow_bindings,
        data_bindings,
        task_result_facts,
    )?;

    Ok(())
}
