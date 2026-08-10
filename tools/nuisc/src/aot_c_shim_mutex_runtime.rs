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
    int64_t shared;
    uint64_t issued_permit_lanes;
    int64_t active_permits;
} NuisSchedulerMutexSlotV1;

typedef struct {
    int64_t token;
    int64_t mutex_handle;
    uint64_t mutex_generation;
    uint64_t acquire_epoch;
    int64_t owner_worker;
    int64_t active;
} NuisSchedulerMutexGuardSlotV1;

typedef struct {
    int64_t token;
    int64_t mutex_handle;
    uint64_t mutex_generation;
    int64_t lane;
    int64_t active;
} NuisSchedulerMutexPermitSlotV1;

static NuisSchedulerMutexSlotV1
    nuis_scheduler_mutex_slots_v1[NUIS_SCHEDULER_MUTEX_CAPACITY_V1];
static NuisSchedulerMutexGuardSlotV1
    nuis_scheduler_mutex_guard_slots_v1[NUIS_SCHEDULER_MUTEX_CAPACITY_V1];
static NuisSchedulerMutexPermitSlotV1
    nuis_scheduler_mutex_permit_slots_v1[NUIS_SCHEDULER_MUTEX_CAPACITY_V1];
static int64_t nuis_scheduler_mutex_next_handle_v1 = 1;
static int64_t nuis_scheduler_mutex_next_guard_v1 = 1;
static int64_t nuis_scheduler_mutex_next_permit_v1 = 1;
static uint64_t nuis_scheduler_mutex_next_generation_v1 = 1;
static uint64_t nuis_scheduler_mutex_visibility_epoch_v1 = 0;
static int64_t nuis_scheduler_mutex_rejected_lock_count_v1 = 0;
static int64_t nuis_scheduler_mutex_rejected_unlock_count_v1 = 0;
static int64_t nuis_scheduler_mutex_successful_unlock_count_v1 = 0;
static int64_t nuis_scheduler_mutex_rejected_permit_count_v1 = 0;
static int64_t nuis_scheduler_mutex_rejected_close_count_v1 = 0;

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

static NuisSchedulerMutexPermitSlotV1* nuis_scheduler_mutex_permit_slot_v1(
    int64_t token
) {
    if (token <= 0) return NULL;
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        NuisSchedulerMutexPermitSlotV1* slot =
            &nuis_scheduler_mutex_permit_slots_v1[index];
        if (slot->active && slot->token == token) return slot;
    }
    return NULL;
}

static NuisSchedulerMutexPermitSlotV1* nuis_scheduler_mutex_free_permit_slot_v1(void) {
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        NuisSchedulerMutexPermitSlotV1* slot =
            &nuis_scheduler_mutex_permit_slots_v1[index];
        if (!slot->active) return slot;
    }
    return NULL;
}

static void nuis_scheduler_mutex_reset_v1(void) {
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        nuis_scheduler_mutex_slots_v1[index].active = 0;
        nuis_scheduler_mutex_slots_v1[index].locked = 0;
        nuis_scheduler_mutex_slots_v1[index].shared = 0;
        nuis_scheduler_mutex_slots_v1[index].issued_permit_lanes = 0;
        nuis_scheduler_mutex_slots_v1[index].active_permits = 0;
        nuis_scheduler_mutex_guard_slots_v1[index].active = 0;
        nuis_scheduler_mutex_permit_slots_v1[index].active = 0;
    }
    nuis_scheduler_mutex_visibility_epoch_v1 = 0;
    nuis_scheduler_mutex_rejected_lock_count_v1 = 0;
    nuis_scheduler_mutex_rejected_unlock_count_v1 = 0;
    nuis_scheduler_mutex_successful_unlock_count_v1 = 0;
    nuis_scheduler_mutex_rejected_permit_count_v1 = 0;
    nuis_scheduler_mutex_rejected_close_count_v1 = 0;
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
        slot->shared = 0;
        slot->issued_permit_lanes = 0;
        slot->active_permits = 0;
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

int64_t nuis_scheduler_mutex_share_i64_v1(int64_t handle) {
    NuisSchedulerMutexSlotV1* mutex = nuis_scheduler_mutex_slot_v1(handle);
    if (mutex == NULL || mutex->locked || mutex->shared) {
        fprintf(stderr, "nuis: scheduler mutex share rejected for handle %lld\n",
            (long long)handle);
        exit(75);
    }
    mutex->shared = 1;
    mutex->issued_permit_lanes = 0;
    mutex->active_permits = 0;
    return mutex->handle;
}

