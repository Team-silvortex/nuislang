pub(crate) fn append_provider_worker_output_helpers(out: &mut String) {
    out.push_str(
        r#"

static void nuis_provider_worker_release_fds(void);
static void nuis_provider_worker_release_output(void);
static int64_t nuis_provider_worker_payload_scalar(
    const char* key,
    const char* value_prefix);
static void nuis_provider_worker_hash(
    const unsigned char* bytes,
    size_t length,
    char output[19]);

static int nuis_provider_worker_append_manifest(
    char* output,
    size_t capacity,
    size_t* length,
    const char* value) {
    int written = snprintf(
        output + *length,
        capacity - *length,
        "%s%s",
        *length == 0 ? "" : ",",
        value);
    if (written <= 0 || (size_t)written >= capacity - *length) return 0;
    *length += (size_t)written;
    return 1;
}

static int nuis_provider_worker_render_output_manifests(
    char lengths[256],
    char hashes[256],
    char modes[256]) {
    if (nuis_provider_worker_output_count == 0) {
        memcpy(lengths, "-", 2);
        memcpy(hashes, "-", 2);
        memcpy(modes, "-", 2);
        return 1;
    }
    size_t length_offset = 0;
    size_t hash_offset = 0;
    size_t mode_offset = 0;
    for (size_t index = 0; index < nuis_provider_worker_output_count; index++) {
        char byte_length[32];
        int rendered = snprintf(
            byte_length,
            sizeof(byte_length),
            "%zu",
            nuis_provider_worker_output_lengths[index]);
        if (rendered <= 0
            || (size_t)rendered >= sizeof(byte_length)
            || !nuis_provider_worker_append_manifest(
                lengths, 256, &length_offset, byte_length)
            || !nuis_provider_worker_append_manifest(
                hashes, 256, &hash_offset, nuis_provider_worker_output_hashes[index])
            || !nuis_provider_worker_append_manifest(
                modes, 256, &mode_offset, nuis_provider_worker_output_modes[index])) {
            return 0;
        }
    }
    return 1;
}

static int nuis_provider_worker_prepare_generic_outputs(
    int64_t ingress_status,
    size_t output_count) {
    if (output_count == 0
        || output_count > nuis_provider_worker_max_output_fds
        || output_count > NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS) {
        return 0;
    }
    nuis_provider_worker_output_count = output_count;
    for (size_t index = 0; index < output_count; index++) {
        uint64_t output_words[3] = {
            (uint64_t)nuis_provider_worker_payload_scalar(
                "capsule_token", "capsule-token:") + index,
            (uint64_t)ingress_status,
            (uint64_t)nuis_provider_worker_input_byte_sum,
        };
        FILE* file = tmpfile();
        if (file == NULL) return 0;
        nuis_provider_worker_output_files[index] = file;
        nuis_provider_worker_output_lengths[index] = sizeof(output_words);
        memcpy(nuis_provider_worker_output_modes[index], "protocol-stdout", 16);
        if (fwrite(output_words, 1, sizeof(output_words), file) != sizeof(output_words)
            || fflush(file) != 0) {
            return 0;
        }
        nuis_provider_worker_hash(
            (const unsigned char*)output_words,
            sizeof(output_words),
            nuis_provider_worker_output_hashes[index]);
    }
    return 1;
}

int64_t nuis_host_provider_worker_reply(int64_t invocation_status) {
    if (nuis_provider_worker_socket < 0) return -1;
    if (invocation_status <= 0) {
        nuis_provider_worker_report_error("nuis-ingress", (int)invocation_status);
        return -1;
    }
    char output_lengths[256];
    char output_hashes[256];
    char output_modes[256];
    if (!nuis_provider_worker_render_output_manifests(
            output_lengths, output_hashes, output_modes)) {
        return -2;
    }
    const char* output_roles =
        nuis_provider_worker_output_count == 0 ? "-" : nuis_provider_worker_output_roles;
    char receipt[4096];
    int header_length = snprintf(
        receipt,
        sizeof(receipt),
        "NUISPWUR7\t%s\t%llu\t%s\t%d\t%zu\t%u\t%lld\t%s\t%s\t%zu\t%s\t%zu\t%s\t%s\t%s\t%s\t%zu\t%s\n",
        nuis_provider_worker_lease,
        nuis_provider_worker_sequence,
        nuis_provider_worker_request_id,
        getpid(),
        nuis_provider_worker_fd_count,
        nuis_provider_worker_input_byte_sum,
        (long long)invocation_status,
        nuis_provider_worker_payload_hash,
        nuis_provider_worker_roles,
        nuis_provider_worker_payload_length,
        nuis_provider_worker_payload_hash,
        nuis_provider_worker_output_count,
        output_roles,
        output_lengths,
        output_hashes,
        output_modes,
        nuis_provider_worker_adapter_protocol_length,
        nuis_provider_worker_adapter_protocol_hash);
    if (header_length <= 0 || (size_t)header_length >= sizeof(receipt)) return -2;
    struct iovec parts[2] = {
        {.iov_base = receipt, .iov_len = (size_t)header_length},
        {
            .iov_base = nuis_provider_worker_adapter_protocol,
            .iov_len = nuis_provider_worker_adapter_protocol_length,
        },
    };
    struct msghdr message = {0};
    message.msg_iov = parts;
    message.msg_iovlen = 2;
    char control[
        CMSG_SPACE(sizeof(int) * NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS)] = {0};
    if (nuis_provider_worker_output_count > 0) {
        message.msg_control = control;
        message.msg_controllen = CMSG_SPACE(
            sizeof(int) * nuis_provider_worker_output_count);
        struct cmsghdr* header = CMSG_FIRSTHDR(&message);
        if (header == NULL) return -2;
        header->cmsg_level = SOL_SOCKET;
        header->cmsg_type = SCM_RIGHTS;
        header->cmsg_len = CMSG_LEN(
            sizeof(int) * nuis_provider_worker_output_count);
        int* output_fds = (int*)CMSG_DATA(header);
        for (size_t index = 0; index < nuis_provider_worker_output_count; index++) {
            output_fds[index] = fileno(nuis_provider_worker_output_files[index]);
        }
    }
    size_t expected = parts[0].iov_len + parts[1].iov_len;
    ssize_t sent = sendmsg(nuis_provider_worker_socket, &message, 0);
    int send_error = errno;
    if (sent != (ssize_t)expected) {
        nuis_provider_worker_report_error(
            "reply-send",
            sent < 0 ? -send_error : (int)sent);
    }
    nuis_provider_worker_release_fds();
    nuis_provider_worker_release_output();
    return sent == (ssize_t)expected ? 0 : -3;
}
"#,
    );
}
