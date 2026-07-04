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
        "argv runtime",
        "../../examples/projects/tooling/argv_runtime_demo",
    ),
    (
        "env runtime",
        "../../examples/projects/tooling/env_runtime_demo",
    ),
    (
        "diagnostic runtime",
        "../../examples/projects/tooling/diagnostic_runtime_demo",
    ),
    (
        "error runtime",
        "../../examples/projects/tooling/error_runtime_demo",
    ),
    (
        "json runtime",
        "../../examples/projects/tooling/json_runtime_demo",
    ),
    (
        "result runtime",
        "../../examples/projects/tooling/result_runtime_demo",
    ),
    (
        "result enum runtime",
        "../../examples/projects/tooling/result_enum_runtime_demo",
    ),
    (
        "result diagnostic",
        "../../examples/projects/tooling/result_diagnostic_demo",
    ),
    (
        "text format runtime",
        "../../examples/projects/tooling/text_format_runtime_demo",
    ),
    (
        "time runtime",
        "../../examples/projects/tooling/time_runtime_demo",
    ),
    (
        "io runtime",
        "../../examples/projects/tooling/io_runtime_demo",
    ),
    (
        "terminal io",
        "../../examples/projects/tooling/terminal_io_demo",
    ),
    (
        "stdin runtime",
        "../../examples/projects/tooling/stdin_runtime_demo",
    ),
    (
        "tty runtime",
        "../../examples/projects/tooling/tty_runtime_demo",
    ),
    (
        "input runtime",
        "../../examples/projects/tooling/input_runtime_demo",
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
        "file read",
        "../../examples/projects/tooling/file_read_demo",
    ),
    (
        "file write",
        "../../examples/projects/tooling/file_write_demo",
    ),
    (
        "file copy",
        "../../examples/projects/tooling/file_copy_demo",
    ),
    (
        "file output",
        "../../examples/projects/tooling/file_output_demo",
    ),
    (
        "file roundtrip",
        "../../examples/projects/tooling/file_roundtrip_demo",
    ),
    (
        "directory create",
        "../../examples/projects/tooling/directory_create_demo",
    ),
    (
        "directory remove",
        "../../examples/projects/tooling/directory_remove_demo",
    ),
    (
        "filesystem report",
        "../../examples/projects/tooling/filesystem_report_demo",
    ),
    (
        "filesystem report file",
        "../../examples/projects/tooling/filesystem_report_file_demo",
    ),
    (
        "filesystem io report",
        "../../examples/projects/tooling/filesystem_io_report_demo",
    ),
    (
        "path analysis",
        "../../examples/projects/tooling/path_analysis_demo",
    ),
    (
        "path copy remove",
        "../../examples/projects/tooling/path_copy_remove_demo",
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
