use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectableCpuUnaryRuntimeOp {
    Join,
    ThreadJoin,
    JoinResult,
    ThreadJoinResult,
    Cancel,
    MutexNew,
    MutexLock,
    MutexUnlock,
}

impl SelectableCpuUnaryRuntimeOp {
    fn prefix(self) -> &'static str {
        match self {
            Self::Join => "cpu_join",
            Self::ThreadJoin => "cpu_thread_join",
            Self::JoinResult => "cpu_join_result",
            Self::ThreadJoinResult => "cpu_thread_join_result",
            Self::Cancel => "cpu_cancel",
            Self::MutexNew => "cpu_mutex_new",
            Self::MutexLock => "cpu_mutex_lock",
            Self::MutexUnlock => "cpu_mutex_unlock",
        }
    }

    fn instruction(self) -> &'static str {
        match self {
            Self::Join => "join",
            Self::ThreadJoin => "thread_join",
            Self::JoinResult => "join_result",
            Self::ThreadJoinResult => "thread_join_result",
            Self::Cancel => "cancel",
            Self::MutexNew => "mutex_new",
            Self::MutexLock => "mutex_lock",
            Self::MutexUnlock => "mutex_unlock",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectableCpuCallRuntimeOp {
    Spawn,
    ThreadSpawn,
}

impl SelectableCpuCallRuntimeOp {
    fn prefix(self) -> &'static str {
        match self {
            Self::Spawn => "cpu_spawn_task",
            Self::ThreadSpawn => "cpu_spawn_thread",
        }
    }

    fn instruction(self) -> &'static str {
        match self {
            Self::Spawn => "spawn_task",
            Self::ThreadSpawn => "spawn_thread",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectableCpuBinaryRuntimeOp {
    Timeout,
}

impl SelectableCpuBinaryRuntimeOp {
    fn prefix(self) -> &'static str {
        match self {
            Self::Timeout => "cpu_timeout",
        }
    }

    fn instruction(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
        }
    }
}

fn extract_selectable_cpu_unary_runtime_expr(
    expr: &NirExpr,
) -> Option<(SelectableCpuUnaryRuntimeOp, &NirExpr)> {
    match expr {
        NirExpr::CpuJoin(input) => Some((SelectableCpuUnaryRuntimeOp::Join, input)),
        NirExpr::CpuThreadJoin(input) => Some((SelectableCpuUnaryRuntimeOp::ThreadJoin, input)),
        NirExpr::CpuJoinResult(input) => Some((SelectableCpuUnaryRuntimeOp::JoinResult, input)),
        NirExpr::CpuThreadJoinResult(input) => {
            Some((SelectableCpuUnaryRuntimeOp::ThreadJoinResult, input))
        }
        NirExpr::CpuCancel(input) => Some((SelectableCpuUnaryRuntimeOp::Cancel, input)),
        NirExpr::CpuMutexNew(input) => Some((SelectableCpuUnaryRuntimeOp::MutexNew, input)),
        NirExpr::CpuMutexLock(input) => Some((SelectableCpuUnaryRuntimeOp::MutexLock, input)),
        NirExpr::CpuMutexUnlock(input) => Some((SelectableCpuUnaryRuntimeOp::MutexUnlock, input)),
        _ => None,
    }
}

fn extract_selectable_cpu_call_runtime_expr(
    expr: &NirExpr,
) -> Option<(SelectableCpuCallRuntimeOp, &str, &[NirExpr])> {
    match expr {
        NirExpr::CpuSpawn { callee, args } => Some((
            SelectableCpuCallRuntimeOp::Spawn,
            callee.as_str(),
            args.as_slice(),
        )),
        NirExpr::CpuThreadSpawn { callee, args } => Some((
            SelectableCpuCallRuntimeOp::ThreadSpawn,
            callee.as_str(),
            args.as_slice(),
        )),
        _ => None,
    }
}

fn extract_selectable_cpu_binary_runtime_expr(
    expr: &NirExpr,
) -> Option<(SelectableCpuBinaryRuntimeOp, &NirExpr, &NirExpr)> {
    match expr {
        NirExpr::CpuTimeout { task, limit } => {
            Some((SelectableCpuBinaryRuntimeOp::Timeout, task, limit))
        }
        _ => None,
    }
}

