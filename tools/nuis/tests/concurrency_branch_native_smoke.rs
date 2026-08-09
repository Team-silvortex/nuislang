use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_{label}_{nonce}"));
    fs::create_dir_all(&dir).expect("create concurrency native smoke directory");
    dir
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

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

#[test]
fn branch_selected_cancel_and_unlock_reach_native_binary_once_per_chain() {
    let output_dir = temp_dir("branch_cancel_unlock");
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/task/task_branch_cancel_unlock_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("run nuis build for branch cancel/unlock demo");
    assert_success(&build, "nuis build branch cancel/unlock demo");

    let yir = read(&output_dir.join("task_branch_cancel_unlock_demo.yir"));
    assert_eq!(yir.matches("cpu.mutex_new").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_lock").count(), 2);
    assert_eq!(yir.matches("cpu.mutex_unlock").count(), 2);
    assert_eq!(yir.matches("cpu.spawn_task").count(), 1);
    assert_eq!(yir.matches("cpu.cancel").count(), 1);
    assert_eq!(yir.matches("cpu.join_result").count(), 1);
    for metadata in [
        "mutex_contract=scheduler-handle-v1",
        "visibility=acquire-release-epoch-v1",
        "authority=linear-guard-v1",
        "payload_policy=i64-native-staged-fallback",
    ] {
        assert_eq!(
            yir.matches(metadata).count(),
            6,
            "every mutex YIR node must carry `{metadata}`"
        );
    }

    let llvm = read(&output_dir.join("task_branch_cancel_unlock_demo.ll"));
    assert_eq!(
        llvm.matches("call void @nuis_scheduler_task_cancel_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_task_spawn_invoker_i64_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_task_join_state_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_new_i64_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_lock_i64_v1")
            .count(),
        2
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_unlock_i64_v1")
            .count(),
        2
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_value_i64_v1")
            .count(),
        1
    );
    assert!(!llvm.contains("deferred lowering for cpu.mutex"));

    let binary = output_dir.join("task_branch_cancel_unlock_demo");
    let left = Command::new(&binary)
        .output()
        .expect("run left native branch");
    assert_eq!(left.status.code(), Some(81));
    let right = Command::new(&binary)
        .arg("right")
        .output()
        .expect("run right native branch");
    assert_eq!(right.status.code(), Some(89));
}

#[test]
fn shared_mutex_permits_cross_two_native_task_boundaries_without_handle_copies() {
    let output_dir = temp_dir("shared_mutex_permits");
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/task/task_shared_mutex_permit_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("run nuis build for shared mutex permit demo");
    assert_success(&build, "nuis build shared mutex permit demo");

    let yir = read(&output_dir.join("task_shared_mutex_permit_demo.yir"));
    assert_eq!(yir.matches("cpu.mutex_share").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_permit ").count(), 2);
    assert_eq!(yir.matches("cpu.mutex_permit_lock").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_lease_value").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_lease_unlock").count(), 1);
    assert_eq!(yir.matches("cpu.spawn_task").count(), 2);
    for metadata in [
        "authority=linear-permit-lease-v1",
        "permit_scope=fixed-two-lane-v1",
        "permit_policy=one-shot-generation-bound-v1",
    ] {
        assert_eq!(
            yir.matches(metadata).count(),
            6,
            "every shared mutex YIR node must carry `{metadata}`"
        );
    }

    let llvm = read(&output_dir.join("task_shared_mutex_permit_demo.ll"));
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_share_i64_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_permit_i64_v1")
            .count(),
        2
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_permit_lock_i64_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_mutex_lease_unlock_i64_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call i64 @nuis_scheduler_task_spawn_invoker_i64_v1")
            .count(),
        2
    );
    assert!(!llvm.contains("deferred lowering for cpu.mutex"));

    let run = Command::new(output_dir.join("task_shared_mutex_permit_demo"))
        .output()
        .expect("run shared mutex permit native binary");
    assert_eq!(run.status.code(), Some(34));
}
