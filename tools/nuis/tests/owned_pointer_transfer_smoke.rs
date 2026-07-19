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
    let dir = std::env::temp_dir().join(format!("nuis_owned_pointer_transfer_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn selected_owned_pointer_transfer_runs_both_native_leaf_paths() {
    let output_dir = temp_dir();
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/memory/selected_owned_pointer_transfer_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("build selected owned pointer transfer demo");
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let yir = read(&output_dir.join("selected_owned_pointer_transfer_demo.yir"));
    assert_eq!(yir.matches("owned_transfer").count(), 2);

    let llvm = read(&output_dir.join("selected_owned_pointer_transfer_demo.ll"));
    assert_eq!(llvm.matches("call ptr @malloc(i64 16)").count(), 1);
    assert!(llvm.contains("call ptr @nuis_fn_consume_left(ptr"));
    assert!(llvm.contains("call ptr @nuis_fn_consume_right(ptr"));
    assert!(llvm.contains("branch_effect_then."));
    assert!(llvm.contains("branch_effect_else."));
    assert!(llvm.contains("branch_effect_merge."));
    assert!(!llvm.contains("deferred lowering for cpu.select_owned_bytes_tree"));

    let binary = output_dir.join("selected_owned_pointer_transfer_demo");
    let right = Command::new(&binary)
        .output()
        .expect("run right transfer leaf");
    assert_eq!(right.status.code(), Some(16));
    assert_eq!(String::from_utf8_lossy(&right.stdout).trim(), "22");

    let left = Command::new(&binary)
        .arg("left")
        .output()
        .expect("run left transfer leaf");
    assert_eq!(left.status.code(), Some(16));
    assert_eq!(String::from_utf8_lossy(&left.stdout).trim(), "21");
}
