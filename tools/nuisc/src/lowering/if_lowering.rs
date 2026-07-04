use super::*;

#[path = "if_lowering_chains.rs"]
mod if_lowering_chains;
#[path = "if_lowering_effects.rs"]
mod if_lowering_effects;
#[path = "if_lowering_runtime.rs"]
mod if_lowering_runtime;

pub(super) use if_lowering_chains::lower_guard_return_chain;
use if_lowering_chains::{
    lower_binding_if_chain, lower_binding_if_chain_with_shared_context, lower_return_if_chain,
    lower_return_if_chain_with_shared_context,
};
pub(super) use if_lowering_effects::stmts_contain_conditional_effect_primitive;
use if_lowering_runtime::{
    lower_direct_selectable_binary_runtime_binding, lower_direct_selectable_binary_runtime_return,
    lower_direct_selectable_call_runtime_binding, lower_direct_selectable_call_runtime_return,
    lower_direct_selectable_runtime_binding, lower_direct_selectable_runtime_return,
};

fn unsupported_if_shape_message(then_body: &[NirStmt], else_body: &[NirStmt]) -> String {
    if stmts_contain_conditional_effect_primitive(then_body)
        || stmts_contain_conditional_effect_primitive(else_body)
    {
        "conditional `if`/lowered-`match` lowering does not yet support branch-local consuming task/thread/mutex runtime primitives such as join-result, lock, unlock, spawn, join, or timeout; hoist those effects before the branch or reduce each branch to pure/select-compatible values"
            .to_owned()
    } else {
        "minimal nuisc lowering currently only supports `if` as matching `print`, matching `let/const`, `return <expr>`, or small terminal branches like `print(...); return ...`"
            .to_owned()
    }
}

