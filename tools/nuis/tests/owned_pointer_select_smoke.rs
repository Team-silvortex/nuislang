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
    let dir = std::env::temp_dir().join(format!("nuis_owned_pointer_select_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn owned_pointer_select_runs_as_native_binary() {
    let output_dir = temp_dir();
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/memory/owned_pointer_select_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("build owned pointer select demo");
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let yir = read(&output_dir.join("owned_pointer_select_demo.yir"));
    assert!(yir.contains("cpu.branch_effect"));
    assert!(yir.contains("owned_ptr"));
    assert!(yir.contains("take_ptr_drop_other"));
    assert!(yir.contains("address_kind=node"));
    assert!(yir.contains("address_kind=buffer"));
    assert!(yir.contains("nullable=true"));

    let llvm = read(&output_dir.join("owned_pointer_select_demo.ll"));
    assert!(llvm.contains("phi ptr"));
    assert_eq!(llvm.matches("call void @free(ptr").count(), 9);
    assert!(!llvm.contains("deferred lowering for cpu.branch_effect"));

    let run = Command::new(output_dir.join("owned_pointer_select_demo"))
        .output()
        .expect("run owned pointer select demo");
    assert_eq!(run.status.code(), Some(78));
}
