use std::{path::Path, process::Command};

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/bootstrap")
        .join(path)
}

#[test]
fn command_bootstrap_cli_accepts_the_compiler_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args([
            "bootstrap-check",
            "--json",
            fixture("accepted/compiler_scanner.ns")
                .to_str()
                .expect("UTF-8 fixture path"),
        ])
        .output()
        .expect("run nuisc bootstrap-check");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"accepted\":true"));
    assert!(stdout.contains("\"semantic_pipeline\":\"checked\""));
    assert!(output.stderr.is_empty());
}

#[test]
fn command_bootstrap_cli_rejects_ffi_with_structured_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args([
            "bootstrap-check",
            "--json",
            fixture("rejected/ffi_address.ns")
                .to_str()
                .expect("UTF-8 fixture path"),
        ])
        .output()
        .expect("run nuisc bootstrap-check");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("\"accepted\":false"));
    assert!(stderr.contains("\"semantic_pipeline\":\"skipped\""));
    assert!(stderr.contains("\"code\":\"NBS003\""));
}
