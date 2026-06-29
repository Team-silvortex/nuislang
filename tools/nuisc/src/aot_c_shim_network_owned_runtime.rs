pub(crate) fn append_c_shim_network_owned_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_network_open_tcp_stream(
    int64_t remote_port,
    int64_t connect_timeout_ms
) {
    int fd = -1;
    struct sockaddr_in addr;
    if (remote_port <= 0) return 0;
    fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return 0;
    if (connect_timeout_ms >= 0) {
        if (!nuis_network_apply_timeout_ms(fd, connect_timeout_ms)) {
            close(fd);
            return 0;
        }
    }
    nuis_network_init_loopback_addr(&addr, remote_port);
    if (connect(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        close(fd);
        return 0;
    }
    return nuis_host_network_register_fd(fd, 1);
}

static int64_t nuis_host_network_open_tcp_listener(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int fd = -1;
    struct sockaddr_in addr;
    int yes = 1;
    if (local_port <= 0 || read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return 0;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));
    if (!nuis_network_apply_timeout_ms(fd, read_timeout_ms + write_timeout_ms)) {
        close(fd);
        return 0;
    }
    nuis_network_init_loopback_addr(&addr, local_port);
    if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        close(fd);
        return 0;
    }
    if (listen(fd, 1) != 0) {
        close(fd);
        return 0;
    }
    return nuis_host_network_register_fd(fd, 3);
}

static int64_t nuis_host_network_open_udp_datagram(
    int64_t local_port,
    int64_t remote_port
) {
    int fd = -1;
    struct sockaddr_in addr;
    fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0) return 0;
    if (local_port > 0) {
        nuis_network_init_loopback_addr(&addr, local_port);
        if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
            close(fd);
            return 0;
        }
    }
    if (remote_port > 0) {
        nuis_network_init_loopback_addr(&addr, remote_port);
        if (connect(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
            close(fd);
            return 0;
        }
    }
    return nuis_host_network_register_fd(fd, 2);
}

static int64_t nuis_host_network_bind_udp_datagram(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int fd = -1;
    struct sockaddr_in addr;
    if (local_port <= 0 || read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0) return 0;
    if (!nuis_network_apply_timeout_ms(fd, read_timeout_ms + write_timeout_ms)) {
        close(fd);
        return 0;
    }
    nuis_network_init_loopback_addr(&addr, local_port);
    if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        close(fd);
        return 0;
    }
    return nuis_host_network_register_fd(fd, 2);
}

static int64_t nuis_host_network_accept_owned(
    int64_t listener_handle,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int listener_fd = -1;
    int accepted_fd = -1;
    if (listener_handle <= 0 || read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    if (nuis_host_network_lookup_kind(listener_handle) != 3) return 0;
    listener_fd = nuis_host_network_lookup_fd(listener_handle);
    if (listener_fd < 0) return 0;
    accepted_fd = accept(listener_fd, NULL, NULL);
    if (accepted_fd < 0) return 0;
    if (!nuis_network_apply_timeout_ms(accepted_fd, read_timeout_ms + write_timeout_ms)) {
        close(accepted_fd);
        return 0;
    }
    return nuis_host_network_register_fd(accepted_fd, 1);
}

static int64_t nuis_host_network_close_owned(int64_t handle) {
    return nuis_host_network_release_fd(handle, 1);
}

static int64_t nuis_host_network_send_owned(
    int64_t handle,
    int64_t stream_window,
    int64_t send_window
) {
    int fd = -1;
    int64_t kind = 0;
    ssize_t sent = 0;
    char buffer[64];
    size_t want = (size_t)send_window;
    if (handle <= 0 || stream_window <= 0 || send_window <= 0) return 0;
    fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    kind = nuis_host_network_lookup_kind(handle);
    if (want > sizeof(buffer)) want = sizeof(buffer);
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    if (kind == 1) {
        const char* request = "GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
        size_t request_len = strlen(request);
        if (want > request_len) want = request_len;
        memcpy(buffer, request, want);
    } else {
        memset(buffer, 's', want);
    }
    sent = send(fd, buffer, want, 0);
    if (sent <= 0) return 0;
    if (kind == 1) {
        shutdown(fd, SHUT_WR);
    }
    return handle + (int64_t)sent;
}

static int64_t nuis_host_network_recv_owned(
    int64_t handle,
    int64_t stream_window,
    int64_t recv_window
) {
    int fd = -1;
    ssize_t received = 0;
    char buffer[64];
    size_t want = (size_t)recv_window;
    if (handle <= 0 || stream_window <= 0 || recv_window <= 0) return 0;
    fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    if (want > sizeof(buffer)) want = sizeof(buffer);
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    received = recv(fd, buffer, want, 0);
    if (received <= 0) return 0;
    return handle + (int64_t)received;
}

static int64_t nuis_host_network_recv_http_status_owned(
    int64_t handle,
    int64_t stream_window,
    int64_t recv_window
) {
    int fd = -1;
    ssize_t received = 0;
    char buffer[128];
    size_t want = (size_t)recv_window;
    int status = 0;
    if (handle <= 0 || stream_window <= 0 || recv_window <= 0) return 0;
    fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    if (want > sizeof(buffer) - 1) want = sizeof(buffer) - 1;
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    received = recv(fd, buffer, want, 0);
    if (received <= 0) return 0;
    buffer[received] = '\0';
    if (sscanf(buffer, "HTTP/%*d.%*d %d", &status) == 1 && status > 0) {
        return (int64_t)status;
    }
    return handle + (int64_t)received;
}

static int64_t nuis_host_network_close(int64_t handle) {
    if (handle <= 0) return 0;
    if (nuis_host_network_close_owned(handle)) return 1;
    return 0;
}
"#,
    );
}