fn extract_binding_name_and_value(stmt: &NirStmt) -> Option<(&String, &NirExpr)> {
    match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            Some((name, value))
        }
        _ => None,
    }
}

fn extract_selectable_cpu_unary_runtime_binding_chain(
    stmts: &[NirStmt],
) -> Option<(String, SelectableCpuUnaryRuntimeOp, &NirExpr)> {
    let first = stmts.first()?;
    let (first_name, first_value) = extract_binding_name_and_value(first)?;
    let (op, input) = extract_selectable_cpu_unary_runtime_expr(first_value)?;

    let mut previous_name = first_name;
    let mut outcome_name = first_name.clone();
    for stmt in &stmts[1..] {
        let (name, value) = extract_binding_name_and_value(stmt)?;
        let NirExpr::Var(var_name) = value else {
            return None;
        };
        if var_name != previous_name {
            return None;
        }
        previous_name = name;
        outcome_name = name.clone();
    }
    Some((outcome_name, op, input))
}

fn extract_selectable_cpu_call_runtime_binding_chain(
    stmts: &[NirStmt],
) -> Option<(String, SelectableCpuCallRuntimeOp, &str, &[NirExpr])> {
    let first = stmts.first()?;
    let (first_name, first_value) = extract_binding_name_and_value(first)?;
    let (op, callee, args) = extract_selectable_cpu_call_runtime_expr(first_value)?;

    let mut previous_name = first_name;
    let mut outcome_name = first_name.clone();
    for stmt in &stmts[1..] {
        let (name, value) = extract_binding_name_and_value(stmt)?;
        let NirExpr::Var(var_name) = value else {
            return None;
        };
        if var_name != previous_name {
            return None;
        }
        previous_name = name;
        outcome_name = name.clone();
    }
    Some((outcome_name, op, callee, args))
}

fn extract_selectable_cpu_binary_runtime_binding_chain(
    stmts: &[NirStmt],
) -> Option<(String, SelectableCpuBinaryRuntimeOp, &NirExpr, &NirExpr)> {
    let first = stmts.first()?;
    let (first_name, first_value) = extract_binding_name_and_value(first)?;
    let (op, lhs, rhs) = extract_selectable_cpu_binary_runtime_expr(first_value)?;

    let mut previous_name = first_name;
    let mut outcome_name = first_name.clone();
    for stmt in &stmts[1..] {
        let (name, value) = extract_binding_name_and_value(stmt)?;
        let NirExpr::Var(var_name) = value else {
            return None;
        };
        if var_name != previous_name {
            return None;
        }
        previous_name = name;
        outcome_name = name.clone();
    }
    Some((outcome_name, op, lhs, rhs))
}

fn extract_selectable_cpu_unary_runtime_return_chain(
    stmts: &[NirStmt],
) -> Option<(SelectableCpuUnaryRuntimeOp, &NirExpr)> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            extract_selectable_cpu_unary_runtime_expr(value)
        }
        _ => {
            let (last_stmt, prefix) = stmts.split_last()?;
            let (op, input, previous_name) = {
                let first = prefix.first()?;
                let (first_name, first_value) = extract_binding_name_and_value(first)?;
                let (op, input) = extract_selectable_cpu_unary_runtime_expr(first_value)?;
                (op, input, first_name)
            };

            let mut previous_name = previous_name;
            for stmt in &prefix[1..] {
                let (name, value) = extract_binding_name_and_value(stmt)?;
                let NirExpr::Var(var_name) = value else {
                    return None;
                };
                if var_name != previous_name {
                    return None;
                }
                previous_name = name;
            }

            match last_stmt {
                NirStmt::Return(Some(NirExpr::Var(var_name)))
                | NirStmt::Expr(NirExpr::Var(var_name))
                    if var_name == previous_name =>
                {
                    Some((op, input))
                }
                _ => None,
            }
        }
    }
}

