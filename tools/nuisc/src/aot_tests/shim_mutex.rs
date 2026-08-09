use super::*;

#[test]
fn scheduler_mutex_handles_reject_contention_and_publish_release_epoch() {
    let dir = temp_dir("scheduler_mutex_visibility");
    let source_path = dir.join("scheduler_mutex_visibility.c");
    let binary_path = dir.join("scheduler_mutex_visibility");
    let mut source = String::new();
    crate::aot_c_shim_runtime::append_c_shim_prelude(&mut source, "0", "0", 0);
    crate::aot_c_shim_runtime::append_c_shim_lifecycle_runtime(&mut source);
    crate::aot_c_shim_text_runtime::append_c_shim_text_runtime(&mut source);
    source.push_str(
        r#"
static int64_t shared_mutex = 0;
static int64_t held_guard = 0;

static int64_t hold_mutex_worker(void* context) {
    (void)context;
    held_guard = nuis_scheduler_mutex_lock_i64_v1(shared_mutex);
    if (held_guard == 0) return 0;
    if (nuis_scheduler_mutex_guard_owner_v1(held_guard) != 2) return 0;
    if (nuis_scheduler_mutex_guard_acquire_epoch_v1(held_guard) != 0) return 0;
    return nuis_scheduler_mutex_value_i64_v1(held_guard) == 17 ? 1 : 0;
}

static int64_t contend_mutex_worker(void* context) {
    (void)context;
    if (nuis_scheduler_mutex_try_lock_i64_v1(shared_mutex) != 0) return 0;
    return nuis_scheduler_mutex_rejected_lock_count_get_v1() == 1 ? 1 : 0;
}

static int64_t observe_release_worker(void* context) {
    (void)context;
    int64_t guard = nuis_scheduler_mutex_lock_i64_v1(shared_mutex);
    if (nuis_scheduler_mutex_guard_owner_v1(guard) != 4) return 0;
    if (nuis_scheduler_mutex_guard_acquire_epoch_v1(guard) != 1) return 0;
    if (nuis_scheduler_mutex_value_i64_v1(guard) != 17) return 0;
    return nuis_scheduler_mutex_unlock_i64_v1(guard) == shared_mutex ? 1 : 0;
}

int64_t nuis_yir_entry(void) {
    shared_mutex = nuis_scheduler_mutex_new_i64_v1(17);
    if (shared_mutex == 0 || nuis_scheduler_mutex_live_count_get_v1() != 1) return 10;

    int64_t holder = nuis_scheduler_task_spawn_invoker_i64_v1(
        hold_mutex_worker, NULL
    );
    int64_t contender = nuis_scheduler_task_spawn_invoker_i64_v1(
        contend_mutex_worker, NULL
    );
    if (holder == 0 || contender == 0) return 11;
    if (nuis_scheduler_task_join_state_v1(holder) != 1) return 12;
    if (nuis_scheduler_task_join_state_v1(contender) != 1) return 13;
    if (nuis_scheduler_task_value_i64_v1(holder) != 1) return 14;
    if (nuis_scheduler_task_value_i64_v1(contender) != 1) return 15;

    if (nuis_scheduler_mutex_unlock_i64_v1(held_guard) != shared_mutex) return 16;
    if (nuis_scheduler_mutex_release_epoch_v1(shared_mutex) != 1) return 17;
    if (nuis_scheduler_mutex_try_unlock_i64_v1(held_guard) != 0) return 18;
    if (nuis_scheduler_mutex_rejected_unlock_count_get_v1() != 1) return 19;

    int64_t observer = nuis_scheduler_task_spawn_invoker_i64_v1(
        observe_release_worker, NULL
    );
    if (observer == 0 || nuis_scheduler_task_join_state_v1(observer) != 1) return 20;
    if (nuis_scheduler_task_value_i64_v1(observer) != 1) return 21;
    if (nuis_scheduler_mutex_release_epoch_v1(shared_mutex) != 2) return 22;
    if (nuis_scheduler_mutex_successful_unlock_count_get_v1() != 2) return 23;

    if (nuis_scheduler_mutex_try_unlock_i64_v1(INT64_MAX) != 0) return 24;
    if (nuis_scheduler_mutex_rejected_unlock_count_get_v1() != 2) return 25;
    if (nuis_lifecycle_shutdown_v1(0) != 0) return 26;
    return nuis_scheduler_mutex_live_count_get_v1() == 0 ? 0 : 27;
}
"#,
    );
    crate::aot_c_shim_runtime::append_c_shim_main(&mut source);
    fs::write(&source_path, source).expect("write scheduler mutex harness");

    let compile = Command::new("clang")
        .arg("-std=c11")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("compile scheduler mutex harness");
    assert!(
        compile.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&binary_path)
        .output()
        .expect("run scheduler mutex harness");
    assert_eq!(
        run.status.code(),
        Some(0),
        "scheduler mutex harness failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn scheduler_shared_mutex_permits_are_lane_bound_and_one_shot() {
    let dir = temp_dir("scheduler_shared_mutex_permit");
    let source_path = dir.join("scheduler_shared_mutex_permit.c");
    let binary_path = dir.join("scheduler_shared_mutex_permit");
    let mut source = String::new();
    crate::aot_c_shim_runtime::append_c_shim_prelude(&mut source, "0", "0", 0);
    crate::aot_c_shim_runtime::append_c_shim_lifecycle_runtime(&mut source);
    crate::aot_c_shim_text_runtime::append_c_shim_text_runtime(&mut source);
    source.push_str(
        r#"
int64_t nuis_yir_entry(void) {
    int64_t shared = nuis_scheduler_mutex_share_i64_v1(
        nuis_scheduler_mutex_new_i64_v1(23)
    );
    int64_t left = nuis_scheduler_mutex_permit_i64_v1(shared, 0);
    int64_t right = nuis_scheduler_mutex_permit_i64_v1(shared, 1);
    if (left == 0 || right == 0) return 10;
    if (left == right) return 11;
    if (nuis_scheduler_mutex_active_permit_count_get_v1(shared) != 2) return 12;

    if (nuis_scheduler_mutex_try_permit_i64_v1(shared, 0) != 0) return 13;
    if (nuis_scheduler_mutex_try_permit_i64_v1(shared, 2) != 0) return 14;
    if (nuis_scheduler_mutex_rejected_permit_count_get_v1() != 2) return 15;

    int64_t left_lease = nuis_scheduler_mutex_permit_lock_i64_v1(left);
    if (nuis_scheduler_mutex_value_i64_v1(left_lease) != 23) return 16;
    if (nuis_scheduler_mutex_active_permit_count_get_v1(shared) != 1) return 17;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(left_lease) != 1) return 18;
    if (nuis_scheduler_mutex_release_epoch_v1(shared) != 1) return 19;
    if (nuis_scheduler_mutex_try_permit_lock_i64_v1(left) != 0) return 20;
    if (nuis_scheduler_mutex_rejected_permit_count_get_v1() != 3) return 21;

    int64_t right_lease = nuis_scheduler_mutex_permit_lock_i64_v1(right);
    if (nuis_scheduler_mutex_value_i64_v1(right_lease) != 23) return 22;
    if (nuis_scheduler_mutex_active_permit_count_get_v1(shared) != 0) return 23;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(right_lease) != 1) return 24;
    if (nuis_scheduler_mutex_release_epoch_v1(shared) != 2) return 25;
    if (nuis_scheduler_mutex_successful_unlock_count_get_v1() != 2) return 26;
    if (nuis_lifecycle_shutdown_v1(0) != 0) return 27;
    return nuis_scheduler_mutex_live_count_get_v1() == 0 ? 0 : 28;
}
"#,
    );
    crate::aot_c_shim_runtime::append_c_shim_main(&mut source);
    fs::write(&source_path, source).expect("write shared mutex permit harness");

    let compile = Command::new("clang")
        .arg("-std=c11")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("compile shared mutex permit harness");
    assert!(
        compile.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&binary_path)
        .output()
        .expect("run shared mutex permit harness");
    assert_eq!(
        run.status.code(),
        Some(0),
        "shared mutex permit harness failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}
