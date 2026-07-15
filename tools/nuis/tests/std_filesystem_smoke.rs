use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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

fn run_binary_with_stdin(binary: &Path, stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("failed to spawn {}: {error}", binary.display()));
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin)
        .unwrap_or_else(|error| panic!("failed to write stdin for {}: {error}", binary.display()));
    child
        .wait_with_output()
        .unwrap_or_else(|error| panic!("failed to wait for {}: {error}", binary.display()))
}

fn run_binary_with_args(binary: &Path, args: &[&str]) -> std::process::Output {
    Command::new(binary)
        .args(args)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "failed to run {} with args {:?}: {error}",
                binary.display(),
                args
            )
        })
}

fn assert_file_contains(path: &Path, needle: &str, context: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        source.contains(needle),
        "expected {context} file {} to contain `{needle}`\n{source}",
        path.display()
    );
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
        "stdin_runtime",
        "../../examples/projects/tooling/stdin_runtime_demo",
    ),
    (
        "filesystem_io_report",
        "../../examples/projects/tooling/filesystem_io_report_demo",
    ),
    (
        "cli_report_file",
        "../../examples/projects/tooling/cli_report_file_demo",
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
        "cli_report_file",
        "../../examples/projects/tooling/cli_report_file_demo",
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

fn assert_official_galaxy_hetero_build(
    label: &str,
    project: &str,
    domain: &str,
    yir_needles: &[&str],
    sidecar_needles: &[&str],
    payload_needles: &[&str],
) {
    let output_dir = temp_dir(label);
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&["build", project, &output_dir_text]);
    assert_success(&build, "nuis build official galaxy hetero smoke");

    let yir_path = output_dir.join(format!("{label}.yir"));
    for needle in yir_needles {
        assert_file_contains(&yir_path, needle, "official galaxy hetero YIR");
    }

    assert_file_contains(
        &output_dir.join(format!("nuis.domain.{domain}.artifact.toml")),
        "schema = \"nuis-domain-build-unit-v1\"",
        "official galaxy hetero artifact",
    );
    assert_file_contains(
        &output_dir.join(format!("nuis.domain.{domain}.payload.toml")),
        "schema = \"nuis-domain-build-payload-v1\"",
        "official galaxy hetero payload",
    );
    assert_file_contains(
        &output_dir.join("nuis.hetero-calculate.plan.toml"),
        "schema = \"nuis-hetero-calculate-link-plan-v1\"",
        "official galaxy hetero plan",
    );

    let sidecar_path = output_dir.join(format!("nuis.domain.{domain}.lowering.ir.txt"));
    for needle in sidecar_needles {
        assert_file_contains(&sidecar_path, needle, "official galaxy hetero sidecar");
    }
    let payload_path = output_dir.join(format!("nuis.domain.{domain}.payload.toml"));
    for needle in payload_needles {
        assert_file_contains(&payload_path, needle, "official galaxy hetero payload");
    }
    assert_file_contains(
        &output_dir.join("nuis.hetero-calculate.plan.toml"),
        &format!("domain_family = \"{domain}\""),
        "official galaxy hetero plan",
    );
}

#[test]
fn std_tooling_light_project_smokes_build_doctor_and_run() {
    assert_std_tooling_project_smoke(STD_TOOLING_LIGHT_SMOKE_PROJECTS);
}

