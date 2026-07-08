use std::{
    fs,
    path::PathBuf,
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

#[test]
fn run_artifact_accepts_void_main_print_exit_zero() {
    let project_dir = temp_dir("void_main_print_project");
    let output_dir = temp_dir("void_main_print_outputs");
    fs::write(
        project_dir.join("nuis.toml"),
        r#"
name = "void_main_print"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
    )
    .unwrap();
    fs::write(
        project_dir.join("main.ns"),
        r#"
mod cpu Main {
  fn main() {
    print(42);
  }
}
"#,
    )
    .unwrap();

    let build = run_nuis(&[
        "build",
        &project_dir.display().to_string(),
        &output_dir.display().to_string(),
    ]);
    assert_success(&build, "nuis build void-main print project");

    let json = run_nuis(&["run-artifact", "--json", &output_dir.display().to_string()]);
    assert_success(&json, "nuis run-artifact --json void-main print project");
    let json_stdout = String::from_utf8_lossy(&json.stdout);
    assert!(json_stdout.contains("\"ready_to_run\":true"));
    assert!(json_stdout.contains("\"binary_resolved\":true"));

    let run = run_nuis(&["run-artifact", &output_dir.display().to_string()]);
    assert_success(&run, "nuis run-artifact void-main print project");
    assert!(String::from_utf8_lossy(&run.stdout).contains("exit_status: 0"));
}
