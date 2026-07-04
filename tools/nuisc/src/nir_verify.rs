mod data;
mod effects;
mod expr;
mod task_result_facts;
mod uses;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{NirExpr, NirFunction, NirModule, NirStmt, NirTypeRef};

use self::data::{infer_data_kind, infer_data_kind_from_type, render_data_expr_name, NirDataKind};
use self::effects::{
    ensure_binding_can_be_rebound, merge_branch_state, merge_control_flow_borrow_bindings,
    merge_control_flow_data_bindings, note_binding_effects,
};
use self::expr::{apply_guaranteed_expr_effects, verify_condition_expr, verify_expr};
use self::task_result_facts::{
    apply_task_result_condition_facts, borrowed_address_alias_source, borrowed_address_binding,
    merge_control_flow_task_result_facts, BorrowBindings, TaskResultStateFact,
};
use self::uses::expr_resource_key;

pub fn verify_nir_module(module: &NirModule) -> Result<(), String> {
    verify_declared_types(module)?;
    for function in &module.functions {
        verify_function(function)
            .map_err(|error| format!("{error} in function `{}`", function.name))?;
    }
    Ok(())
}

fn verify_declared_types(module: &NirModule) -> Result<(), String> {
    for function in &module.externs {
        for param in &function.params {
            verify_type_ref(&param.ty)?;
        }
        verify_type_ref(&function.return_type)?;
    }
    for interface in &module.extern_interfaces {
        for method in &interface.methods {
            for param in &method.params {
                verify_type_ref(&param.ty)?;
            }
            verify_type_ref(&method.return_type)?;
        }
    }
    for definition in &module.structs {
        for field in &definition.fields {
            verify_type_ref(&field.ty)?;
        }
    }
    for function in &module.functions {
        for param in &function.params {
            verify_type_ref(&param.ty)?;
        }
        if let Some(return_type) = &function.return_type {
            verify_type_ref(return_type)?;
        }
        for stmt in &function.body {
            match stmt {
                NirStmt::Let { ty, .. } => {
                    if let Some(ty) = ty {
                        verify_type_ref(ty)?;
                    }
                }
                NirStmt::Const { ty, .. } => verify_type_ref(ty)?,
                NirStmt::Print(_)
                | NirStmt::Await(_)
                | NirStmt::Expr(_)
                | NirStmt::Return(_)
                | NirStmt::If { .. }
                | NirStmt::While { .. }
                | NirStmt::Break
                | NirStmt::Continue => {}
            }
        }
    }
    Ok(())
}

fn verify_type_ref(ty: &NirTypeRef) -> Result<(), String> {
    ty.validate_container_contract()
        .map_err(|error| format!("nir verify: invalid type `{}`: {error}", ty.render()))
}

fn borrowed_address_kind_label(expr: &NirExpr, borrow_bindings: &BorrowBindings) -> &'static str {
    match borrowed_address_binding(expr, borrow_bindings) {
        Some(binding) if binding.via_traversal => "borrowed traversal alias",
        Some(_) => "borrowed address alias",
        None => "borrowed address alias",
    }
}

fn owned_address_error(
    operation: &str,
    expr: &NirExpr,
    borrow_bindings: &BorrowBindings,
) -> String {
    format!(
        "nir verify: {operation} expects owned address, found {} `{}`",
        borrowed_address_kind_label(expr, borrow_bindings),
        render_data_expr_name(expr),
    )
}

fn owned_structural_address_error(
    operation: &str,
    expr: &NirExpr,
    borrow_bindings: &BorrowBindings,
) -> String {
    format!(
        "nir verify: {operation} requires owned structural address, found {} `{}`",
        borrowed_address_kind_label(expr, borrow_bindings),
        render_data_expr_name(expr),
    )
}

fn ensure_owned_address_target(
    operation: &str,
    expr: &NirExpr,
    borrow_bindings: &BorrowBindings,
) -> Result<(), String> {
    if borrowed_address_alias_source(expr, borrow_bindings).is_some() {
        return Err(owned_address_error(operation, expr, borrow_bindings));
    }
    Ok(())
}

fn expr_is_fixed_readable_carry_source(expr: &NirExpr) -> bool {
    matches!(expr, NirExpr::LoadValue(_) | NirExpr::LoadAt { .. })
}

fn verify_fixed_readable_carry_source_expr(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String> {
    match expr {
        NirExpr::LoadValue(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        ),
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
            )
        }
        _ => Err("nir verify: expected fixed readable carry source candidate".to_owned()),
    }
}

fn verify_function(function: &NirFunction) -> Result<(), String> {
    let mut moved = BTreeSet::<String>::new();
    let mut borrows = BTreeMap::<String, usize>::new();
    let mut borrow_bindings = BorrowBindings::new();
    let mut data_bindings = function
        .params
        .iter()
        .map(|param| (param.name.clone(), infer_data_kind_from_type(&param.ty)))
        .collect::<BTreeMap<_, _>>();
    let mut task_result_facts = BTreeMap::<String, TaskResultStateFact>::new();

    for stmt in &function.body {
        verify_stmt(
            stmt,
            &mut moved,
            &mut borrows,
            &mut borrow_bindings,
            &mut data_bindings,
            &mut task_result_facts,
        )?;
    }

    Ok(())
}

