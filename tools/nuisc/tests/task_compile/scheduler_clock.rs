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
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                if fields.iter().any(|(field, value)| {
                    field == "global_domain_id" && matches!(value, NirExpr::Binary { .. })
                }) && fields.iter().any(|(field, value)| {
                    field == "global_epoch_ns"
                        && matches!(
                            value,
                            NirExpr::CpuExternCall { callee, .. } if callee == "host_clock_epoch_ns"
                        )
                }) && fields.iter().any(|(field, value)| {
                    field == "monotonic_ns"
                        && matches!(
                            value,
                            NirExpr::CpuExternCall { callee, .. } if callee == "host_monotonic_time_ns"
                        )
                }) && fields.iter().any(|(field, value)| {
                    field == "global_tick" && matches!(value, NirExpr::CpuTickI64 { .. })
                }) && fields.iter().any(|(field, value)| {
                    field == "global_scale_ppm"
                        && matches!(
                            value,
                            NirExpr::CpuExternCall { callee, .. } if callee == "host_clock_scale_ppm"
                        )
                })
        )
    }));
}