fn extract_selectable_cpu_call_runtime_return_chain(
    stmts: &[NirStmt],
) -> Option<(SelectableCpuCallRuntimeOp, &str, &[NirExpr])> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            extract_selectable_cpu_call_runtime_expr(value)
        }
        _ => {
            let (last_stmt, prefix) = stmts.split_last()?;
            let (op, callee, args, previous_name) = {
                let first = prefix.first()?;
                let (first_name, first_value) = extract_binding_name_and_value(first)?;
                let (op, callee, args) = extract_selectable_cpu_call_runtime_expr(first_value)?;
                (op, callee, args, first_name)
            };

            let mut previous_name = previous_name;
            for stmt in &prefix[1..] {
                let (name, value) = extract_binding_name_and_value(stmt)?;
                let NirExpr::Var(var_name) = value else {
                    return None;
                };
                if var_name != previous_name {
                    return None;
                }
                previous_name = name;
            }

            match last_stmt {
                NirStmt::Return(Some(NirExpr::Var(var_name)))
                | NirStmt::Expr(NirExpr::Var(var_name))
                    if var_name == previous_name =>
                {
                    Some((op, callee, args))
                }
                _ => None,
            }
        }
    }
}

fn extract_selectable_cpu_binary_runtime_return_chain(
    stmts: &[NirStmt],
) -> Option<(SelectableCpuBinaryRuntimeOp, &NirExpr, &NirExpr)> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            extract_selectable_cpu_binary_runtime_expr(value)
        }
        _ => {
            let (last_stmt, prefix) = stmts.split_last()?;
            let (op, lhs, rhs, previous_name) = {
                let first = prefix.first()?;
                let (first_name, first_value) = extract_binding_name_and_value(first)?;
                let (op, lhs, rhs) = extract_selectable_cpu_binary_runtime_expr(first_value)?;
                (op, lhs, rhs, first_name)
            };

            let mut previous_name = previous_name;
            for stmt in &prefix[1..] {
                let (name, value) = extract_binding_name_and_value(stmt)?;
                let NirExpr::Var(var_name) = value else {
                    return None;
                };
                if var_name != previous_name {
                    return None;
                }
                previous_name = name;
            }

            match last_stmt {
                NirStmt::Return(Some(NirExpr::Var(var_name)))
                | NirStmt::Expr(NirExpr::Var(var_name))
                    if var_name == previous_name =>
                {
                    Some((op, lhs, rhs))
                }
                _ => None,
            }
        }
    }
}

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

