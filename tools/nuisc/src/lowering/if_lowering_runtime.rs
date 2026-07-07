use super::*;

#[path = "if_lowering_runtime_extract.rs"]
mod if_lowering_runtime_extract;

use if_lowering_runtime_extract::{
    extract_selectable_cpu_binary_runtime_binding_chain,
    extract_selectable_cpu_binary_runtime_expr, extract_selectable_cpu_binary_runtime_return_chain,
    extract_selectable_cpu_call_runtime_binding_chain, extract_selectable_cpu_call_runtime_expr,
    extract_selectable_cpu_call_runtime_return_chain,
    extract_selectable_cpu_unary_runtime_binding_chain, extract_selectable_cpu_unary_runtime_expr,
    extract_selectable_cpu_unary_runtime_return_chain, SelectableCpuBinaryRuntimeOp,
    SelectableCpuCallRuntimeOp, SelectableCpuUnaryRuntimeOp,
};

fn build_selectable_cpu_unary_runtime_expr(
    op: SelectableCpuUnaryRuntimeOp,
    input: &NirExpr,
) -> NirExpr {
    match op {
        SelectableCpuUnaryRuntimeOp::Join => NirExpr::CpuJoin(Box::new(input.clone())),
        SelectableCpuUnaryRuntimeOp::ThreadJoin => NirExpr::CpuThreadJoin(Box::new(input.clone())),
        SelectableCpuUnaryRuntimeOp::JoinResult => NirExpr::CpuJoinResult(Box::new(input.clone())),
        SelectableCpuUnaryRuntimeOp::ThreadJoinResult => {
            NirExpr::CpuThreadJoinResult(Box::new(input.clone()))
        }
        SelectableCpuUnaryRuntimeOp::Cancel => NirExpr::CpuCancel(Box::new(input.clone())),
        SelectableCpuUnaryRuntimeOp::MutexNew => NirExpr::CpuMutexNew(Box::new(input.clone())),
        SelectableCpuUnaryRuntimeOp::MutexLock => NirExpr::CpuMutexLock(Box::new(input.clone())),
        SelectableCpuUnaryRuntimeOp::MutexUnlock => {
            NirExpr::CpuMutexUnlock(Box::new(input.clone()))
        }
    }
}

fn lower_selected_cpu_unary_runtime_effect(
    condition_name: String,
    lhs_value: &NirExpr,
    rhs_value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some((lhs_op, lhs_input)) = extract_selectable_cpu_unary_runtime_expr(lhs_value) else {
        return Ok(None);
    };
    let Some((rhs_op, rhs_input)) = extract_selectable_cpu_unary_runtime_expr(rhs_value) else {
        return Ok(None);
    };
    if lhs_op != rhs_op {
        return Ok(None);
    }

    let selected_input = if matches!(
        lhs_op,
        SelectableCpuUnaryRuntimeOp::Join
            | SelectableCpuUnaryRuntimeOp::JoinResult
            | SelectableCpuUnaryRuntimeOp::ThreadJoin
            | SelectableCpuUnaryRuntimeOp::ThreadJoinResult
            | SelectableCpuUnaryRuntimeOp::Cancel
    ) {
        if let (
            Some((lhs_bin_op, lhs_bin_lhs, lhs_bin_rhs)),
            Some((rhs_bin_op, rhs_bin_lhs, rhs_bin_rhs)),
        ) = (
            extract_selectable_cpu_binary_runtime_expr(lhs_input),
            extract_selectable_cpu_binary_runtime_expr(rhs_input),
        ) {
            if let Some(lowered) = lower_selected_cpu_binary_runtime_effect(
                condition_name.clone(),
                SelectableCpuBinaryRuntimeCandidate {
                    op: lhs_bin_op,
                    lhs: lhs_bin_lhs,
                    rhs: lhs_bin_rhs,
                },
                SelectableCpuBinaryRuntimeCandidate {
                    op: rhs_bin_op,
                    lhs: rhs_bin_lhs,
                    rhs: rhs_bin_rhs,
                },
                state,
                bindings,
            )? {
                lowered
            } else {
                let lhs_name = lower_expr(lhs_input, state, bindings)?;
                let rhs_name = lower_expr(rhs_input, state, bindings)?;
                lower_select(condition_name, lhs_name, rhs_name, state)?
            }
        } else if let (
            Some((lhs_call_op, lhs_callee, lhs_args)),
            Some((rhs_call_op, rhs_callee, rhs_args)),
        ) = (
            extract_selectable_cpu_call_runtime_expr(lhs_input),
            extract_selectable_cpu_call_runtime_expr(rhs_input),
        ) {
            if let Some(lowered) = lower_selected_cpu_call_runtime_effect(
                condition_name.clone(),
                SelectableCpuCallRuntimeCandidate {
                    op: lhs_call_op,
                    callee: lhs_callee,
                    args: lhs_args,
                },
                SelectableCpuCallRuntimeCandidate {
                    op: rhs_call_op,
                    callee: rhs_callee,
                    args: rhs_args,
                },
                state,
                bindings,
            )? {
                lowered
            } else {
                let lhs_name = lower_expr(lhs_input, state, bindings)?;
                let rhs_name = lower_expr(rhs_input, state, bindings)?;
                lower_select(condition_name, lhs_name, rhs_name, state)?
            }
        } else {
            let lhs_name = lower_expr(lhs_input, state, bindings)?;
            let rhs_name = lower_expr(rhs_input, state, bindings)?;
            lower_select(condition_name, lhs_name, rhs_name, state)?
        }
    } else {
        let lhs_name = lower_expr(lhs_input, state, bindings)?;
        let rhs_name = lower_expr(rhs_input, state, bindings)?;
        lower_select(condition_name, lhs_name, rhs_name, state)?
    };
    let name = next_name(state, lhs_op.prefix());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: lhs_op.instruction().to_owned(),
            args: vec![selected_input.clone()],
        },
    });
    push_dep_edges(state, &selected_input, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: selected_input,
        to: name.clone(),
    });
    Ok(Some(name))
}