fn verify_stmt(
    stmt: &NirStmt,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    borrow_bindings: &mut BorrowBindings,
    data_bindings: &mut BTreeMap<String, NirDataKind>,
    task_result_facts: &mut BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String> {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            ensure_binding_can_be_rebound(name, borrows, borrow_bindings)?;
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            borrows.remove(name);
            borrow_bindings.remove(name);
            data_bindings.remove(name);
            task_result_facts.remove(name);
            note_binding_effects(value, name, moved, borrows, borrow_bindings);
            data_bindings.insert(
                name.clone(),
                infer_data_kind(value, data_bindings)
                    .merge_with_type_hint(ty.as_ref().map(infer_data_kind_from_type)),
            );
        }
        NirStmt::Const { name, ty, value } => {
            ensure_binding_can_be_rebound(name, borrows, borrow_bindings)?;
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            borrows.remove(name);
            borrow_bindings.remove(name);
            data_bindings.remove(name);
            task_result_facts.remove(name);
            note_binding_effects(value, name, moved, borrows, borrow_bindings);
            data_bindings.insert(
                name.clone(),
                infer_data_kind(value, data_bindings)
                    .merge_with_type_hint(Some(infer_data_kind_from_type(ty))),
            );
        }
        NirStmt::Print(value) | NirStmt::Await(value) | NirStmt::Expr(value) => {
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            note_binding_effects(value, "_", moved, borrows, borrow_bindings);
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            verify_condition_expr(
                condition,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            apply_guaranteed_expr_effects(condition, moved, borrows, borrow_bindings, false);
            let mut then_moved = moved.clone();
            let mut then_borrows = borrows.clone();
            let mut then_borrow_bindings = borrow_bindings.clone();
            let mut then_data_bindings = data_bindings.clone();
            let mut then_task_result_facts = task_result_facts.clone();
            let mut else_task_result_facts = task_result_facts.clone();
            apply_task_result_condition_facts(
                condition,
                &mut then_task_result_facts,
                &mut else_task_result_facts,
            );
            for stmt in then_body {
                verify_stmt(
                    stmt,
                    &mut then_moved,
                    &mut then_borrows,
                    &mut then_borrow_bindings,
                    &mut then_data_bindings,
                    &mut then_task_result_facts,
                )?;
            }
            let mut else_moved = moved.clone();
            let mut else_borrows = borrows.clone();
            let mut else_borrow_bindings = borrow_bindings.clone();
            let mut else_data_bindings = data_bindings.clone();
            for stmt in else_body {
                verify_stmt(
                    stmt,
                    &mut else_moved,
                    &mut else_borrows,
                    &mut else_borrow_bindings,
                    &mut else_data_bindings,
                    &mut else_task_result_facts,
                )?;
            }
            match (
                block_always_terminates(then_body),
                block_always_terminates(else_body),
            ) {
                (true, false) => {
                    *moved = else_moved;
                    *borrows = else_borrows;
                    *borrow_bindings = else_borrow_bindings;
                    *data_bindings = else_data_bindings;
                    *task_result_facts = else_task_result_facts;
                }
                (false, true) => {
                    *moved = then_moved;
                    *borrows = then_borrows;
                    *borrow_bindings = then_borrow_bindings;
                    *data_bindings = then_data_bindings;
                    *task_result_facts = then_task_result_facts;
                }
                _ => {
                    merge_branch_state(
                        moved,
                        borrows,
                        &then_moved,
                        &then_borrows,
                        &else_moved,
                        &else_borrows,
                    );
                    merge_control_flow_borrow_bindings(
                        borrow_bindings,
                        &then_borrow_bindings,
                        &else_borrow_bindings,
                    );
                    merge_control_flow_data_bindings(
                        data_bindings,
                        &then_data_bindings,
                        &else_data_bindings,
                    );
                    merge_control_flow_task_result_facts(
                        task_result_facts,
                        &then_task_result_facts,
                        &else_task_result_facts,
                    );
                }
            }
        }
        NirStmt::While { condition, body } => {
            verify_condition_expr(
                condition,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            apply_guaranteed_expr_effects(condition, moved, borrows, borrow_bindings, false);
            let pre_loop_moved = moved.clone();
            let pre_loop_borrows = borrows.clone();
            let pre_loop_borrow_bindings = borrow_bindings.clone();
            let pre_loop_data_bindings = data_bindings.clone();
            let pre_loop_task_result_facts = task_result_facts.clone();
            let mut loop_moved = moved.clone();
            let mut loop_borrows = borrows.clone();
            let mut loop_borrow_bindings = borrow_bindings.clone();
            let mut loop_data_bindings = data_bindings.clone();
            let mut loop_task_result_facts = task_result_facts.clone();
            for stmt in body {
                verify_stmt(
                    stmt,
                    &mut loop_moved,
                    &mut loop_borrows,
                    &mut loop_borrow_bindings,
                    &mut loop_data_bindings,
                    &mut loop_task_result_facts,
                )?;
            }
            merge_branch_state(
                moved,
                borrows,
                &loop_moved,
                &loop_borrows,
                &pre_loop_moved,
                &pre_loop_borrows,
            );
            merge_control_flow_borrow_bindings(
                borrow_bindings,
                &loop_borrow_bindings,
                &pre_loop_borrow_bindings,
            );
            merge_control_flow_data_bindings(
                data_bindings,
                &loop_data_bindings,
                &pre_loop_data_bindings,
            );
            merge_control_flow_task_result_facts(
                task_result_facts,
                &loop_task_result_facts,
                &pre_loop_task_result_facts,
            );
        }
        NirStmt::Break | NirStmt::Continue => {}
        NirStmt::Return(value) => {
            if let Some(value) = value {
                verify_expr(
                    value,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
    }
    Ok(())
}

fn block_always_terminates(body: &[NirStmt]) -> bool {
    body.iter().any(stmt_always_terminates)
}

fn stmt_always_terminates(stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Return(_) | NirStmt::Break | NirStmt::Continue => true,
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => block_always_terminates(then_body) && block_always_terminates(else_body),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
