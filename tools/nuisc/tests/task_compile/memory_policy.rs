use super::*;

#[test]
fn compiles_task_memory_roundtrip_project() {
    let project = Path::new("../../examples/projects/task/task_memory_roundtrip_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task memory roundtrip project should compile");
}

#[test]
fn compiles_task_memory_result_branch_project() {
    let project = Path::new("../../examples/projects/task/task_memory_result_branch_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task memory result branch project should compile");
}

#[test]
fn compiles_task_memory_result_batch_project() {
    let project = Path::new("../../examples/projects/task/task_memory_result_batch_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task memory result batch project should compile");
}

#[test]
fn compiles_task_memory_session_policy_project() {
    let project = Path::new("../../examples/projects/task/task_memory_session_policy_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task memory session policy project should compile");
}

#[test]
fn lowers_task_memory_session_policy_project_with_task_memory_session_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_memory_session_policy_demo");

    let capture_session = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_memory_session")
        .expect("expected capture_task_memory_session function");
    assert!(capture_session.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "primary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuSpawn { callee, .. } if callee == "primary"
                )
        )
    }));
    assert!(capture_session.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "secondary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuSpawn { callee, .. } if callee == "secondary"
                )
        )
    }));
    assert!(capture_session.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "fallback_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuTimeout { task: inner, .. }
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
                )
        )
    }));
    let stage_session = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "stage_session_value")
        .expect("expected stage_session_value function");
    assert!(
        stage_session.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::AllocBuffer { .. },
                } if name == "scratch"
                    && ty.name == "Buffer"
                    && ty.is_ref
            )
        }),
        "expected alloc_buffer staging let: {:?}",
        stage_session.body,
    );
    assert!(
        stage_session
            .body
            .iter()
            .any(|stmt| { matches!(stmt, NirStmt::Expr(NirExpr::StoreAt { .. })) }),
        "expected store_at staging expr: {:?}",
        stage_session.body,
    );
    assert!(
        stage_session
            .body
            .iter()
            .any(|stmt| { matches!(stmt, NirStmt::Expr(NirExpr::Free(_))) }),
        "expected free staging expr: {:?}",
        stage_session.body,
    );
}

#[test]
fn compiles_task_memory_session_packet_project() {
    let project = Path::new("../../examples/projects/task/task_memory_session_packet_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task memory session packet project should compile");
}

#[test]
fn compiles_task_result_policy_branch_project() {
    let project = Path::new("../../examples/projects/task/task_result_policy_branch_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task result policy branch project should compile");
}

#[test]
fn compiles_task_result_enum_project() {
    let project = Path::new("../../examples/projects/task/task_result_enum_demo");
    nuisc::pipeline::compile_project(project).expect("task result enum project should compile");
}

#[test]
fn lowers_task_result_policy_branch_project_with_branch_selection_shape() {
    let artifacts = compiled_project("../../examples/projects/task/task_result_policy_branch_demo");

    let select_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "select_value")
        .expect("expected select_value function");
    assert_eq!(
        select_value
            .body
            .iter()
            .filter(|stmt| matches!(stmt, NirStmt::If { .. }))
            .count(),
        3
    );
    assert!(matches!(
        select_value.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Int(0))))
    ));

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_policy")
        .expect("expected capture_task_result_policy function");
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "primary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "secondary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "secondary")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "fallback_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuTimeout { task: inner, .. }
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
                )
        )
    }));
}
