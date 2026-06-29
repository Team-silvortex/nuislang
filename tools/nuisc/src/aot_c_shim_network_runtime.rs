pub(crate) fn append_c_shim_network_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_network_register_fd(int fd, int64_t kind) {
    if (fd < 0) return 0;
    if (nuis_host_network_fd_len >= 256) {
        close(fd);
        return 0;
    }
    nuis_host_network_fds[nuis_host_network_fd_len] = fd;
    nuis_host_network_fd_kinds[nuis_host_network_fd_len] = kind;
    nuis_host_network_fd_len += 1;
    return nuis_host_network_fd_len;
}

static int nuis_host_network_lookup_fd(int64_t handle) {
    if (handle <= 0 || handle > nuis_host_network_fd_len) return -1;
    return nuis_host_network_fds[handle - 1];
}

static int64_t nuis_host_network_lookup_kind(int64_t handle) {
    if (handle <= 0 || handle > nuis_host_network_fd_len) return 0;
    return nuis_host_network_fd_kinds[handle - 1];
}

static int64_t nuis_host_network_release_fd(int64_t handle, int close_fd) {
    int fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    nuis_host_network_fds[handle - 1] = -1;
    nuis_host_network_fd_kinds[handle - 1] = 0;
    if (close_fd && close(fd) != 0) return 0;
    return 1;
}

static void nuis_network_init_loopback_addr(struct sockaddr_in* addr, int64_t port) {
    memset(addr, 0, sizeof(*addr));
    addr->sin_family = AF_INET;
    addr->sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr->sin_port = htons((uint16_t)port);
}

static int nuis_network_apply_timeout_ms(int fd, int64_t timeout_ms) {
    struct timeval tv;
    if (timeout_ms < 0) return 0;
    tv.tv_sec = (time_t)(timeout_ms / 1000);
    tv.tv_usec = (suseconds_t)((timeout_ms % 1000) * 1000);
    if (setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) != 0) return 0;
    if (setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv)) != 0) return 0;
    return 1;
}
"#,
    );
}
