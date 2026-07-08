use super::*;

#[test]
fn compiles_task_cli_tooling_project() {
    let project = Path::new("../../examples/projects/task/task_cli_tooling_demo");
    nuisc::pipeline::compile_project(project).expect("task cli tooling project should compile");
}

#[test]
fn compiles_task_thread_mutex_project() {
    let project = Path::new("../../examples/projects/task/task_thread_mutex_demo");
    nuisc::pipeline::compile_project(project).expect("task thread/mutex project should compile");
}

#[test]
fn lowers_task_cli_tooling_project_with_timeout_and_host_io_shape() {
    let artifacts = compiled_project("../../examples/projects/task/task_cli_tooling_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuExternCall { callee, .. },
            } if name == "argv_count"
                && ty.render() == "i64"
                && callee == "host_argv_count"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCompleted(_),
                then_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
                    if callee == "emit_completed_cli"
            )
        )
    }));
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "emit_timeout_cli"
    ));

    let emit_completed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "emit_completed_cli")
        .expect("expected emit_completed_cli function");
    assert!(emit_completed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuExternCall { callee, .. },
            } if name == "stdout_code"
                && ty.render() == "i64"
                && callee == "host_stdout_write"
        )
    }));

    let emit_timeout = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "emit_timeout_cli")
        .expect("expected emit_timeout_cli function");
    assert!(emit_timeout.body.iter().any(|stmt| {
        matches!(
        stmt,
        NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, .. },
        } if name == "stderr_code"
            && ty.render() == "i64"
            && callee == "host_stderr_write"
        )
    }));
}

#[test]
fn lowers_task_thread_mutex_project_with_thread_and_lock_shape() {
    let artifacts = compiled_project("../../examples/projects/task/task_thread_mutex_demo");

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_thread_mutex")
        .expect("expected capture_thread_mutex function");
    let mutex_snapshot = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "mutex_snapshot__i64")
        .expect("expected specialized mutex_snapshot helper");
    let join_thread_result = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "join_thread_result__i64")
        .expect("expected specialized join_thread_result helper");
    let join_thread_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "join_thread_value__i64")
        .expect("expected specialized join_thread_value helper");
    let capture_direct_join = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_thread_mutex_direct_join")
        .expect("expected capture_thread_mutex_direct_join function");

    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "first_snapshot"
                && ty.render() == "MutexSnapshot<i64>"
                && callee == "mutex_snapshot__i64"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "second_snapshot"
                && ty.render() == "MutexSnapshot<i64>"
                && callee == "mutex_snapshot__i64"
        )
    }));
    assert!(mutex_snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexLock(_),
            } if name == "guard" && ty.render() == "MutexGuard<i64>"
        )
    }));
    assert!(mutex_snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexLock(_),
            } if name == "guard" && ty.render() == "MutexGuard<i64>"
        )
    }));
    assert!(mutex_snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexValue(_),
            } if name == "value" && ty.render() == "i64"
        )
    }));
    assert!(mutex_snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexUnlock(_),
            } if name == "reopened" && ty.render() == "Mutex<i64>"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuThreadSpawn { callee, .. },
            } if name == "worker" && ty.render() == "Thread<i64>" && callee == "ping"
        )
    }));
    assert!(join_thread_result
        .body
        .iter()
        .any(|stmt| { matches!(stmt, NirStmt::Return(Some(NirExpr::CpuThreadJoinResult(_)))) }));
    assert!(join_thread_value
        .body
        .iter()
        .any(|stmt| { matches!(stmt, NirStmt::Return(Some(NirExpr::CpuThreadJoin(_)))) }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "joined"
                && ty.render() == "TaskResult<i64>"
                && callee == "join_thread_result__i64"
        )
    }));
    assert!(capture_direct_join.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "thread_value"
                && ty.render() == "i64"
                && callee == "join_thread_value__i64"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCompleted(_),
                ..
            }
        )
    }));
}
