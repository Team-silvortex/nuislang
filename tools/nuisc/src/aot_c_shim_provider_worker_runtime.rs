pub(crate) fn append_c_shim_provider_worker_runtime(out: &mut String) {
    out.push_str(
        r#"

#include <sys/uio.h>
#include <sys/wait.h>
#include <errno.h>

#define NUIS_PROVIDER_WORKER_MAX_FDS 16
#define NUIS_PROVIDER_WORKER_MAX_FRAME_BYTES (64 * 1024)

static int nuis_provider_worker_socket = -1;
static int nuis_provider_worker_fds[NUIS_PROVIDER_WORKER_MAX_FDS];
static size_t nuis_provider_worker_fd_count = 0;
static unsigned char nuis_provider_worker_frame[NUIS_PROVIDER_WORKER_MAX_FRAME_BYTES];
static unsigned char* nuis_provider_worker_payload = NULL;
static size_t nuis_provider_worker_payload_length = 0;
static unsigned long long nuis_provider_worker_sequence = 0;
static char nuis_provider_worker_lease[512];
static char nuis_provider_worker_request_id[512];
static char nuis_provider_worker_payload_hash[32];
static char nuis_provider_worker_roles[1024];
static int64_t nuis_provider_worker_provider = 0;
static int64_t nuis_provider_worker_capability = 0;
static int nuis_provider_worker_close_requested = 0;
static FILE* nuis_provider_worker_output_file = NULL;
static size_t nuis_provider_worker_output_length = 0;
static char nuis_provider_worker_output_hash[19] = "0x0000000000000000";
static char nuis_provider_worker_output_roles[1024] = "-";
static unsigned char nuis_provider_worker_adapter_protocol[NUIS_PROVIDER_WORKER_MAX_FRAME_BYTES];
static size_t nuis_provider_worker_adapter_protocol_length = 0;
static char nuis_provider_worker_adapter_protocol_hash[19] = "0xcbf29ce484222325";
static unsigned int nuis_provider_worker_input_byte_sum = 0;
"#,
    );
    crate::aot_c_shim_provider_worker_control::append_provider_worker_control_helpers(out);
    crate::aot_c_shim_provider_worker_result::append_provider_worker_result_helpers(out);
    out.push_str(
        r#"

static void nuis_provider_worker_release_output(void) {
    if (nuis_provider_worker_output_file != NULL) {
        fclose(nuis_provider_worker_output_file);
        nuis_provider_worker_output_file = NULL;
    }
    nuis_provider_worker_output_length = 0;
    memcpy(nuis_provider_worker_output_hash, "0x0000000000000000", 19);
    memcpy(nuis_provider_worker_output_roles, "-", 2);
    nuis_provider_worker_adapter_protocol_length = 0;
    memcpy(nuis_provider_worker_adapter_protocol_hash, "0xcbf29ce484222325", 19);
    nuis_provider_worker_input_byte_sum = 0;
}

static void nuis_provider_worker_release_fds(void) {
    for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
        close(nuis_provider_worker_fds[index]);
    }
    nuis_provider_worker_fd_count = 0;
}

static int64_t nuis_provider_worker_receive_fail(int64_t status) {
    nuis_provider_worker_report_error("request-receive", (int)status);
    nuis_provider_worker_release_fds();
    nuis_provider_worker_release_output();
    return status;
}

static void nuis_provider_worker_hash(const unsigned char* bytes, size_t length, char output[19]) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    for (size_t index = 0; index < length; index++) {
        hash ^= bytes[index];
        hash *= UINT64_C(0x100000001b3);
    }
    snprintf(output, 19, "0x%016llx", (unsigned long long)hash);
}

static int nuis_provider_worker_hash_file(const char* path, char output[19]) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 0;
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    unsigned char bytes[4096];
    size_t count = 0;
    while ((count = fread(bytes, 1, sizeof(bytes), file)) > 0) {
        for (size_t index = 0; index < count; index++) {
            hash ^= bytes[index];
            hash *= UINT64_C(0x100000001b3);
        }
    }
    int valid = feof(file) && fclose(file) == 0;
    if (!valid) return 0;
    snprintf(output, 19, "0x%016llx", (unsigned long long)hash);
    return 1;
}

static size_t nuis_provider_worker_role_count(const char* roles) {
    if (strcmp(roles, "-") == 0) return 0;
    size_t count = 1;
    for (const char* cursor = roles; *cursor != '\0'; cursor++) {
        if (*cursor == ',') count++;
    }
    return count;
}