struct SelectableCpuCallRuntimeCandidate<'a> {
    op: SelectableCpuCallRuntimeOp,
    callee: &'a str,
    args: &'a [NirExpr],
}

fn lower_selected_cpu_call_runtime_effect(
    condition_name: String,
    lhs: SelectableCpuCallRuntimeCandidate<'_>,
    rhs: SelectableCpuCallRuntimeCandidate<'_>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    if lhs.op != rhs.op || lhs.callee != rhs.callee || lhs.args.len() != rhs.args.len() {
        return Ok(None);
    }

    let mut selected_bindings = bindings.clone();
    let mut selected_args = Vec::with_capacity(lhs.args.len());
    for (index, (lhs_arg, rhs_arg)) in lhs.args.iter().zip(rhs.args.iter()).enumerate() {
        let lhs_name = lower_expr(lhs_arg, state, bindings)?;
        let rhs_name = lower_expr(rhs_arg, state, bindings)?;
        let selected_name = lower_select(condition_name.clone(), lhs_name, rhs_name, state)?;
        let temp_name = format!("__nuis_selected_runtime_arg_{index}");
        selected_bindings.insert(temp_name.clone(), selected_name);
        selected_args.push(NirExpr::Var(temp_name));
    }

    let returned = super::body_lowering::lower_async_call_boundary(
        lhs.callee,
        &selected_args,
        state,
        &selected_bindings,
    )?;
    let name = next_name(state, lhs.op.prefix());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: lhs.op.instruction().to_owned(),
            args: vec![lhs.callee.to_owned(), returned.clone()],
        },
    });
    push_dep_edges(state, &returned, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: returned,
        to: name.clone(),
    });
    Ok(Some(name))
}

struct SelectableCpuBinaryRuntimeCandidate<'a> {
    op: SelectableCpuBinaryRuntimeOp,
    lhs: &'a NirExpr,
    rhs: &'a NirExpr,
}

