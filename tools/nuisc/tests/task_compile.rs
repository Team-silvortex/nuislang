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