static int64_t nuis_provider_worker_payload_scalar(
    const char* key,
    const char* value_prefix) {
    if (nuis_provider_worker_payload == NULL) return -1;
    size_t key_length = strlen(key);
    size_t prefix_length = strlen(value_prefix);
    const unsigned char* cursor = nuis_provider_worker_payload;
    const unsigned char* end = cursor + nuis_provider_worker_payload_length;
    while (cursor < end) {
        const unsigned char* newline = memchr(cursor, '\n', (size_t)(end - cursor));
        const unsigned char* line_end = newline == NULL ? end : newline;
        size_t line_length = (size_t)(line_end - cursor);
        if (line_length > key_length + 1 + prefix_length
            && memcmp(cursor, key, key_length) == 0
            && cursor[key_length] == '='
            && memcmp(cursor + key_length + 1, value_prefix, prefix_length) == 0) {
            const unsigned char* digits = cursor + key_length + 1 + prefix_length;
            uint64_t value = 0;
            if (digits == line_end) return -1;
            for (const unsigned char* digit = digits; digit < line_end; digit++) {
                if (*digit < '0' || *digit > '9') return -1;
                uint64_t component = (uint64_t)(*digit - '0');
                if (value > ((uint64_t)INT64_MAX - component) / 10) return -1;
                value = value * 10 + component;
            }
            return (int64_t)value;
        }
        if (newline == NULL) break;
        cursor = newline + 1;
    }
    return -1;
}

static int nuis_provider_worker_payload_text(
    const char* key,
    char* output,
    size_t output_capacity) {
    if (nuis_provider_worker_payload == NULL || output_capacity == 0) return 0;
    size_t key_length = strlen(key);
    const unsigned char* cursor = nuis_provider_worker_payload;
    const unsigned char* end = cursor + nuis_provider_worker_payload_length;
    while (cursor < end) {
        const unsigned char* newline = memchr(cursor, '\n', (size_t)(end - cursor));
        const unsigned char* line_end = newline == NULL ? end : newline;
        size_t line_length = (size_t)(line_end - cursor);
        if (line_length > key_length + 1
            && memcmp(cursor, key, key_length) == 0
            && cursor[key_length] == '=') {
            size_t value_length = line_length - key_length - 1;
            if (value_length + 1 > output_capacity) return 0;
            memcpy(output, cursor + key_length + 1, value_length);
            output[value_length] = '\0';
            return 1;
        }
        if (newline == NULL) break;
        cursor = newline + 1;
    }
    return 0;
}

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
    char launch[16] = {0};
    ssize_t launch_length = recv(nuis_provider_worker_socket, launch, sizeof(launch), 0);
    if (launch_length != 9 || memcmp(launch, "NUISPWUH0", 9) != 0) {
        nuis_provider_worker_socket = -1;
        return -4;
    }
    char handshake[64];
    int length = snprintf(handshake, sizeof(handshake), "NUISPWUH1\t%d", getpid());
    if (length <= 0 || send(nuis_provider_worker_socket, handshake, (size_t)length, 0) != length) {
        nuis_provider_worker_socket = -1;
        return -5;
    }
    return 0;
}