fn lower_selected_cpu_binary_runtime_effect(
    condition_name: String,
    lhs: SelectableCpuBinaryRuntimeCandidate<'_>,
    rhs: SelectableCpuBinaryRuntimeCandidate<'_>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    if lhs.op != rhs.op {
        return Ok(None);
    }

    let selected_lhs = if lhs.op == SelectableCpuBinaryRuntimeOp::Timeout {
        if let Some(lowered) = lower_selected_cpu_unary_runtime_effect(
            condition_name.clone(),
            lhs.lhs,
            rhs.lhs,
            state,
            bindings,
        )? {
            lowered
        } else if let (
            Some((lhs_call_op, lhs_callee, lhs_args)),
            Some((rhs_call_op, rhs_callee, rhs_args)),
        ) = (
            extract_selectable_cpu_call_runtime_expr(lhs.lhs),
            extract_selectable_cpu_call_runtime_expr(rhs.lhs),
        ) {
            if let Some(lowered) = lower_selected_cpu_call_runtime_effect(
                condition_name.clone(),
                SelectableCpuCallRuntimeCandidate {
                    op: lhs_call_op,
                    callee: lhs_callee,
                    args: lhs_args,
                },
                SelectableCpuCallRuntimeCandidate {
                    op: rhs_call_op,
                    callee: rhs_callee,
                    args: rhs_args,
                },
                state,
                bindings,
            )? {
                lowered
            } else {
                lower_select(
                    condition_name.clone(),
                    lower_expr(lhs.lhs, state, bindings)?,
                    lower_expr(rhs.lhs, state, bindings)?,
                    state,
                )?
            }
        } else {
            lower_select(
                condition_name.clone(),
                lower_expr(lhs.lhs, state, bindings)?,
                lower_expr(rhs.lhs, state, bindings)?,
                state,
            )?
        }
    } else {
        lower_select(
            condition_name.clone(),
            lower_expr(lhs.lhs, state, bindings)?,
            lower_expr(rhs.lhs, state, bindings)?,
            state,
        )?
    };
    let selected_rhs = lower_select(
        condition_name,
        lower_expr(lhs.rhs, state, bindings)?,
        lower_expr(rhs.rhs, state, bindings)?,
        state,
    )?;

    let name = next_name(state, lhs.op.prefix());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: lhs.op.instruction().to_owned(),
            args: vec![selected_lhs.clone(), selected_rhs.clone()],
        },
    });
    push_dep_edges(state, &selected_lhs, &name);
    push_dep_edges(state, &selected_rhs, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: selected_lhs,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: selected_rhs,
        to: name.clone(),
    });
    Ok(Some(name))
}

pub(super) fn lower_direct_selectable_runtime_binding(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some((lhs_name, lhs_op, lhs_input)) =
        extract_selectable_cpu_unary_runtime_binding_chain(then_body)
    else {
        return Ok(None);
    };
    let Some((rhs_name, rhs_op, rhs_input)) =
        extract_selectable_cpu_unary_runtime_binding_chain(else_body)
    else {
        return Ok(None);
    };
    if lhs_name != rhs_name || lhs_op != rhs_op {
        return Ok(None);
    }

    let lhs_expr = build_selectable_cpu_unary_runtime_expr(lhs_op, lhs_input);
    let rhs_expr = build_selectable_cpu_unary_runtime_expr(rhs_op, rhs_input);
    let Some(value) = lower_selected_cpu_unary_runtime_effect(
        condition_name,
        &lhs_expr,
        &rhs_expr,
        state,
        bindings,
    )?
    else {
        return Ok(None);
    };
    super::body_lowering::chain_statement_effect(state, &value);
    Ok(Some(LoweredIfOutcome::Bind {
        name: lhs_name,
        value,
    }))
}

pub(super) fn lower_direct_selectable_call_runtime_binding(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some((lhs_name, lhs_op, lhs_callee, lhs_args)) =
        extract_selectable_cpu_call_runtime_binding_chain(then_body)
    else {
        return Ok(None);
    };
    let Some((rhs_name, rhs_op, rhs_callee, rhs_args)) =
        extract_selectable_cpu_call_runtime_binding_chain(else_body)
    else {
        return Ok(None);
    };
    if lhs_name != rhs_name {
        return Ok(None);
    }

    let Some(value) = lower_selected_cpu_call_runtime_effect(
        condition_name,
        SelectableCpuCallRuntimeCandidate {
            op: lhs_op,
            callee: lhs_callee,
            args: lhs_args,
        },
        SelectableCpuCallRuntimeCandidate {
            op: rhs_op,
            callee: rhs_callee,
            args: rhs_args,
        },
        state,
        bindings,
    )?
    else {
        return Ok(None);
    };
    super::body_lowering::chain_statement_effect(state, &value);
    Ok(Some(LoweredIfOutcome::Bind {
        name: lhs_name,
        value,
    }))
}

