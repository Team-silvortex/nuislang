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
    let dir = std::env::temp_dir().join(format!("nuis_owned_bytes_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn assert_file_contains(path: &Path, needle: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        source.contains(needle),
        "expected {} to contain `{needle}`",
        path.display()
    );
}

#[test]
fn task_owned_bytes_payload_runs_and_drops_native_blob() {
    let output_dir = temp_dir();
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/task/task_owned_bytes_payload_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("build task owned Bytes payload");
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let yir = output_dir.join("task_owned_bytes_payload_demo.yir");
    for instruction in [
        "cpu.copy_buffer_owned",
        "cpu.owned_bytes_len",
        "cpu.drop_owned_bytes",
        "cpu.guard_drop_owned_bytes_return",
        "Packet{bytes:Bytes}",
    ] {
        assert_file_contains(&yir, instruction);
    }

    let llvm = output_dir.join("task_owned_bytes_payload_demo.ll");
    for instruction in [
        "call ptr @nuis_scheduler_owned_blob_copy_v1(ptr",
        "call ptr @nuis_scheduler_owned_aggregate_take_blob_v1(ptr",
        "call void @nuis_scheduler_owned_blob_drop_v1(ptr",
        "guard_drop_bytes_return_then.",
        "guard_drop_bytes_return_cont.",
    ] {
        assert_file_contains(&llvm, instruction);
    }

    let binary = Command::new(output_dir.join("task_owned_bytes_payload_demo"))
        .output()
        .expect("run task owned Bytes payload binary");
    assert_eq!(binary.status.code(), Some(24));
}
