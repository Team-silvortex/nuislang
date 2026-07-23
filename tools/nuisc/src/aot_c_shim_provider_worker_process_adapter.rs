pub(crate) fn append_provider_worker_process_adapter_helpers(out: &mut String) {
    out.push_str(
        r#"

static size_t nuis_provider_worker_adapter_payload_offsets[
    NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS];
static size_t nuis_provider_worker_adapter_mapped_lengths[
    NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS];
static size_t nuis_provider_worker_adapter_packet_lengths[
    NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS];

static int nuis_provider_worker_parse_size_manifest(
    const char* manifest,
    size_t expected_count,
    size_t values[NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS]) {
    if (manifest == NULL || *manifest == '\0' || expected_count == 0
        || expected_count > NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS) return 0;
    const char* cursor = manifest;
    for (size_t index = 0; index < expected_count; index++) {
        char* end = NULL;
        unsigned long long value = strtoull(cursor, &end, 10);
        if (end == cursor || value == 0 || value > SIZE_MAX) return 0;
        values[index] = (size_t)value;
        if (index + 1 == expected_count) return *end == '\0';
        if (*end != ',') return 0;
        cursor = end + 1;
    }
    return 0;
}

static int nuis_provider_worker_parse_u64_manifest(
    const char* manifest,
    size_t expected_count,
    uint64_t values[NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS]) {
    if (manifest == NULL || *manifest == '\0' || expected_count == 0
        || expected_count > NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS) return 0;
    const char* cursor = manifest;
    for (size_t index = 0; index < expected_count; index++) {
        char* end = NULL;
        errno = 0;
        unsigned long long value = strtoull(cursor, &end, 10);
        if (end == cursor || errno == ERANGE) return 0;
        values[index] = (uint64_t)value;
        if (index + 1 == expected_count) return *end == '\0';
        if (*end != ',') return 0;
        cursor = end + 1;
    }
    return 0;
}

static int nuis_provider_worker_output_role_at(
    size_t requested,
    char* output,
    size_t capacity) {
    const char* start = nuis_provider_worker_output_roles;
    for (size_t index = 0; index < requested; index++) {
        start = strchr(start, ',');
        if (start == NULL) return 0;
        start++;
    }
    const char* end = strchr(start, ',');
    size_t length = end == NULL ? strlen(start) : (size_t)(end - start);
    if (length == 0 || length >= capacity) return 0;
    memcpy(output, start, length);
    output[length] = '\0';
    return 1;
}

static int nuis_provider_worker_prepare_adapter_outputs(
    size_t output_count,
    const size_t semantic_lengths[NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS]) {
    long page_size_value = sysconf(_SC_PAGESIZE);
    if (page_size_value <= 0 || output_count == 0
        || output_count > nuis_provider_worker_max_output_fds
        || output_count > NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS) return 0;
    size_t page_size = (size_t)page_size_value;
    size_t payload_offset = ((56 + page_size - 1) / page_size) * page_size;
    nuis_provider_worker_output_count = output_count;
    for (size_t index = 0; index < output_count; index++) {
        size_t mapped_length =
            ((semantic_lengths[index] + page_size - 1) / page_size) * page_size;
        if (mapped_length < semantic_lengths[index]
            || payload_offset > SIZE_MAX - mapped_length) return 0;
        size_t packet_length = payload_offset + mapped_length;
        FILE* output_file = tmpfile();
        if (output_file == NULL) return 0;
        nuis_provider_worker_output_files[index] = output_file;
        memcpy(
            nuis_provider_worker_output_modes[index],
            "nuispfd1-result",
            16);
        nuis_provider_worker_adapter_payload_offsets[index] = payload_offset;
        nuis_provider_worker_adapter_mapped_lengths[index] = mapped_length;
        nuis_provider_worker_adapter_packet_lengths[index] = packet_length;
        unsigned char carrier_header[56] = {0};
        memcpy(carrier_header, "NUISPFD1", 8);
        uint32_t frame_count = 1;
        uint32_t encoded_page_size = (uint32_t)page_size;
        uint64_t encoded_payload_offset = (uint64_t)payload_offset;
        uint64_t encoded_output_length = (uint64_t)semantic_lengths[index];
        uint64_t encoded_mapped_length = (uint64_t)mapped_length;
        memcpy(carrier_header + 8, &frame_count, sizeof(frame_count));
        memcpy(carrier_header + 12, &encoded_page_size, sizeof(encoded_page_size));
        memcpy(
            carrier_header + 24,
            &encoded_payload_offset,
            sizeof(encoded_payload_offset));
        memcpy(
            carrier_header + 32,
            &encoded_output_length,
            sizeof(encoded_output_length));
        memcpy(
            carrier_header + 40,
            &encoded_mapped_length,
            sizeof(encoded_mapped_length));
        if (fwrite(carrier_header, 1, sizeof(carrier_header), output_file)
                != sizeof(carrier_header)
            || fflush(output_file) != 0
            || ftruncate(fileno(output_file), (off_t)packet_length) != 0) return 0;
    }
    return 1;
}

static int nuis_provider_worker_expose_adapter_outputs(
    const size_t semantic_lengths[NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS]) {
    char manifest[2048];
    size_t offset = 0;
    for (size_t index = 0; index < nuis_provider_worker_output_count; index++) {
        char role[128];
        if (!nuis_provider_worker_output_role_at(index, role, sizeof(role))) return 0;
        int fd = fileno(nuis_provider_worker_output_files[index]);
        int flags = fcntl(fd, F_GETFD);
        int written = snprintf(
            manifest + offset,
            sizeof(manifest) - offset,
            "%s%s=fd:%d:%zu:%zu:48",
            index == 0 ? "" : ",",
            role,
            fd,
            nuis_provider_worker_adapter_payload_offsets[index],
            semantic_lengths[index]);
        if (flags < 0
            || fcntl(fd, F_SETFD, flags & ~FD_CLOEXEC) < 0
            || written <= 0
            || (size_t)written >= sizeof(manifest) - offset) return 0;
        offset += (size_t)written;
    }
    char slot_zero[128];
    int slot_zero_length = snprintf(
        slot_zero,
        sizeof(slot_zero),
        "fd:%d:%zu:%zu:48",
        fileno(nuis_provider_worker_output_files[0]),
        nuis_provider_worker_adapter_payload_offsets[0],
        semantic_lengths[0]);
    return slot_zero_length > 0
        && (size_t)slot_zero_length < sizeof(slot_zero)
        && setenv("NUIS_PROVIDER_OUTPUT_FD", slot_zero, 1) == 0
        && setenv("NUIS_PROVIDER_OUTPUT_FDS", manifest, 1) == 0;
}

static int nuis_provider_worker_protocol_hash_manifest(
    size_t expected_count,
    uint64_t hashes[NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS]) {
    if (expected_count == 1) {
        return nuis_provider_worker_protocol_u64("output_hash", &hashes[0]);
    }
    const char* key = "output_hashes";
    size_t key_length = strlen(key);
    const unsigned char* cursor = nuis_provider_worker_adapter_protocol;
    const unsigned char* end =
        cursor + nuis_provider_worker_adapter_protocol_length;
    while (cursor < end) {
        const unsigned char* newline = memchr(cursor, '\n', (size_t)(end - cursor));
        const unsigned char* line_end = newline == NULL ? end : newline;
        if ((size_t)(line_end - cursor) > key_length + 1
            && memcmp(cursor, key, key_length) == 0
            && cursor[key_length] == '=') {
            char manifest[256];
            size_t length = (size_t)(line_end - cursor - key_length - 1);
            if (length == 0 || length >= sizeof(manifest)) return 0;
            memcpy(manifest, cursor + key_length + 1, length);
            manifest[length] = '\0';
            return nuis_provider_worker_parse_u64_manifest(
                manifest, expected_count, hashes);
        }
        if (newline == NULL) break;
        cursor = newline + 1;
    }
    return 0;
}

static int nuis_provider_worker_validate_adapter_outputs(
    const size_t semantic_lengths[NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS]) {
    uint64_t declared_hashes[NUIS_PROVIDER_WORKER_MAX_OUTPUT_FDS] = {0};
    if (!nuis_provider_worker_protocol_hash_manifest(
            nuis_provider_worker_output_count,
            declared_hashes)) return 0;
    for (size_t index = 0; index < nuis_provider_worker_output_count; index++) {
        uint64_t stored_hash = 0;
        uint64_t actual_hash = 0;
        uint64_t packet_hash = 0;
        int fd = fileno(nuis_provider_worker_output_files[index]);
        if (pread(fd, &stored_hash, sizeof(stored_hash), 48) != sizeof(stored_hash)
            || !nuis_provider_worker_hash_fd(
                fd,
                nuis_provider_worker_adapter_payload_offsets[index],
                semantic_lengths[index],
                &actual_hash)
            || actual_hash != declared_hashes[index]
            || stored_hash != declared_hashes[index]
            || !nuis_provider_worker_hash_fd(
                fd,
                0,
                nuis_provider_worker_adapter_packet_lengths[index],
                &packet_hash)) return 0;
        nuis_provider_worker_output_lengths[index] =
            nuis_provider_worker_adapter_packet_lengths[index];
        snprintf(
            nuis_provider_worker_output_hashes[index],
            sizeof(nuis_provider_worker_output_hashes[index]),
            "0x%016llx",
            (unsigned long long)packet_hash);
    }
    return 1;
}
"#,
    );
}
