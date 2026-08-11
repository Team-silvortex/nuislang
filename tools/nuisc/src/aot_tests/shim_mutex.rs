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
fn scheduler_mutex_scalar_protocol_preserves_signed_i32_bits() {
    let dir = temp_dir("scheduler_mutex_scalar_i32");
    let source_path = dir.join("scheduler_mutex_scalar_i32.c");
    let binary_path = dir.join("scheduler_mutex_scalar_i32");
    let mut source = String::new();
    crate::aot_c_shim_runtime::append_c_shim_prelude(&mut source, "0", "0", 0);
    crate::aot_c_shim_runtime::append_c_shim_lifecycle_runtime(&mut source);
    crate::aot_c_shim_text_runtime::append_c_shim_text_runtime(&mut source);
    source.push_str(
        r#"
int64_t nuis_yir_entry(void) {
    if (nuis_scheduler_mutex_new_scalar_v1(1, 99) != 0) return 10;
    int64_t shared = nuis_scheduler_mutex_share_i64_v1(
        nuis_scheduler_mutex_new_scalar_v1(-17, NUIS_SCHEDULER_MUTEX_SCALAR_I32_V1),
        1
    );
    int64_t permit = nuis_scheduler_mutex_permit_i64_v1(shared, 0);
    int64_t lease = nuis_scheduler_mutex_permit_lock_i64_v1(permit);
    if (nuis_scheduler_mutex_value_scalar_v1(
        lease, NUIS_SCHEDULER_MUTEX_SCALAR_I32_V1
    ) != -17) return 11;
    if (nuis_scheduler_mutex_lease_replace_scalar_v1(
        lease, 23, NUIS_SCHEDULER_MUTEX_SCALAR_I32_V1
    ) != -17) return 12;
    if (nuis_scheduler_mutex_value_scalar_v1(
        lease, NUIS_SCHEDULER_MUTEX_SCALAR_I32_V1
    ) != 23) return 13;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(lease) != 1) return 14;
    if (nuis_scheduler_mutex_shared_close_i64_v1(shared) != 0) return 15;
    if (nuis_lifecycle_shutdown_v1(0) != 0) return 16;
    return nuis_scheduler_mutex_live_count_get_v1() == 0 ? 0 : 17;
}
"#,
    );
    crate::aot_c_shim_runtime::append_c_shim_main(&mut source);
    fs::write(&source_path, source).expect("write scalar mutex harness");

    let compile = Command::new("clang")
        .arg("-std=c11")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("compile scalar mutex harness");
    assert!(
        compile.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&binary_path)
        .output()
        .expect("run scalar mutex harness");
    assert_eq!(run.status.code(), Some(0), "{:?}", run);
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
        nuis_scheduler_mutex_new_i64_v1(23), 2
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
    if (nuis_scheduler_mutex_lease_replace_i64_v1(left_lease, 29) != 23) return 16;
    if (nuis_scheduler_mutex_release_epoch_v1(shared) != 1) return 17;
    if (nuis_scheduler_mutex_value_i64_v1(left_lease) != 29) return 18;
    if (nuis_scheduler_mutex_active_permit_count_get_v1(shared) != 1) return 19;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(left_lease) != 1) return 20;
    if (nuis_scheduler_mutex_release_epoch_v1(shared) != 2) return 21;
    if (nuis_scheduler_mutex_try_permit_lock_i64_v1(left) != 0) return 22;
    if (nuis_scheduler_mutex_rejected_permit_count_get_v1() != 3) return 23;

    int64_t right_lease = nuis_scheduler_mutex_permit_lock_i64_v1(right);
    if (nuis_scheduler_mutex_value_i64_v1(right_lease) != 29) return 24;
    if (nuis_scheduler_mutex_active_permit_count_get_v1(shared) != 0) return 25;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(right_lease) != 1) return 26;
    if (nuis_scheduler_mutex_release_epoch_v1(shared) != 3) return 27;
    if (nuis_scheduler_mutex_successful_unlock_count_get_v1() != 2) return 28;
    if (nuis_scheduler_mutex_shared_close_i64_v1(shared) != 0) return 29;
    if (nuis_scheduler_mutex_live_count_get_v1() != 0) return 30;

    int64_t closing = nuis_scheduler_mutex_share_i64_v1(
        nuis_scheduler_mutex_new_i64_v1(31), 2
    );
    int64_t pending = nuis_scheduler_mutex_permit_i64_v1(closing, 0);
    int64_t active = nuis_scheduler_mutex_permit_i64_v1(closing, 1);
    int64_t active_lease = nuis_scheduler_mutex_permit_lock_i64_v1(active);
    if (nuis_scheduler_mutex_try_shared_close_i64_v1(closing) != -1) return 31;
    if (nuis_scheduler_mutex_rejected_close_count_get_v1() != 1) return 32;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(active_lease) != 1) return 33;
    if (nuis_scheduler_mutex_shared_close_i64_v1(closing) != 1) return 34;
    if (nuis_scheduler_mutex_try_permit_lock_i64_v1(pending) != 0) return 35;
    if (nuis_scheduler_mutex_try_permit_i64_v1(closing, 1) != 0) return 36;
    if (nuis_scheduler_mutex_rejected_permit_count_get_v1() != 5) return 37;
    if (nuis_scheduler_mutex_try_shared_close_i64_v1(closing) != -1) return 38;
    if (nuis_scheduler_mutex_rejected_close_count_get_v1() != 2) return 39;
    if (nuis_scheduler_mutex_live_count_get_v1() != 0) return 40;

    int64_t wide = nuis_scheduler_mutex_share_i64_v1(
        nuis_scheduler_mutex_new_i64_v1(41), 3
    );
    if (nuis_scheduler_mutex_permit_cardinality_get_v1(wide) != 3) return 41;
    int64_t third = nuis_scheduler_mutex_permit_i64_v1(wide, 2);
    if (third == 0) return 42;
    if (nuis_scheduler_mutex_try_permit_i64_v1(wide, 3) != 0) return 43;
    if (nuis_scheduler_mutex_rejected_permit_count_get_v1() != 6) return 44;
    int64_t third_lease = nuis_scheduler_mutex_permit_lock_i64_v1(third);
    if (nuis_scheduler_mutex_value_i64_v1(third_lease) != 41) return 45;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(third_lease) != 1) return 46;
    if (nuis_scheduler_mutex_shared_close_i64_v1(wide) != 0) return 47;

    int64_t maximum = nuis_scheduler_mutex_share_i64_v1(
        nuis_scheduler_mutex_new_i64_v1(43), 64
    );
    if (nuis_scheduler_mutex_permit_cardinality_get_v1(maximum) != 64) return 48;
    int64_t last = nuis_scheduler_mutex_permit_i64_v1(maximum, 63);
    if (last == 0) return 49;
    if (nuis_scheduler_mutex_try_permit_i64_v1(maximum, 64) != 0) return 50;
    if (nuis_scheduler_mutex_rejected_permit_count_get_v1() != 7) return 51;
    int64_t last_lease = nuis_scheduler_mutex_permit_lock_i64_v1(last);
    if (nuis_scheduler_mutex_value_i64_v1(last_lease) != 43) return 52;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(last_lease) != 1) return 53;
    if (nuis_scheduler_mutex_shared_close_i64_v1(maximum) != 0) return 54;
    if (nuis_lifecycle_shutdown_v1(0) != 0) return 55;
    return nuis_scheduler_mutex_live_count_get_v1() == 0 ? 0 : 56;
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

