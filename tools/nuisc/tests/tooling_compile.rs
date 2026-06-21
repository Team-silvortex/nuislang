use std::path::Path;

#[test]
fn compiles_cli_runtime_tooling_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo");
    nuisc::pipeline::compile_project(project).expect("cli runtime tooling project should compile");
}

#[test]
fn compiles_cli_session_tooling_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_session_demo");
    nuisc::pipeline::compile_project(project).expect("cli session tooling project should compile");
}

#[test]
fn compiles_cli_report_session_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_report_session_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli report session tooling project should compile");
}

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

#[test]
fn compiles_cli_compile_workflow_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_compile_workflow_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli compile workflow tooling project should compile");
}

#[test]
fn compiles_cli_build_pipeline_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_build_pipeline_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli build pipeline tooling project should compile");
}

#[test]
fn compiles_cli_workflow_automation_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_workflow_automation_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli workflow automation tooling project should compile");
}

#[test]
fn compiles_cli_project_build_report_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_project_build_report_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli project build report tooling project should compile");
}

#[test]
fn compiles_benchmark_report_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/benchmark_report_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("benchmark report tooling project should compile");
}

#[test]
fn compiles_benchmark_report_count_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/benchmark_report_count_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("benchmark report count tooling project should compile");
}

#[test]
fn compiles_benchmark_report_file_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/benchmark_report_file_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("benchmark report file tooling project should compile");
}

#[test]
fn compiles_cli_pgm_info_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_info_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli pgm info tooling project should compile");
}

#[test]
fn compiles_cli_pgm_invert_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_invert_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli pgm invert tooling project should compile");
}

#[test]
fn compiles_cli_pgm_threshold_tooling_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_threshold_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("cli pgm threshold tooling project should compile");
}
