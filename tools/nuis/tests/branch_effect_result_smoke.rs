use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_branch_effect_result_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn branch_action_result_runs_both_native_paths() {
    let output_dir = temp_dir();
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/memory/branch_effect_result_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("build branch effect result demo");
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let yir = read(&output_dir.join("branch_effect_result_demo.yir"));
    assert!(yir.contains("cpu.branch_effect"));

    let llvm = read(&output_dir.join("branch_effect_result_demo.ll"));
    assert!(llvm.contains("phi i64"));
    assert!(!llvm.contains("deferred lowering for cpu.branch_effect"));

    let binary = output_dir.join("branch_effect_result_demo");
    let run = Command::new(&binary)
        .output()
        .expect("run both branch action result paths");
    assert_eq!(run.status.code(), Some(114));
}
