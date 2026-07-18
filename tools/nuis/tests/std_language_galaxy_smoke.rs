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
    let dir = std::env::temp_dir().join(format!("nuis_{label}_{nonce}"));
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

fn assert_file_contains(path: &Path, needle: &str, context: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        source.contains(needle),
        "expected {context} file {} to contain `{needle}`\n{source}",
        path.display()
    );
}

fn assert_file_not_contains(path: &Path, needle: &str, context: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        !source.contains(needle),
        "expected {context} file {} not to contain `{needle}`\n{source}",
        path.display()
    );
}

#[test]
fn std_language_galaxy_project_runs_std_result_hof_surface() {
    let output_dir = temp_dir("std_language_galaxy_bootstrap");
    let output_dir_text = output_dir.display().to_string();
    let source = fs::read_to_string(
        "../../examples/projects/state/std_language_galaxy_bootstrap_demo/main.ns",
    )
    .expect("read std language galaxy bootstrap source");
    assert!(source.contains("use cpu StdLanguageCore"));
    assert!(source.contains("use cpu StdLanguageOps"));
    assert!(!source.contains("result_map<i64, i64, Error>"));
    assert!(!source.contains("ok<i64, Error>"));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/state/std_language_galaxy_bootstrap_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build std language galaxy bootstrap");

    assert_file_contains(
        &output_dir.join("nuis.project.galaxy.txt"),
        "library_modules=lib/task_contracts.ns",
        "std language galaxy project galaxy index",
    );
    assert_file_contains(
        &output_dir.join("nuis.project.galaxy.txt"),
        "lib/language_core.ns",
        "std language galaxy project galaxy index",
    );
    assert_file_contains(
        &output_dir.join("nuis.project.galaxy.txt"),
        "lib/language_ops.ns",
        "std language galaxy project galaxy index",
    );
    assert_file_contains(
        &output_dir.join("nuis.project.imports.txt"),
        "use\tcpu.StdLanguageOps\tcpu.StdLanguageCore\tresolution=local-visible:galaxy-auto-inject",
        "std language galaxy project imports",
    );
    assert_file_contains(
        &output_dir.join("std_language_galaxy_bootstrap_demo.nir.txt"),
        "StdLanguageOps.__hof_result_map___lambda_build_report_0__i64__i64__Error",
        "std language galaxy bootstrap NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_galaxy_bootstrap_demo.nir.txt"),
        "__hof_result_map___lambda_pipeline_0__i64__i64__Error",
        "std language galaxy bootstrap NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_galaxy_bootstrap_demo.nir.txt"),
        "ok__i64__Error(raw_score)",
        "std language galaxy bootstrap NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_galaxy_bootstrap_demo.nir.txt"),
        "impl.Addable.for.i64.add(item, item)",
        "std language galaxy bootstrap NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_galaxy_bootstrap_demo.yir"),
        "edge lifetime",
        "std language galaxy bootstrap YIR",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_galaxy_bootstrap_demo.ll"),
        "deferred lowering",
        "std language galaxy bootstrap LLVM",
    );

    let binary = Command::new(output_dir.join("std_language_galaxy_bootstrap_demo"))
        .output()
        .expect("run std language galaxy bootstrap binary");
    assert_eq!(binary.status.code(), Some(55));
}