pub(super) fn lower_if_pair(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<LoweredIfOutcome, String> {
    if then_body.is_empty() && else_body.is_empty() {
        return Ok(LoweredIfOutcome::Continued);
    }

    if let Some(lowered) = lower_guard_return_with_surviving_binding(
        &condition_name,
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
    }

    if then_body.len() != 1 || else_body.len() != 1 {
        if else_body.is_empty() {
            if let Some(then_branch) = prepare_terminal_branch(then_body, &state.pure_helpers) {
                match then_branch {
                    PreparedTerminalBranch::Return(value) => {
                        let lowered = lower_expr(&value, state, bindings)?;
                        lower_guard_return(condition_name, lowered, state);
                    }
                    PreparedTerminalBranch::PrintReturn { print, returned } => {
                        let print_name = lower_expr(&print, state, bindings)?;
                        let return_name = lower_expr(&returned, state, bindings)?;
                        lower_guard_print_return(condition_name, print_name, return_name, state);
                    }
                }
                return Ok(LoweredIfOutcome::Continued);
            }
        }
        if let (Some(then_branch), Some(else_branch)) = (
            prepare_terminal_branch(then_body, &state.pure_helpers),
            prepare_terminal_branch(else_body, &state.pure_helpers),
        ) {
            match (then_branch, else_branch) {
                (
                    PreparedTerminalBranch::PrintReturn {
                        print: then_print,
                        returned: then_return,
                    },
                    PreparedTerminalBranch::PrintReturn {
                        print: else_print,
                        returned: else_return,
                    },
                ) => {
                    let then_print_name = lower_expr(&then_print, state, bindings)?;
                    let then_return_name = lower_expr(&then_return, state, bindings)?;
                    let else_print_name = lower_expr(&else_print, state, bindings)?;
                    let else_return_name = lower_expr(&else_return, state, bindings)?;
                    lower_branch_print_return(
                        condition_name,
                        then_print_name,
                        then_return_name,
                        else_print_name,
                        else_return_name,
                        state,
                    );
                    return Ok(LoweredIfOutcome::Continued);
                }
                (PreparedTerminalBranch::Return(lhs), PreparedTerminalBranch::Return(rhs)) => {
                    let lhs_name = lower_expr(&lhs, state, bindings)?;
                    let rhs_name = lower_expr(&rhs, state, bindings)?;
                    let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
                    return Ok(LoweredIfOutcome::Returned(selected));
                }
                _ => {}
            }
        }
        if let (Some(lhs), Some(rhs)) = (
            lower_return_if_chain(then_body, state, bindings)?,
            lower_return_if_chain(else_body, state, bindings)?,
        ) {
            let selected = lower_select(condition_name, lhs, rhs, state)?;
            return Ok(LoweredIfOutcome::Returned(selected));
        }
        if let Some(lowered) = lower_direct_selectable_runtime_return(
            condition_name.clone(),
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        if let Some(lowered) = lower_direct_selectable_call_runtime_return(
            condition_name.clone(),
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        if let Some(lowered) = lower_direct_selectable_binary_runtime_return(
            condition_name.clone(),
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        let pure_helpers = state.pure_helpers.clone();
        if let (Some((lhs_name, lhs_value)), Some((rhs_name, rhs_value))) = (
            lower_binding_if_chain(then_body, state, bindings, &pure_helpers)?,
            lower_binding_if_chain(else_body, state, bindings, &pure_helpers)?,
        ) {
            if lhs_name == rhs_name {
                let selected = lower_select(condition_name, lhs_value, rhs_value, state)?;
                return Ok(LoweredIfOutcome::Bind {
                    name: lhs_name,
                    value: selected,
                });
            }
        }
        if let Some(lowered) = lower_direct_selectable_runtime_binding(
            condition_name.clone(),
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        if let Some(lowered) = lower_direct_selectable_call_runtime_binding(
            condition_name.clone(),
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        if let Some(lowered) = lower_direct_selectable_binary_runtime_binding(
            condition_name.clone(),
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        if let Some(lowered) = lower_binding_if_chain_with_shared_context(
            &condition_name,
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        if let Some(lowered) = lower_return_if_chain_with_shared_context(
            &condition_name,
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        return Err(unsupported_if_shape_message(then_body, else_body));
    }

    if let (Some(lhs), Some(rhs)) = (
        lower_return_if_chain(then_body, state, bindings)?,
        lower_return_if_chain(else_body, state, bindings)?,
    ) {
        let selected = lower_select(condition_name, lhs, rhs, state)?;
        return Ok(LoweredIfOutcome::Returned(selected));
    }

    if let Some(lowered) = lower_direct_selectable_runtime_return(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
    }
    if let Some(lowered) = lower_direct_selectable_call_runtime_return(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
    }
    if let Some(lowered) = lower_direct_selectable_binary_runtime_return(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
    }

    let pure_helpers = state.pure_helpers.clone();
    if let (Some((lhs_name, lhs_value)), Some((rhs_name, rhs_value))) = (
        lower_binding_if_chain(then_body, state, bindings, &pure_helpers)?,
        lower_binding_if_chain(else_body, state, bindings, &pure_helpers)?,
    ) {
        if lhs_name == rhs_name {
            let selected = lower_select(condition_name, lhs_value, rhs_value, state)?;
            return Ok(LoweredIfOutcome::Bind {
                name: lhs_name,
                value: selected,
            });
        }
    }

    if let Some(lowered) = lower_direct_selectable_runtime_binding(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
    }
    if let Some(lowered) = lower_direct_selectable_call_runtime_binding(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
    }
    if let Some(lowered) = lower_direct_selectable_binary_runtime_binding(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
    }

    match (&then_body[0], &else_body[0]) {
        (NirStmt::Print(lhs), NirStmt::Print(rhs)) => {
            let lhs_name = lower_expr(lhs, state, bindings)?;
            let rhs_name = lower_expr(rhs, state, bindings)?;
            let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
            let print_name = format!("print_{}", state.print_counter);
            state.print_counter += 1;
            state.yir.nodes.push(Node {
                name: print_name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "print".to_owned(),
                    args: vec![selected.clone()],
                },
            });
            push_dep_edges(state, &selected, &print_name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: selected,
                to: print_name,
            });
            Ok(LoweredIfOutcome::Printed)
        }
        (
            NirStmt::Let {
                name: lhs_name,
                value: lhs_value,
                ..
            },
            NirStmt::Let {
                name: rhs_name,
                value: rhs_value,
                ..
            },
        )
        | (
            NirStmt::Const {
                name: lhs_name,
                value: lhs_value,
                ..
            },
            NirStmt::Const {
                name: rhs_name,
                value: rhs_value,
                ..
            },
        ) if lhs_name == rhs_name => {
            let lhs_value = lower_expr(lhs_value, state, bindings)?;
            let rhs_value = lower_expr(rhs_value, state, bindings)?;
            let selected = lower_select(condition_name, lhs_value, rhs_value, state)?;
            Ok(LoweredIfOutcome::Bind {
                name: lhs_name.clone(),
                value: selected,
            })
        }
        (NirStmt::Return(Some(lhs)), NirStmt::Return(Some(rhs))) => {
            let lhs_name = lower_expr(lhs, state, bindings)?;
            let rhs_name = lower_expr(rhs, state, bindings)?;
            let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
            Ok(LoweredIfOutcome::Returned(selected))
        }
        _ => Err(unsupported_if_shape_message(then_body, else_body)),
    }
}

fn lower_guard_return_with_surviving_binding(
    condition_name: &str,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some(PreparedTerminalBranch::Return(returned)) =
        prepare_terminal_branch(then_body, &state.pure_helpers)
    else {
        return Ok(None);
    };
    let pure_helpers = state.pure_helpers.clone();
    let Some((surviving_name, surviving_value)) =
        lower_binding_if_chain(else_body, state, bindings, &pure_helpers)?
    else {
        return Ok(None);
    };
    let returned_name = lower_expr(&returned, state, bindings)?;
    lower_guard_return(condition_name.to_owned(), returned_name, state);
    Ok(Some(LoweredIfOutcome::Bind {
        name: surviving_name,
        value: surviving_value,
    }))
}
