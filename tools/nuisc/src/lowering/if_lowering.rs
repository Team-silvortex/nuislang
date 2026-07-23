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
    let shape = format!(
        "then=[{}]; else=[{}]",
        then_body
            .iter()
            .map(describe_if_stmt_shape)
            .collect::<Vec<_>>()
            .join(","),
        else_body
            .iter()
            .map(describe_if_stmt_shape)
            .collect::<Vec<_>>()
            .join(",")
    );
    if stmts_contain_conditional_effect_primitive(then_body)
        || stmts_contain_conditional_effect_primitive(else_body)
    {
        format!("conditional `if`/lowered-`match` lowering does not yet support branch-local consuming task/thread/mutex runtime primitives such as join-result, lock, unlock, spawn, join, or timeout; hoist those effects before the branch or reduce each branch to pure/select-compatible values ({shape})")
    } else {
        format!("minimal nuisc lowering currently only supports `if` as matching `print`, matching `let/const`, `return <expr>`, or small terminal branches like `print(...); return ...` ({shape})")
    }
}

fn describe_if_stmt_shape(stmt: &NirStmt) -> &'static str {
    match stmt {
        NirStmt::Let { .. } => "let",
        NirStmt::Const { .. } => "const",
        NirStmt::Expr(NirExpr::Call { .. }) => "expr-call",
        NirStmt::Expr(NirExpr::CpuExternCall { .. }) => "expr-cpu-extern",
        NirStmt::Expr(_) => "expr",
        NirStmt::Return(Some(NirExpr::Call { .. })) => "return-call",
        NirStmt::Return(Some(NirExpr::CpuExternCall { .. })) => "return-cpu-extern",
        NirStmt::Return(Some(_)) => "return",
        NirStmt::Return(None) => "return-empty",
        NirStmt::Print(_) => "print",
        NirStmt::If { .. } => "if",
        NirStmt::While { .. } => "while",
        NirStmt::Await(_) => "await",
        NirStmt::Break => "break",
        NirStmt::Continue => "continue",
    }
}