int64_t nuis_host_provider_worker_receive(void) {
    if (nuis_provider_worker_socket < 0) return -1;
    nuis_provider_worker_release_fds();
    nuis_provider_worker_release_output();
    char control[CMSG_SPACE(sizeof(int) * NUIS_PROVIDER_WORKER_MAX_FDS)] = {0};
    struct iovec iov = {
        .iov_base = nuis_provider_worker_frame,
        .iov_len = sizeof(nuis_provider_worker_frame) - 1,
    };
    struct msghdr message = {0};
    message.msg_iov = &iov;
    message.msg_iovlen = 1;
    message.msg_control = control;
    message.msg_controllen = sizeof(control);
    ssize_t received = recvmsg(nuis_provider_worker_socket, &message, 0);
    if (received <= 0) {
        nuis_provider_worker_report_error("request-receive", -2);
        return -2;
    }
    for (struct cmsghdr* header = CMSG_FIRSTHDR(&message); header != NULL;
         header = CMSG_NXTHDR(&message, header)) {
        if (header->cmsg_level != SOL_SOCKET || header->cmsg_type != SCM_RIGHTS) {
            return nuis_provider_worker_receive_fail(-6);
        }
        if (header->cmsg_len < CMSG_LEN(0)) {
            return nuis_provider_worker_receive_fail(-6);
        }
        size_t descriptor_bytes = header->cmsg_len - CMSG_LEN(0);
        if (descriptor_bytes % sizeof(int) != 0) {
            return nuis_provider_worker_receive_fail(-6);
        }
        size_t count = descriptor_bytes / sizeof(int);
        int* raw_fds = (int*)CMSG_DATA(header);
        if (nuis_provider_worker_fd_count + count > NUIS_PROVIDER_WORKER_MAX_FDS) {
            for (size_t index = 0; index < count; index++) close(raw_fds[index]);
            return nuis_provider_worker_receive_fail(-7);
        }
        memcpy(
            nuis_provider_worker_fds + nuis_provider_worker_fd_count,
            raw_fds,
            count * sizeof(int));
        nuis_provider_worker_fd_count += count;
    }
    for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
        int flags = fcntl(nuis_provider_worker_fds[index], F_GETFD);
        if (flags < 0
            || fcntl(nuis_provider_worker_fds[index], F_SETFD, flags | FD_CLOEXEC) < 0) {
            return nuis_provider_worker_receive_fail(-9);
        }
    }
    if ((message.msg_flags & (MSG_TRUNC | MSG_CTRUNC)) != 0) {
        return nuis_provider_worker_receive_fail(-2);
    }
    nuis_provider_worker_frame[received] = '\0';
    unsigned char* header_end = memchr(nuis_provider_worker_frame, '\n', (size_t)received);
    if (header_end == NULL) return nuis_provider_worker_receive_fail(-3);
    *header_end = '\0';
    nuis_provider_worker_payload = header_end + 1;
    size_t header_length = (size_t)(header_end - nuis_provider_worker_frame);
    char magic[16] = {0};
    size_t declared_count = 0;
    if (sscanf(
            (char*)nuis_provider_worker_frame,
            "%15[^\t]\t%511[^\t]\t%llu\t%511[^\t]\t%zu\t%31[^\t]\t%zu\t%1023s",
            magic,
            nuis_provider_worker_lease,
            &nuis_provider_worker_sequence,
            nuis_provider_worker_request_id,
            &nuis_provider_worker_payload_length,
            nuis_provider_worker_payload_hash,
            &declared_count,
            nuis_provider_worker_roles) != 8
        || strcmp(magic, "NUISPWU2") != 0
        || declared_count > NUIS_PROVIDER_WORKER_MAX_FDS
        || nuis_provider_worker_role_count(nuis_provider_worker_roles) != declared_count
        || header_length + 1 + nuis_provider_worker_payload_length != (size_t)received) {
        return nuis_provider_worker_receive_fail(-4);
    }
    char actual_hash[19];
    nuis_provider_worker_hash(
        nuis_provider_worker_payload,
        nuis_provider_worker_payload_length,
        actual_hash);
    if (strcmp(actual_hash, nuis_provider_worker_payload_hash) != 0) {
        return nuis_provider_worker_receive_fail(-5);
    }
    if (nuis_provider_worker_fd_count != declared_count) {
        return nuis_provider_worker_receive_fail(-8);
    }
    nuis_provider_worker_close_requested =
        strcmp(nuis_provider_worker_request_id, "__close__") == 0;
    return 1;
}

int64_t nuis_host_provider_worker_request(void) {
    return (int64_t)nuis_provider_worker_sequence + 1;
}

int64_t nuis_host_provider_worker_descriptor_table(void) {
    return nuis_provider_worker_fd_count == 0
        ? 0
        : (int64_t)nuis_provider_worker_sequence + 1;
}

int64_t nuis_host_provider_worker_descriptor_count(void) {
    return (int64_t)nuis_provider_worker_fd_count;
}

int64_t nuis_host_provider_worker_provider_key(void) {
    return nuis_provider_worker_provider;
}

int64_t nuis_host_provider_worker_capability_hash(void) {
    return nuis_provider_worker_capability;
}

int64_t nuis_host_provider_worker_capsule_token(void) {
    return nuis_provider_worker_payload_scalar("capsule_token", "capsule-token:");
}

int64_t nuis_host_provider_worker_capsule_input_count(void) {
    return nuis_provider_worker_payload_scalar("inputs", "");
}