#[test]
fn official_galaxy_hetero_projects_emit_shader_and_kernel_artifacts() {
    assert_official_galaxy_hetero_build(
        "pixelmagic_pipeline_demo",
        "../../examples/projects/domains/pixelmagic_pipeline_demo",
        "shader",
        &[
            "shader.begin_pass",
            "shader.draw_instanced",
            "shader.inline_wgsl",
            "PixelMagicContracts.shader_pipeline_total",
        ],
        &[
            "shader_stage_model = \"metal-render-pipeline\"",
            "lowering_capabilities",
            "pipeline_lowering = \"metal-render-pipeline-state\"",
            "execution_route = \"unified-render-graph\"",
        ],
        &[
            "backend_family = \"metal\"",
            "target_device = \"apple-silicon-gpu\"",
            "shader.inline_wgsl",
        ],
    );

    assert_official_galaxy_hetero_build(
        "witsage_kernel_demo",
        "../../examples/projects/domains/witsage_kernel_demo",
        "kernel",
        &[
            "kernel.tensor",
            "kernel.reduce_mean_axis",
            "kernel.topk_axis",
            "WitSageContracts.kernel_pipeline_total",
        ],
        &[
            "kernel_ir = \"coreml-program\"",
            "kernel_entry_model = \"mlmodelc-function\"",
            "tensor_lowering = \"ranked-tensor-graph\"",
        ],
        &[
            "backend_family = \"coreml\"",
            "target_device = \"apple-ane\"",
            "kernel.reduce_mean_axis",
        ],
    );
}