#[test]
fn scheduler_mutex_slots_admit_simultaneous_host_threads_without_aliasing() {
    let dir = temp_dir("scheduler_mutex_host_threads");
    let source_path = dir.join("scheduler_mutex_host_threads.c");
    let binary_path = dir.join("scheduler_mutex_host_threads");
    let mut source = String::from("#include <pthread.h>\n");
    crate::aot_c_shim_runtime::append_c_shim_prelude(&mut source, "0", "0", 0);
    crate::aot_c_shim_runtime::append_c_shim_lifecycle_runtime(&mut source);
    crate::aot_c_shim_text_runtime::append_c_shim_text_runtime(&mut source);
    source.push_str(
        r#"
#define NUIS_HOST_MUTEX_WORKERS_V1 32

typedef struct {
    int64_t index;
    int64_t handle;
    int64_t lease;
    int64_t result;
} NuisHostMutexWorkerV1;

static atomic_int nuis_host_mutex_ready_v1 = 0;
static atomic_int nuis_host_mutex_leases_ready_v1 = 0;
static atomic_int nuis_host_mutex_start_v1 = 0;
static atomic_int nuis_host_mutex_release_v1 = 0;

static void* nuis_host_mutex_worker_v1(void* opaque) {
    NuisHostMutexWorkerV1* worker = (NuisHostMutexWorkerV1*)opaque;
    atomic_fetch_add_explicit(&nuis_host_mutex_ready_v1, 1, memory_order_release);
    while (atomic_load_explicit(&nuis_host_mutex_start_v1, memory_order_acquire) == 0) {
    }

    int64_t expected = 1000 + worker->index;
    int64_t handle = nuis_scheduler_mutex_new_i64_v1(expected);
    worker->handle = handle;
    if (handle == 0) {
        worker->result = 10;
    } else {
        int64_t shared = nuis_scheduler_mutex_share_i64_v1(handle, 1);
        int64_t permit = nuis_scheduler_mutex_try_permit_i64_v1(shared, 0);
        int64_t lease = permit == 0
            ? 0
            : nuis_scheduler_mutex_try_permit_lock_i64_v1(permit);
        if (shared != handle || permit == 0 || lease == 0) {
            worker->result = 11;
        } else if (nuis_scheduler_mutex_value_i64_v1(lease) != expected) {
            worker->result = 12;
        } else {
            worker->lease = lease;
        }
    }

    atomic_fetch_add_explicit(
        &nuis_host_mutex_leases_ready_v1,
        1,
        memory_order_release
    );
    while (atomic_load_explicit(&nuis_host_mutex_release_v1, memory_order_acquire) == 0) {
    }

    if (worker->result != 0) return NULL;
    if (nuis_scheduler_mutex_lease_unlock_i64_v1(worker->lease) != 1) {
        worker->result = 13;
    } else if (nuis_scheduler_mutex_try_shared_close_i64_v1(handle) != 0) {
        worker->result = 14;
    } else {
        worker->result = 0;
    }
    return NULL;
}

int64_t nuis_yir_entry(void) {
    alarm(10);
    pthread_t threads[NUIS_HOST_MUTEX_WORKERS_V1];
    NuisHostMutexWorkerV1 workers[NUIS_HOST_MUTEX_WORKERS_V1] = {0};
    int64_t created = 0;
    for (int64_t index = 0; index < NUIS_HOST_MUTEX_WORKERS_V1; index += 1) {
        workers[index].index = index;
        if (pthread_create(
            &threads[index],
            NULL,
            nuis_host_mutex_worker_v1,
            &workers[index]
        ) != 0) {
            atomic_store_explicit(&nuis_host_mutex_start_v1, 1, memory_order_release);
            atomic_store_explicit(&nuis_host_mutex_release_v1, 1, memory_order_release);
            for (int64_t joined = 0; joined < created; joined += 1) {
                pthread_join(threads[joined], NULL);
            }
            return 20;
        }
        created += 1;
    }

    while (atomic_load_explicit(&nuis_host_mutex_ready_v1, memory_order_acquire)
        != NUIS_HOST_MUTEX_WORKERS_V1) {
    }
    atomic_store_explicit(&nuis_host_mutex_start_v1, 1, memory_order_release);
    while (atomic_load_explicit(
        &nuis_host_mutex_leases_ready_v1,
        memory_order_acquire
    ) != NUIS_HOST_MUTEX_WORKERS_V1) {
    }

    int64_t live_while_held = nuis_scheduler_mutex_live_count_get_v1();
    atomic_store_explicit(&nuis_host_mutex_release_v1, 1, memory_order_release);
    for (int64_t index = 0; index < NUIS_HOST_MUTEX_WORKERS_V1; index += 1) {
        if (pthread_join(threads[index], NULL) != 0) return 21;
    }
    if (live_while_held != NUIS_HOST_MUTEX_WORKERS_V1) return 22;

    for (int64_t index = 0; index < NUIS_HOST_MUTEX_WORKERS_V1; index += 1) {
        if (workers[index].result != 0 || workers[index].handle <= 0) return 23;
        for (int64_t other = index + 1; other < NUIS_HOST_MUTEX_WORKERS_V1; other += 1) {
            if (workers[index].handle == workers[other].handle) return 24;
        }
    }
    if (nuis_scheduler_mutex_successful_unlock_count_get_v1()
        != NUIS_HOST_MUTEX_WORKERS_V1) return 25;
    if (nuis_scheduler_mutex_live_count_get_v1() != 0) return 26;
    if (nuis_lifecycle_shutdown_v1(0) != 0) return 27;
    return nuis_scheduler_mutex_live_count_get_v1() == 0 ? 0 : 28;
}
"#,
    );
    crate::aot_c_shim_runtime::append_c_shim_main(&mut source);
    fs::write(&source_path, source).expect("write host-thread mutex harness");

    let compile = Command::new("clang")
        .arg("-std=c11")
        .arg("-pthread")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("compile host-thread mutex harness");
    assert!(
        compile.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&binary_path)
        .output()
        .expect("run host-thread mutex harness");
    assert_eq!(
        run.status.code(),
        Some(0),
        "host-thread mutex harness failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}
