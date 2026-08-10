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
        "payload_policy=scalar-i32-i64-native-staged-fallback-v1",
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
    assert_eq!(yir.matches("cpu.mutex_share ").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_permit ").count(), 2);
    assert_eq!(yir.matches("cpu.mutex_permit_lock").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_lease_value").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_lease_unlock").count(), 1);
    assert_eq!(yir.matches("cpu.mutex_shared_close").count(), 1);
    assert_eq!(yir.matches("cpu.spawn_task").count(), 2);
    for metadata in [
        "authority=linear-permit-lease-v1",
        "permit_cardinality=share-literal-1-to-64-v1",
        "permit_policy=one-shot-generation-bound-v1",
        "lifecycle=explicit-close-revoke-v1",
        "mutation=lease-replace-release-epoch-v1",
    ] {
        assert_eq!(
            yir.matches(metadata).count(),
            7,
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
        llvm.matches("call i64 @nuis_scheduler_mutex_shared_close_i64_v1")
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

#[test]
fn lease_replace_is_visible_across_native_task_permits() {
    let output_dir = temp_dir("shared_mutex_replace");
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/task/task_shared_mutex_replace_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("run nuis build for shared mutex replace demo");
    assert_success(&build, "nuis build shared mutex replace demo");

    let yir = read(&output_dir.join("task_shared_mutex_replace_demo.yir"));
    for (instruction, expected) in [
        ("cpu.mutex_share ", 1),
        ("cpu.mutex_permit ", 2),
        ("cpu.mutex_permit_lock", 2),
        ("cpu.mutex_lease_replace", 1),
        ("cpu.mutex_lease_value", 2),
        ("cpu.mutex_lease_unlock", 2),
        ("cpu.mutex_shared_close", 1),
        ("cpu.spawn_task", 2),
    ] {
        assert_eq!(
            yir.matches(instruction).count(),
            expected,
            "unexpected `{instruction}` YIR count"
        );
    }
    assert_eq!(
        yir.matches("mutation=lease-replace-release-epoch-v1")
            .count(),
        11
    );

    let llvm = read(&output_dir.join("task_shared_mutex_replace_demo.ll"));
    for (call, expected) in [
        ("call i64 @nuis_scheduler_mutex_share_i64_v1", 1),
        ("call i64 @nuis_scheduler_mutex_permit_i64_v1", 2),
        ("call i64 @nuis_scheduler_mutex_permit_lock_i64_v1", 2),
        ("call i64 @nuis_scheduler_mutex_lease_replace_i64_v1", 1),
        ("call i64 @nuis_scheduler_mutex_value_i64_v1", 2),
        ("call i64 @nuis_scheduler_mutex_lease_unlock_i64_v1", 2),
        ("call i64 @nuis_scheduler_mutex_shared_close_i64_v1", 1),
        ("call i64 @nuis_scheduler_task_spawn_invoker_i64_v1", 2),
    ] {
        assert_eq!(
            llvm.matches(call).count(),
            expected,
            "unexpected `{call}` LLVM call count"
        );
    }
    assert!(!llvm.contains("deferred lowering for cpu.mutex"));

    let run = Command::new(output_dir.join("task_shared_mutex_replace_demo"))
        .output()
        .expect("run shared mutex replace native binary");
    assert_eq!(run.status.code(), Some(65));
}

#[test]
fn i32_shared_mutex_payload_crosses_native_task_boundaries_without_type_erasure() {
    let output_dir = temp_dir("shared_mutex_i32");
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/task/task_shared_mutex_i32_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("run nuis build for i32 shared mutex demo");
    assert_success(&build, "nuis build i32 shared mutex demo");

    let yir = read(&output_dir.join("task_shared_mutex_i32_demo.yir"));
    assert!(yir.contains("MutexPermit<i32>"), "{yir}");
    for (instruction, expected) in [
        ("cpu.mutex_share ", 1),
        ("cpu.mutex_permit ", 2),
        ("cpu.mutex_permit_lock", 2),
        ("cpu.mutex_lease_replace", 1),
        ("cpu.mutex_lease_value", 2),
        ("cpu.mutex_lease_unlock", 2),
        ("cpu.mutex_shared_close", 1),
        ("cpu.spawn_task", 2),
    ] {
        assert_eq!(yir.matches(instruction).count(), expected, "{instruction}");
    }

    let llvm = read(&output_dir.join("task_shared_mutex_i32_demo.ll"));
    for (call, expected) in [
        ("call i64 @nuis_scheduler_mutex_new_scalar_v1", 1),
        ("call i64 @nuis_scheduler_mutex_value_scalar_v1", 2),
        ("call i64 @nuis_scheduler_mutex_lease_replace_scalar_v1", 1),
        ("call i64 @nuis_scheduler_task_spawn_invoker_i64_v1", 2),
    ] {
        assert_eq!(llvm.matches(call).count(), expected, "{call}");
    }
    assert!(llvm.contains("define i32 @nuis_fn_replace(i64 %arg0, i32 %arg1)"));
    assert!(llvm.contains("define i32 @nuis_fn_observe(i64 %arg0)"));
    assert!(!llvm.contains("deferred lowering for cpu.mutex"));

    let run = Command::new(output_dir.join("task_shared_mutex_i32_demo"))
        .output()
        .expect("run i32 shared mutex native binary");
    assert_eq!(run.status.code(), Some(63));
}

#[test]
fn static_three_lane_cardinality_reaches_native_runtime() {
    let output_dir = temp_dir("shared_mutex_cardinality");
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/task/task_shared_mutex_cardinality_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("run nuis build for shared mutex cardinality demo");
    assert_success(&build, "nuis build shared mutex cardinality demo");

    let yir = read(&output_dir.join("task_shared_mutex_cardinality_demo.yir"));
    for (instruction, expected) in [
        ("cpu.mutex_share ", 1),
        ("cpu.mutex_permit ", 3),
        ("cpu.mutex_permit_lock", 1),
        ("cpu.mutex_lease_value", 1),
        ("cpu.mutex_lease_unlock", 1),
        ("cpu.mutex_shared_close", 1),
        ("cpu.spawn_task", 3),
    ] {
        assert_eq!(
            yir.matches(instruction).count(),
            expected,
            "unexpected `{instruction}` YIR count"
        );
    }
    assert_eq!(
        yir.matches("permit_cardinality=share-literal-1-to-64-v1")
            .count(),
        8
    );

    let llvm = read(&output_dir.join("task_shared_mutex_cardinality_demo.ll"));
    for (call, expected) in [
        ("call i64 @nuis_scheduler_mutex_share_i64_v1", 1),
        ("call i64 @nuis_scheduler_mutex_permit_i64_v1", 3),
        ("call i64 @nuis_scheduler_mutex_permit_lock_i64_v1", 1),
        ("call i64 @nuis_scheduler_mutex_value_i64_v1", 1),
        ("call i64 @nuis_scheduler_mutex_lease_unlock_i64_v1", 1),
        ("call i64 @nuis_scheduler_mutex_shared_close_i64_v1", 1),
        ("call i64 @nuis_scheduler_task_spawn_invoker_i64_v1", 3),
    ] {
        assert_eq!(
            llvm.matches(call).count(),
            expected,
            "unexpected `{call}` LLVM call count"
        );
    }
    let share_call = llvm
        .lines()
        .find(|line| line.contains("call i64 @nuis_scheduler_mutex_share_i64_v1"))
        .expect("static share call");
    assert!(share_call.contains(", i64 3)"));
    assert!(!llvm.contains("deferred lowering for cpu.mutex"));

    let run = Command::new(output_dir.join("task_shared_mutex_cardinality_demo"))
        .output()
        .expect("run shared mutex cardinality native binary");
    assert_eq!(run.status.code(), Some(33));
}

#[test]
fn branch_selected_shared_mutex_capabilities_reach_native_binary_once() {
    let output_dir = temp_dir("shared_mutex_branch");
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args([
            "build",
            "../../examples/projects/task/task_shared_mutex_branch_demo",
            output_dir.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("run nuis build for shared mutex branch demo");
    assert_success(&build, "nuis build shared mutex branch demo");

    let yir = read(&output_dir.join("task_shared_mutex_branch_demo.yir"));
    for instruction in [
        "cpu.mutex_new",
        "cpu.mutex_share ",
        "cpu.mutex_permit ",
        "cpu.mutex_permit_lock",
        "cpu.mutex_lease_replace",
        "cpu.mutex_lease_value",
        "cpu.mutex_lease_unlock",
        "cpu.mutex_shared_close",
    ] {
        assert_eq!(
            yir.matches(instruction).count(),
            1,
            "branch-selected `{instruction}` must reach YIR once"
        );
    }
    assert_eq!(yir.matches("cpu.select").count(), 2);
    assert_eq!(
        yir.matches("permit_cardinality=share-literal-1-to-64-v1")
            .count(),
        7
    );

    let llvm = read(&output_dir.join("task_shared_mutex_branch_demo.ll"));
    for call in [
        "call i64 @nuis_scheduler_mutex_new_i64_v1",
        "call i64 @nuis_scheduler_mutex_share_i64_v1",
        "call i64 @nuis_scheduler_mutex_permit_i64_v1",
        "call i64 @nuis_scheduler_mutex_permit_lock_i64_v1",
        "call i64 @nuis_scheduler_mutex_lease_replace_i64_v1",
        "call i64 @nuis_scheduler_mutex_value_i64_v1",
        "call i64 @nuis_scheduler_mutex_lease_unlock_i64_v1",
        "call i64 @nuis_scheduler_mutex_shared_close_i64_v1",
    ] {
        assert_eq!(
            llvm.matches(call).count(),
            1,
            "branch-selected `{call}` must reach LLVM once"
        );
    }
    let share_call = llvm
        .lines()
        .find(|line| line.contains("call i64 @nuis_scheduler_mutex_share_i64_v1"))
        .expect("branch-selected share call");
    assert!(share_call.contains(", i64 3)"));
    assert!(!llvm.contains("deferred lowering for cpu.mutex"));

    let binary = output_dir.join("task_shared_mutex_branch_demo");
    let left = Command::new(&binary)
        .output()
        .expect("run left shared mutex branch");
    assert_eq!(left.status.code(), Some(25));
    let right = Command::new(&binary)
        .arg("right")
        .output()
        .expect("run right shared mutex branch");
    assert_eq!(right.status.code(), Some(43));
}
