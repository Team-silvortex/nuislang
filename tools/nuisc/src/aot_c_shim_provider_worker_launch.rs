pub(crate) fn append_provider_worker_launch_helpers(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_provider_worker_launch_scalar(const char* name) {
    const char* text = getenv(name);
    if (text == NULL) return 0;
    char* end = NULL;
    long long value = strtoll(text, &end, 10);
    if (*text == '\0' || *end != '\0' || value <= 0) return 0;
    return (int64_t)value;
}

int64_t nuis_host_provider_worker_launch_provider_key(void) {
    return nuis_provider_worker_launch_scalar("NUIS_PROVIDER_WORKER_PROVIDER_KEY");
}

int64_t nuis_host_provider_worker_launch_capability_hash(void) {
    return nuis_provider_worker_launch_scalar("NUIS_PROVIDER_WORKER_CAPABILITY_HASH");
}

int64_t nuis_host_provider_worker_open(int64_t provider, int64_t capability) {
    if (nuis_provider_worker_socket >= 0 || provider <= 0 || capability <= 0) return -1;
    const char* fd_text = getenv("NUIS_PROVIDER_WORKER_SOCKET_FD");
    if (fd_text == NULL) return -2;
    char* end = NULL;
    long parsed = strtol(fd_text, &end, 10);
    if (*fd_text == '\0' || *end != '\0' || parsed < 0 || parsed > INT_MAX) return -3;
    nuis_provider_worker_socket = (int)parsed;
    nuis_provider_worker_provider = provider;
    nuis_provider_worker_capability = capability;
    const char* descriptor_contract = getenv("NUIS_PROVIDER_WORKER_DESCRIPTOR_CONTRACT");
    nuis_provider_worker_max_semantic_fds = (size_t)nuis_provider_worker_launch_scalar(
        "NUIS_PROVIDER_WORKER_MAX_SEMANTIC_DESCRIPTORS");
    nuis_provider_worker_max_control_fds = (size_t)nuis_provider_worker_launch_scalar(
        "NUIS_PROVIDER_WORKER_MAX_CONTROL_DESCRIPTORS");
    const char* output_descriptor_contract = getenv(
        "NUIS_PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CONTRACT");
    nuis_provider_worker_max_output_fds = (size_t)nuis_provider_worker_launch_scalar(
        "NUIS_PROVIDER_WORKER_MAX_OUTPUT_DESCRIPTORS");
    if (descriptor_contract == NULL
        || strcmp(
            descriptor_contract,
            "nuis-provider-worker-descriptor-capability-v1") != 0
        || nuis_provider_worker_max_semantic_fds == 0
        || nuis_provider_worker_max_control_fds == 0
        || nuis_provider_worker_max_semantic_fds
            + nuis_provider_worker_max_control_fds > NUIS_PROVIDER_WORKER_MAX_FDS) {
        nuis_provider_worker_socket = -1;
        return -4;
    }
    if (output_descriptor_contract == NULL
        || strcmp(
            output_descriptor_contract,
            "nuis-provider-worker-output-descriptor-capability-v1") != 0
        || nuis_provider_worker_max_output_fds == 0
        || nuis_provider_worker_max_output_fds > NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS) {
        nuis_provider_worker_socket = -1;
        return -5;
    }
    char launch[16] = {0};
    ssize_t launch_length = recv(nuis_provider_worker_socket, launch, sizeof(launch), 0);
    if (launch_length != 9 || memcmp(launch, "NUISPWUH0", 9) != 0) {
        nuis_provider_worker_socket = -1;
        return -6;
    }
    char handshake[256];
    int length = snprintf(
        handshake,
        sizeof(handshake),
        "NUISPWUH3\t%d\t%s\t%zu\t%zu\t%zu\t%s\t%zu",
        getpid(),
        descriptor_contract,
        nuis_provider_worker_max_semantic_fds,
        nuis_provider_worker_max_control_fds,
        nuis_provider_worker_max_semantic_fds + nuis_provider_worker_max_control_fds,
        output_descriptor_contract,
        nuis_provider_worker_max_output_fds);
    if (length <= 0 || send(nuis_provider_worker_socket, handshake, (size_t)length, 0) != length) {
        nuis_provider_worker_socket = -1;
        return -7;
    }
    return 0;
}
"#,
    );
}
