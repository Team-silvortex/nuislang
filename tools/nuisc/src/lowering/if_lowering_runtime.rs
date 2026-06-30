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
                lhs_bin_op,
                lhs_bin_lhs,
                lhs_bin_rhs,
                rhs_bin_op,
                rhs_bin_lhs,
                rhs_bin_rhs,
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
                lhs_call_op,
                lhs_callee,
                lhs_args,
                rhs_call_op,
                rhs_callee,
                rhs_args,
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

fn lower_selected_cpu_call_runtime_effect(
    condition_name: String,
    lhs_op: SelectableCpuCallRuntimeOp,
    lhs_callee: &str,
    lhs_args: &[NirExpr],
    rhs_op: SelectableCpuCallRuntimeOp,
    rhs_callee: &str,
    rhs_args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    if lhs_op != rhs_op || lhs_callee != rhs_callee || lhs_args.len() != rhs_args.len() {
        return Ok(None);
    }

    let mut selected_bindings = bindings.clone();
    let mut selected_args = Vec::with_capacity(lhs_args.len());
    for (index, (lhs_arg, rhs_arg)) in lhs_args.iter().zip(rhs_args.iter()).enumerate() {
        let lhs_name = lower_expr(lhs_arg, state, bindings)?;
        let rhs_name = lower_expr(rhs_arg, state, bindings)?;
        let selected_name = lower_select(condition_name.clone(), lhs_name, rhs_name, state)?;
        let temp_name = format!("__nuis_selected_runtime_arg_{index}");
        selected_bindings.insert(temp_name.clone(), selected_name);
        selected_args.push(NirExpr::Var(temp_name));
    }

    let returned = super::body_lowering::lower_async_call_boundary(
        lhs_callee,
        &selected_args,
        state,
        &selected_bindings,
    )?;
    let name = next_name(state, lhs_op.prefix());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: lhs_op.instruction().to_owned(),
            args: vec![lhs_callee.to_owned(), returned.clone()],
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

fn lower_selected_cpu_binary_runtime_effect(
    condition_name: String,
    lhs_op: SelectableCpuBinaryRuntimeOp,
    lhs_lhs: &NirExpr,
    lhs_rhs: &NirExpr,
    rhs_op: SelectableCpuBinaryRuntimeOp,
    rhs_lhs: &NirExpr,
    rhs_rhs: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    if lhs_op != rhs_op {
        return Ok(None);
    }

    let selected_lhs = if lhs_op == SelectableCpuBinaryRuntimeOp::Timeout {
        if let Some(lowered) = lower_selected_cpu_unary_runtime_effect(
            condition_name.clone(),
            lhs_lhs,
            rhs_lhs,
            state,
            bindings,
        )? {
            lowered
        } else if let (
            Some((lhs_call_op, lhs_callee, lhs_args)),
            Some((rhs_call_op, rhs_callee, rhs_args)),
        ) = (
            extract_selectable_cpu_call_runtime_expr(lhs_lhs),
            extract_selectable_cpu_call_runtime_expr(rhs_lhs),
        ) {
            if let Some(lowered) = lower_selected_cpu_call_runtime_effect(
                condition_name.clone(),
                lhs_call_op,
                lhs_callee,
                lhs_args,
                rhs_call_op,
                rhs_callee,
                rhs_args,
                state,
                bindings,
            )? {
                lowered
            } else {
                lower_select(
                    condition_name.clone(),
                    lower_expr(lhs_lhs, state, bindings)?,
                    lower_expr(rhs_lhs, state, bindings)?,
                    state,
                )?
            }
        } else {
            lower_select(
                condition_name.clone(),
                lower_expr(lhs_lhs, state, bindings)?,
                lower_expr(rhs_lhs, state, bindings)?,
                state,
            )?
        }
    } else {
        lower_select(
            condition_name.clone(),
            lower_expr(lhs_lhs, state, bindings)?,
            lower_expr(rhs_lhs, state, bindings)?,
            state,
        )?
    };
    let selected_rhs = lower_select(
        condition_name,
        lower_expr(lhs_rhs, state, bindings)?,
        lower_expr(rhs_rhs, state, bindings)?,
        state,
    )?;

    let name = next_name(state, lhs_op.prefix());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: lhs_op.instruction().to_owned(),
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
        lhs_op,
        lhs_callee,
        lhs_args,
        rhs_op,
        rhs_callee,
        rhs_args,
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
        lhs_op,
        lhs_lhs,
        lhs_rhs,
        rhs_op,
        rhs_lhs,
        rhs_rhs,
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
        lhs_op,
        lhs_callee,
        lhs_args,
        rhs_op,
        rhs_callee,
        rhs_args,
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
        lhs_op,
        lhs_lhs,
        lhs_rhs,
        rhs_op,
        rhs_lhs,
        rhs_rhs,
        state,
        bindings,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(LoweredIfOutcome::Returned(value)))
}