#[test]
fn std_language_galaxy_cli_report_runs_through_text_and_io_contracts() {
    let output_dir = temp_dir("std_language_cli_report");
    let output_dir_text = output_dir.display().to_string();
    let source =
        fs::read_to_string("../../examples/projects/tooling/std_language_cli_report_demo/main.ns")
            .expect("read std language cli report source");
    assert!(source.contains("use cpu StdLanguageCore"));
    assert!(source.contains("use cpu StdLanguageOps"));
    assert!(source.contains("use cpu StdTextContracts"));
    assert!(source.contains("use cpu StdIoContracts"));
    assert!(source.contains("host_stdout_write"));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/std_language_cli_report_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build std language cli report");

    assert_file_contains(
        &output_dir.join("nuis.project.imports.txt"),
        "use\tcpu.StdLanguageOps\tcpu.StdLanguageCore\tresolution=local-visible:galaxy-auto-inject",
        "std language cli report imports",
    );
    assert_file_contains(
        &output_dir.join("std_language_cli_report_demo.nir.txt"),
        "StdLanguageOps.build_report(seed)",
        "std language cli report NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_cli_report_demo.nir.txt"),
        "extern \"c\" host_stdout_write(report_handle)",
        "std language cli report NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_cli_report_demo.nir.txt"),
        "StdTextContracts.report_validation_total(report_handle, report_len)",
        "std language cli report NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_cli_report_demo.nir.txt"),
        "StdIoContracts.output_report_validation_total(report_len, stdout_written, stdout_flushed)",
        "std language cli report NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_cli_report_demo.yir"),
        "host_stdout_write",
        "std language cli report YIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_cli_report_demo.yir"),
        "host_text_len",
        "std language cli report YIR",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_cli_report_demo.ll"),
        "deferred lowering",
        "std language cli report LLVM",
    );

    let binary = Command::new(output_dir.join("std_language_cli_report_demo"))
        .output()
        .expect("run std language cli report binary");
    assert_eq!(binary.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&binary.stdout);
    assert!(stdout.contains("nuis std language report"));
    assert!(stdout.contains("route: std-language-cli"));
    assert!(stdout.contains("status: ready"));
}

#[test]
fn std_language_galaxy_report_file_runs_through_report_contracts() {
    let output_dir = temp_dir("std_language_report_file");
    let output_dir_text = output_dir.display().to_string();
    let source =
        fs::read_to_string("../../examples/projects/tooling/std_language_report_file_demo/main.ns")
            .expect("read std language report file source");
    assert!(source.contains("use cpu StdLanguageCore"));
    assert!(source.contains("use cpu StdLanguageOps"));
    assert!(source.contains("use cpu StdReportContracts"));
    assert!(source.contains("host_file_write"));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/std_language_report_file_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build std language report file");

    assert_file_contains(
        &output_dir.join("nuis.project.imports.txt"),
        "use\tcpu.StdLanguageOps\tcpu.StdLanguageCore\tresolution=local-visible:galaxy-auto-inject",
        "std language report file imports",
    );
    assert_file_contains(
        &output_dir.join("std_language_report_file_demo.nir.txt"),
        "StdLanguageOps.build_report(seed)",
        "std language report file NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_report_file_demo.nir.txt"),
        "extern \"c\" host_file_write(output_file_handle, report_handle)",
        "std language report file NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_report_file_demo.nir.txt"),
        "StdReportContracts.cli_report_file_validation_total",
        "std language report file NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_report_file_demo.yir"),
        "host_file_write",
        "std language report file YIR",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_report_file_demo.ll"),
        "deferred lowering",
        "std language report file LLVM",
    );

    let report_path = output_dir.join("std-language-report.txt");
    let binary = Command::new(output_dir.join("std_language_report_file_demo"))
        .arg(&report_path)
        .output()
        .expect("run std language report file binary");
    assert_eq!(binary.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&binary.stdout);
    assert!(stdout.contains("nuis std language file report"));
    assert!(stdout.contains("route: std-language-report-file"));
    let report_text = fs::read_to_string(&report_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", report_path.display()));
    assert!(report_text.contains("nuis std language file report"));
    assert!(report_text.contains("output: argv-file-and-stdout"));
}