int64_t nuis_host_provider_worker_capsule_output_count(void) {
    return nuis_provider_worker_payload_scalar("outputs", "");
}

int64_t nuis_host_provider_worker_is_close(void) {
    return nuis_provider_worker_close_requested;
}

static int nuis_provider_worker_invoke_process_adapter(void) {
    char control[NUIS_PROVIDER_WORKER_MAX_FRAME_BYTES];
    char* fields[39];
    char launch_contract[128];
    char executable[2048];
    char executable_hash[32];
    if (!nuis_provider_worker_payload_text(
            "adapter_control", control, sizeof(control))) {
        return -1;
    }
    size_t field_count =
        nuis_provider_worker_split_tabs(control, fields, sizeof(fields) / sizeof(fields[0]));
    if (field_count < 7
        || strcmp(fields[0], "nuis-provider-worker-adapter-control-v1") != 0
        || strcmp(fields[1], "nuis-provider-worker-process-adapter-v4") != 0) {
        return -1;
    }
    char* output_end = NULL;
    char* count_end = NULL;
    long long output_byte_length = strtoll(fields[5], &output_end, 10);
    long long argument_count = strtoll(fields[6], &count_end, 10);
    int launch_length = snprintf(
        launch_contract, sizeof(launch_contract), "%s", fields[1]);
    int executable_length = snprintf(
        executable, sizeof(executable), "%s", fields[2]);
    int hash_length = snprintf(
        executable_hash, sizeof(executable_hash), "%s", fields[3]);
    if (launch_length <= 0
        || (size_t)launch_length >= sizeof(launch_contract)
        || executable_length <= 0
        || (size_t)executable_length >= sizeof(executable)
        || hash_length <= 0
        || (size_t)hash_length >= sizeof(executable_hash)
        || fields[4][0] == '\0'
        || *fields[5] == '\0'
        || *output_end != '\0'
        || *fields[6] == '\0'
        || *count_end != '\0'
        || argument_count <= 0
        || argument_count > 32
        || output_byte_length <= 0
        || field_count != 7 + (size_t)argument_count) {
        return -1;
    }
    char actual_hash[19];
    if (!nuis_provider_worker_hash_file(executable, actual_hash)
        || strcmp(actual_hash, executable_hash) != 0) {
        return -2;
    }
    nuis_provider_worker_output_file = tmpfile();
    if (nuis_provider_worker_output_file == NULL) return -3;
    FILE* protocol_file = tmpfile();
    if (protocol_file == NULL) {
        nuis_provider_worker_release_output();
        return -3;
    }
    long page_size_value = sysconf(_SC_PAGESIZE);
    if (page_size_value <= 0) {
        fclose(protocol_file);
        nuis_provider_worker_release_output();
        return -3;
    }
    size_t page_size = (size_t)page_size_value;
    size_t payload_offset = ((56 + page_size - 1) / page_size) * page_size;
    size_t mapped_length =
        (((size_t)output_byte_length + page_size - 1) / page_size) * page_size;
    if (mapped_length < (size_t)output_byte_length
        || payload_offset > SIZE_MAX - mapped_length) {
        fclose(protocol_file);
        nuis_provider_worker_release_output();
        return -3;
    }
    size_t packet_length = payload_offset + mapped_length;
    unsigned char carrier_header[56] = {0};
    memcpy(carrier_header, "NUISPFD1", 8);
    uint32_t frame_count = 1;
    uint32_t encoded_page_size = (uint32_t)page_size;
    uint64_t encoded_payload_offset = (uint64_t)payload_offset;
    uint64_t encoded_output_length = (uint64_t)output_byte_length;
    uint64_t encoded_mapped_length = (uint64_t)mapped_length;
    memcpy(carrier_header + 8, &frame_count, sizeof(frame_count));
    memcpy(carrier_header + 12, &encoded_page_size, sizeof(encoded_page_size));
    memcpy(carrier_header + 24, &encoded_payload_offset, sizeof(encoded_payload_offset));
    memcpy(carrier_header + 32, &encoded_output_length, sizeof(encoded_output_length));
    memcpy(carrier_header + 40, &encoded_mapped_length, sizeof(encoded_mapped_length));
    if (fwrite(carrier_header, 1, sizeof(carrier_header), nuis_provider_worker_output_file)
            != sizeof(carrier_header)
        || fflush(nuis_provider_worker_output_file) != 0
        || ftruncate(fileno(nuis_provider_worker_output_file), (off_t)packet_length) != 0) {
        fclose(protocol_file);
        nuis_provider_worker_release_output();
        return -3;
    }
    pid_t child = fork();
    if (child < 0) {
        fclose(protocol_file);
        nuis_provider_worker_release_output();
        return -4;
    }
    if (child == 0) {
        if (dup2(fileno(protocol_file), STDOUT_FILENO) < 0
            || dup2(fileno(protocol_file), STDERR_FILENO) < 0) {
            _exit(126);
        }
        int output_fd = fileno(nuis_provider_worker_output_file);
        int output_flags = fcntl(output_fd, F_GETFD);
        char output_descriptor[128];
        int output_descriptor_length = snprintf(
            output_descriptor,
            sizeof(output_descriptor),
            "fd:%d:%zu:%lld:48",
            output_fd,
            payload_offset,
            (long long)output_byte_length);
        if (output_flags < 0
            || fcntl(output_fd, F_SETFD, output_flags & ~FD_CLOEXEC) < 0
            || output_descriptor_length <= 0
            || (size_t)output_descriptor_length >= sizeof(output_descriptor)
            || setenv("NUIS_PROVIDER_OUTPUT_FD", output_descriptor, 1) != 0) {
            _exit(126);
        }
        for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
            int flags = fcntl(nuis_provider_worker_fds[index], F_GETFD);
            if (flags < 0
                || fcntl(
                    nuis_provider_worker_fds[index],
                    F_SETFD,
                    flags & ~FD_CLOEXEC) < 0) {
                _exit(126);
            }
        }
        char resolved_arguments[32][2048];
        char* arguments[34];
        arguments[0] = executable;
        for (int64_t index = 0; index < argument_count; index++) {
            const char* encoded = fields[7 + index];
            int resolved_length = 0;
            if (strncmp(encoded, "literal:", 8) == 0 && encoded[8] != '\0') {
                resolved_length = snprintf(
                    resolved_arguments[index],
                    sizeof(resolved_arguments[index]),
                    "%s",
                    encoded + 8);
            } else if (strncmp(encoded, "verified-path:", 14) == 0
                && strlen(encoded + 14) > 19
                && encoded[32] == ':') {
                char expected_hash[19];
                memcpy(expected_hash, encoded + 14, 18);
                expected_hash[18] = '\0';
                const char* path = encoded + 33;
                char actual_path_hash[19];
                if (*path == '\0'
                    || !nuis_provider_worker_hash_file(path, actual_path_hash)
                    || strcmp(expected_hash, actual_path_hash) != 0) {
                    _exit(126);
                }
                resolved_length = snprintf(
                    resolved_arguments[index],
                    sizeof(resolved_arguments[index]),
                    "%s",
                    path);
            } else if (strncmp(encoded, "descriptor-path:", 16) == 0) {
                char tail = '\0';
                size_t descriptor_index = 0;
                if (sscanf(encoded + 16, "%zu%c", &descriptor_index, &tail) != 1
                    || descriptor_index >= nuis_provider_worker_fd_count) {
                    _exit(126);
                }
                resolved_length = snprintf(
                    resolved_arguments[index],
                    sizeof(resolved_arguments[index]),
                    "/dev/fd/%d",
                    nuis_provider_worker_fds[descriptor_index]);
            } else if (strncmp(encoded, "descriptor-carrier:", 19) == 0) {
                char tail = '\0';
                size_t descriptor_index = 0;
                unsigned long long frame = 0;
                unsigned long long packet_length = 0;
                unsigned long long packet_hash = 0;
                if (sscanf(
                        encoded + 19,
                        "%zu:%llu:%llu:%llu%c",
                        &descriptor_index,
                        &frame,
                        &packet_length,
                        &packet_hash,
                        &tail) != 4
                    || descriptor_index >= nuis_provider_worker_fd_count) {
                    _exit(126);
                }
                const char* metadata = strchr(encoded + 19, ':');
                if (metadata == NULL || metadata[1] == '\0') _exit(126);
                resolved_length = snprintf(
                    resolved_arguments[index],
                    sizeof(resolved_arguments[index]),
                    "fd:%d:%s",
                    nuis_provider_worker_fds[descriptor_index],
                    metadata + 1);
            } else {
                _exit(126);
            }
            if (resolved_length <= 0
                || (size_t)resolved_length >= sizeof(resolved_arguments[index])) {
                _exit(126);
            }
            arguments[index + 1] = resolved_arguments[index];
        }
        arguments[argument_count + 1] = NULL;
        execv(executable, arguments);
        _exit(127);
    }
    int status = 0;
    if (waitpid(child, &status, 0) != child
        || !WIFEXITED(status)
        || WEXITSTATUS(status) != 0) {
        if (fflush(protocol_file) == 0
            && fseek(protocol_file, 0, SEEK_SET) == 0) {
            unsigned char diagnostic[4096];
            size_t count = 0;
            while ((count = fread(
                       diagnostic,
                       1,
                       sizeof(diagnostic),
                       protocol_file)) > 0) {
                fwrite(diagnostic, 1, count, stderr);
            }
            fflush(stderr);
        }
        fclose(protocol_file);
        nuis_provider_worker_release_output();
        return -5;
    }
    if (fflush(protocol_file) != 0
        || fseek(protocol_file, 0, SEEK_END) != 0) {
        fclose(protocol_file);
        nuis_provider_worker_release_output();
        return -5;
    }
    long protocol_length = ftell(protocol_file);
    if (protocol_length <= 0 || protocol_length > NUIS_PROVIDER_WORKER_MAX_FRAME_BYTES
        || fseek(protocol_file, 0, SEEK_SET) != 0
        || fread(
               nuis_provider_worker_adapter_protocol,
               1,
               (size_t)protocol_length,
               protocol_file) != (size_t)protocol_length) {
        fclose(protocol_file);
        nuis_provider_worker_release_output();
        return -6;
    }
    fclose(protocol_file);
    nuis_provider_worker_adapter_protocol_length = (size_t)protocol_length;
    nuis_provider_worker_hash(
        nuis_provider_worker_adapter_protocol,
        nuis_provider_worker_adapter_protocol_length,
        nuis_provider_worker_adapter_protocol_hash);
    uint64_t declared_output_hash = 0;
    uint64_t stored_output_hash = 0;
    uint64_t actual_output_hash = 0;
    uint64_t packet_hash = 0;
    if (!nuis_provider_worker_protocol_u64("output_hash", &declared_output_hash)
        || pread(
               fileno(nuis_provider_worker_output_file),
               &stored_output_hash,
               sizeof(stored_output_hash),
               48) != sizeof(stored_output_hash)
        || !nuis_provider_worker_hash_fd(
            fileno(nuis_provider_worker_output_file),
            payload_offset,
            (size_t)output_byte_length,
            &actual_output_hash)
        || actual_output_hash != declared_output_hash
        || stored_output_hash != declared_output_hash
        || !nuis_provider_worker_hash_fd(
            fileno(nuis_provider_worker_output_file),
            0,
            packet_length,
            &packet_hash)) {
        fprintf(
            stderr,
            "provider worker direct-result validation failed: declared=%llu stored=%llu actual=%llu bytes=%lld packet=%zu\n",
            (unsigned long long)declared_output_hash,
            (unsigned long long)stored_output_hash,
            (unsigned long long)actual_output_hash,
            (long long)output_byte_length,
            packet_length);
        fflush(stderr);
        nuis_provider_worker_release_output();
        return -7;
    }
    nuis_provider_worker_output_length = packet_length;
    snprintf(
        nuis_provider_worker_output_hash,
        sizeof(nuis_provider_worker_output_hash),
        "0x%016llx",
        (unsigned long long)packet_hash);
    return 0;
}

