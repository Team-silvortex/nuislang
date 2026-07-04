use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_std_tooling_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {:?}: {error}", args))
}

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

const STD_TOOLING_LIGHT_SMOKE_PROJECTS: &[(&str, &str)] = &[
    (
        "file_read",
        "../../examples/projects/tooling/file_read_demo",
    ),
    (
        "path_analysis",
        "../../examples/projects/tooling/path_analysis_demo",
    ),
    (
        "text_pipeline",
        "../../examples/projects/tooling/text_pipeline_demo",
    ),
    (
        "argv_runtime",
        "../../examples/projects/tooling/argv_runtime_demo",
    ),
];

const STD_TOOLING_FULL_SMOKE_PROJECTS: &[(&str, &str)] = &[
    (
        "file_read",
        "../../examples/projects/tooling/file_read_demo",
    ),
    (
        "directory_remove",
        "../../examples/projects/tooling/directory_remove_demo",
    ),
    (
        "filesystem_report_file",
        "../../examples/projects/tooling/filesystem_report_file_demo",
    ),
    (
        "path_analysis",
        "../../examples/projects/tooling/path_analysis_demo",
    ),
    (
        "path_copy_remove",
        "../../examples/projects/tooling/path_copy_remove_demo",
    ),
    (
        "text_pipeline",
        "../../examples/projects/tooling/text_pipeline_demo",
    ),
    (
        "io_report",
        "../../examples/projects/tooling/io_report_demo",
    ),
    (
        "benchmark_report_file",
        "../../examples/projects/tooling/benchmark_report_file_demo",
    ),
    (
        "hetero_proxy_benchmark",
        "../../examples/projects/tooling/hetero_proxy_benchmark_demo",
    ),
    (
        "argv_runtime",
        "../../examples/projects/tooling/argv_runtime_demo",
    ),
    (
        "env_runtime",
        "../../examples/projects/tooling/env_runtime_demo",
    ),
    (
        "error_runtime",
        "../../examples/projects/tooling/error_runtime_demo",
    ),
    (
        "time_runtime",
        "../../examples/projects/tooling/time_runtime_demo",
    ),
    (
        "diagnostic_runtime",
        "../../examples/projects/tooling/diagnostic_runtime_demo",
    ),
    (
        "json_runtime",
        "../../examples/projects/tooling/json_runtime_demo",
    ),
    (
        "result_runtime",
        "../../examples/projects/tooling/result_runtime_demo",
    ),
    (
        "result_diagnostic",
        "../../examples/projects/tooling/result_diagnostic_demo",
    ),
    (
        "result_enum_runtime",
        "../../examples/projects/tooling/result_enum_runtime_demo",
    ),
    (
        "net_result_enum",
        "../../examples/projects/domains/net_result_enum_recipe_demo",
    ),
    (
        "text_format_runtime",
        "../../examples/projects/tooling/text_format_runtime_demo",
    ),
    (
        "text_report_builder",
        "../../examples/projects/tooling/text_report_builder_demo",
    ),
    (
        "text_report_json",
        "../../examples/projects/tooling/text_report_json_demo",
    ),
    (
        "filesystem_io_report",
        "../../examples/projects/tooling/filesystem_io_report_demo",
    ),
    (
        "host_text_runtime",
        "../../examples/projects/tooling/host_text_runtime_demo",
    ),
    (
        "io_runtime",
        "../../examples/projects/tooling/io_runtime_demo",
    ),
    (
        "terminal_io",
        "../../examples/projects/tooling/terminal_io_demo",
    ),
    (
        "stdin_runtime",
        "../../examples/projects/tooling/stdin_runtime_demo",
    ),
    (
        "tty_runtime",
        "../../examples/projects/tooling/tty_runtime_demo",
    ),
];

fn assert_std_tooling_project_smoke(projects: &[(&str, &str)]) {
    for (label, project) in projects {
        let output_dir = temp_dir(label);
        let output_dir_text = output_dir.display().to_string();

        let build = run_nuis(&["build", project, &output_dir_text]);
        assert_success(&build, "nuis build std tooling smoke");

        let doctor = run_nuis(&["artifact-doctor", &output_dir_text]);
        assert_success(&doctor, "nuis artifact-doctor std tooling smoke");
        let doctor_stdout = String::from_utf8_lossy(&doctor.stdout);
        assert!(
            doctor_stdout.contains("artifact_diagnostic_code: ready_to_run"),
            "artifact-doctor did not report ready_to_run for {label}\n{doctor_stdout}"
        );

        let run = run_nuis(&["run-artifact", &output_dir_text]);
        assert_success(&run, "nuis run-artifact std tooling smoke");
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            run_stdout.contains("exit_status: 0"),
            "run-artifact did not report exit_status: 0 for {label}\n{run_stdout}"
        );

        assert!(
            Path::new(&output_dir_text)
                .join("nuis.build.manifest.toml")
                .exists(),
            "expected build manifest for {label}"
        );
    }
}

#[test]
fn std_tooling_light_project_smokes_build_doctor_and_run() {
    assert_std_tooling_project_smoke(STD_TOOLING_LIGHT_SMOKE_PROJECTS);
}

#[test]
#[ignore = "full std tooling smoke builds/runs every representative std filesystem/text/io/benchmark project"]
fn std_tooling_full_project_smokes_build_doctor_and_run() {
    assert_std_tooling_project_smoke(STD_TOOLING_FULL_SMOKE_PROJECTS);
}
