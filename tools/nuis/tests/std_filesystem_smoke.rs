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

fn run_nsld(args: &[&str]) -> std::process::Output {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_nsld").map(PathBuf::from) {
        return Command::new(path)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run nsld {:?}: {error}", args));
    }
    let fallback = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/nsld");
    if fallback.exists() {
        return Command::new(fallback)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run nsld {:?}: {error}", args));
    }
    Command::new("cargo")
        .args(["run", "-q", "-p", "nsld", "--"])
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsld through cargo {:?}: {error}", args))
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

fn assert_output_files_have_schema(output_dir: &Path, files: &[(&str, &str)], context: &str) {
    for (file, schema) in files {
        let path = output_dir.join(file);
        assert!(
            path.exists(),
            "expected {context} output file `{file}` in {}",
            output_dir.display()
        );
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let expected = format!("schema = \"{schema}\"");
        assert!(
            source.contains(&expected),
            "expected {context} output file `{file}` to contain `{expected}`\n{source}"
        );
    }
}

const STD_TOOLING_LIGHT_SMOKE_PROJECTS: &[(&str, &str)] = &[
    (
        "file_read",
        "../../examples/projects/tooling/file_read_demo",
    ),
    (
        "file_write",
        "../../examples/projects/tooling/file_write_demo",
    ),
    (
        "file_roundtrip",
        "../../examples/projects/tooling/file_roundtrip_demo",
    ),
    (
        "directory_create",
        "../../examples/projects/tooling/directory_create_demo",
    ),
    (
        "path_analysis",
        "../../examples/projects/tooling/path_analysis_demo",
    ),
    (
        "path_safety",
        "../../examples/projects/tooling/path_safety_demo",
    ),
    (
        "text_pipeline",
        "../../examples/projects/tooling/text_pipeline_demo",
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
        "path_safety",
        "../../examples/projects/tooling/path_safety_demo",
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
fn std_path_safety_release_check_reports_nsld_drive_command_set() {
    let output_dir = temp_dir("path_safety_release_check");
    let output_dir_text = output_dir.display().to_string();
    let release_check = run_nuis(&[
        "release-check",
        "../../examples/projects/tooling/path_safety_demo",
        &output_dir_text,
    ]);
    assert_success(&release_check, "nuis release-check std path safety smoke");
    let stdout = String::from_utf8_lossy(&release_check.stdout);
    assert!(
        stdout.contains("release-check: nsld-drive"),
        "release-check output did not report nsld-drive block\n{stdout}"
    );
    assert!(
        stdout.contains("protocol: nsld-drive-command-set-v1"),
        "release-check output did not report nsld drive command protocol\n{stdout}"
    );
    assert!(
        stdout.contains("recommended_first_json_command: nsld drive "),
        "release-check output did not report recommended dry-run json command\n{stdout}"
    );
    assert!(
        stdout.contains("dry_run_mutates_artifacts: false"),
        "release-check output did not mark dry-run as non-mutating\n{stdout}"
    );
    assert!(
        stdout.contains("apply_next_mutates_artifacts: true"),
        "release-check output did not mark apply-next as mutating\n{stdout}"
    );
    assert!(
        stdout.contains("release-check: ok"),
        "release-check output did not finish cleanly\n{stdout}"
    );
    assert!(
        output_dir.join("nuis.build.manifest.toml").exists(),
        "expected build manifest for std path safety release-check"
    );

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_path_text = manifest_path.display().to_string();
    let nsld_drive = run_nsld(&["drive", &manifest_path_text, "--json"]);
    assert_success(&nsld_drive, "nsld drive dry-run std path safety smoke");
    let nsld_stdout = String::from_utf8_lossy(&nsld_drive.stdout);
    assert!(
        nsld_stdout.contains("\"kind\":\"nsld_drive_dry_run\""),
        "nsld drive did not report dry-run json\n{nsld_stdout}"
    );
    assert!(
        nsld_stdout.contains("\"would_execute\":true"),
        "nsld drive dry-run did not find an executable next action\n{nsld_stdout}"
    );
    assert!(
        nsld_stdout.contains("\"mutates_artifacts\":false"),
        "nsld drive dry-run reported mutation\n{nsld_stdout}"
    );
    assert!(
        nsld_stdout.contains("\"command_id\":\"emit-inputs\""),
        "nsld drive dry-run did not select the first artifact-chain action\n{nsld_stdout}"
    );

    let nsld_apply = run_nsld(&["drive", &manifest_path_text, "--apply", "--json"]);
    assert_success(&nsld_apply, "nsld drive apply std path safety smoke");
    let nsld_apply_stdout = String::from_utf8_lossy(&nsld_apply.stdout);
    assert!(
        nsld_apply_stdout.contains("\"kind\":\"nsld_drive_apply\""),
        "nsld drive apply did not report apply json\n{nsld_apply_stdout}"
    );
    assert!(
        nsld_apply_stdout.contains("\"applied\":true"),
        "nsld drive apply did not apply the first action\n{nsld_apply_stdout}"
    );
    assert!(
        nsld_apply_stdout.contains("\"mutates_artifacts\":true"),
        "nsld drive apply did not report artifact mutation\n{nsld_apply_stdout}"
    );
    assert!(
        nsld_apply_stdout.contains("\"command_id\":\"emit-inputs\""),
        "nsld drive apply did not apply emit-inputs\n{nsld_apply_stdout}"
    );
    assert!(
        output_dir.join("nuis.nsld.link-inputs.toml").exists(),
        "expected nsld link-inputs stage after apply"
    );

    let nsld_until_clean = run_nsld(&[
        "drive",
        &manifest_path_text,
        "--apply",
        "--until-clean",
        "--json",
    ]);
    assert_success(
        &nsld_until_clean,
        "nsld drive until-clean std path safety smoke",
    );
    let nsld_until_clean_stdout = String::from_utf8_lossy(&nsld_until_clean.stdout);
    assert!(
        nsld_until_clean_stdout.contains("\"kind\":\"nsld_drive_until_clean\""),
        "nsld drive until-clean did not report until-clean json\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout.contains("\"mutates_artifacts\":true"),
        "nsld drive until-clean did not report artifact mutation\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout.contains("\"applied_steps\":7"),
        "nsld drive until-clean did not apply the expected remaining stage count\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout.contains("\"stop_reason\":\"repeated-next-action\""),
        "nsld drive until-clean did not stop at the expected repeated finalization boundary\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout.contains("\"stop_command_id\":\"emit-final-executable-pipeline\""),
        "nsld drive until-clean did not report the final pipeline stop command\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout
            .contains("\"stop_action_reason\":\"first missing optional artifact stage `final-executable-output`\""),
        "nsld drive until-clean did not explain the final executable output boundary\n{nsld_until_clean_stdout}"
    );
    assert_output_files_have_schema(
        &output_dir,
        &[
            (
                "nuis.nsld.link-inputs.toml",
                "nuis-nsld-link-input-table-v1",
            ),
            ("nuis.nsld.link-units.toml", "nuis-nsld-link-unit-table-v1"),
            ("nuis.nsld.link-bundle.toml", "nuis-nsld-link-bundle-v1"),
            ("nuis.nsld.assemble-plan.toml", "nuis-nsld-assemble-plan-v1"),
            (
                "nuis.nsld.section-manifest.toml",
                "nuis-nsld-section-manifest-v1",
            ),
            ("nuis.nsld.object-plan.toml", "nuis-nsld-object-plan-v1"),
            (
                "nuis.nsld.object-writer-input.toml",
                "nuis-nsld-object-writer-input-v1",
            ),
            (
                "nuis.nsld.final-executable-pipeline.toml",
                "nuis-nsld-final-executable-pipeline-v1",
            ),
        ],
        "nsld until-clean",
    );
}

#[test]
#[ignore = "full std tooling smoke builds/runs every representative std filesystem/text/io/benchmark project"]
fn std_tooling_full_project_smokes_build_doctor_and_run() {
    assert_std_tooling_project_smoke(STD_TOOLING_FULL_SMOKE_PROJECTS);
}