int64_t nuis_host_provider_worker_invoke_capsule(int64_t ingress_status) {
    if (nuis_provider_worker_socket < 0
        || nuis_provider_worker_close_requested
        || ingress_status <= 0
        || nuis_provider_worker_payload_scalar("invoker_token", "invoker-token:") <= 0
        || nuis_provider_worker_payload_scalar("outputs", "") != 1
        || !nuis_provider_worker_payload_text(
            "output_roles",
            nuis_provider_worker_output_roles,
            sizeof(nuis_provider_worker_output_roles))
        || strchr(nuis_provider_worker_output_roles, ',') != NULL) {
        fprintf(stderr, "provider worker capsule metadata validation failed\n");
        fflush(stderr);
        nuis_provider_worker_report_error("capsule-metadata", -1);
        return -1;
    }
    for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
        unsigned char value = 0;
        if (pread(nuis_provider_worker_fds[index], &value, 1, 0) != 1) {
            fprintf(
                stderr,
                "provider worker input descriptor %zu is unreadable\n",
                index);
            fflush(stderr);
            nuis_provider_worker_report_error("input-descriptor", -2);
            return -2;
        }
        nuis_provider_worker_input_byte_sum += value;
    }
    if (nuis_provider_worker_payload_has("adapter_control")) {
        int adapter_status = nuis_provider_worker_invoke_process_adapter();
        if (adapter_status != 0) {
            fprintf(
                stderr,
                "provider worker process adapter failed with status %d\n",
                adapter_status);
            fflush(stderr);
            nuis_provider_worker_report_error("process-adapter", adapter_status);
            return -3;
        }
        return ingress_status;
    }
    uint64_t output_words[3] = {
        (uint64_t)nuis_provider_worker_payload_scalar("capsule_token", "capsule-token:"),
        (uint64_t)ingress_status,
        (uint64_t)nuis_provider_worker_input_byte_sum,
    };
    nuis_provider_worker_output_file = tmpfile();
    if (nuis_provider_worker_output_file == NULL) return -3;
    nuis_provider_worker_output_length = sizeof(output_words);
    if (fwrite(
            output_words,
            1,
            nuis_provider_worker_output_length,
            nuis_provider_worker_output_file) != nuis_provider_worker_output_length
        || fflush(nuis_provider_worker_output_file) != 0) {
        nuis_provider_worker_release_output();
        return -4;
    }
    nuis_provider_worker_hash(
        (const unsigned char*)output_words,
        nuis_provider_worker_output_length,
        nuis_provider_worker_output_hash);
    return ingress_status;
}

