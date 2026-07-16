use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_semantics::model::{NirExpr, NirStmt};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_language_{label}_{nonce}"));
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

fn assert_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context} missing `{needle}`\nfull output:\n{haystack}"
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

fn assert_binary_exit(binary: &Path, expected: i32, context: &str) {
    let output = Command::new(binary)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", binary.display()));
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{context} should exit with {expected}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn task_result_enum_project_anchors_language_bootstrap_smoke() {
    let output_dir = temp_dir("task_result_enum_bootstrap");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_result_enum_demo",
        &output_dir_text,
    ]);
    assert_success(
        &build,
        "nuis build task result enum language bootstrap smoke",
    );

    let run_json = run_nuis(&["run-artifact", &output_dir_text, "--json"]);
    assert_success(
        &run_json,
        "nuis run-artifact json task result enum language bootstrap smoke",
    );
    let run_json_stdout = String::from_utf8_lossy(&run_json.stdout);
    assert_contains(
        &run_json_stdout,
        "\"run_artifact_prelaunch_status\":\"ready\"",
        "task result enum run-artifact json",
    );
    assert_contains(
        &run_json_stdout,
        "\"link_plan_final_stage\":\"host-native-link\"",
        "task result enum run-artifact json",
    );

    assert_file_contains(
        &output_dir.join("task_result_enum_demo.nir.txt"),
        "__hof_result_map_raise_value__i64__i64__ErrorEnvelope",
        "task result enum NIR",
    );
    assert_file_contains(
        &output_dir.join("task_result_enum_demo.nir.txt"),
        "Result<i64, ErrorEnvelope>",
        "task result enum NIR",
    );
    assert_file_contains(
        &output_dir.join("task_result_enum_demo.yir"),
        "cpu.variant_is",
        "task result enum YIR",
    );
    assert_file_contains(
        &output_dir.join("task_result_enum_demo.yir"),
        "cpu.task_completed",
        "task result enum YIR",
    );
    assert_file_contains(
        &output_dir.join("task_result_enum_demo.yir"),
        "c host_error_code",
        "task result enum YIR",
    );
    assert_file_contains(
        &output_dir.join("task_result_enum_demo.ll"),
        "declare i64 @host_error_code(i64)",
        "task result enum LLVM IR",
    );
    assert_file_contains(
        &output_dir.join("nuis.project.host_ffi.txt"),
        "policy=signature-whitelist-required",
        "task result enum host FFI project report",
    );

    assert_binary_exit(
        &output_dir.join("task_result_enum_demo"),
        39,
        "task_result_enum_demo should execute the Result/task/error path to a deterministic exit code",
    );
}

#[test]
fn generic_trait_and_glm_buffer_projects_anchor_language_bootstrap_smoke() {
    let generic_output_dir = temp_dir("generic_trait_bound_bootstrap");
    let generic_output_dir_text = generic_output_dir.display().to_string();
    let generic_build = run_nuis(&[
        "build",
        "../../examples/projects/state/generic_method_bound_guarded_nested_match_demo",
        &generic_output_dir_text,
    ]);
    assert_success(
        &generic_build,
        "nuis build generic trait-bound language bootstrap smoke",
    );

    assert_file_contains(
        &generic_output_dir.join("generic_method_bound_guarded_nested_match_demo.ast.txt"),
        "trait Addable",
        "generic trait-bound AST",
    );
    assert_file_contains(
        &generic_output_dir.join("generic_method_bound_guarded_nested_match_demo.ast.txt"),
        "type Outer<T> = Alias<T>",
        "generic alias AST",
    );
    assert_file_contains(
        &generic_output_dir.join("generic_method_bound_guarded_nested_match_demo.nir.txt"),
        "fn bump__i64(value: i64) -> i64",
        "generic monomorphized NIR",
    );
    assert_file_contains(
        &generic_output_dir.join("generic_method_bound_guarded_nested_match_demo.nir.txt"),
        "impl.Addable.for.i64.add(local, local)",
        "generic trait method call NIR",
    );
    assert_binary_exit(
        &generic_output_dir.join("generic_method_bound_guarded_nested_match_demo"),
        8,
        "generic trait-bound guarded nested match binary",
    );

    let glm_output_dir = temp_dir("glm_buffer_bootstrap");
    let glm_output_dir_text = glm_output_dir.display().to_string();
    let glm_build = run_nuis(&[
        "build",
        "../../examples/projects/state/glm_buffer_roundtrip_state_demo",
        &glm_output_dir_text,
    ]);
    assert_success(&glm_build, "nuis build GLM buffer language bootstrap smoke");

    assert_file_contains(
        &glm_output_dir.join("glm_buffer_roundtrip_state_demo.nir.txt"),
        "let len: i64 = buffer_len(buffer)",
        "GLM buffer NIR",
    );
    assert_file_contains(
        &glm_output_dir.join("glm_buffer_roundtrip_state_demo.nir.txt"),
        "expr free(buffer)",
        "GLM buffer NIR",
    );
    assert_file_contains(
        &glm_output_dir.join("glm_buffer_roundtrip_state_demo.yir"),
        "cpu.store_at",
        "GLM buffer YIR",
    );
    assert_file_contains(
        &glm_output_dir.join("glm_buffer_roundtrip_state_demo.yir"),
        "edge lifetime",
        "GLM buffer YIR",
    );
    assert_file_contains(
        &glm_output_dir.join("glm_buffer_roundtrip_state_demo.ll"),
        "declare void @free(ptr)",
        "GLM buffer LLVM IR",
    );
    assert_binary_exit(
        &glm_output_dir.join("glm_buffer_roundtrip_state_demo"),
        10,
        "GLM buffer roundtrip binary",
    );
}