fn lower_direct_selectable_runtime_binding(
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

fn lower_direct_selectable_call_runtime_binding(
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

fn lower_direct_selectable_binary_runtime_binding(
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

fn lower_direct_selectable_runtime_return(
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

fn lower_direct_selectable_call_runtime_return(
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

fn lower_direct_selectable_binary_runtime_return(
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

fn is_branch_local_runtime_observer(expr: &NirExpr) -> bool {
    matches!(
        expr,
        NirExpr::CpuTaskCompleted(_)
            | NirExpr::CpuTaskTimedOut(_)
            | NirExpr::CpuTaskCancelled(_)
            | NirExpr::CpuTaskValue(_)
            | NirExpr::CpuMutexValue(_)
    )
}

fn is_branch_local_runtime_consumer(expr: &NirExpr) -> bool {
    matches!(
        expr,
        NirExpr::Await(_)
            | NirExpr::CpuSpawn { .. }
            | NirExpr::CpuThreadSpawn { .. }
            | NirExpr::CpuJoin(_)
            | NirExpr::CpuThreadJoin(_)
            | NirExpr::CpuCancel(_)
            | NirExpr::CpuJoinResult(_)
            | NirExpr::CpuThreadJoinResult(_)
            | NirExpr::CpuMutexNew(_)
            | NirExpr::CpuMutexLock(_)
            | NirExpr::CpuMutexUnlock(_)
            | NirExpr::CpuTimeout { .. }
    )
}

fn expr_contains_conditional_effect_primitive(expr: &NirExpr) -> bool {
    match expr {
        NirExpr::Await(inner) => !matches!(
            inner.as_ref(),
            NirExpr::Call { .. } | NirExpr::MethodCall { .. }
        ),
        _ if is_branch_local_runtime_consumer(expr) => true,
        _ if is_branch_local_runtime_observer(expr) => false,
        NirExpr::Borrow(inner)
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
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRow(inner)
        | NirExpr::KernelCol(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner)
        | NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::KernelArgmax(inner)
        | NirExpr::KernelArgmin(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::FieldAccess { base: inner, .. }
        | NirExpr::KernelReshape { input: inner, .. }
        | NirExpr::KernelBroadcast { input: inner, .. }
        | NirExpr::KernelSort(inner)
        | NirExpr::DataResult { value: inner, .. }
        | NirExpr::ShaderResult { value: inner, .. }
        | NirExpr::KernelResult { value: inner, .. }
        | NirExpr::NetworkResult { value: inner, .. } => {
            expr_contains_conditional_effect_primitive(inner)
        }
        NirExpr::Binary { lhs, rhs, .. }
        | NirExpr::KernelMatmul { lhs, rhs }
        | NirExpr::KernelZip { lhs, rhs, .. }
        | NirExpr::KernelAddBias {
            input: lhs,
            bias: rhs,
        } => {
            expr_contains_conditional_effect_primitive(lhs)
                || expr_contains_conditional_effect_primitive(rhs)
        }
        NirExpr::AllocNode { value, next } => {
            expr_contains_conditional_effect_primitive(value)
                || expr_contains_conditional_effect_primitive(next)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_contains_conditional_effect_primitive(len)
                || expr_contains_conditional_effect_primitive(fill)
        }
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => {
            expr_contains_conditional_effect_primitive(target)
                || expr_contains_conditional_effect_primitive(value)
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            expr_contains_conditional_effect_primitive(buffer)
                || expr_contains_conditional_effect_primitive(index)
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        }
        | NirExpr::DataWriteWindow {
            window: buffer,
            index,
            value,
        } => {
            expr_contains_conditional_effect_primitive(buffer)
                || expr_contains_conditional_effect_primitive(index)
                || expr_contains_conditional_effect_primitive(value)
        }
        NirExpr::Call { args, .. }
        | NirExpr::MethodCall { args, .. }
        | NirExpr::CpuExternCall { args, .. } => args
            .iter()
            .any(|arg| expr_contains_conditional_effect_primitive(arg)),
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_conditional_effect_primitive(value)),
        NirExpr::KernelMap { input, scalar, .. } | NirExpr::KernelMapAxis { input, scalar, .. } => {
            expr_contains_conditional_effect_primitive(input)
                || scalar
                    .as_ref()
                    .is_some_and(|value| expr_contains_conditional_effect_primitive(value))
        }
        NirExpr::KernelElementAt { input, row, col } => {
            expr_contains_conditional_effect_primitive(input)
                || expr_contains_conditional_effect_primitive(row)
                || expr_contains_conditional_effect_primitive(col)
        }
        NirExpr::KernelArgmaxAxis { input, .. }
        | NirExpr::KernelArgminAxis { input, .. }
        | NirExpr::KernelReduceSumAxis { input, .. }
        | NirExpr::KernelReduceMaxAxis { input, .. }
        | NirExpr::KernelReduceMeanAxis { input, .. }
        | NirExpr::KernelSortAxis { input, .. }
        | NirExpr::KernelTopkAxis { input, .. }
        | NirExpr::KernelTopk { input, .. } => expr_contains_conditional_effect_primitive(input),
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_)
        | NirExpr::Instantiate { .. }
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::DataCopyWindow { .. }
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. }
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::ShaderProfileTargetRef { .. }
        | NirExpr::ShaderProfileViewportRef { .. }
        | NirExpr::ShaderProfilePipelineRef { .. }
        | NirExpr::ShaderProfileVertexCountRef { .. }
        | NirExpr::ShaderProfileInstanceCountRef { .. }
        | NirExpr::ShaderProfilePacketColorSlotRef { .. }
        | NirExpr::ShaderProfilePacketSpeedSlotRef { .. }
        | NirExpr::ShaderProfilePacketRadiusSlotRef { .. }
        | NirExpr::ShaderProfilePacketTagRef { .. }
        | NirExpr::ShaderProfileMaterialModeRef { .. }
        | NirExpr::ShaderProfilePassKindRef { .. }
        | NirExpr::ShaderProfilePacketFieldCountRef { .. }
        | NirExpr::DataProfileMarkerRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelTensor { .. } => false,
        _ => false,
    }
}

fn stmt_contains_conditional_effect_primitive(stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value)
        | NirStmt::Return(Some(value)) => expr_contains_conditional_effect_primitive(value),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_conditional_effect_primitive(condition)
                || then_body
                    .iter()
                    .any(stmt_contains_conditional_effect_primitive)
                || else_body
                    .iter()
                    .any(stmt_contains_conditional_effect_primitive)
        }
        NirStmt::While { condition, body } => {
            expr_contains_conditional_effect_primitive(condition)
                || body.iter().any(stmt_contains_conditional_effect_primitive)
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}

pub(super) fn stmts_contain_conditional_effect_primitive(stmts: &[NirStmt]) -> bool {
    stmts.iter().any(stmt_contains_conditional_effect_primitive)
}

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

fn lower_return_if_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            if expr_contains_conditional_effect_primitive(value) {
                return Ok(None);
            }
            Ok(Some(lower_expr(value, state, bindings)?))
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let pure_helpers = state.pure_helpers.clone();
            let Some((name, value)) = extract_pure_branch_binding(binding, &pure_helpers) else {
                return Ok(None);
            };
            let substituted: Vec<NirStmt> = tail
                .iter()
                .map(|stmt| {
                    super::loop_purity::substitute_stmt_bindings(
                        stmt,
                        &[(name.clone(), value.clone())],
                    )
                })
                .collect();
            lower_return_if_chain(&substituted, state, bindings)
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            let condition_name = lower_expr(condition, state, bindings)?;
            let Some(lhs) = lower_return_if_chain(then_body, state, bindings)? else {
                return Ok(None);
            };
            let Some(rhs) = lower_return_if_chain(else_body, state, bindings)? else {
                return Ok(None);
            };
            Ok(Some(lower_select(condition_name, lhs, rhs, state)?))
        }
        _ => Ok(None),
    }
}