int64_t nuis_scheduler_mutex_try_shared_close_i64_v1(int64_t handle) {
    NuisSchedulerMutexSlotV1* mutex = nuis_scheduler_mutex_slot_v1(handle);
    if (mutex == NULL || !mutex->shared || mutex->locked) {
        nuis_scheduler_mutex_rejected_close_count_v1 += 1;
        return -1;
    }
    int64_t revoked = 0;
    for (int64_t index = 0; index < NUIS_SCHEDULER_MUTEX_CAPACITY_V1; index += 1) {
        NuisSchedulerMutexPermitSlotV1* permit =
            &nuis_scheduler_mutex_permit_slots_v1[index];
        if (!permit->active || permit->mutex_handle != mutex->handle
            || permit->mutex_generation != mutex->generation) {
            continue;
        }
        permit->active = 0;
        revoked += 1;
    }
    atomic_thread_fence(memory_order_release);
    nuis_scheduler_mutex_visibility_epoch_v1 += 1;
    mutex->release_epoch = nuis_scheduler_mutex_visibility_epoch_v1;
    mutex->issued_permit_lanes = 0;
    mutex->active_permits = 0;
    mutex->shared = 0;
    mutex->active = 0;
    return revoked;
}

int64_t nuis_scheduler_mutex_shared_close_i64_v1(int64_t handle) {
    int64_t revoked = nuis_scheduler_mutex_try_shared_close_i64_v1(handle);
    if (revoked >= 0) return revoked;
    fprintf(stderr, "nuis: scheduler shared mutex close rejected for handle %lld\n",
        (long long)handle);
    exit(79);
}

int64_t nuis_scheduler_mutex_try_permit_i64_v1(int64_t handle, int64_t lane) {
    NuisSchedulerMutexSlotV1* mutex = nuis_scheduler_mutex_slot_v1(handle);
    NuisSchedulerMutexPermitSlotV1* permit =
        nuis_scheduler_mutex_free_permit_slot_v1();
    uint64_t lane_bit = lane >= 0 && lane <= 1 ? ((uint64_t)1 << lane) : 0;
    if (mutex == NULL || !mutex->shared || lane_bit == 0
        || (mutex->issued_permit_lanes & lane_bit) != 0 || permit == NULL
        || nuis_scheduler_mutex_next_permit_v1 == INT64_MAX) {
        nuis_scheduler_mutex_rejected_permit_count_v1 += 1;
        return 0;
    }
    permit->token = nuis_scheduler_mutex_next_permit_v1++;
    permit->mutex_handle = mutex->handle;
    permit->mutex_generation = mutex->generation;
    permit->lane = lane;
    permit->active = 1;
    mutex->issued_permit_lanes |= lane_bit;
    mutex->active_permits += 1;
    return permit->token;
}

int64_t nuis_scheduler_mutex_permit_i64_v1(int64_t handle, int64_t lane) {
    int64_t permit = nuis_scheduler_mutex_try_permit_i64_v1(handle, lane);
    if (permit != 0) return permit;
    fprintf(stderr, "nuis: scheduler mutex permit rejected for handle %lld lane %lld\n",
        (long long)handle, (long long)lane);
    exit(76);
}

int64_t nuis_scheduler_mutex_try_permit_lock_i64_v1(int64_t permit_token) {
    NuisSchedulerMutexPermitSlotV1* permit =
        nuis_scheduler_mutex_permit_slot_v1(permit_token);
    NuisSchedulerMutexSlotV1* mutex = permit == NULL
        ? NULL
        : nuis_scheduler_mutex_slot_v1(permit->mutex_handle);
    if (permit == NULL || mutex == NULL || !mutex->shared
        || mutex->generation != permit->mutex_generation) {
        nuis_scheduler_mutex_rejected_permit_count_v1 += 1;
        return 0;
    }
    int64_t guard = nuis_scheduler_mutex_try_lock_i64_v1(mutex->handle);
    if (guard == 0) return 0;
    permit->active = 0;
    mutex->active_permits -= 1;
    return guard;
}

int64_t nuis_scheduler_mutex_permit_lock_i64_v1(int64_t permit_token) {
    int64_t guard = nuis_scheduler_mutex_try_permit_lock_i64_v1(permit_token);
    if (guard != 0) return guard;
    fprintf(stderr, "nuis: scheduler mutex permit lock rejected for token %lld\n",
        (long long)permit_token);
    exit(77);
}

int64_t nuis_scheduler_mutex_lease_unlock_i64_v1(int64_t guard_token) {
    int64_t handle = nuis_scheduler_mutex_try_unlock_i64_v1(guard_token);
    if (handle != 0) return 1;
    fprintf(stderr, "nuis: scheduler mutex lease unlock rejected for guard %lld\n",
        (long long)guard_token);
    exit(78);
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

int64_t nuis_scheduler_mutex_rejected_permit_count_get_v1(void) {
    return nuis_scheduler_mutex_rejected_permit_count_v1;
}

int64_t nuis_scheduler_mutex_rejected_close_count_get_v1(void) {
    return nuis_scheduler_mutex_rejected_close_count_v1;
}

int64_t nuis_scheduler_mutex_active_permit_count_get_v1(int64_t handle) {
    NuisSchedulerMutexSlotV1* mutex = nuis_scheduler_mutex_slot_v1(handle);
    return mutex == NULL ? -1 : mutex->active_permits;
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