#[test]
fn generic_result_buffer_lambda_project_combines_language_bootstrap_features() {
    let output_dir = temp_dir("generic_result_buffer_lambda_bootstrap");
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&[
        "build",
        "../../examples/projects/state/generic_result_buffer_lambda_helper_demo",
        &output_dir_text,
    ]);
    assert_success(
        &build,
        "nuis build generic Result/buffer/lambda language bootstrap smoke",
    );

    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.ast.txt"),
        "trait Addable",
        "generic Result/buffer/lambda AST",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.ast.txt"),
        "enum Result<T, E>",
        "generic Result/buffer/lambda AST",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.ast.txt"),
        "return Result.Err(error)",
        "generic Result/buffer/lambda AST",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.ast.txt"),
        "if (seed == 3)",
        "generic Result/buffer/lambda AST",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.nir.txt"),
        "__hof_result_map___lambda_build_report_0__i64__i64__HelperError",
        "generic Result/buffer/lambda NIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.nir.txt"),
        "err__i64__HelperError(HelperError.Invalid",
        "generic Result/buffer/lambda NIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.nir.txt"),
        "__hof_result_map___lambda_build_report_2__i64__i64__HelperError",
        "generic Result/buffer/lambda NIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.nir.txt"),
        "ok__i64__HelperError(seed)",
        "generic Result/buffer/lambda NIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.nir.txt"),
        "return Result.Err<i64, HelperError>",
        "generic Result/buffer/lambda NIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.nir.txt"),
        "impl.Addable.for.i64.add(item, item)",
        "generic Result/buffer/lambda NIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.nir.txt"),
        "let len: i64 = buffer_len(buffer)",
        "generic Result/buffer/lambda NIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.yir"),
        "cpu.store_at",
        "generic Result/buffer/lambda YIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.yir"),
        "cpu.variant_is",
        "generic Result/buffer/lambda YIR",
    );
    assert_file_contains(
        &output_dir.join("generic_result_buffer_lambda_helper_demo.yir"),
        "HelperReport",
        "generic Result/buffer/lambda YIR",
    );
    assert_binary_exit(
        &output_dir.join("generic_result_buffer_lambda_helper_demo"),
        59,
        "generic Result/buffer/lambda helper binary",
    );
}

#[test]
fn no_annotation_try_await_result_hof_project_anchors_language_bootstrap_smoke() {
    let output_dir = temp_dir("no_annotation_try_await_result_hof_bootstrap");
    let output_dir_text = output_dir.display().to_string();
    let source = fs::read_to_string(
        "../../examples/projects/task/task_no_annotation_try_await_result_hof_demo/main.ns",
    )
    .expect("read no-annotation try/await Result project source");
    assert_contains(
        &source,
        "let mapped = map_result",
        "no-annotation try/await Result project source",
    );
    assert_contains(
        &source,
        "Result.Ok(await work(fetch(seed)?))",
        "no-annotation try/await Result project source",
    );
    let module = nuisc::frontend::parse_nuis_module(&source)
        .expect("parse no-annotation try/await Result project source");

    let compute = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .expect("compute function should remain present");
    assert!(compute.is_async);
    assert!(compute.body.iter().any(mapped_binding_is_concrete_result));

    let mapper = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_map_result___lambda_compute_0")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Result<i64, Error>"
                )
        })
        .expect("map_result lambda should be specialized for Result<i64, Error>");
    assert!(mapper.generic_params.is_empty());
    assert!(matches!(
        mapper.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<i64, Error>"
    ));

    let build = run_nuis(&[
        "build",
        "../../examples/projects/task/task_no_annotation_try_await_result_hof_demo",
        &output_dir_text,
    ]);
    assert_success(
        &build,
        "nuis build no-annotation try/await Result HOF language bootstrap smoke",
    );
    assert_file_contains(
        &output_dir.join("task_no_annotation_try_await_result_hof_demo.yir"),
        "cpu.async_value",
        "no-annotation try/await Result HOF YIR",
    );
    assert_binary_exit(
        &output_dir.join("task_no_annotation_try_await_result_hof_demo"),
        4,
        "no-annotation try/await Result HOF binary",
    );
}

fn mapped_binding_is_concrete_result(stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        } => {
            name == "mapped"
                && ty.render() == "Result<i64, Error>"
                && callee == "__hof_map_result___lambda_compute_0__i64__i64__Error"
        }
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            then_body.iter().any(mapped_binding_is_concrete_result)
                || else_body.iter().any(mapped_binding_is_concrete_result)
        }
        _ => false,
    }
}