pub(super) fn lower_guard_return_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            if expr_contains_conditional_effect_primitive(value) {
                return Ok(None);
            }
            Ok(Some(lower_expr(value, state, bindings)?))
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let pure_helpers = state.pure_helpers.clone();
            let Some((name, value)) = extract_pure_branch_binding(binding, &pure_helpers) else {
                return Ok(None);
            };
            let substituted: Vec<NirStmt> = tail
                .iter()
                .map(|stmt| {
                    super::loop_purity::substitute_stmt_bindings(
                        stmt,
                        &[(name.clone(), value.clone())],
                    )
                })
                .collect();
            lower_guard_return_chain(&substituted, state, bindings)
        }
        [
            NirStmt::If {
                condition,
                then_body,
                else_body,
            },
            tail @ ..,
        ] if else_body.is_empty() => {
            let condition_name = lower_expr(condition, state, bindings)?;
            let Some(lhs) = lower_return_if_chain(then_body, state, bindings)? else {
                return Ok(None);
            };
            let Some(rhs) = lower_guard_return_chain(tail, state, bindings)? else {
                return Ok(None);
            };
            Ok(Some(lower_select(condition_name, lhs, rhs, state)?))
        }
        _ => Ok(None),
    }
}

fn lower_binding_if_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    pure_helpers: &BTreeSet<String>,
) -> Result<Option<(String, String)>, String> {
    match stmts {
        [NirStmt::Let { name, value, .. }] | [NirStmt::Const { name, value, .. }] => {
            if expr_contains_conditional_effect_primitive(value) {
                return Ok(None);
            }
            Ok(Some((name.clone(), lower_expr(value, state, bindings)?)))
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let Some((name, value)) = extract_pure_branch_binding(binding, pure_helpers) else {
                return Ok(None);
            };
            let substituted: Vec<NirStmt> = tail
                .iter()
                .map(|stmt| {
                    super::loop_purity::substitute_stmt_bindings(
                        stmt,
                        &[(name.clone(), value.clone())],
                    )
                })
                .collect();
            lower_binding_if_chain(&substituted, state, bindings, pure_helpers)
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            let condition_name = lower_expr(condition, state, bindings)?;
            let Some((lhs_name, lhs_value)) =
                lower_binding_if_chain(then_body, state, bindings, pure_helpers)?
            else {
                return Ok(None);
            };
            let Some((rhs_name, rhs_value)) =
                lower_binding_if_chain(else_body, state, bindings, pure_helpers)?
            else {
                return Ok(None);
            };
            if lhs_name != rhs_name {
                return Ok(None);
            }
            Ok(Some((
                lhs_name,
                lower_select(condition_name, lhs_value, rhs_value, state)?,
            )))
        }
        _ => Ok(None),
    }
}

