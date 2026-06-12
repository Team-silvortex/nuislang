use std::path::Path;

use nuis_semantics::model::{NirExpr, NirStmt};

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
}

#[test]
fn compiles_task_recursive_async_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task recursive async project should compile");
}

#[test]
fn compiles_task_mutual_recursive_async_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_mutual_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task mutual recursive async project should compile");
}

#[test]
fn compiles_task_generic_recursive_async_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task generic recursive async project should compile");
}

#[test]
fn lowers_task_generic_recursive_async_project_with_specialized_async_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_recursive_async_demo",
    );

    let specialized = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "bounce__i64")
        .expect("expected specialized async generic recursive helper");
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());

    assert!(
        artifacts
            .yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "async_call"),
        "expected async_call node in compiled task generic recursive project"
    );
    assert!(
        artifacts
            .yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "await"),
        "expected await node in compiled task generic recursive project"
    );
}

#[test]
fn compiles_task_generic_mutual_recursive_async_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_mutual_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task generic mutual recursive async project should compile");
}

#[test]
fn compiles_task_recursive_async_result_family_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_result_family_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async result family project should compile");
}

#[test]
fn compiles_task_recursive_async_payload_alias_hof_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_payload_alias_hof_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async payload alias hof project should compile");
}

#[test]
fn compiles_task_memory_roundtrip_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_roundtrip_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory roundtrip project should compile");
}

#[test]
fn compiles_task_memory_result_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory result branch project should compile");
}

#[test]
fn compiles_task_memory_result_batch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_batch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory result batch project should compile");
}

#[test]
fn compiles_task_memory_session_policy_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_policy_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory session policy project should compile");
}

#[test]
fn lowers_task_memory_session_policy_project_with_task_memory_session_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_policy_demo",
    );

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
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_packet_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory session packet project should compile");
}

#[test]
fn compiles_task_result_policy_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result policy branch project should compile");
}

#[test]
fn lowers_task_result_policy_branch_project_with_branch_selection_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo",
    );

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

#[test]
fn compiles_task_fallback_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task fallback branch project should compile");
}

#[test]
fn lowers_task_fallback_branch_project_with_timeout_fallback_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo",
    );

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
        2
    );
    assert!(matches!(
        select_value.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Int(0))))
    ));

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_fallback")
        .expect("expected capture_task_fallback function");
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "primary_task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuSpawn { callee, .. },
            } if name == "fallback_task" && ty.render() == "Task<i64>" && callee == "fallback"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "primary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "primary_task")
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
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "fallback_task")
        )
    }));
}

#[test]
fn compiles_task_result_family_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_family_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result family branch project should compile");
}

#[test]
fn lowers_task_result_family_branch_project_with_result_family_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_family_branch_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_family")
        .expect("expected capture_task_result_family function");
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "completed_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "timed_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuTimeout { task: inner, .. }
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
                )
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "cancelled_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuCancel(inner)
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
                )
        )
    }));

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
            then_body,
            ..
        }) if matches!(then_body.as_slice(), [NirStmt::Return(Some(NirExpr::Int(1)))])
    ));

    let encode_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_value")
        .expect("expected encode_value function");
    assert!(matches!(
        encode_value.body.first(),
        Some(NirStmt::If {
            then_body,
            ..
        }) if matches!(
            then_body.as_slice(),
            [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
        )
    ));
}

#[test]
fn compiles_task_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task batch branch project should compile");
}

#[test]
fn lowers_task_batch_branch_project_with_batch_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_batch")
        .expect("expected capture_task_batch function");
    for (task_name, callee) in [
        ("alpha_task", "alpha"),
        ("beta_task", "beta"),
        ("gamma_task", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuSpawn { callee: stmt_callee, .. },
                } if name == task_name && ty.render() == "Task<i64>" && stmt_callee == callee
            )
        }));
    }
    for (result_name, task_name) in [
        ("alpha_result", "alpha_task"),
        ("beta_result", "beta_task"),
        ("gamma_result", "gamma_task"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::Var(bound_task) if bound_task == task_name)
            )
        }));
    }
    assert!(matches!(
        capture_batch.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "alpha_completed" && matches!(value, NirExpr::Call { callee, .. } if callee == "encode_completed")
                })
                && fields.iter().any(|(field, value)| {
                    field == "batch_value" && matches!(value, NirExpr::Binary { .. })
                })
    ));
}

#[test]
fn compiles_task_result_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result batch branch project should compile");
}