#[test]
fn std_tooling_observable_cli_smoke_checks_reports_and_stdin() {
    let report_output_dir = temp_dir("filesystem_io_report_observable");
    let report_output_dir_text = report_output_dir.display().to_string();
    let report_build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/filesystem_io_report_demo",
        &report_output_dir_text,
    ]);
    assert_success(
        &report_build,
        "nuis build filesystem IO report observable smoke",
    );

    let report_json = run_nuis(&["run-artifact", &report_output_dir_text, "--json"]);
    assert_success(
        &report_json,
        "nuis run-artifact json filesystem IO report observable smoke",
    );
    let report_json_stdout = String::from_utf8_lossy(&report_json.stdout);
    assert!(
        report_json_stdout.contains("\"run_artifact_prelaunch_kind\":\"host-binary\""),
        "run-artifact json did not expose host-binary prelaunch for filesystem report\n{report_json_stdout}"
    );
    assert!(
        report_json_stdout.contains("\"run_artifact_prelaunch_status\":\"ready\""),
        "run-artifact json did not expose ready prelaunch for filesystem report\n{report_json_stdout}"
    );

    let report_run = run_nuis(&["run-artifact", &report_output_dir_text]);
    assert_success(
        &report_run,
        "nuis run-artifact filesystem IO report observable smoke",
    );
    let report_stdout = String::from_utf8_lossy(&report_run.stdout);
    let report_stderr = String::from_utf8_lossy(&report_run.stderr);
    let run_artifact_index = report_stdout
        .find("run-artifact:")
        .expect("filesystem report stdout should include run-artifact summary");
    let report_stdout_prefix = &report_stdout[..run_artifact_index];
    assert!(
        !report_stdout_prefix.is_empty()
            && report_stdout_prefix.chars().all(|ch| ch.is_ascii_digit()),
        "filesystem report stdout should start with observable report digits before run-artifact summary\n{report_stdout}"
    );
    assert!(
        !report_stderr.trim().is_empty()
            && report_stderr.trim().chars().all(|ch| ch.is_ascii_digit()),
        "filesystem report stderr should contain observable report digits\n{report_stderr}"
    );
    assert!(
        report_stdout.contains("exit_status: 0")
            && report_stdout.contains("prelaunch_status: ready")
            && report_stdout.contains("link_plan_final_stage: host-native-link"),
        "filesystem report run-artifact output did not expose expected launch fields\n{report_stdout}"
    );
    assert_file_contains(
        &report_output_dir.join("filesystem_io_report_demo.ll"),
        "host_stdout_write",
        "filesystem IO report LLVM",
    );
    assert_file_contains(
        &report_output_dir.join("filesystem_io_report_demo.yir"),
        "host_stderr_write",
        "filesystem IO report YIR",
    );

    let stdin_output_dir = temp_dir("stdin_runtime_observable");
    let stdin_output_dir_text = stdin_output_dir.display().to_string();
    let stdin_build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/stdin_runtime_demo",
        &stdin_output_dir_text,
    ]);
    assert_success(&stdin_build, "nuis build stdin observable smoke");
    assert_file_contains(
        &stdin_output_dir.join("stdin_runtime_demo.ll"),
        "host_stdin_read",
        "stdin runtime LLVM",
    );
    assert_file_contains(
        &stdin_output_dir.join("stdin_runtime_demo.yir"),
        "host_stdin_read",
        "stdin runtime YIR",
    );
    let stdin_binary = stdin_output_dir.join("stdin_runtime_demo");
    let stdin_run = run_binary_with_stdin(
        &stdin_binary,
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
    );
    assert_success(&stdin_run, "direct stdin runtime binary smoke");
    assert!(
        stdin_run.stdout.is_empty() && stdin_run.stderr.is_empty(),
        "stdin runtime should consume piped stdin without producing report output\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&stdin_run.stdout),
        String::from_utf8_lossy(&stdin_run.stderr)
    );
    let stdin_json = run_nuis(&["run-artifact", &stdin_output_dir_text, "--json"]);
    assert_success(&stdin_json, "nuis run-artifact json stdin observable smoke");
    let stdin_json_stdout = String::from_utf8_lossy(&stdin_json.stdout);
    assert!(
        stdin_json_stdout.contains("\"run_artifact_prelaunch_kind\":\"host-binary\"")
            && stdin_json_stdout.contains("\"run_artifact_prelaunch_status\":\"ready\"")
            && stdin_json_stdout.contains("\"binary_resolved\":true"),
        "stdin run-artifact json did not expose expected executable readiness\n{stdin_json_stdout}"
    );

    let wc_output_dir = temp_dir("cli_wc_observable");
    let wc_output_dir_text = wc_output_dir.display().to_string();
    let wc_build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/cli_wc_demo",
        &wc_output_dir_text,
    ]);
    assert_success(&wc_build, "nuis build cli wc observable smoke");
    assert_file_contains(
        &wc_output_dir.join("cli_wc_demo.ll"),
        "host_argv_count",
        "cli wc LLVM argv bridge",
    );
    assert_file_contains(
        &wc_output_dir.join("cli_wc_demo.ll"),
        "host_file_read",
        "cli wc LLVM file read bridge",
    );
    assert_file_contains(
        &wc_output_dir.join("cli_wc_demo.yir"),
        "host_text_word_count",
        "cli wc YIR text word-count bridge",
    );
    let wc_input_path = wc_output_dir.join("wc-input.txt");
    fs::write(&wc_input_path, b"alpha beta\ngamma delta\nepsilon\n")
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", wc_input_path.display()));
    let wc_input_text = wc_input_path.display().to_string();
    let wc_binary = wc_output_dir.join("cli_wc_demo");
    let wc_run = run_binary_with_args(&wc_binary, &[&wc_input_text]);
    assert_success(&wc_run, "direct cli wc binary smoke");
    let wc_stdout = String::from_utf8_lossy(&wc_run.stdout);
    assert!(
        wc_stdout.contains("bytes: 31\n")
            && wc_stdout.contains("text_len: 31\n")
            && wc_stdout.contains("lines: 3\n")
            && wc_stdout.contains("words: 5\n")
            && wc_stdout.contains("scan_ns: "),
        "cli wc stdout did not expose expected text report\n{wc_stdout}"
    );
    assert!(
        wc_run.stderr.is_empty(),
        "cli wc should not write stderr on success\n{}",
        String::from_utf8_lossy(&wc_run.stderr)
    );

    let report_file_output_dir = temp_dir("cli_report_file_observable");
    let report_file_output_dir_text = report_file_output_dir.display().to_string();
    let report_file_build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/cli_report_file_demo",
        &report_file_output_dir_text,
    ]);
    assert_success(&report_file_build, "nuis build cli report file smoke");
    assert_file_contains(
        &report_file_output_dir.join("cli_report_file_demo.ll"),
        "host_file_write",
        "cli report file LLVM file writer",
    );
    assert_file_contains(
        &report_file_output_dir.join("cli_report_file_demo.yir"),
        "StdReportContracts.cli_report_file_status_total",
        "cli report file YIR std report contract",
    );
    let report_output_path = report_file_output_dir.join("nuis-report.txt");
    let report_output_text = report_output_path.display().to_string();
    let report_file_binary = report_file_output_dir.join("cli_report_file_demo");
    let report_file_run = run_binary_with_args(&report_file_binary, &[&report_output_text]);
    assert_success(&report_file_run, "direct cli report file binary smoke");
    let report_file_stdout = String::from_utf8_lossy(&report_file_run.stdout);
    assert!(
        report_file_stdout.contains("route: argv-file-text")
            && report_file_stdout.contains("status: ready"),
        "cli report file stdout did not expose expected report\n{report_file_stdout}"
    );
    assert_file_contains(
        &report_output_path,
        "output: report-file",
        "cli report file generated report",
    );

    let pixelmagic_output_dir = temp_dir("pixelmagic_report_file_observable");
    let pixelmagic_output_dir_text = pixelmagic_output_dir.display().to_string();
    let pixelmagic_build = run_nuis(&[
        "build",
        "../../examples/projects/domains/pixelmagic_report_file_demo",
        &pixelmagic_output_dir_text,
    ]);
    assert_success(&pixelmagic_build, "nuis build pixelmagic report file smoke");
    assert_file_contains(
        &pixelmagic_output_dir.join("pixelmagic_report_file_demo.yir"),
        "StdReportContracts.cli_report_file_validation_total",
        "pixelmagic report file YIR std report contract",
    );
    assert_file_contains(
        &pixelmagic_output_dir.join("pixelmagic_report_file_demo.yir"),
        "PixelMagicContracts.filter_chain_total",
        "pixelmagic report file YIR package contract",
    );
    let pixelmagic_report_path = pixelmagic_output_dir.join("pixelmagic-report.txt");
    let pixelmagic_report_text = pixelmagic_report_path.display().to_string();
    let pixelmagic_binary = pixelmagic_output_dir.join("pixelmagic_report_file_demo");
    let pixelmagic_run = run_binary_with_args(&pixelmagic_binary, &[&pixelmagic_report_text]);
    assert_success(
        &pixelmagic_run,
        "direct pixelmagic report file binary smoke",
    );
    let pixelmagic_stdout = String::from_utf8_lossy(&pixelmagic_run.stdout);
    assert!(
        pixelmagic_stdout.contains("package: pixelmagic")
            && pixelmagic_stdout.contains("route: std-report-file")
            && pixelmagic_stdout.contains("workload: image-analysis"),
        "pixelmagic report stdout did not expose expected report\n{pixelmagic_stdout}"
    );
    assert_file_contains(
        &pixelmagic_report_path,
        "status: ready",
        "pixelmagic generated report",
    );

    let witsage_output_dir = temp_dir("witsage_report_file_observable");
    let witsage_output_dir_text = witsage_output_dir.display().to_string();
    let witsage_build = run_nuis(&[
        "build",
        "../../examples/projects/domains/witsage_report_file_demo",
        &witsage_output_dir_text,
    ]);
    assert_success(&witsage_build, "nuis build witsage report file smoke");
    assert_file_contains(
        &witsage_output_dir.join("witsage_report_file_demo.yir"),
        "StdReportContracts.cli_report_file_validation_total",
        "witsage report file YIR std report contract",
    );
    assert_file_contains(
        &witsage_output_dir.join("witsage_report_file_demo.yir"),
        "WitSageContracts.classifier_pipeline_total",
        "witsage report file YIR package contract",
    );
    let witsage_report_path = witsage_output_dir.join("witsage-report.txt");
    let witsage_report_text = witsage_report_path.display().to_string();
    let witsage_binary = witsage_output_dir.join("witsage_report_file_demo");
    let witsage_run = run_binary_with_args(&witsage_binary, &[&witsage_report_text]);
    assert_success(&witsage_run, "direct witsage report file binary smoke");
    let witsage_stdout = String::from_utf8_lossy(&witsage_run.stdout);
    assert!(
        witsage_stdout.contains("package: witsage")
            && witsage_stdout.contains("route: std-report-file")
            && witsage_stdout.contains("workload: classical-ml"),
        "witsage report stdout did not expose expected report\n{witsage_stdout}"
    );
    assert_file_contains(
        &witsage_report_path,
        "status: ready",
        "witsage generated report",
    );
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
        stdout.contains("final_executable_output_ready: false"),
        "release-check output should not report the host final executable as Nsld-owned ready before nsld drive\n{stdout}"
    );
    assert!(
        stdout.contains("final_executable_output_path_present: true"),
        "release-check output did not report the host final executable output path\n{stdout}"
    );
    assert!(
        stdout.contains("final_executable_output_nsld_owned: <unknown>"),
        "release-check output should not guess final executable ownership before nsld drive\n{stdout}"
    );
    assert!(
        stdout.contains(
            "final_executable_output_first_blocker: final-executable-output:ownership-unknown"
        ),
        "release-check output did not surface the final executable output blocker\n{stdout}"
    );
    assert!(
        stdout
            .contains("final_executable_output_blocker: final-executable-output:ownership-unknown"),
        "release-check output did not surface the final executable output blocker list\n{stdout}"
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
        nsld_until_clean_stdout.contains("\"stop_reason\":\"host-finalizer-policy-required\""),
        "nsld drive until-clean should stop at the explicit host finalizer policy boundary\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout.contains("\"stop_command_id\":\"final-executable-output\""),
        "nsld drive until-clean should report the final executable output boundary command\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout.contains(
            "\"stop_source\":\"final-output-boundary\""
        ),
        "nsld drive until-clean should classify the stop as the final output boundary\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout
            .contains("\"messages\":[\"applied emit-units\"")
            && nsld_until_clean_stdout.contains("\"read-only-boundary:final-executable-output\""),
        "nsld drive until-clean should materialize the chain before the read-only host boundary\n{nsld_until_clean_stdout}"
    );
    assert!(
        nsld_until_clean_stdout.contains("\"last_command_id\":\"emit-final-executable-pipeline\""),
        "nsld drive until-clean did not finish at the final executable pipeline stage\n{nsld_until_clean_stdout}"
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

    let run_artifact_json = run_nuis(&["run-artifact", &manifest_path_text, "--json"]);
    assert_success(
        &run_artifact_json,
        "nuis run-artifact json after nsld until-clean",
    );
    let run_artifact_stdout = String::from_utf8_lossy(&run_artifact_json.stdout);
    assert!(
        run_artifact_stdout.contains("\"nsld_final_executable_output_ready\":false"),
        "run-artifact json should distinguish host runnable output from Nsld-owned final output readiness\n{run_artifact_stdout}"
    );
    assert!(
        run_artifact_stdout.contains("\"nsld_final_executable_output_path_present\":true"),
        "run-artifact json did not report the host final output path after nsld drive\n{run_artifact_stdout}"
    );
    assert!(
        run_artifact_stdout.contains("\"nsld_final_executable_output_nsld_owned\":false"),
        "run-artifact json did not resolve final output ownership after nsld drive\n{run_artifact_stdout}"
    );
    assert!(
        run_artifact_stdout
            .contains("\"nsld_final_executable_output_blockers\":[\"final-executable-output:not-nsld-owned\"]"),
        "run-artifact json did not report the not-nsld-owned final output blocker after nsld drive\n{run_artifact_stdout}"
    );
}

#[test]
#[ignore = "full std tooling smoke builds/runs every representative std filesystem/text/io/benchmark project"]
fn std_tooling_full_project_smokes_build_doctor_and_run() {
    assert_std_tooling_project_smoke(STD_TOOLING_FULL_SMOKE_PROJECTS);
}
