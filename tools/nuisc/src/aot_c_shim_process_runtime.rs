pub(crate) fn append_c_shim_process_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_process_id(void) {
    return (int64_t)getpid();
}

static int64_t nuis_host_process_status(void) {
    return 0;
}

static int64_t nuis_host_process_exit_code(int64_t status) {
    int raw = (int)status;
    if (WIFEXITED(raw)) return (int64_t)WEXITSTATUS(raw);
    if (WIFSIGNALED(raw)) return (int64_t)(128 + WTERMSIG(raw));
    return status;
}

static char* nuis_host_build_shell_command(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t env_handle
) {
    const char* program = nuis_host_text_lookup(program_handle);
    const char* argv_text = nuis_host_text_lookup(argv_handle);
    const char* env_text = nuis_host_text_lookup(env_handle);
    if (program == NULL || program[0] == '\0') return NULL;
    int has_argv = argv_text != NULL && argv_text[0] != '\0';
    int has_env = env_text != NULL && env_text[0] != '\0';
    size_t program_len = strlen(program);
    size_t argv_len = has_argv ? strlen(argv_text) : 0;
    size_t env_len = has_env ? strlen(env_text) : 0;
    size_t total = program_len + 1;
    if (has_argv) total += 1 + argv_len;
    if (has_env) total += 4 + env_len + 1;
    char* command = (char*)malloc(total);
    if (command == NULL) return NULL;
    if (has_env) {
        if (has_argv) {
            snprintf(command, total, "env %s %s %s", env_text, program, argv_text);
        } else {
            snprintf(command, total, "env %s %s", env_text, program);
        }
    } else if (has_argv) {
        snprintf(command, total, "%s %s", program, argv_text);
    } else {
        snprintf(command, total, "%s", program);
    }
    return command;
}

static int64_t nuis_host_now_monotonic_ns_raw(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) return 0;
    return (int64_t)ts.tv_sec * 1000000000LL + (int64_t)ts.tv_nsec;
}

static int64_t nuis_host_deadline_ns_from_timeout_ms(int64_t timeout_ms) {
    if (timeout_ms <= 0) return 0;
    int64_t now = nuis_host_now_monotonic_ns_raw();
    if (now <= 0) return 0;
    return now + (timeout_ms * 1000000LL);
}

static int nuis_host_timeout_expired(int64_t deadline_ns) {
    if (deadline_ns <= 0) return 0;
    int64_t now = nuis_host_now_monotonic_ns_raw();
    if (now <= 0) return 0;
    return now >= deadline_ns;
}

static int nuis_host_apply_timeout_to_pid(
    pid_t pid,
    int* done_slot,
    int64_t* status_slot,
    int* timed_out_slot,
    int64_t deadline_ns
) {
    if (*done_slot) return 0;
    if (!nuis_host_timeout_expired(deadline_ns)) return 0;
    kill(pid, SIGKILL);
    int status = 0;
    pid_t result = waitpid(pid, &status, 0);
    if (result < 0) return 0;
    *done_slot = 1;
    *status_slot = (int64_t)status;
    *timed_out_slot = 1;
    return 1;
}

static int64_t nuis_host_command_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
);

static int64_t nuis_host_subprocess_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t env_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
);

static pid_t nuis_host_spawn_shell(char* program, int64_t cwd_handle) {
    if (program == NULL || program[0] == '\0') return -1;
    pid_t pid = fork();
    if (pid < 0) return -1;
    if (pid == 0) {
        const char* cwd = nuis_host_text_lookup(cwd_handle);
        if (cwd != NULL && cwd[0] != '\0') {
            if (chdir(cwd) != 0) _exit(127);
        }
        execlp("sh", "sh", "-c", program, (char*)NULL);
        _exit(127);
    }
    return pid;
}

static int64_t nuis_host_command_spawn(int64_t program_handle, int64_t argv_handle) {
    return nuis_host_command_spawn_in(program_handle, argv_handle, 0, 0);
}

static int64_t nuis_host_command_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
) {
    if (nuis_host_command_len >= 256) return 0;
    char* command = nuis_host_build_shell_command(program_handle, argv_handle, 0);
    pid_t pid = nuis_host_spawn_shell(command, cwd_handle);
    free(command);
    if (pid < 0) return 0;
    nuis_host_command_pids[nuis_host_command_len] = pid;
    nuis_host_command_status_slots[nuis_host_command_len] = 0;
    nuis_host_command_done[nuis_host_command_len] = 0;
    nuis_host_command_timed_out[nuis_host_command_len] = 0;
    nuis_host_command_deadline_ns[nuis_host_command_len] =
        nuis_host_deadline_ns_from_timeout_ms(timeout_ms);
    nuis_host_command_len += 1;
    return nuis_host_command_len;
}