pub(super) fn lower_direct_selectable_binary_runtime_binding(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some((lhs_name, lhs_op, lhs_lhs, lhs_rhs)) =
        extract_selectable_cpu_binary_runtime_binding_chain(then_body)
    else {
        return Ok(None);
    };
    let Some((rhs_name, rhs_op, rhs_lhs, rhs_rhs)) =
        extract_selectable_cpu_binary_runtime_binding_chain(else_body)
    else {
        return Ok(None);
    };
    if lhs_name != rhs_name {
        return Ok(None);
    }

    let Some(value) = lower_selected_cpu_binary_runtime_effect(
        condition_name,
        SelectableCpuBinaryRuntimeCandidate {
            op: lhs_op,
            lhs: lhs_lhs,
            rhs: lhs_rhs,
        },
        SelectableCpuBinaryRuntimeCandidate {
            op: rhs_op,
            lhs: rhs_lhs,
            rhs: rhs_rhs,
        },
        state,
        bindings,
    )?
    else {
        return Ok(None);
    };
    super::body_lowering::chain_statement_effect(state, &value);
    Ok(Some(LoweredIfOutcome::Bind {
        name: lhs_name,
        value,
    }))
}

pub(super) fn lower_direct_selectable_runtime_return(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some((lhs_op, lhs_input)) = extract_selectable_cpu_unary_runtime_return_chain(then_body)
    else {
        return Ok(None);
    };
    let Some((rhs_op, rhs_input)) = extract_selectable_cpu_unary_runtime_return_chain(else_body)
    else {
        return Ok(None);
    };
    if lhs_op != rhs_op {
        return Ok(None);
    }

    let lhs_expr = build_selectable_cpu_unary_runtime_expr(lhs_op, lhs_input);
    let rhs_expr = build_selectable_cpu_unary_runtime_expr(rhs_op, rhs_input);
    let Some(value) = lower_selected_cpu_unary_runtime_effect(
        condition_name,
        &lhs_expr,
        &rhs_expr,
        state,
        bindings,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(LoweredIfOutcome::Returned(value)))
}

pub(super) fn lower_direct_selectable_call_runtime_return(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some((lhs_op, lhs_callee, lhs_args)) =
        extract_selectable_cpu_call_runtime_return_chain(then_body)
    else {
        return Ok(None);
    };
    let Some((rhs_op, rhs_callee, rhs_args)) =
        extract_selectable_cpu_call_runtime_return_chain(else_body)
    else {
        return Ok(None);
    };

    let Some(value) = lower_selected_cpu_call_runtime_effect(
        condition_name,
        SelectableCpuCallRuntimeCandidate {
            op: lhs_op,
            callee: lhs_callee,
            args: lhs_args,
        },
        SelectableCpuCallRuntimeCandidate {
            op: rhs_op,
            callee: rhs_callee,
            args: rhs_args,
        },
        state,
        bindings,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(LoweredIfOutcome::Returned(value)))
}

pub(super) fn lower_direct_selectable_binary_runtime_return(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some((lhs_op, lhs_lhs, lhs_rhs)) =
        extract_selectable_cpu_binary_runtime_return_chain(then_body)
    else {
        return Ok(None);
    };
    let Some((rhs_op, rhs_lhs, rhs_rhs)) =
        extract_selectable_cpu_binary_runtime_return_chain(else_body)
    else {
        return Ok(None);
    };

    let Some(value) = lower_selected_cpu_binary_runtime_effect(
        condition_name,
        SelectableCpuBinaryRuntimeCandidate {
            op: lhs_op,
            lhs: lhs_lhs,
            rhs: lhs_rhs,
        },
        SelectableCpuBinaryRuntimeCandidate {
            op: rhs_op,
            lhs: rhs_lhs,
            rhs: rhs_rhs,
        },
        state,
        bindings,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(LoweredIfOutcome::Returned(value)))
}
