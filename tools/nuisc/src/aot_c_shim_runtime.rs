pub(crate) fn append_c_shim_prelude(
    out: &mut String,
    network_lifecycle_enabled: &str,
    hetero_lifecycle_enabled: &str,
    hetero_lifecycle_surface_slots: usize,
) {
    out.push_str(
        r#"#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>
#include <fcntl.h>
#include <unistd.h>
#include <time.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <dirent.h>
#include <signal.h>
#include <sys/wait.h>

extern int64_t nuis_yir_entry(void);
static void nuis_host_text_release_all_v1(void);

static int nuis_argc = 0;
static char** nuis_argv = NULL;
static char* nuis_host_text_slots[4096];
static size_t nuis_host_text_slot_lens[4096];
static uint64_t nuis_host_text_slot_hashes[4096];
static int64_t nuis_host_text_intern_table[8192];
static int64_t nuis_host_text_len = 0;
static DIR* nuis_host_dir_slots[256];
static int64_t nuis_host_dir_entry_counts[256];
static int64_t nuis_host_dir_len = 0;
static int nuis_host_network_fds[256];
static int64_t nuis_host_network_fd_kinds[256];
static int64_t nuis_host_network_fd_len = 0;
static pid_t nuis_host_command_pids[256];
static int64_t nuis_host_command_status_slots[256];
static int nuis_host_command_done[256];
static int nuis_host_command_timed_out[256];
static int64_t nuis_host_command_deadline_ns[256];
static int64_t nuis_host_command_len = 0;
static pid_t nuis_host_subprocess_pids[256];
static int64_t nuis_host_subprocess_status_slots[256];
static int nuis_host_subprocess_done[256];
static int nuis_host_subprocess_timed_out[256];
static int64_t nuis_host_subprocess_deadline_ns[256];
static int64_t nuis_host_subprocess_len = 0;
static int64_t nuis_scheduler_task_payloads[256];
static int64_t nuis_scheduler_task_states[256];
static int64_t nuis_scheduler_task_ready_ticks[256];
static int64_t nuis_scheduler_task_deadline_ticks[256];
typedef int64_t (*NuisSchedulerTaskInvokerI64)(void*);
typedef void* (*NuisSchedulerPayloadMoveV1)(void*);
typedef void (*NuisSchedulerPayloadDropV1)(void*);
typedef struct {
    void* data;
    int64_t size;
    int64_t alignment;
    uint64_t type_id;
    NuisSchedulerPayloadMoveV1 move_hook;
    NuisSchedulerPayloadDropV1 drop_hook;
} NuisSchedulerOwnedPayloadV1;
typedef struct {
    int64_t kind;
    NuisSchedulerTaskInvokerI64 invoker;
    void* context;
    NuisSchedulerOwnedPayloadV1 owned_template;
} NuisSchedulerTaskThunkPacket;
static NuisSchedulerTaskThunkPacket nuis_scheduler_task_thunk_packets[256];
static NuisSchedulerOwnedPayloadV1 nuis_scheduler_task_owned_payloads[256];
static int64_t nuis_scheduler_task_payload_kinds[256];
static int64_t nuis_scheduler_task_len = 0;
"#,
    );
    out.push_str(&format!(
        "static int64_t nuis_lifecycle_network_enabled = {network_lifecycle_enabled};\n"
    ));
    out.push_str(&format!(
        "static int64_t nuis_lifecycle_hetero_enabled = {hetero_lifecycle_enabled};\n"
    ));
    out.push_str(&format!(
        "static int64_t nuis_lifecycle_hetero_surface_slots = {hetero_lifecycle_surface_slots};\n"
    ));
}