fn lower_binding_if_chain_with_shared_context(
    condition_name: &str,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let (shared_prefix, then_core, else_core, shared_suffix) =
        split_shared_branch_context(then_body, else_body);
    if shared_prefix.is_empty() && shared_suffix.is_empty() {
        return Ok(None);
    }

    let pure_helpers = state.pure_helpers.clone();
    let mut local_bindings = bindings.clone();
    super::body_lowering::lower_linear_stmts(shared_prefix, state, &mut local_bindings)?;

    let Some((lhs_name, lhs_value)) =
        lower_binding_if_chain(then_core, state, &local_bindings, &pure_helpers)?
    else {
        return Ok(None);
    };
    let Some((rhs_name, rhs_value)) =
        lower_binding_if_chain(else_core, state, &local_bindings, &pure_helpers)?
    else {
        return Ok(None);
    };
    if lhs_name != rhs_name {
        return Ok(None);
    }

    let selected = lower_select(condition_name.to_owned(), lhs_value, rhs_value, state)?;
    local_bindings.insert(lhs_name.clone(), selected.clone());

    let outcome_name = if shared_suffix.is_empty() {
        lhs_name
    } else {
        let suffix_last_bound =
            super::body_lowering::lower_linear_stmts(shared_suffix, state, &mut local_bindings)?;
        suffix_last_bound.unwrap_or(lhs_name)
    };
    let Some(outcome_value) = local_bindings.get(&outcome_name).cloned() else {
        return Err(format!(
            "minimal nuisc lowering expected shared branch binding `{outcome_name}` to be available after shared suffix lowering"
        ));
    };

    Ok(Some(LoweredIfOutcome::Bind {
        name: outcome_name,
        value: outcome_value,
    }))
}

fn lower_return_if_chain_with_shared_context(
    condition_name: &str,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let (shared_prefix, then_core, else_core, shared_suffix) =
        split_shared_branch_context(then_body, else_body);
    if shared_suffix.is_empty() {
        return Ok(None);
    }

    let pure_helpers = state.pure_helpers.clone();
    let mut local_bindings = bindings.clone();
    super::body_lowering::lower_linear_stmts(shared_prefix, state, &mut local_bindings)?;

    let Some((lhs_name, lhs_value)) =
        lower_binding_if_chain(then_core, state, &local_bindings, &pure_helpers)?
    else {
        return Ok(None);
    };
    let Some((rhs_name, rhs_value)) =
        lower_binding_if_chain(else_core, state, &local_bindings, &pure_helpers)?
    else {
        return Ok(None);
    };
    if lhs_name != rhs_name {
        return Ok(None);
    }

    let selected = lower_select(condition_name.to_owned(), lhs_value, rhs_value, state)?;
    local_bindings.insert(lhs_name, selected);

    let Some(returned) = lower_return_if_chain(shared_suffix, state, &local_bindings)? else {
        return Ok(None);
    };
    Ok(Some(LoweredIfOutcome::Returned(returned)))
}

fn split_shared_branch_context<'a>(
    then_body: &'a [NirStmt],
    else_body: &'a [NirStmt],
) -> (&'a [NirStmt], &'a [NirStmt], &'a [NirStmt], &'a [NirStmt]) {
    let shared_prefix_len = then_body
        .iter()
        .zip(else_body.iter())
        .take_while(|(lhs, rhs)| lhs == rhs)
        .count();

    let then_remaining = &then_body[shared_prefix_len..];
    let else_remaining = &else_body[shared_prefix_len..];

    let max_shared_suffix_len = then_remaining.len().min(else_remaining.len());
    let mut shared_suffix_len = 0usize;
    while shared_suffix_len < max_shared_suffix_len
        && then_remaining[then_remaining.len() - 1 - shared_suffix_len]
            == else_remaining[else_remaining.len() - 1 - shared_suffix_len]
    {
        shared_suffix_len += 1;
    }

    let then_core_end = then_remaining.len().saturating_sub(shared_suffix_len);
    let else_core_end = else_remaining.len().saturating_sub(shared_suffix_len);

    (
        &then_body[..shared_prefix_len],
        &then_remaining[..then_core_end],
        &else_remaining[..else_core_end],
        &then_remaining[then_core_end..],
    )
}