static int64_t nuis_host_command_status(int64_t command_handle) {
    if (command_handle <= 0 || command_handle > nuis_host_command_len) return 0;
    int64_t idx = command_handle - 1;
    if (nuis_host_command_done[idx]) return nuis_host_command_status_slots[idx];
    if (nuis_host_apply_timeout_to_pid(
            nuis_host_command_pids[idx],
            &nuis_host_command_done[idx],
            &nuis_host_command_status_slots[idx],
            &nuis_host_command_timed_out[idx],
            nuis_host_command_deadline_ns[idx]
        )) {
        return nuis_host_command_status_slots[idx];
    }
    int status = 0;
    pid_t result = waitpid(nuis_host_command_pids[idx], &status, WNOHANG);
    if (result == nuis_host_command_pids[idx]) {
        nuis_host_command_done[idx] = 1;
        nuis_host_command_status_slots[idx] = (int64_t)status;
    }
    return nuis_host_command_status_slots[idx];
}

static int64_t nuis_host_command_wait(int64_t command_handle) {
    if (command_handle <= 0 || command_handle > nuis_host_command_len) return 0;
    int64_t idx = command_handle - 1;
    if (nuis_host_command_done[idx]) return nuis_host_command_status_slots[idx];
    if (nuis_host_apply_timeout_to_pid(
            nuis_host_command_pids[idx],
            &nuis_host_command_done[idx],
            &nuis_host_command_status_slots[idx],
            &nuis_host_command_timed_out[idx],
            nuis_host_command_deadline_ns[idx]
        )) {
        return nuis_host_command_status_slots[idx];
    }
    int status = 0;
    pid_t result = waitpid(nuis_host_command_pids[idx], &status, 0);
    if (result < 0) return 0;
    nuis_host_command_done[idx] = 1;
    nuis_host_command_status_slots[idx] = (int64_t)status;
    return nuis_host_command_status_slots[idx];
}

static int64_t nuis_host_command_wait_exit(int64_t command_handle) {
    if (command_handle > 0 && command_handle <= nuis_host_command_len) {
        int64_t idx = command_handle - 1;
        if (nuis_host_command_timed_out[idx]) return 124;
    }
    int64_t raw = nuis_host_command_wait(command_handle);
    if (command_handle > 0 && command_handle <= nuis_host_command_len) {
        int64_t idx = command_handle - 1;
        if (nuis_host_command_timed_out[idx]) return 124;
    }
    return nuis_host_process_exit_code(raw);
}

static int64_t nuis_host_subprocess_spawn(int64_t program_handle, int64_t argv_handle, int64_t env_handle) {
    return nuis_host_subprocess_spawn_in(program_handle, argv_handle, env_handle, 0, 0);
}

static int64_t nuis_host_subprocess_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t env_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
) {
    if (nuis_host_subprocess_len >= 256) return 0;
    char* command = nuis_host_build_shell_command(program_handle, argv_handle, env_handle);
    pid_t pid = nuis_host_spawn_shell(command, cwd_handle);
    free(command);
    if (pid < 0) return 0;
    nuis_host_subprocess_pids[nuis_host_subprocess_len] = pid;
    nuis_host_subprocess_status_slots[nuis_host_subprocess_len] = 0;
    nuis_host_subprocess_done[nuis_host_subprocess_len] = 0;
    nuis_host_subprocess_timed_out[nuis_host_subprocess_len] = 0;
    nuis_host_subprocess_deadline_ns[nuis_host_subprocess_len] =
        nuis_host_deadline_ns_from_timeout_ms(timeout_ms);
    nuis_host_subprocess_len += 1;
    return nuis_host_subprocess_len;
}

static int64_t nuis_host_subprocess_signal(int64_t process_handle, int64_t signal) {
    if (process_handle <= 0 || process_handle > nuis_host_subprocess_len) return 0;
    int64_t idx = process_handle - 1;
    if (nuis_host_subprocess_done[idx]) return 0;
    return kill(nuis_host_subprocess_pids[idx], (int)signal) == 0 ? 1 : 0;
}

static int64_t nuis_host_subprocess_join(int64_t process_handle) {
    if (process_handle <= 0 || process_handle > nuis_host_subprocess_len) return 0;
    int64_t idx = process_handle - 1;
    if (nuis_host_subprocess_done[idx]) return nuis_host_subprocess_status_slots[idx];
    if (nuis_host_apply_timeout_to_pid(
            nuis_host_subprocess_pids[idx],
            &nuis_host_subprocess_done[idx],
            &nuis_host_subprocess_status_slots[idx],
            &nuis_host_subprocess_timed_out[idx],
            nuis_host_subprocess_deadline_ns[idx]
        )) {
        return nuis_host_subprocess_status_slots[idx];
    }
    int status = 0;
    pid_t result = waitpid(nuis_host_subprocess_pids[idx], &status, 0);
    if (result < 0) return 0;
    nuis_host_subprocess_done[idx] = 1;
    nuis_host_subprocess_status_slots[idx] = (int64_t)status;
    return nuis_host_subprocess_status_slots[idx];
}

static int64_t nuis_host_subprocess_join_exit(int64_t process_handle) {
    if (process_handle > 0 && process_handle <= nuis_host_subprocess_len) {
        int64_t idx = process_handle - 1;
        if (nuis_host_subprocess_timed_out[idx]) return 124;
    }
    int64_t raw = nuis_host_subprocess_join(process_handle);
    if (process_handle > 0 && process_handle <= nuis_host_subprocess_len) {
        int64_t idx = process_handle - 1;
        if (nuis_host_subprocess_timed_out[idx]) return 124;
    }
    return nuis_host_process_exit_code(raw);
}
"#,
    );
}
