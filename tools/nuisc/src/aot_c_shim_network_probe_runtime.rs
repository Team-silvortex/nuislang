pub(crate) fn append_c_shim_network_probe_runtime(out: &mut String) {
    out.push_str(
        r#"

static int nuis_network_try_connect_probe(
    int64_t local_port,
    int64_t remote_port,
    int64_t connect_timeout_ms
) {
    int listener = -1;
    int client = -1;
    int accepted = -1;
    int ok = 0;
    struct sockaddr_in listener_addr;
    struct sockaddr_in client_addr;

    listener = socket(AF_INET, SOCK_STREAM, 0);
    if (listener < 0) goto done;
    {
        int yes = 1;
        setsockopt(listener, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));
    }
    nuis_network_init_loopback_addr(&listener_addr, remote_port);
    if (bind(listener, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    if (listen(listener, 1) != 0) goto done;

    client = socket(AF_INET, SOCK_STREAM, 0);
    if (client < 0) goto done;
    if (!nuis_network_apply_timeout_ms(client, connect_timeout_ms)) goto done;
    if (local_port > 0) {
        nuis_network_init_loopback_addr(&client_addr, local_port);
        if (bind(client, (struct sockaddr*)&client_addr, sizeof(client_addr)) != 0) goto done;
    }
    if (connect(client, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    accepted = accept(listener, NULL, NULL);
    if (accepted < 0) goto done;
    ok = 1;

done:
    if (accepted >= 0) close(accepted);
    if (client >= 0) close(client);
    if (listener >= 0) close(listener);
    return ok;
}

static int nuis_network_try_accept_probe(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int listener = -1;
    int client = -1;
    int accepted = -1;
    int ok = 0;
    struct sockaddr_in listener_addr;

    listener = socket(AF_INET, SOCK_STREAM, 0);
    if (listener < 0) goto done;
    {
        int yes = 1;
        setsockopt(listener, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));
    }
    if (!nuis_network_apply_timeout_ms(listener, read_timeout_ms + write_timeout_ms)) goto done;
    nuis_network_init_loopback_addr(&listener_addr, local_port);
    if (bind(listener, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    if (listen(listener, 1) != 0) goto done;

    client = socket(AF_INET, SOCK_STREAM, 0);
    if (client < 0) goto done;
    if (!nuis_network_apply_timeout_ms(client, write_timeout_ms)) goto done;
    if (connect(client, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    accepted = accept(listener, NULL, NULL);
    if (accepted < 0) goto done;
    if (!nuis_network_apply_timeout_ms(accepted, read_timeout_ms + write_timeout_ms)) goto done;
    ok = 1;

done:
    if (accepted >= 0) close(accepted);
    if (client >= 0) close(client);
    if (listener >= 0) close(listener);
    return ok;
}

static int nuis_network_try_send_probe(int64_t stream_window, int64_t send_window) {
    int fds[2] = {-1, -1};
    int ok = 0;
    char buffer[64];
    size_t want = (size_t)send_window;
    if (want > sizeof(buffer)) want = sizeof(buffer);
    if (want == 0) want = 1;
    memset(buffer, 'n', want);
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, fds) != 0) goto done;
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    if (send(fds[0], buffer, want, 0) < 0) goto done;
    ok = 1;

done:
    if (fds[0] >= 0) close(fds[0]);
    if (fds[1] >= 0) close(fds[1]);
    return ok;
}

static int nuis_network_try_recv_probe(int64_t stream_window, int64_t recv_window) {
    int fds[2] = {-1, -1};
    int ok = 0;
    char send_buffer[64];
    char recv_buffer[64];
    size_t want = (size_t)recv_window;
    if (want > sizeof(send_buffer)) want = sizeof(send_buffer);
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    memset(send_buffer, 'y', want);
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, fds) != 0) goto done;
    if (send(fds[0], send_buffer, want, 0) < 0) goto done;
    if (recv(fds[1], recv_buffer, want, 0) < 0) goto done;
    ok = 1;

done:
    if (fds[0] >= 0) close(fds[0]);
    if (fds[1] >= 0) close(fds[1]);
    return ok;
}

static int64_t nuis_host_network_connect_probe(
    int64_t local_port,
    int64_t remote_port,
    int64_t connect_timeout_ms
) {
    if (local_port <= 0 || remote_port <= 0) return 0;
    if (connect_timeout_ms < 0) return 0;
    return nuis_network_try_connect_probe(local_port, remote_port, connect_timeout_ms) ? 1 : 0;
}

static int64_t nuis_host_network_accept_probe(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    if (local_port <= 0) return 0;
    if (read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    return nuis_network_try_accept_probe(local_port, read_timeout_ms, write_timeout_ms) ? 1 : 0;
}

static int64_t nuis_host_network_send_probe(
    int64_t stream_window,
    int64_t send_window,
    int64_t remote_port
) {
    if (stream_window <= 0 || send_window <= 0 || remote_port <= 0) return 0;
    (void)remote_port;
    return nuis_network_try_send_probe(stream_window, send_window) ? 1 : 0;
}

static int64_t nuis_host_network_recv_probe(
    int64_t stream_window,
    int64_t recv_window,
    int64_t local_port
) {
    if (stream_window <= 0 || recv_window <= 0 || local_port <= 0) return 0;
    (void)local_port;
    return nuis_network_try_recv_probe(stream_window, recv_window) ? 1 : 0;
}
"#,
    );
}
