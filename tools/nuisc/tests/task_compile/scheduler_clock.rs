use super::*;

#[test]
fn compiles_task_scheduler_observe_project() {
    let project = Path::new("../../examples/projects/task/task_scheduler_observe_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task scheduler observe project should compile");
}

#[test]
fn lowers_task_scheduler_observe_project_with_scheduler_and_timeout_shapes() {
    let artifacts = compiled_project("../../examples/projects/task/task_scheduler_observe_demo");

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_scheduler_project")
        .expect("expected capture_task_scheduler_project function");
    assert!(capture
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuBindCore(0)))));
    assert!(capture.body.iter().any(|stmt| {
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
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTickI64 { .. },
            } if name == "scheduler_tick" && ty.render() == "i64"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuExternCall { callee, .. },
            } if name == "monotonic_ns"
                && ty.render() == "i64"
                && callee == "host_monotonic_time_ns"
        )
    }));
}

#[test]
fn compiles_task_clock_observe_project() {
    let project = Path::new("../../examples/projects/task/task_clock_observe_demo");
    nuisc::pipeline::compile_project(project).expect("task clock observe project should compile");
}

#[test]
fn lowers_task_clock_observe_project_with_clock_host_observer_shapes() {
    let artifacts = compiled_project("../../examples/projects/task/task_clock_observe_demo");

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_clock_project")
        .expect("expected capture_task_clock_project function");
    assert!(capture.body.iter().any(|stmt| {
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
    for (name, callee) in [
        ("global_epoch_ns", "host_clock_epoch_ns"),
        ("monotonic_ns", "host_monotonic_time_ns"),
        ("global_scale_ppm", "host_clock_scale_ppm"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: binding_name,
                    ty: Some(ty),
                    value: NirExpr::CpuExternCall { callee: call, .. },
                } if binding_name == name && ty.render() == "i64" && call == callee
            )
        }));
    }
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTickI64 { .. },
            } if name == "global_tick" && ty.render() == "i64"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCompleted(_),
                then_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Var(name)))] if name == "completed_summary"
            )
        )
    }));
}
