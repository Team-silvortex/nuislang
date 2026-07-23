pub(crate) fn append_provider_worker_control_helpers(out: &mut String) {
    out.push_str(
        r#"

static void nuis_provider_worker_hash(
    const unsigned char* bytes,
    size_t length,
    char output[19]);
static int nuis_provider_worker_payload_text(
    const char* key,
    char* output,
    size_t output_capacity);

static int nuis_provider_worker_payload_has(const char* key) {
    if (nuis_provider_worker_payload == NULL) return 0;
    size_t key_length = strlen(key);
    const unsigned char* cursor = nuis_provider_worker_payload;
    const unsigned char* end = cursor + nuis_provider_worker_payload_length;
    while (cursor < end) {
        const unsigned char* newline = memchr(cursor, '\n', (size_t)(end - cursor));
        const unsigned char* line_end = newline == NULL ? end : newline;
        size_t line_length = (size_t)(line_end - cursor);
        if (line_length > key_length
            && memcmp(cursor, key, key_length) == 0
            && cursor[key_length] == '=') {
            return 1;
        }
        if (newline == NULL) break;
        cursor = newline + 1;
    }
    return 0;
}

static size_t nuis_provider_worker_split_tabs(
    char* text,
    char** fields,
    size_t field_capacity) {
    if (*text == '\0' || field_capacity == 0) return 0;
    size_t count = 0;
    char* cursor = text;
    while (1) {
        if (count == field_capacity || *cursor == '\0') return 0;
        fields[count++] = cursor;
        char* separator = strchr(cursor, '\t');
        if (separator == NULL) return count;
        *separator = '\0';
        cursor = separator + 1;
    }
}

static int nuis_provider_worker_classify_descriptor_roles(void) {
    nuis_provider_worker_input_fd_count = nuis_provider_worker_fd_count;
    const char* control = strstr(nuis_provider_worker_roles, "control.adapter");
    if (control == NULL) return 1;
    const char* last_separator = strrchr(nuis_provider_worker_roles, ',');
    const char* last_role =
        last_separator == NULL ? nuis_provider_worker_roles : last_separator + 1;
    if (control != last_role
        || strcmp(last_role, "control.adapter") != 0
        || nuis_provider_worker_fd_count == 0) {
        return 0;
    }
    nuis_provider_worker_input_fd_count--;
    return 1;
}

static int nuis_provider_worker_load_adapter_control(
    char* output,
    size_t output_capacity) {
    if (nuis_provider_worker_payload_text(
            "adapter_control", output, output_capacity)) {
        return 1;
    }
    char reference[128];
    char* fields[3];
    if (!nuis_provider_worker_payload_text(
            "adapter_control_ref", reference, sizeof(reference))) return -1;
    if (nuis_provider_worker_split_tabs(reference, fields, 3) != 3
        || strcmp(
            fields[0],
            "nuis-provider-worker-adapter-control-carrier-v1") != 0
        || nuis_provider_worker_input_fd_count >= nuis_provider_worker_fd_count) return -2;
    char* length_end = NULL;
    unsigned long long declared_length = strtoull(fields[1], &length_end, 10);
    if (*fields[1] == '\0'
        || *length_end != '\0'
        || declared_length == 0
        || declared_length >= output_capacity
        || strlen(fields[2]) != 18
        || strncmp(fields[2], "0x", 2) != 0) {
        return -3;
    }
    int descriptor = nuis_provider_worker_fds[nuis_provider_worker_input_fd_count];
    ssize_t received = pread(
        descriptor, output, (size_t)declared_length, 0);
    unsigned char extra = 0;
    if (received != (ssize_t)declared_length
        || pread(descriptor, &extra, 1, (off_t)declared_length) != 0) {
        return -4;
    }
    output[declared_length] = '\0';
    char actual_hash[19];
    nuis_provider_worker_hash(
        (const unsigned char*)output,
        (size_t)declared_length,
        actual_hash);
    return strcmp(actual_hash, fields[2]) == 0 ? 1 : -5;
}
"#,
    );
}