#[test]
fn lowers_task_result_batch_branch_project_with_result_batch_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_batch")
        .expect("expected capture_task_result_batch function");
    for (result_name, callee) in [
        ("alpha_result", "alpha"),
        ("beta_result", "beta"),
        ("gamma_result", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::CpuSpawn { callee: stmt_callee, .. } if stmt_callee == callee)
            )
        }));
    }
    assert!(matches!(
        capture_batch.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskResultBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "alpha_completed" && matches!(value, NirExpr::Call { callee, .. } if callee == "encode_completed")
                })
                && fields.iter().any(|(field, value)| {
                    field == "batch_value" && matches!(value, NirExpr::Binary { .. })
                })
    ));
}

#[test]
fn compiles_task_windowed_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_windowed_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task windowed batch branch project should compile");
}

#[test]
fn lowers_task_windowed_batch_branch_project_with_windowed_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_windowed_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_batch")
        .expect("expected capture_task_batch function");
    for (task_name, callee) in [
        ("alpha_task", "alpha"),
        ("beta_task", "beta"),
        ("gamma_task", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuSpawn { callee: stmt_callee, .. },
                } if name == task_name && ty.render() == "Task<i64>" && stmt_callee == callee
            )
        }));
    }
    for (result_name, task_name) in [
        ("alpha_result", "alpha_task"),
        ("beta_result", "beta_task"),
        ("gamma_result", "gamma_task"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::Var(bound_task) if bound_task == task_name)
            )
        }));
    }

    let capture_windowed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_windowed_batch")
        .expect("expected capture_task_windowed_batch function");
    assert!(capture_windowed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "summary"
                && ty.render() == "TaskBatchSummary"
                && callee == "capture_task_batch"
        )
    }));
    assert!(matches!(
        capture_windowed.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskWindowedBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "preview_value" && matches!(value, NirExpr::Binary { .. })
                })
                && fields.iter().any(|(field, value)| {
                    field == "final_value"
                        && matches!(
                            value,
                            NirExpr::FieldAccess { field: inner_field, .. } if inner_field == "batch_value"
                        )
                })
    ));
}

#[test]
fn compiles_task_result_windowed_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_windowed_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result windowed batch branch project should compile");
}

#[test]
fn lowers_task_result_windowed_batch_branch_project_with_result_windowed_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_windowed_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_batch")
        .expect("expected capture_task_result_batch function");
    for (result_name, callee) in [
        ("alpha_result", "alpha"),
        ("beta_result", "beta"),
        ("gamma_result", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::CpuSpawn { callee: stmt_callee, .. } if stmt_callee == callee)
            )
        }));
    }

    let encode_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_value")
        .expect("expected encode_value function");
    assert!(matches!(
        encode_value.body.first(),
        Some(NirStmt::If {
            then_body,
            ..
        }) if matches!(
            then_body.as_slice(),
            [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
        )
    ));

    let capture_windowed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_windowed_batch")
        .expect("expected capture_task_result_windowed_batch function");
    assert!(capture_windowed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "summary"
                && ty.render() == "TaskResultBatchSummary"
                && callee == "capture_task_result_batch"
        )
    }));
    assert!(matches!(
        capture_windowed.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskResultWindowedBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "preview_value" && matches!(value, NirExpr::Binary { .. })
                })
                && fields.iter().any(|(field, value)| {
                    field == "final_value"
                        && matches!(
                            value,
                            NirExpr::FieldAccess { field: inner_field, .. } if inner_field == "batch_value"
                        )
                })
    ));
}

#[test]
fn compiles_task_lifecycle_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task lifecycle branch project should compile");
}

#[test]
fn lowers_task_lifecycle_branch_project_with_timeout_branch_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo",
    );

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
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let { name, .. },
                    NirStmt::Print(_),
                    NirStmt::Return(Some(_))
                ] if name == "summary"
            ) && matches!(
                else_body.as_slice(),
                [
                    NirStmt::Let { name, .. },
                    NirStmt::Print(_),
                    NirStmt::Return(Some(_))
                ] if name == "summary"
            )
        )
    }));
}

#[test]
fn compiles_task_httpish_response_packet_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_packet_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish response packet project should compile");
}

#[test]
fn compiles_task_httpish_session_policy_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_session_policy_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish session policy project should compile");
}

#[test]
fn compiles_task_httpish_response_slots_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_slots_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish response slots project should compile");
}

#[test]
fn compiles_task_httpish_header_session_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_header_session_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish header session project should compile");
}