fn owned_bytes_move_source(
    expr: &NirExpr,
    state: &LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<String> {
    let NirExpr::Move(value) = expr else {
        return None;
    };
    let NirExpr::Var(binding) = value.as_ref() else {
        return None;
    };
    let source = bindings.get(binding)?;
    state
        .yir
        .nodes
        .iter()
        .find(|node| node.name == *source)
        .filter(|node| {
            matches!(
                node.op.instruction.as_str(),
                "copy_buffer_owned"
                    | "move_owned_bytes"
                    | "param_owned_bytes"
                    | "call_owned_bytes"
                    | "loop_owned_result"
                    | "select_owned_bytes"
                    | "select_owned_bytes_drop_unselected"
                    | "branch_call_owned_bytes"
            )
        })
        .map(|_| source.clone())
}

fn lower_owned_bytes_return_select(
    condition_name: String,
    then_expr: &NirExpr,
    else_expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some(then_name) = owned_bytes_move_source(then_expr, state, bindings) else {
        return Ok(None);
    };
    let Some(else_name) = owned_bytes_move_source(else_expr, state, bindings) else {
        return Ok(None);
    };
    if then_name != else_name {
        return Ok(Some(lower_select_owned_bytes_drop_unselected(
            condition_name,
            then_name,
            else_name,
            state,
        )));
    }
    Ok(Some(lower_select_owned_bytes(
        condition_name,
        then_name,
        else_name,
        state,
    )))
}

fn lower_prepared_host_call_return(
    calls: &[PreparedHostCall],
    returned: &PreparedHostCallReturn,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<(LoweredHostCallChain, PreparedHostCallReturnSpec), String> {
    let calls = calls
        .iter()
        .map(|call| {
            let args = call
                .args
                .iter()
                .map(|arg| lower_expr(arg, state, bindings))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                call.result_name.clone(),
                call.abi.clone(),
                call.callee.clone(),
                args,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let returned = match returned {
        PreparedHostCallReturn::Expr(returned) => {
            PreparedHostCallReturnSpec::Value(lower_expr(returned, state, bindings)?)
        }
        PreparedHostCallReturn::CompareCallResult {
            result_name,
            op,
            expected,
            matched,
            unmatched,
        } => PreparedHostCallReturnSpec::CompareCallResult {
            result_name: result_name.clone(),
            op: *op,
            expected: lower_expr(expected, state, bindings)?,
            matched: lower_expr(matched, state, bindings)?,
            unmatched: lower_expr(unmatched, state, bindings)?,
        },
        PreparedHostCallReturn::WriteFlushExitCode {
            write_name,
            flush_name,
            offset,
        } => PreparedHostCallReturnSpec::WriteFlushExitCode {
            write_name: write_name.clone(),
            flush_name: flush_name.clone(),
            offset: *offset,
        },
    };
    Ok((calls, returned))
}

fn extract_single_return_call(stmts: &[NirStmt]) -> Option<(&str, &[NirExpr])> {
    let ([NirStmt::Return(Some(NirExpr::Call { callee, args }))]
    | [NirStmt::Expr(NirExpr::Call { callee, args })]) = stmts
    else {
        return None;
    };
    Some((callee.as_str(), args.as_slice()))
}

fn lower_matching_call_return(
    condition_name: &str,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some((then_callee, then_args)) = extract_single_return_call(then_body) else {
        return Ok(None);
    };
    let Some((else_callee, else_args)) = extract_single_return_call(else_body) else {
        return Ok(None);
    };
    if then_callee != else_callee || then_args.len() != else_args.len() {
        return Ok(None);
    }

    let mut selected_bindings = bindings.clone();
    let mut selected_args = Vec::with_capacity(then_args.len());
    for (index, (then_arg, else_arg)) in then_args.iter().zip(else_args.iter()).enumerate() {
        let then_name = lower_expr(then_arg, state, bindings)?;
        let else_name = lower_expr(else_arg, state, bindings)?;
        let selected_name = lower_select(condition_name.to_owned(), then_name, else_name, state)?;
        let temp_name = format!("__nuis_selected_call_arg_{index}");
        selected_bindings.insert(temp_name.clone(), selected_name);
        selected_args.push(NirExpr::Var(temp_name));
    }

    let returned = lower_call_expr(then_callee, &selected_args, state, &selected_bindings)?;
    Ok(Some(LoweredIfOutcome::Returned(returned)))
}

pub(super) fn lower_if_pair(
    condition_name: String,
    condition_expr: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<LoweredIfOutcome, String> {
    if then_body.is_empty() && else_body.is_empty() {
        return Ok(LoweredIfOutcome::Continued);
    }

    if then_body == else_body {
        if let [NirStmt::Expr(effect)] = then_body {
            lower_expr(effect, state, bindings)?;
            return Ok(LoweredIfOutcome::Continued);
        }
    }

    if let Some(lowered) = lower_branch_effect(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(lowered);
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

    if let Some(returned) = lower_conditional_owned_return_call(
        condition_name.clone(),
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(LoweredIfOutcome::Returned(returned));
    }

    if let Some(returned) = lower_nested_owned_return_tree(
        condition_name.clone(),
        condition_expr,
        then_body,
        else_body,
        state,
        bindings,
    )? {
        return Ok(LoweredIfOutcome::Returned(returned));
    }

    if let Some(lowered) =
        lower_matching_call_return(&condition_name, then_body, else_body, state, bindings)?
    {
        return Ok(lowered);
    }

    if let ([NirStmt::Return(Some(then_expr))], [NirStmt::Return(Some(else_expr))]) =
        (then_body, else_body)
    {
        if let Some(selected) = lower_owned_bytes_return_select(
            condition_name.clone(),
            then_expr,
            else_expr,
            state,
            bindings,
        )? {
            return Ok(LoweredIfOutcome::Returned(selected));
        }
    }

    if else_body.is_empty() {
        if let Some(returned) = lower_guard_return_chain(then_body, state, bindings)? {
            lower_guard_return(condition_name.clone(), returned, state);
            return Ok(LoweredIfOutcome::Continued);
        }
    }

    if then_body.len() != 1 || else_body.len() != 1 {
        if else_body.is_empty() {
            if let Some(then_branch) = prepare_terminal_branch(then_body, &state.pure_helpers) {
                match then_branch {
                    PreparedTerminalBranch::Return(value) => {
                        let lowered = lower_expr(&value, state, bindings)?;
                        lower_guard_return(condition_name, lowered, state);
                    }
                    PreparedTerminalBranch::DropOwnedBytesReturn { bytes, returned } => {
                        let bytes = lower_expr(&bytes, state, bindings)?;
                        let returned = lower_expr(&returned, state, bindings)?;
                        lower_guard_drop_owned_bytes_return(condition_name, bytes, returned, state);
                    }
                    PreparedTerminalBranch::PrintReturn { print, returned } => {
                        let print_name = lower_expr(&print, state, bindings)?;
                        let return_name = lower_expr(&returned, state, bindings)?;
                        lower_guard_print_return(condition_name, print_name, return_name, state);
                    }
                    PreparedTerminalBranch::HostCallReturn { calls, returned } => {
                        let (calls, returned) =
                            lower_prepared_host_call_return(&calls, &returned, state, bindings)?;
                        lower_guard_host_call_return(condition_name, calls, returned, state);
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
                    if let Some(selected) = lower_owned_bytes_return_select(
                        condition_name.clone(),
                        &lhs,
                        &rhs,
                        state,
                        bindings,
                    )? {
                        return Ok(LoweredIfOutcome::Returned(selected));
                    }
                    let lhs_name = lower_expr(&lhs, state, bindings)?;
                    let rhs_name = lower_expr(&rhs, state, bindings)?;
                    let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
                    return Ok(LoweredIfOutcome::Returned(selected));
                }
                (
                    PreparedTerminalBranch::DropOwnedBytesReturn {
                        bytes: then_bytes,
                        returned: then_returned,
                    },
                    PreparedTerminalBranch::DropOwnedBytesReturn {
                        bytes: else_bytes,
                        returned: else_returned,
                    },
                ) => {
                    let then_bytes = lower_expr(&then_bytes, state, bindings)?;
                    let then_returned = lower_expr(&then_returned, state, bindings)?;
                    let else_bytes = lower_expr(&else_bytes, state, bindings)?;
                    let else_returned = lower_expr(&else_returned, state, bindings)?;
                    lower_branch_drop_owned_bytes_return(
                        condition_name,
                        then_bytes,
                        then_returned,
                        else_bytes,
                        else_returned,
                        state,
                    );
                    return Ok(LoweredIfOutcome::Continued);
                }
                (
                    PreparedTerminalBranch::HostCallReturn {
                        calls: then_calls,
                        returned: then_returned,
                    },
                    PreparedTerminalBranch::HostCallReturn {
                        calls: else_calls,
                        returned: else_returned,
                    },
                ) => {
                    let (then_calls, then_returned) = lower_prepared_host_call_return(
                        &then_calls,
                        &then_returned,
                        state,
                        bindings,
                    )?;
                    let (else_calls, else_returned) = lower_prepared_host_call_return(
                        &else_calls,
                        &else_returned,
                        state,
                        bindings,
                    )?;
                    lower_branch_host_call_return(
                        condition_name,
                        then_calls,
                        then_returned,
                        else_calls,
                        else_returned,
                        state,
                    );
                    return Ok(LoweredIfOutcome::Continued);
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
            if !is_terminal_branch_pure_expr(lhs, &state.pure_helpers)
                || !is_terminal_branch_pure_expr(rhs, &state.pure_helpers)
            {
                return Err(unsupported_if_shape_message(then_body, else_body));
            }
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
    if else_body.is_empty()
        || !else_body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Let { .. } | NirStmt::Const { .. }))
        || !else_body.iter().all(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { .. } | NirStmt::Const { .. } | NirStmt::Expr(_)
            )
        })
    {
        return Ok(None);
    }
    let returned_name = lower_expr(&returned, state, bindings)?;
    let mut local_bindings = bindings.clone();
    let Some(surviving_name) =
        super::body_lowering::lower_linear_stmts(else_body, state, &mut local_bindings)?
    else {
        return Ok(None);
    };
    let Some(surviving_value) = local_bindings.get(&surviving_name).cloned() else {
        return Err(format!(
            "minimal nuisc lowering expected surviving branch binding `{surviving_name}` after guarded return"
        ));
    };
    let selected_value = lower_select(
        condition_name.to_owned(),
        returned_name,
        surviving_value,
        state,
    )?;
    Ok(Some(LoweredIfOutcome::Bind {
        name: surviving_name,
        value: selected_value,
    }))
}