int64_t nuis_host_provider_worker_reply(int64_t invocation_status) {
    if (nuis_provider_worker_socket < 0) return -1;
    if (invocation_status <= 0) {
        nuis_provider_worker_report_error("nuis-ingress", (int)invocation_status);
        return -1;
    }
    size_t output_count = nuis_provider_worker_output_file == NULL ? 0 : 1;
    const char* output_roles =
        output_count == 0 ? "-" : nuis_provider_worker_output_roles;
    const char* output_hash =
        output_count == 0 ? "0x0000000000000000" : nuis_provider_worker_output_hash;
    const char* output_mode = output_count == 0
        ? "none"
        : (nuis_provider_worker_adapter_protocol_length == 0
            ? "protocol-stdout"
            : "nuispfd1-result");
    char receipt[4096];
    int header_length = snprintf(
        receipt,
        sizeof(receipt),
        "NUISPWUR6\t%s\t%llu\t%s\t%d\t%zu\t%u\t%lld\t%s\t%s\t%zu\t%s\t%zu\t%s\t%zu\t%s\t%s\t%zu\t%s\n",
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
        output_count,
        output_roles,
        nuis_provider_worker_output_length,
        output_hash,
        output_mode,
        nuis_provider_worker_adapter_protocol_length,
        nuis_provider_worker_adapter_protocol_hash);
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
    char control[CMSG_SPACE(sizeof(int))] = {0};
    if (output_count == 1) {
        message.msg_control = control;
        message.msg_controllen = sizeof(control);
        struct cmsghdr* header = CMSG_FIRSTHDR(&message);
        if (header == NULL) return -2;
        header->cmsg_level = SOL_SOCKET;
        header->cmsg_type = SCM_RIGHTS;
        header->cmsg_len = CMSG_LEN(sizeof(int));
        int output_fd = fileno(nuis_provider_worker_output_file);
        memcpy(CMSG_DATA(header), &output_fd, sizeof(output_fd));
    }
    size_t expected = parts[0].iov_len + parts[1].iov_len;
    ssize_t sent = sendmsg(nuis_provider_worker_socket, &message, 0);
    int send_error = errno;
    if (sent != (ssize_t)expected) {
        nuis_provider_worker_report_error("reply-send", sent < 0 ? -send_error : (int)sent);
    }
    nuis_provider_worker_release_fds();
    nuis_provider_worker_release_output();
    return sent == (ssize_t)expected ? 0 : -3;
}

int64_t nuis_host_provider_worker_close(void) {
    nuis_provider_worker_release_fds();
    nuis_provider_worker_release_output();
    if (nuis_provider_worker_socket >= 0) close(nuis_provider_worker_socket);
    nuis_provider_worker_socket = -1;
    return 0;
}
"#,
    );
}
