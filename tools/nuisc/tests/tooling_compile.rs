use std::path::Path;

const TOOLING_PROJECTS: &[(&str, &str)] = &[
    (
        "cli runtime",
        "../../examples/projects/tooling/cli_runtime_demo",
    ),
    ("cli cat", "../../examples/projects/tooling/cli_cat_demo"),
    ("cli wc", "../../examples/projects/tooling/cli_wc_demo"),
    (
        "cli session",
        "../../examples/projects/tooling/cli_session_demo",
    ),
    (
        "cli report session",
        "../../examples/projects/tooling/cli_report_session_demo",
    ),
    (
        "command runtime",
        "../../examples/projects/tooling/command_runtime_demo",
    ),
    (
        "subprocess runtime",
        "../../examples/projects/tooling/subprocess_runtime_demo",
    ),
    (
        "workflow runtime",
        "../../examples/projects/tooling/workflow_runtime_demo",
    ),
    (
        "native artifact closure",
        "../../examples/projects/tooling/native_artifact_closure_demo",
    ),
    (
        "cli compile workflow",
        "../../examples/projects/tooling/cli_compile_workflow_demo",
    ),
    (
        "cli build pipeline",
        "../../examples/projects/tooling/cli_build_pipeline_demo",
    ),
    (
        "cli workflow automation",
        "../../examples/projects/tooling/cli_workflow_automation_demo",
    ),
    (
        "cli project build report",
        "../../examples/projects/tooling/cli_project_build_report_demo",
    ),
    (
        "benchmark report",
        "../../examples/projects/tooling/benchmark_report_demo",
    ),
    (
        "benchmark report count",
        "../../examples/projects/tooling/benchmark_report_count_demo",
    ),
    (
        "benchmark report file",
        "../../examples/projects/tooling/benchmark_report_file_demo",
    ),
    (
        "hetero proxy benchmark",
        "../../examples/projects/tooling/hetero_proxy_benchmark_demo",
    ),
    (
        "host text runtime",
        "../../examples/projects/tooling/host_text_runtime_demo",
    ),
    (
        "text pipeline",
        "../../examples/projects/tooling/text_pipeline_demo",
    ),
    (
        "text report builder",
        "../../examples/projects/tooling/text_report_builder_demo",
    ),
    (
        "text report json",
        "../../examples/projects/tooling/text_report_json_demo",
    ),
    (
        "cli pgm info",
        "../../examples/projects/tooling/cli_pgm_info_demo",
    ),
    (
        "cli pgm invert",
        "../../examples/projects/tooling/cli_pgm_invert_demo",
    ),
    (
        "cli pgm threshold",
        "../../examples/projects/tooling/cli_pgm_threshold_demo",
    ),
];

#[test]
fn compiles_tooling_projects() {
    for (label, path) in TOOLING_PROJECTS {
        nuisc::pipeline::compile_project(Path::new(path))
            .unwrap_or_else(|error| panic!("{label} tooling project should compile: {error}"));
    }
}
