use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectableCpuUnaryRuntimeOp {
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
    pub(super) fn prefix(self) -> &'static str {
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

    pub(super) fn instruction(self) -> &'static str {
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
pub(super) enum SelectableCpuCallRuntimeOp {
    Spawn,
    ThreadSpawn,
}

impl SelectableCpuCallRuntimeOp {
    pub(super) fn prefix(self) -> &'static str {
        match self {
            Self::Spawn => "cpu_spawn_task",
            Self::ThreadSpawn => "cpu_spawn_thread",
        }
    }

    pub(super) fn instruction(self) -> &'static str {
        match self {
            Self::Spawn => "spawn_task",
            Self::ThreadSpawn => "spawn_thread",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectableCpuBinaryRuntimeOp {
    Timeout,
    ReadyAfter,
}

impl SelectableCpuBinaryRuntimeOp {
    pub(super) fn prefix(self) -> &'static str {
        match self {
            Self::Timeout => "cpu_timeout",
            Self::ReadyAfter => "cpu_ready_after",
        }
    }

    pub(super) fn instruction(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::ReadyAfter => "ready_after",
        }
    }
}

pub(super) fn extract_selectable_cpu_unary_runtime_expr(
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

pub(super) fn extract_selectable_cpu_call_runtime_expr(
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

pub(super) fn extract_selectable_cpu_binary_runtime_expr(
    expr: &NirExpr,
) -> Option<(SelectableCpuBinaryRuntimeOp, &NirExpr, &NirExpr)> {
    match expr {
        NirExpr::CpuTimeout { task, limit } => {
            Some((SelectableCpuBinaryRuntimeOp::Timeout, task, limit))
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            Some((SelectableCpuBinaryRuntimeOp::ReadyAfter, task, delay))
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

pub(super) fn extract_selectable_cpu_unary_runtime_binding_chain(
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

pub(super) fn extract_selectable_cpu_call_runtime_binding_chain(
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

pub(super) fn extract_selectable_cpu_binary_runtime_binding_chain(
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

pub(super) fn extract_selectable_cpu_unary_runtime_return_chain(
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

pub(super) fn extract_selectable_cpu_call_runtime_return_chain(
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

pub(super) fn extract_selectable_cpu_binary_runtime_return_chain(
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
