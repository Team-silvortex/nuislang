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

static void nuis_lifecycle_state_reset(void) {
    nuis_lifecycle_state.phase = 0;
    nuis_lifecycle_state.tick_count = 0;
    nuis_lifecycle_state.task_poll_count = 0;
    nuis_lifecycle_state.summary_flush_count = 0;
    nuis_lifecycle_state.network_bridge_progress_count = 0;
    nuis_lifecycle_state.hetero_submission_progress_count = 0;
    nuis_lifecycle_state.last_status = 0;
    nuis_lifecycle_state.yalivia_rpc_enabled = 1;
}

static int64_t nuis_lifecycle_on_bridge_bind_v1(void) {
    return 0;
}

static int64_t nuis_lifecycle_on_scheduler_tick_v1(int64_t tick) {
    return tick;
}

static int64_t nuis_lifecycle_on_task_poll_v1(void) {
    nuis_lifecycle_state.task_poll_count += 1;
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

static int64_t nuis_lifecycle_shutdown_v1(int64_t status) {
    (void)nuis_lifecycle_on_result_commit_v1(status);
    (void)nuis_lifecycle_on_summary_flush_v1();
    (void)nuis_lifecycle_on_shutdown_prepare_v1(status);
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
