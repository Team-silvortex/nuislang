use super::*;

#[test]
fn compiles_task_runtime_project() {
    let project = Path::new("../../examples/projects/task/task_runtime_demo");
    nuisc::pipeline::compile_project(project).expect("task runtime project should compile");
}

#[test]
fn lowers_task_runtime_project_with_completed_timeout_and_cancelled_shapes() {
    let artifacts = compiled_project("../../examples/projects/task/task_runtime_demo");

    let capture_lifecycle = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_lifecycle")
        .expect("expected capture_task_lifecycle function");
    assert!(capture_lifecycle.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "completed_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "completed_task")
        )
    }));
    assert!(capture_lifecycle.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "timed_task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(capture_lifecycle.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuCancel(inner),
            } if name == "cancelled_task"
                && ty.render() == "Task<i64>"
                && matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));

    let encode_timed_out = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_timed_out")
        .expect("expected encode_timed_out function");
    assert!(matches!(
        encode_timed_out.body.first(),
        Some(NirStmt::If {
            condition: NirExpr::CpuTaskTimedOut(_),
            ..
        })
    ));

    let encode_cancelled = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_cancelled")
        .expect("expected encode_cancelled function");
    assert!(matches!(
        encode_cancelled.body.first(),
        Some(NirStmt::If {
            condition: NirExpr::CpuTaskCancelled(_),
            ..
        })
    ));
}

#[test]
fn compiles_task_status_observe_project() {
    let project = Path::new("../../examples/projects/task/task_status_observe_demo");
    nuisc::pipeline::compile_project(project).expect("task status observe project should compile");
}

#[test]
fn lowers_task_status_observe_project_with_status_observer_shapes() {
    let artifacts = compiled_project("../../examples/projects/task/task_status_observe_demo");

    for (name, predicate) in [
        ("capture_completed", "completed"),
        ("capture_timed_out", "timed_out"),
        ("capture_cancelled", "cancelled"),
    ] {
        let function = artifacts
            .nir
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("expected {name} function"));
        match predicate {
            "completed" => assert!(matches!(
                function.body.first(),
                Some(NirStmt::If {
                    condition: NirExpr::CpuTaskCompleted(_),
                    ..
                })
            )),
            "timed_out" => assert!(matches!(
                function.body.first(),
                Some(NirStmt::If {
                    condition: NirExpr::CpuTaskTimedOut(_),
                    ..
                })
            )),
            "cancelled" => assert!(matches!(
                function.body.first(),
                Some(NirStmt::If {
                    condition: NirExpr::CpuTaskCancelled(_),
                    ..
                })
            )),
            _ => unreachable!(),
        }
    }

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
                value: NirExpr::CpuJoinResult(_),
            } if name == "completed_result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "timed_result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "cancelled_result" && ty.render() == "TaskResult<i64>"
        )
    }));
}

#[test]
fn compiles_task_completed_observe_project() {
    let project = Path::new("../../examples/projects/task/task_completed_observe_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task completed observe project should compile");
}

#[test]
fn lowers_task_completed_observe_project_with_join_result_and_task_value_shape() {
    let artifacts = compiled_project("../../examples/projects/task/task_completed_observe_demo");

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
                value: NirExpr::CpuJoinResult(task),
            } if name == "result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "task")
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
                [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
            )
        )
    }));
}

#[test]
fn compiles_task_compare_observe_project() {
    let project = Path::new("../../examples/projects/task/task_compare_observe_demo");
    nuisc::pipeline::compile_project(project).expect("task compare observe project should compile");
}

#[test]
fn lowers_task_compare_observe_project_with_direct_and_observed_join_shapes() {
    let artifacts = compiled_project("../../examples/projects/task/task_compare_observe_demo");

    let capture_direct = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_direct_value")
        .expect("expected capture_direct_value function");
    assert!(capture_direct.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Return(Some(NirExpr::CpuJoin(task)))
                if matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "task")
        )
    }));

    let capture_observed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_observed_value")
        .expect("expected capture_observed_value function");
    assert!(capture_observed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(capture_observed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCompleted(_),
                then_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
            )
        )
    }));
}
