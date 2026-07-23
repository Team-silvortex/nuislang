pub(crate) fn append_c_shim_provider_worker_runtime(out: &mut String) {
    out.push_str(
        r#"

#include <sys/uio.h>

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

static void nuis_provider_worker_release_fds(void) {
    for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
        close(nuis_provider_worker_fds[index]);
    }
    nuis_provider_worker_fd_count = 0;
}

static int64_t nuis_provider_worker_receive_fail(int64_t status) {
    nuis_provider_worker_release_fds();
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

static size_t nuis_provider_worker_role_count(const char* roles) {
    if (strcmp(roles, "-") == 0) return 0;
    size_t count = 1;
    for (const char* cursor = roles; *cursor != '\0'; cursor++) {
        if (*cursor == ',') count++;
    }
    return count;
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

int64_t nuis_host_provider_worker_is_close(void) {
    return nuis_provider_worker_close_requested;
}

int64_t nuis_host_provider_worker_reply(int64_t ingress_status) {
    if (nuis_provider_worker_socket < 0 || ingress_status <= 0) return -1;
    unsigned int byte_sum = 0;
    for (size_t index = 0; index < nuis_provider_worker_fd_count; index++) {
        unsigned char value = 0;
        if (pread(nuis_provider_worker_fds[index], &value, 1, 0) != 1) {
            nuis_provider_worker_release_fds();
            return -2;
        }
        byte_sum += value;
    }
    char receipt[4096];
    int header_length = snprintf(
        receipt,
        sizeof(receipt),
        "NUISPWUR3\t%s\t%llu\t%s\t%d\t%zu\t%u\t%s\t%s\t%zu\t%s\n",
        nuis_provider_worker_lease,
        nuis_provider_worker_sequence,
        nuis_provider_worker_request_id,
        getpid(),
        nuis_provider_worker_fd_count,
        byte_sum,
        nuis_provider_worker_payload_hash,
        nuis_provider_worker_roles,
        nuis_provider_worker_payload_length,
        nuis_provider_worker_payload_hash);
    struct iovec parts[2] = {
        {.iov_base = receipt, .iov_len = (size_t)header_length},
        {.iov_base = nuis_provider_worker_payload, .iov_len = nuis_provider_worker_payload_length},
    };
    struct msghdr message = {0};
    message.msg_iov = parts;
    message.msg_iovlen = 2;
    size_t expected = parts[0].iov_len + parts[1].iov_len;
    ssize_t sent = sendmsg(nuis_provider_worker_socket, &message, 0);
    nuis_provider_worker_release_fds();
    return sent == (ssize_t)expected ? 0 : -3;
}

int64_t nuis_host_provider_worker_close(void) {
    nuis_provider_worker_release_fds();
    if (nuis_provider_worker_socket >= 0) close(nuis_provider_worker_socket);
    nuis_provider_worker_socket = -1;
    return 0;
}
"#,
    );
}
