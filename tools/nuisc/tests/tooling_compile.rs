use std::path::Path;

#[test]
fn compiles_command_runtime_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("command runtime tooling project should compile");
}

#[test]
fn compiles_subprocess_runtime_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/subprocess_runtime_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("subprocess runtime tooling project should compile");
}

#[test]
fn compiles_workflow_runtime_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("workflow runtime tooling project should compile");
}

#[test]
fn compiles_native_artifact_closure_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/native_artifact_closure_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("native artifact closure tooling project should compile");
}
