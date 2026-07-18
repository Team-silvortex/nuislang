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

fn assert_llvm_entry_not_contains(path: &Path, needle: &str, context: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let entry = source
        .split("define i64 @nuis_yir_entry()")
        .nth(1)
        .unwrap_or_else(|| panic!("expected LLVM entry in {}", path.display()));
    assert!(
        !entry.contains(needle),
        "expected {context} LLVM entry {} not to contain `{needle}`\n{entry}",
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
        "call i64 @nuis_scheduler_task_spawn_invoker_i64_v1(ptr @nuis_task_invoker_ping",
        "std language task cli scheduler thunk spawn ABI",
    );
    assert_llvm_entry_not_contains(
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
fn task_timeout_reaches_native_scheduler_terminal_state() {
    let output_dir = temp_dir("task_timeout_runtime");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_lifecycle_branch_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build task timeout runtime");

    let llvm = output_dir.join("task_lifecycle_branch_demo.ll");
    assert_file_contains(
        &llvm,
        "call void @nuis_scheduler_task_timeout_v1",
        "task timeout scheduler ABI",
    );
    assert_file_contains(&llvm, "icmp eq i64", "task timeout runtime state branch");

    let binary = Command::new(output_dir.join("task_lifecycle_branch_demo"))
        .output()
        .expect("run task timeout runtime binary");
    assert_eq!(binary.status.code(), Some(74));
    assert!(
        String::from_utf8_lossy(&binary.stdout).contains("task_lifecycle_branch_demo: timed out")
    );
}

#[test]
fn task_ready_delay_orders_completion_against_timeout() {
    let output_dir = temp_dir("task_ready_delay_ordering");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_ready_delay_ordering_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build task ready delay ordering");

    let llvm = output_dir.join("task_ready_delay_ordering_demo.ll");
    assert_file_contains(
        &llvm,
        "call void @nuis_scheduler_task_ready_after_v1",
        "task ready delay scheduler ABI",
    );
    assert_file_contains(
        &llvm,
        "call void @nuis_scheduler_task_timeout_v1",
        "task timeout scheduler ABI",
    );
    assert_file_not_contains(
        &llvm,
        "deferred lowering for cpu.ready_after",
        "task ready delay lowering",
    );

    let binary = Command::new(output_dir.join("task_ready_delay_ordering_demo"))
        .output()
        .expect("run task ready delay ordering binary");
    assert_eq!(binary.status.code(), Some(18));
}

#[test]
fn task_cancel_reaches_native_scheduler_terminal_state() {
    let output_dir = temp_dir("task_cancel_runtime");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_cancel_branch_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build task cancel runtime");

    let llvm = output_dir.join("task_cancel_branch_demo.ll");
    assert_file_contains(
        &llvm,
        "call i64 @nuis_scheduler_task_spawn_invoker_i64_v1(ptr @nuis_task_invoker_ping, ptr null)",
        "task cancel deferred zero-argument thunk",
    );
    assert_file_contains(
        &llvm,
        "call void @nuis_scheduler_task_cancel_v1",
        "task cancel scheduler ABI",
    );
    assert_file_contains(&llvm, "icmp eq i64", "task cancel runtime state branch");
    assert_llvm_entry_not_contains(
        &llvm,
        "call i64 @nuis_fn_ping(",
        "cancelled eager task thunk",
    );

    let binary = Command::new(output_dir.join("task_cancel_branch_demo"))
        .output()
        .expect("run task cancel runtime binary");
    assert_eq!(binary.status.code(), Some(71));
    assert!(String::from_utf8_lossy(&binary.stdout).contains("task_cancel_branch_demo: cancelled"));
}

#[test]
fn zero_argument_task_thunk_completes_from_scheduler_poll() {
    let output_dir = temp_dir("task_zero_argument_thunk");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_completed_observe_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build zero-argument task thunk");

    let llvm = output_dir.join("task_completed_observe_demo.ll");
    assert_file_contains(
        &llvm,
        "call i64 @nuis_scheduler_task_spawn_invoker_i64_v1(ptr @nuis_task_invoker_ping, ptr null)",
        "zero-argument scheduler thunk ABI",
    );
    assert_llvm_entry_not_contains(
        &llvm,
        "call i64 @nuis_fn_ping()",
        "eager zero-argument task thunk",
    );
    assert_file_contains(
        &llvm,
        "call i64 @nuis_scheduler_task_value_i64_v1",
        "zero-argument scheduler task payload",
    );

    let binary = Command::new(output_dir.join("task_completed_observe_demo"))
        .output()
        .expect("run zero-argument task thunk binary");
    assert_eq!(binary.status.code(), Some(7));
}

#[test]
fn binary_task_thunk_preserves_recursive_result_family() {
    let output_dir = temp_dir("task_binary_thunk");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_recursive_async_result_family_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build binary task thunk");

    let llvm_path = output_dir.join("task_recursive_async_result_family_demo.ll");
    let llvm = fs::read_to_string(&llvm_path).expect("read binary task thunk LLVM");
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_task_spawn_invoker_i64_v1")
            .count(),
        3
    );
    let entry = llvm
        .split("define i64 @nuis_yir_entry()")
        .nth(1)
        .expect("binary task thunk LLVM entry");
    assert!(!entry.contains("call i64 @nuis_fn_sum_down("));

    let binary = Command::new(output_dir.join("task_recursive_async_result_family_demo"))
        .output()
        .expect("run binary task thunk result family");
    assert_eq!(binary.status.code(), Some(14));
}

#[test]
fn task_context_supports_more_than_direct_call_arity() {
    let output_dir = temp_dir("task_context_arity");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_context_arity_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build task context arity");

    let llvm = output_dir.join("task_context_arity_demo.ll");
    assert_file_contains(
        &llvm,
        "call ptr @malloc(i64 32)",
        "four-argument task context",
    );
    assert_file_contains(
        &llvm,
        "%task_arg3_ptr = getelementptr i8, ptr %context, i64 24",
        "fourth task context argument",
    );
    assert_file_contains(
        &llvm,
        "call i64 @nuis_scheduler_task_spawn_invoker_i64_v1(ptr @nuis_task_invoker_sum_four",
        "common task context spawn ABI",
    );
    assert_llvm_entry_not_contains(
        &llvm,
        "call i64 @nuis_fn_sum_four(",
        "eager four-argument task",
    );

    let binary = Command::new(output_dir.join("task_context_arity_demo"))
        .output()
        .expect("run task context arity binary");
    assert_eq!(binary.status.code(), Some(10));
}

#[test]
fn task_context_normalizes_bool_and_i32_scalars() {
    let output_dir = temp_dir("task_scalar_context");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_scalar_context_demo",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build task scalar context");

    let llvm = output_dir.join("task_scalar_context_demo.ll");
    assert_file_contains(
        &llvm,
        "define i64 @nuis_task_invoker_pick_i32(ptr %context)",
        "i32 task invoker",
    );
    assert_file_contains(
        &llvm,
        "trunc i64 %task_arg0_packed to i1",
        "bool task argument unpack",
    );
    assert_file_contains(
        &llvm,
        "trunc i64 %task_arg1_packed to i32",
        "i32 task argument unpack",
    );
    assert_file_contains(
        &llvm,
        "sext i32 %task_result to i64",
        "i32 task result pack",
    );
    assert_file_contains(
        &llvm,
        "zext i1 %task_result to i64",
        "bool task result pack",
    );
    assert_llvm_entry_not_contains(
        &llvm,
        "call i32 @nuis_fn_pick_i32(",
        "eager i32 task helper",
    );
    assert_llvm_entry_not_contains(
        &llvm,
        "call i1 @nuis_fn_identity_bool(",
        "eager bool task helper",
    );

    let binary = Command::new(output_dir.join("task_scalar_context_demo"))
        .output()
        .expect("run task scalar context binary");
    assert_eq!(binary.status.code(), Some(23));
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