#[test]
fn std_language_galaxy_workflow_runs_through_cli_contracts() {
    let output_dir = temp_dir("std_language_workflow");
    let output_dir_text = output_dir.display().to_string();
    let source =
        fs::read_to_string("../../examples/projects/tooling/std_language_workflow_demo/main.ns")
            .expect("read std language workflow source");
    assert!(source.contains("use cpu StdCliContracts"));
    assert!(source.contains("use cpu StdLanguageCore"));
    assert!(source.contains("use cpu StdLanguageOps"));
    assert!(source.contains("host_command_spawn"));
    assert!(source.contains("build_report"));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/std_language_workflow_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build std language workflow");

    assert_file_contains(
        &output_dir.join("nuis.project.imports.txt"),
        "use\tcpu.StdLanguageOps\tcpu.StdLanguageCore\tresolution=local-visible:galaxy-auto-inject",
        "std language workflow imports",
    );
    assert_file_contains(
        &output_dir.join("std_language_workflow_demo.nir.txt"),
        "StdLanguageOps.build_report",
        "std language workflow NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_workflow_demo.nir.txt"),
        "extern \"c\" host_command_spawn",
        "std language workflow NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_workflow_demo.nir.txt"),
        "StdCliContracts.workflow_validation_total",
        "std language workflow NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_workflow_demo.yir"),
        "host_command_spawn",
        "std language workflow YIR",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_workflow_demo.ll"),
        "deferred lowering",
        "std language workflow LLVM",
    );

    let binary = Command::new(output_dir.join("std_language_workflow_demo"))
        .output()
        .expect("run std language workflow binary");
    assert_eq!(binary.status.code(), Some(0));
}

#[test]
fn std_language_galaxy_task_cli_runs_through_task_contracts() {
    let output_dir = temp_dir("std_language_task_cli");
    let output_dir_text = output_dir.display().to_string();
    let source =
        fs::read_to_string("../../examples/projects/task/std_language_task_cli_demo/main.ns")
            .expect("read std language task cli source");
    assert!(source.contains("use cpu StdLanguageCore"));
    assert!(source.contains("use cpu StdLanguageOps"));
    assert!(source.contains("use cpu StdTaskContracts"));
    assert!(source.contains("spawn(ping"));
    assert!(source.contains("join_result"));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/std_language_task_cli_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build std language task cli");

    assert_file_contains(
        &output_dir.join("nuis.project.imports.txt"),
        "use\tcpu.StdLanguageOps\tcpu.StdLanguageCore\tresolution=local-visible:galaxy-auto-inject",
        "std language task cli imports",
    );
    assert_file_contains(
        &output_dir.join("std_language_task_cli_demo.nir.txt"),
        "StdLanguageOps.build_report",
        "std language task cli NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_task_cli_demo.nir.txt"),
        "StdTaskContracts.cli_total",
        "std language task cli NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_task_cli_demo.yir"),
        "cpu.spawn_task",
        "std language task cli YIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_task_cli_demo.yir"),
        "host_stdout_write",
        "std language task cli YIR",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_task_cli_demo.ll"),
        "deferred lowering for cpu.timeout",
        "std language task cli LLVM",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_task_cli_demo.ll"),
        "deferred lowering for cpu.join_result",
        "std language task cli LLVM",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_task_cli_demo.ll"),
        "deferred lowering for cpu.task_value",
        "std language task cli LLVM",
    );
    assert_file_contains(
        &output_dir.join("std_language_task_cli_demo.ll"),
        "call i64 @nuis_scheduler_task_spawn_thunk_i64_v1(ptr @nuis_fn_ping",
        "std language task cli scheduler thunk spawn ABI",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_task_cli_demo.ll"),
        "call i64 @nuis_fn_ping(",
        "std language task cli eager async helper call",
    );
    assert_file_contains(
        &output_dir.join("std_language_task_cli_demo.ll"),
        "call i64 @nuis_scheduler_task_join_state_v1",
        "std language task cli scheduler join ABI",
    );
    assert_file_contains(
        &output_dir.join("std_language_task_cli_demo.ll"),
        "call i64 @nuis_scheduler_task_value_i64_v1",
        "std language task cli scheduler value ABI",
    );

    let binary = Command::new(output_dir.join("std_language_task_cli_demo"))
        .output()
        .expect("run std language task cli binary");
    assert_eq!(binary.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&binary.stdout);
    let stderr = String::from_utf8_lossy(&binary.stderr);
    assert!(stdout.contains("std language task completed"));
    assert!(!stderr.contains("std language task timed out"));
}

