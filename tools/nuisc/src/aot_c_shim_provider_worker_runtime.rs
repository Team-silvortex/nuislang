pub(crate) fn append_c_shim_provider_worker_runtime(out: &mut String) {
    out.push_str(
        r#"

#include <sys/uio.h>
#include <sys/wait.h>

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
static unsigned int nuis_provider_worker_input_byte_sum = 0;

static void nuis_provider_worker_release_output(void) {
    if (nuis_provider_worker_output_file != NULL) {
        fclose(nuis_provider_worker_output_file);
        nuis_provider_worker_output_file = NULL;
    }
    nuis_provider_worker_output_length = 0;
    memcpy(nuis_provider_worker_output_hash, "0x0000000000000000", 19);
    memcpy(nuis_provider_worker_output_roles, "-", 2);
    nuis_provider_worker_input_byte_sum = 0;
}

static void nuis_provider_worker_release_fds(void) {
    for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
        close(nuis_provider_worker_fds[index]);
    }
    nuis_provider_worker_fd_count = 0;
}

static int64_t nuis_provider_worker_receive_fail(int64_t status) {
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
    if (received <= 0) return -2;
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
    char launch_contract[128];
    char executable[2048];
    char executable_hash[32];
    char scalar[128];
    if (nuis_provider_worker_fd_count != 1
        || !nuis_provider_worker_payload_text(
            "adapter_launch_contract", launch_contract, sizeof(launch_contract))
        || strcmp(launch_contract, "nuis-provider-worker-process-adapter-v1") != 0
        || !nuis_provider_worker_payload_text(
            "adapter_executable", executable, sizeof(executable))
        || !nuis_provider_worker_payload_text(
            "adapter_executable_hash", executable_hash, sizeof(executable_hash))
        || !nuis_provider_worker_payload_text(
            "adapter_scalar_argument", scalar, sizeof(scalar))) {
        return -1;
    }
    char actual_hash[19];
    if (!nuis_provider_worker_hash_file(executable, actual_hash)
        || strcmp(actual_hash, executable_hash) != 0) {
        return -2;
    }
    nuis_provider_worker_output_file = tmpfile();
    if (nuis_provider_worker_output_file == NULL) return -3;
    pid_t child = fork();
    if (child < 0) {
        nuis_provider_worker_release_output();
        return -4;
    }
    if (child == 0) {
        int input_fd = nuis_provider_worker_fds[0];
        int flags = fcntl(input_fd, F_GETFD);
        if (flags < 0
            || fcntl(input_fd, F_SETFD, flags & ~FD_CLOEXEC) < 0
            || dup2(fileno(nuis_provider_worker_output_file), STDOUT_FILENO) < 0) {
            _exit(126);
        }
        char input_path[64];
        if (snprintf(input_path, sizeof(input_path), "/dev/fd/%d", input_fd) <= 0) {
            _exit(126);
        }
        execl(executable, executable, input_path, scalar, (char*)NULL);
        _exit(127);
    }
    int status = 0;
    if (waitpid(child, &status, 0) != child
        || !WIFEXITED(status)
        || WEXITSTATUS(status) != 0
        || fflush(nuis_provider_worker_output_file) != 0
        || fseek(nuis_provider_worker_output_file, 0, SEEK_END) != 0) {
        nuis_provider_worker_release_output();
        return -5;
    }
    long output_length = ftell(nuis_provider_worker_output_file);
    if (output_length <= 0 || output_length > NUIS_PROVIDER_WORKER_MAX_FRAME_BYTES
        || fseek(nuis_provider_worker_output_file, 0, SEEK_SET) != 0) {
        nuis_provider_worker_release_output();
        return -6;
    }
    unsigned char buffer[4096];
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    size_t remaining = (size_t)output_length;
    while (remaining > 0) {
        size_t requested = remaining < sizeof(buffer) ? remaining : sizeof(buffer);
        size_t count = fread(buffer, 1, requested, nuis_provider_worker_output_file);
        if (count != requested) {
            nuis_provider_worker_release_output();
            return -7;
        }
        for (size_t index = 0; index < count; index++) {
            hash ^= buffer[index];
            hash *= UINT64_C(0x100000001b3);
        }
        remaining -= count;
    }
    nuis_provider_worker_output_length = (size_t)output_length;
    snprintf(
        nuis_provider_worker_output_hash,
        sizeof(nuis_provider_worker_output_hash),
        "0x%016llx",
        (unsigned long long)hash);
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
        return -1;
    }
    for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
        unsigned char value = 0;
        if (pread(nuis_provider_worker_fds[index], &value, 1, 0) != 1) {
            return -2;
        }
        nuis_provider_worker_input_byte_sum += value;
    }
    char adapter_launch_contract[128];
    if (nuis_provider_worker_payload_text(
            "adapter_launch_contract",
            adapter_launch_contract,
            sizeof(adapter_launch_contract))) {
        return nuis_provider_worker_invoke_process_adapter() == 0 ? ingress_status : -3;
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
    if (nuis_provider_worker_socket < 0 || invocation_status <= 0) return -1;
    size_t output_count = nuis_provider_worker_output_file == NULL ? 0 : 1;
    const char* output_roles =
        output_count == 0 ? "-" : nuis_provider_worker_output_roles;
    const char* output_hash =
        output_count == 0 ? "0x0000000000000000" : nuis_provider_worker_output_hash;
    char receipt[4096];
    int header_length = snprintf(
        receipt,
        sizeof(receipt),
        "NUISPWUR5\t%s\t%llu\t%s\t%d\t%zu\t%u\t%lld\t%s\t%s\t%zu\t%s\t%zu\t%s\t%zu\t%s\n",
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
        output_hash);
    struct iovec parts[2] = {
        {.iov_base = receipt, .iov_len = (size_t)header_length},
        {.iov_base = nuis_provider_worker_payload, .iov_len = nuis_provider_worker_payload_length},
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
