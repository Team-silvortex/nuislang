pub(crate) fn append_provider_worker_result_helpers(out: &mut String) {
    out.push_str(
        r#"
static void nuis_provider_worker_report_error(const char* stage, int status) {
    if (nuis_provider_worker_socket < 0) return;
    char frame[256];
    int length = snprintf(
        frame,
        sizeof(frame),
        "NUISPWUE1\t%s\t%d",
        stage,
        status);
    if (length > 0 && (size_t)length < sizeof(frame)) {
        send(nuis_provider_worker_socket, frame, (size_t)length, 0);
    }
}

static int nuis_provider_worker_hash_fd(
    int fd,
    size_t offset,
    size_t length,
    uint64_t* output) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    unsigned char bytes[4096];
    size_t consumed = 0;
    while (consumed < length) {
        size_t requested = length - consumed < sizeof(bytes)
            ? length - consumed
            : sizeof(bytes);
        ssize_t count = pread(fd, bytes, requested, (off_t)(offset + consumed));
        if (count != (ssize_t)requested) return 0;
        for (size_t index = 0; index < requested; index++) {
            hash ^= bytes[index];
            hash *= UINT64_C(0x100000001b3);
        }
        consumed += requested;
    }
    *output = hash;
    return 1;
}

static int nuis_provider_worker_protocol_u64(const char* key, uint64_t* output) {
    size_t key_length = strlen(key);
    const unsigned char* cursor = nuis_provider_worker_adapter_protocol;
    const unsigned char* end = cursor + nuis_provider_worker_adapter_protocol_length;
    while (cursor < end) {
        const unsigned char* newline = memchr(cursor, '\n', (size_t)(end - cursor));
        const unsigned char* line_end = newline == NULL ? end : newline;
        if ((size_t)(line_end - cursor) > key_length + 1
            && memcmp(cursor, key, key_length) == 0
            && cursor[key_length] == '=') {
            uint64_t value = 0;
            const unsigned char* digit = cursor + key_length + 1;
            if (digit == line_end) return 0;
            for (; digit < line_end; digit++) {
                if (*digit < '0' || *digit > '9') return 0;
                uint64_t component = (uint64_t)(*digit - '0');
                if (value > (UINT64_MAX - component) / 10) return 0;
                value = value * 10 + component;
            }
            *output = value;
            return 1;
        }
        if (newline == NULL) break;
        cursor = newline + 1;
    }
    return 0;
}
"#,
    );
}
