use std::path::Path;

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
