use std::path::Path;

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
