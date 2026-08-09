pub(crate) fn append_c_shim_mutex_runtime(out: &mut String) {
    out.push_str(
        r#"

#define NUIS_SCHEDULER_MUTEX_CAPACITY_V1 256

typedef struct {
    int64_t handle;
    int64_t value;
    uint64_t generation;
    uint64_t release_epoch;
    int64_t active;
    int64_t locked;
} NuisSchedulerMutexSlotV1;

typedef struct {
    int64_t token;
    int64_t mutex_handle;
    uint64_t mutex_generation;
    uint64_t acquire_epoch;
    int64_t owner_worker;
    int64_t active;
} NuisSchedulerMutexGuardSlotV1;

static NuisSchedulerMutexSlotV1
    nuis_scheduler_mutex_slots_v1[NUIS_SCHEDULER_MUTEX_CAPACITY_V1];
static NuisSchedulerMutexGuardSlotV1
    nuis_scheduler_mutex_guard_slots_v1[NUIS_SCHEDULER_MUTEX_CAPACITY_V1];
static int64_t nuis_scheduler_mutex_next_handle_v1 = 1;
static int64_t nuis_scheduler_mutex_next_guard_v1 = 1;
static uint64_t nuis_scheduler_mutex_next_generation_v1 = 1;
static uint64_t nuis_scheduler_mutex_visibility_epoch_v1 = 0;
static int64_t nuis_scheduler_mutex_rejected_lock_count_v1 = 0;
static int64_t nuis_scheduler_mutex_rejected_unlock_count_v1 = 0;
static int64_t nuis_scheduler_mutex_successful_unlock_count_v1 = 0;

static NuisSchedulerMutexSlotV1* nuis_scheduler_mutex_slot_v1(int64_t handle) {
    if (handle <= 0) return NULL;
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        NuisSchedulerMutexSlotV1* slot = &nuis_scheduler_mutex_slots_v1[index];
        if (slot->active && slot->handle == handle) return slot;
    }
    return NULL;
}

static NuisSchedulerMutexGuardSlotV1* nuis_scheduler_mutex_guard_slot_v1(
    int64_t token
) {
    if (token <= 0) return NULL;
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        NuisSchedulerMutexGuardSlotV1* slot =
            &nuis_scheduler_mutex_guard_slots_v1[index];
        if (slot->active && slot->token == token) return slot;
    }
    return NULL;
}

static NuisSchedulerMutexGuardSlotV1* nuis_scheduler_mutex_free_guard_slot_v1(void) {
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        NuisSchedulerMutexGuardSlotV1* slot =
            &nuis_scheduler_mutex_guard_slots_v1[index];
        if (!slot->active) return slot;
    }
    return NULL;
}

static void nuis_scheduler_mutex_reset_v1(void) {
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        nuis_scheduler_mutex_slots_v1[index].active = 0;
        nuis_scheduler_mutex_slots_v1[index].locked = 0;
        nuis_scheduler_mutex_guard_slots_v1[index].active = 0;
    }
    nuis_scheduler_mutex_visibility_epoch_v1 = 0;
    nuis_scheduler_mutex_rejected_lock_count_v1 = 0;
    nuis_scheduler_mutex_rejected_unlock_count_v1 = 0;
    nuis_scheduler_mutex_successful_unlock_count_v1 = 0;
}

int64_t nuis_scheduler_mutex_new_i64_v1(int64_t value) {
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        NuisSchedulerMutexSlotV1* slot = &nuis_scheduler_mutex_slots_v1[index];
        if (slot->active) continue;
        if (nuis_scheduler_mutex_next_handle_v1 == INT64_MAX) return 0;
        slot->handle = nuis_scheduler_mutex_next_handle_v1++;
        slot->value = value;
        slot->generation = nuis_scheduler_mutex_next_generation_v1++;
        slot->release_epoch = nuis_scheduler_mutex_visibility_epoch_v1;
        slot->active = 1;
        slot->locked = 0;
        return slot->handle;
    }
    return 0;
}

int64_t nuis_scheduler_mutex_try_lock_i64_v1(int64_t handle) {
    NuisSchedulerMutexSlotV1* mutex = nuis_scheduler_mutex_slot_v1(handle);
    NuisSchedulerMutexGuardSlotV1* guard = nuis_scheduler_mutex_free_guard_slot_v1();
    if (mutex == NULL || mutex->locked || guard == NULL
        || nuis_scheduler_mutex_next_guard_v1 == INT64_MAX) {
        nuis_scheduler_mutex_rejected_lock_count_v1 += 1;
        return 0;
    }
    mutex->locked = 1;
    atomic_thread_fence(memory_order_acquire);
    guard->token = nuis_scheduler_mutex_next_guard_v1++;
    guard->mutex_handle = mutex->handle;
    guard->mutex_generation = mutex->generation;
    guard->acquire_epoch = mutex->release_epoch;
    guard->owner_worker = nuis_scheduler_current_worker_id_v1;
    guard->active = 1;
    return guard->token;
}

