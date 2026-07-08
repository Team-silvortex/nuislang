use super::*;

#[test]
fn compiles_task_httpish_response_packet_project() {
    let project = Path::new("../../examples/projects/task/task_httpish_response_packet_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task httpish response packet project should compile");
}

#[test]
fn compiles_task_httpish_session_policy_project() {
    let project = Path::new("../../examples/projects/task/task_httpish_session_policy_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task httpish session policy project should compile");
}

#[test]
fn compiles_task_httpish_response_slots_project() {
    let project = Path::new("../../examples/projects/task/task_httpish_response_slots_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task httpish response slots project should compile");
}

#[test]
fn compiles_task_httpish_header_session_project() {
    let project = Path::new("../../examples/projects/task/task_httpish_header_session_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task httpish header session project should compile");
}
