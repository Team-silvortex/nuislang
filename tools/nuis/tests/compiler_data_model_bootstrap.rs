use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_compiler_data_model_{nonce}"));
    fs::create_dir_all(&dir).expect("create compiler data model output directory");
    dir
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {args:?}: {error}"))
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

fn assert_file_not_contains(path: &Path, needle: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        !source.contains(needle),
        "expected {} not to contain `{needle}`",
        path.display()
    );
}

#[test]
fn compiler_data_model_bootstrap_builds_and_runs_as_pure_nuis() {
    let project = "../../examples/projects/tooling/bootstrap_compiler_data_model_demo";
    let output_dir = temp_dir();
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&["build", project, &output_dir_text]);
    assert_success(&build, "compiler data model build");
    assert_file_not_contains(
        &output_dir.join("bootstrap_compiler_data_model_demo.ll"),
        "deferred lowering",
    );

    let run = Command::new(output_dir.join("bootstrap_compiler_data_model_demo"))
        .output()
        .expect("run compiler data model binary");
    assert_eq!(
        run.status.code(),
        Some(43),
        "compiler data model binary should return its deterministic compiler score"
    );

    fs::remove_dir_all(output_dir).expect("remove compiler data model output directory");
}