int64_t nuis_scheduler_mutex_lock_i64_v1(int64_t handle) {
    int64_t guard = nuis_scheduler_mutex_try_lock_i64_v1(handle);
    if (guard != 0) return guard;
    fprintf(stderr, "nuis: scheduler mutex lock rejected for handle %lld\n",
        (long long)handle);
    exit(72);
}

int64_t nuis_scheduler_mutex_value_i64_v1(int64_t guard_token) {
    NuisSchedulerMutexGuardSlotV1* guard =
        nuis_scheduler_mutex_guard_slot_v1(guard_token);
    NuisSchedulerMutexSlotV1* mutex = guard == NULL
        ? NULL
        : nuis_scheduler_mutex_slot_v1(guard->mutex_handle);
    if (guard == NULL || mutex == NULL || !mutex->locked
        || mutex->generation != guard->mutex_generation) {
        fprintf(stderr, "nuis: scheduler mutex value rejected for guard %lld\n",
            (long long)guard_token);
        exit(73);
    }
    return mutex->value;
}

int64_t nuis_scheduler_mutex_try_unlock_i64_v1(int64_t guard_token) {
    NuisSchedulerMutexGuardSlotV1* guard =
        nuis_scheduler_mutex_guard_slot_v1(guard_token);
    NuisSchedulerMutexSlotV1* mutex = guard == NULL
        ? NULL
        : nuis_scheduler_mutex_slot_v1(guard->mutex_handle);
    if (guard == NULL || mutex == NULL || !mutex->locked
        || mutex->generation != guard->mutex_generation) {
        nuis_scheduler_mutex_rejected_unlock_count_v1 += 1;
        return 0;
    }
    atomic_thread_fence(memory_order_release);
    nuis_scheduler_mutex_visibility_epoch_v1 += 1;
    mutex->release_epoch = nuis_scheduler_mutex_visibility_epoch_v1;
    mutex->locked = 0;
    guard->active = 0;
    nuis_scheduler_mutex_successful_unlock_count_v1 += 1;
    return mutex->handle;
}

int64_t nuis_scheduler_mutex_unlock_i64_v1(int64_t guard_token) {
    int64_t handle = nuis_scheduler_mutex_try_unlock_i64_v1(guard_token);
    if (handle != 0) return handle;
    fprintf(stderr, "nuis: scheduler mutex unlock rejected for guard %lld\n",
        (long long)guard_token);
    exit(74);
}

int64_t nuis_scheduler_mutex_guard_owner_v1(int64_t guard_token) {
    NuisSchedulerMutexGuardSlotV1* guard =
        nuis_scheduler_mutex_guard_slot_v1(guard_token);
    return guard == NULL ? 0 : guard->owner_worker;
}

int64_t nuis_scheduler_mutex_guard_acquire_epoch_v1(int64_t guard_token) {
    NuisSchedulerMutexGuardSlotV1* guard =
        nuis_scheduler_mutex_guard_slot_v1(guard_token);
    return guard == NULL ? -1 : (int64_t)guard->acquire_epoch;
}

int64_t nuis_scheduler_mutex_release_epoch_v1(int64_t handle) {
    NuisSchedulerMutexSlotV1* mutex = nuis_scheduler_mutex_slot_v1(handle);
    return mutex == NULL ? -1 : (int64_t)mutex->release_epoch;
}

int64_t nuis_scheduler_mutex_rejected_lock_count_get_v1(void) {
    return nuis_scheduler_mutex_rejected_lock_count_v1;
}

int64_t nuis_scheduler_mutex_rejected_unlock_count_get_v1(void) {
    return nuis_scheduler_mutex_rejected_unlock_count_v1;
}

int64_t nuis_scheduler_mutex_successful_unlock_count_get_v1(void) {
    return nuis_scheduler_mutex_successful_unlock_count_v1;
}

int64_t nuis_scheduler_mutex_live_count_get_v1(void) {
    int64_t count = 0;
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        if (nuis_scheduler_mutex_slots_v1[index].active) count += 1;
    }
    return count;
}
"#,
    );
}