pub(crate) fn append_c_shim_lifecycle_runtime(out: &mut String) {
    out.push_str(
        r#"

typedef struct {
    int64_t phase;
    int64_t tick_count;
    int64_t task_poll_count;
    int64_t summary_flush_count;
    int64_t network_bridge_progress_count;
    int64_t hetero_submission_progress_count;
    int64_t last_status;
    int64_t yalivia_rpc_enabled;
} NuisLifecycleState;

static NuisLifecycleState nuis_lifecycle_state = {0, 0, 0, 0, 0, 0, 0, 1};

static NuisSchedulerOwnedPayloadV1 nuis_scheduler_empty_owned_payload_v1(void) {
    NuisSchedulerOwnedPayloadV1 payload = {0};
    return payload;
}

static int nuis_scheduler_owned_payload_valid_v1(
    const NuisSchedulerOwnedPayloadV1* payload
) {
    if (payload == NULL || payload->data == NULL || payload->size <= 0) return 0;
    if (payload->alignment <= 0
        || (payload->alignment & (payload->alignment - 1)) != 0) return 0;
    if (payload->type_id == 0 || payload->drop_hook == NULL) return 0;
    return 1;
}

void nuis_scheduler_payload_free_v1(void* data) {
    free(data);
}

void nuis_scheduler_owned_payload_drop_v1(NuisSchedulerOwnedPayloadV1* payload) {
    if (payload == NULL) return;
    if (payload->data != NULL && payload->drop_hook != NULL) {
        payload->drop_hook(payload->data);
    }
    *payload = nuis_scheduler_empty_owned_payload_v1();
}

static void nuis_scheduler_task_release_owned_payload_v1(int64_t index) {
    if (index < 0 || index >= 256) return;
    NuisSchedulerOwnedPayloadV1* payload = &nuis_scheduler_task_owned_payloads[index];
    if (nuis_scheduler_task_payload_kinds[index] == 1) {
        nuis_scheduler_owned_payload_drop_v1(payload);
    } else {
        *payload = nuis_scheduler_empty_owned_payload_v1();
    }
    nuis_scheduler_task_payload_kinds[index] = 0;
}

static void nuis_scheduler_task_release_context_v1(int64_t index) {
    NuisSchedulerTaskThunkPacket* packet = &nuis_scheduler_task_thunk_packets[index];
    free(packet->context);
    packet->kind = 0;
    packet->invoker = NULL;
    packet->context = NULL;
    packet->owned_template = nuis_scheduler_empty_owned_payload_v1();
}

static void nuis_lifecycle_state_reset(void) {
    for (int64_t index = 0; index < nuis_scheduler_task_len; index += 1) {
        nuis_scheduler_task_release_context_v1(index);
        nuis_scheduler_task_release_owned_payload_v1(index);
    }
    nuis_host_text_release_all_v1();
    nuis_lifecycle_state.phase = 0;
    nuis_lifecycle_state.tick_count = 0;
    nuis_lifecycle_state.task_poll_count = 0;
    nuis_lifecycle_state.summary_flush_count = 0;
    nuis_lifecycle_state.network_bridge_progress_count = 0;
    nuis_lifecycle_state.hetero_submission_progress_count = 0;
    nuis_lifecycle_state.last_status = 0;
    nuis_lifecycle_state.yalivia_rpc_enabled = 1;
    nuis_scheduler_task_len = 0;
}

static int64_t nuis_lifecycle_on_bridge_bind_v1(void) {
    return 0;
}

static int64_t nuis_lifecycle_on_scheduler_tick_v1(int64_t tick) {
    return tick;
}

static int64_t nuis_scheduler_task_execute_thunk_v1(int64_t index) {
    NuisSchedulerTaskThunkPacket* packet = &nuis_scheduler_task_thunk_packets[index];
    if ((packet->kind != 1 && packet->kind != 2) || packet->invoker == NULL) {
        return nuis_scheduler_task_payloads[index];
    }
    int64_t kind = packet->kind;
    NuisSchedulerTaskInvokerI64 invoker = packet->invoker;
    void* context = packet->context;
    NuisSchedulerOwnedPayloadV1 owned_template = packet->owned_template;
    packet->kind = 0;
    packet->invoker = NULL;
    packet->context = NULL;
    packet->owned_template = nuis_scheduler_empty_owned_payload_v1();
    int64_t result = invoker(context);
    free(context);
    if (kind == 2 && result != 0) {
        owned_template.data = (void*)(uintptr_t)result;
        nuis_scheduler_task_owned_payloads[index] = owned_template;
        nuis_scheduler_task_payload_kinds[index] = 1;
        return 0;
    }
    if (kind == 2) {
        nuis_scheduler_task_states[index] = 4;
        return 0;
    }
    return result;
}

static int64_t nuis_lifecycle_on_task_poll_v1(void) {
    nuis_lifecycle_state.task_poll_count += 1;
    for (int64_t index = 0; index < nuis_scheduler_task_len; index += 1) {
        if (nuis_scheduler_task_states[index] == 0
            && nuis_scheduler_task_ready_ticks[index] <= nuis_lifecycle_state.tick_count) {
            nuis_scheduler_task_payloads[index] =
                nuis_scheduler_task_execute_thunk_v1(index);
            if (nuis_scheduler_task_states[index] == 0) {
                nuis_scheduler_task_states[index] = 1;
            }
        } else if (nuis_scheduler_task_states[index] == 0
            && nuis_scheduler_task_deadline_ticks[index] >= 0
            && nuis_scheduler_task_deadline_ticks[index] <= nuis_lifecycle_state.tick_count) {
            nuis_scheduler_task_release_context_v1(index);
            nuis_scheduler_task_release_owned_payload_v1(index);
            nuis_scheduler_task_states[index] = 2;
        }
    }
    return nuis_lifecycle_state.task_poll_count;
}

static int64_t nuis_lifecycle_on_result_commit_v1(int64_t status) {
    nuis_lifecycle_state.last_status = status;
    return status;
}

static int64_t nuis_lifecycle_on_summary_flush_v1(void) {
    nuis_lifecycle_state.summary_flush_count += 1;
    return nuis_lifecycle_state.summary_flush_count;
}

static int64_t nuis_lifecycle_sample_network_bridge_progress_v1(void) {
    return nuis_host_network_fd_len;
}

static int64_t nuis_lifecycle_on_network_bridge_progress_v1(void) {
    if (nuis_lifecycle_network_enabled == 0) return 0;
    int64_t observed = nuis_lifecycle_sample_network_bridge_progress_v1();
    if (observed > nuis_lifecycle_state.network_bridge_progress_count) {
        nuis_lifecycle_state.network_bridge_progress_count = observed;
    } else if (observed > 0) {
        nuis_lifecycle_state.network_bridge_progress_count += 1;
    }
    return nuis_lifecycle_state.network_bridge_progress_count;
}

static int64_t nuis_lifecycle_sample_hetero_submission_progress_v1(void) {
    return nuis_lifecycle_hetero_surface_slots;
}

static int64_t nuis_lifecycle_on_hetero_submission_progress_v1(void) {
    if (nuis_lifecycle_hetero_enabled == 0) return 0;
    int64_t observed = nuis_lifecycle_sample_hetero_submission_progress_v1();
    if (observed > nuis_lifecycle_state.hetero_submission_progress_count) {
        nuis_lifecycle_state.hetero_submission_progress_count = observed;
    } else if (observed > 0) {
        nuis_lifecycle_state.hetero_submission_progress_count += 1;
    }
    return nuis_lifecycle_state.hetero_submission_progress_count;
}

static int64_t nuis_lifecycle_on_managed_rpc_v1(void) {
    return nuis_lifecycle_state.yalivia_rpc_enabled;
}

static int64_t nuis_lifecycle_on_shutdown_prepare_v1(int64_t status) {
    nuis_lifecycle_state.last_status = status;
    return status;
}

static int64_t nuis_lifecycle_bootstrap_entry_v1(void) {
    nuis_lifecycle_state_reset();
    nuis_lifecycle_state.phase = 1;
    (void)nuis_lifecycle_on_bridge_bind_v1();
    (void)nuis_lifecycle_on_managed_rpc_v1();
    return 0;
}

static int64_t nuis_lifecycle_tick_once_v1(void) {
    if (nuis_lifecycle_state.phase == 0) return 0;
    if (nuis_lifecycle_state.phase == 3) return nuis_lifecycle_state.last_status;
    nuis_lifecycle_state.phase = 2;
    nuis_lifecycle_state.tick_count += 1;
    (void)nuis_lifecycle_on_scheduler_tick_v1(nuis_lifecycle_state.tick_count);
    (void)nuis_lifecycle_on_task_poll_v1();
    (void)nuis_lifecycle_on_network_bridge_progress_v1();
    (void)nuis_lifecycle_on_hetero_submission_progress_v1();
    return nuis_lifecycle_state.tick_count;
}

int64_t nuis_scheduler_task_spawn_i64_v1(int64_t payload) {
    if (nuis_scheduler_task_len >= 256) return 0;
    int64_t index = nuis_scheduler_task_len;
    nuis_scheduler_task_len += 1;
    nuis_scheduler_task_payloads[index] = payload;
    nuis_scheduler_task_states[index] = 0;
    nuis_scheduler_task_ready_ticks[index] = nuis_lifecycle_state.tick_count + 1;
    nuis_scheduler_task_deadline_ticks[index] = -1;
    nuis_scheduler_task_thunk_packets[index].kind = 0;
    nuis_scheduler_task_thunk_packets[index].invoker = NULL;
    nuis_scheduler_task_thunk_packets[index].context = NULL;
    nuis_scheduler_task_thunk_packets[index].owned_template = nuis_scheduler_empty_owned_payload_v1();
    nuis_scheduler_task_owned_payloads[index] = nuis_scheduler_empty_owned_payload_v1();
    nuis_scheduler_task_payload_kinds[index] = 0;
    return index + 1;
}

int64_t nuis_scheduler_task_spawn_owned_v1(
    const NuisSchedulerOwnedPayloadV1* payload_ref
) {
    if (!nuis_scheduler_owned_payload_valid_v1(payload_ref)) return 0;
    NuisSchedulerOwnedPayloadV1 payload = *payload_ref;
    if (nuis_scheduler_task_len >= 256) {
        nuis_scheduler_owned_payload_drop_v1(&payload);
        return 0;
    }
    void* owned_data = payload.data;
    if (payload.move_hook != NULL) {
        owned_data = payload.move_hook(payload.data);
        if (owned_data == NULL) return 0;
    }
    int64_t index = nuis_scheduler_task_len;
    nuis_scheduler_task_len += 1;
    payload.data = owned_data;
    nuis_scheduler_task_payloads[index] = 0;
    nuis_scheduler_task_states[index] = 0;
    nuis_scheduler_task_ready_ticks[index] = nuis_lifecycle_state.tick_count + 1;
    nuis_scheduler_task_deadline_ticks[index] = -1;
    nuis_scheduler_task_thunk_packets[index].kind = 0;
    nuis_scheduler_task_thunk_packets[index].invoker = NULL;
    nuis_scheduler_task_thunk_packets[index].context = NULL;
    nuis_scheduler_task_thunk_packets[index].owned_template = nuis_scheduler_empty_owned_payload_v1();
    nuis_scheduler_task_owned_payloads[index] = payload;
    nuis_scheduler_task_payload_kinds[index] = 1;
    return index + 1;
}

int64_t nuis_scheduler_task_spawn_invoker_i64_v1(
    NuisSchedulerTaskInvokerI64 invoker,
    void* context
) {
    if (invoker == NULL || nuis_scheduler_task_len >= 256) {
        free(context);
        return 0;
    }
    int64_t index = nuis_scheduler_task_len;
    nuis_scheduler_task_len += 1;
    nuis_scheduler_task_payloads[index] = 0;
    nuis_scheduler_task_states[index] = 0;
    nuis_scheduler_task_ready_ticks[index] = nuis_lifecycle_state.tick_count + 1;
    nuis_scheduler_task_deadline_ticks[index] = -1;
    nuis_scheduler_task_thunk_packets[index].kind = 1;
    nuis_scheduler_task_thunk_packets[index].invoker = invoker;
    nuis_scheduler_task_thunk_packets[index].context = context;
    nuis_scheduler_task_thunk_packets[index].owned_template = nuis_scheduler_empty_owned_payload_v1();
    nuis_scheduler_task_owned_payloads[index] = nuis_scheduler_empty_owned_payload_v1();
    nuis_scheduler_task_payload_kinds[index] = 0;
    return index + 1;
}

int64_t nuis_scheduler_task_spawn_owned_invoker_v1(
    NuisSchedulerTaskInvokerI64 invoker,
    void* context,
    int64_t size,
    int64_t alignment,
    uint64_t type_id,
    NuisSchedulerPayloadDropV1 drop_hook
) {
    if (invoker == NULL || size <= 0 || alignment <= 0
        || (alignment & (alignment - 1)) != 0
        || type_id == 0 || drop_hook == NULL
        || nuis_scheduler_task_len >= 256) {
        free(context);
        return 0;
    }
    int64_t index = nuis_scheduler_task_len;
    nuis_scheduler_task_len += 1;
    nuis_scheduler_task_payloads[index] = 0;
    nuis_scheduler_task_states[index] = 0;
    nuis_scheduler_task_ready_ticks[index] = nuis_lifecycle_state.tick_count + 1;
    nuis_scheduler_task_deadline_ticks[index] = -1;
    nuis_scheduler_task_thunk_packets[index].kind = 2;
    nuis_scheduler_task_thunk_packets[index].invoker = invoker;
    nuis_scheduler_task_thunk_packets[index].context = context;
    nuis_scheduler_task_thunk_packets[index].owned_template = (NuisSchedulerOwnedPayloadV1) {
        .data = NULL,
        .size = size,
        .alignment = alignment,
        .type_id = type_id,
        .move_hook = NULL,
        .drop_hook = drop_hook,
    };
    nuis_scheduler_task_owned_payloads[index] = nuis_scheduler_empty_owned_payload_v1();
    nuis_scheduler_task_payload_kinds[index] = 0;
    return index + 1;
}

void nuis_scheduler_task_ready_after_v1(int64_t task_handle, int64_t delay) {
    if (task_handle <= 0 || task_handle > nuis_scheduler_task_len) return;
    int64_t index = task_handle - 1;
    if (nuis_scheduler_task_states[index] != 0) return;
    if (delay < 0) delay = 0;
    if (delay > INT64_MAX - nuis_lifecycle_state.tick_count) {
        nuis_scheduler_task_ready_ticks[index] = INT64_MAX;
    } else {
        nuis_scheduler_task_ready_ticks[index] = nuis_lifecycle_state.tick_count + delay;
    }
}

void nuis_scheduler_task_timeout_v1(int64_t task_handle, int64_t limit) {
    if (task_handle <= 0 || task_handle > nuis_scheduler_task_len) return;
    int64_t index = task_handle - 1;
    if (nuis_scheduler_task_states[index] != 0) return;
    if (limit <= 0) {
        nuis_scheduler_task_release_context_v1(index);
        nuis_scheduler_task_release_owned_payload_v1(index);
        nuis_scheduler_task_states[index] = 2;
        return;
    }
    if (limit > INT64_MAX - nuis_lifecycle_state.tick_count) {
        nuis_scheduler_task_deadline_ticks[index] = INT64_MAX;
    } else {
        nuis_scheduler_task_deadline_ticks[index] = nuis_lifecycle_state.tick_count + limit;
    }
}

void nuis_scheduler_task_cancel_v1(int64_t task_handle) {
    if (task_handle <= 0 || task_handle > nuis_scheduler_task_len) return;
    int64_t index = task_handle - 1;
    if (nuis_scheduler_task_states[index] == 0) {
        nuis_scheduler_task_release_context_v1(index);
        nuis_scheduler_task_release_owned_payload_v1(index);
        nuis_scheduler_task_states[index] = 3;
    }
}

int64_t nuis_scheduler_task_join_state_v1(int64_t task_handle) {
    if (task_handle <= 0 || task_handle > nuis_scheduler_task_len) return 3;
    int64_t index = task_handle - 1;
    while (nuis_scheduler_task_states[index] == 0) {
        (void)nuis_lifecycle_tick_once_v1();
    }
    return nuis_scheduler_task_states[index];
}

void nuis_scheduler_task_require_completed_v1(int64_t task_handle) {
    int64_t state = nuis_scheduler_task_join_state_v1(task_handle);
    if (state == 1) return;
    fprintf(stderr, "nuis: direct task join reached terminal state %lld\n", (long long)state);
    exit(70);
}

int64_t nuis_scheduler_task_value_i64_v1(int64_t task_handle) {
    if (task_handle <= 0 || task_handle > nuis_scheduler_task_len) return 0;
    int64_t index = task_handle - 1;
    if (nuis_scheduler_task_states[index] != 1) return 0;
    return nuis_scheduler_task_payloads[index];
}

int64_t nuis_scheduler_task_take_owned_v1(
    int64_t task_handle,
    NuisSchedulerOwnedPayloadV1* out_payload
) {
    if (out_payload == NULL) return 0;
    *out_payload = nuis_scheduler_empty_owned_payload_v1();
    if (task_handle <= 0 || task_handle > nuis_scheduler_task_len) return 0;
    int64_t index = task_handle - 1;
    if (nuis_scheduler_task_states[index] != 1
        || nuis_scheduler_task_payload_kinds[index] != 1) return 0;
    *out_payload = nuis_scheduler_task_owned_payloads[index];
    nuis_scheduler_task_owned_payloads[index] = nuis_scheduler_empty_owned_payload_v1();
    nuis_scheduler_task_payload_kinds[index] = 0;
    return 1;
}

static int64_t nuis_lifecycle_shutdown_v1(int64_t status) {
    (void)nuis_lifecycle_on_result_commit_v1(status);
    (void)nuis_lifecycle_on_summary_flush_v1();
    (void)nuis_lifecycle_on_shutdown_prepare_v1(status);
    for (int64_t index = 0; index < nuis_scheduler_task_len; index += 1) {
        nuis_scheduler_task_release_context_v1(index);
        nuis_scheduler_task_release_owned_payload_v1(index);
    }
    nuis_host_text_release_all_v1();
    nuis_lifecycle_state.phase = 3;
    nuis_lifecycle_state.last_status = status;
    return status;
}

static int64_t nuis_lifecycle_yalivia_rpc_hook_v1(void) {
    return nuis_lifecycle_state.yalivia_rpc_enabled;
}
"#,
    );
}

pub(crate) fn append_c_shim_main(out: &mut String) {
    out.push_str(
        r#"

int main(int argc, char** argv) {
    nuis_argc = argc;
    nuis_argv = argv;
    if (nuis_lifecycle_bootstrap_entry_v1() != 0) {
        return 1;
    }
    int64_t entry_status = nuis_yir_entry();
    (void)nuis_lifecycle_tick_once_v1();
    return (int)nuis_lifecycle_shutdown_v1(entry_status);
}
"#,
    );
}