#[test]
fn std_language_galaxy_build_pipeline_runs_through_cli_contracts() {
    let output_dir = temp_dir("std_language_build_pipeline");
    let output_dir_text = output_dir.display().to_string();
    let source = fs::read_to_string(
        "../../examples/projects/tooling/std_language_build_pipeline_demo/main.ns",
    )
    .expect("read std language build pipeline source");
    assert!(source.contains("use cpu StdCliContracts"));
    assert!(source.contains("use cpu StdLanguageCore"));
    assert!(source.contains("use cpu StdLanguageOps"));
    assert!(source.contains("build_pipeline_total"));
    assert!(source.contains("host_command_spawn"));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/std_language_build_pipeline_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build std language build pipeline");

    assert_file_contains(
        &output_dir.join("nuis.project.imports.txt"),
        "use\tcpu.StdLanguageOps\tcpu.StdLanguageCore\tresolution=local-visible:galaxy-auto-inject",
        "std language build pipeline imports",
    );
    assert_file_contains(
        &output_dir.join("std_language_build_pipeline_demo.nir.txt"),
        "StdLanguageOps.build_report",
        "std language build pipeline NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_build_pipeline_demo.nir.txt"),
        "StdCliContracts.build_pipeline_total",
        "std language build pipeline NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_build_pipeline_demo.nir.txt"),
        "extern \"c\" host_command_spawn",
        "std language build pipeline NIR",
    );
    assert_file_contains(
        &output_dir.join("std_language_build_pipeline_demo.yir"),
        "host_command_spawn",
        "std language build pipeline YIR",
    );
    assert_file_not_contains(
        &output_dir.join("std_language_build_pipeline_demo.ll"),
        "deferred lowering",
        "std language build pipeline LLVM",
    );

    let binary = Command::new(output_dir.join("std_language_build_pipeline_demo"))
        .output()
        .expect("run std language build pipeline binary");
    assert_eq!(binary.status.code(), Some(0));
}

#[test]
fn cli_build_pipeline_keeps_std_language_gate_native() {
    let output_dir = temp_dir("cli_build_pipeline_language_gate");
    let output_dir_text = output_dir.display().to_string();
    let source =
        fs::read_to_string("../../examples/projects/tooling/cli_build_pipeline_demo/main.ns")
            .expect("read cli build pipeline source");
    assert!(source.contains("use cpu StdLanguageCore"));
    assert!(source.contains("use cpu StdLanguageOps"));
    assert!(source.contains("language_pipeline_gate"));
    assert!(source.contains("build_report"));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/tooling/cli_build_pipeline_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build cli build pipeline language gate");

    assert_file_contains(
        &output_dir.join("nuis.project.imports.txt"),
        "use\tcpu.StdLanguageOps\tcpu.StdLanguageCore\tresolution=local-visible:galaxy-auto-inject",
        "cli build pipeline language imports",
    );
    assert_file_contains(
        &output_dir.join("cli_build_pipeline_demo.nir.txt"),
        "StdLanguageOps.build_report",
        "cli build pipeline language NIR",
    );
    assert_file_contains(
        &output_dir.join("cli_build_pipeline_demo.yir"),
        "StdLanguageOps.build_report",
        "cli build pipeline language YIR",
    );
    assert_file_not_contains(
        &output_dir.join("cli_build_pipeline_demo.ll"),
        "deferred lowering",
        "cli build pipeline LLVM",
    );

    let binary = Command::new(output_dir.join("cli_build_pipeline_demo"))
        .output()
        .expect("run cli build pipeline binary");
    assert_eq!(binary.status.code(), Some(0));
}
