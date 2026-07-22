#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#define MAX_FDS 16
#define MAX_FRAME_BYTES (64 * 1024)

static int send_text(int socket_fd, const char *text) {
    size_t length = strlen(text);
    return send(socket_fd, text, length, 0) == (ssize_t)length ? 0 : 1;
}

static void fnv1a64_hex(const unsigned char *bytes, size_t length, char output[19]) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    for (size_t index = 0; index < length; index++) {
        hash ^= bytes[index];
        hash *= UINT64_C(0x100000001b3);
    }
    snprintf(output, 19, "0x%016llx", (unsigned long long)hash);
}

static size_t role_count(const char *manifest) {
    if (strcmp(manifest, "-") == 0) return 0;
    size_t count = 1;
    for (const char *cursor = manifest; *cursor != '\0'; cursor++) {
        if (*cursor == ',') count++;
    }
    return count;
}

int main(void) {
    const char *fd_text = getenv("NUIS_PROVIDER_WORKER_SOCKET_FD");
    if (fd_text == NULL) return 2;
    char *end = NULL;
    long parsed_fd = strtol(fd_text, &end, 10);
    if (*fd_text == '\0' || *end != '\0' || parsed_fd < 0 || parsed_fd > 0x7fffffff) return 3;
    int socket_fd = (int)parsed_fd;
    char handshake[64];
    snprintf(handshake, sizeof(handshake), "NUISPWUH1\t%d", getpid());
    if (send_text(socket_fd, handshake) != 0) return 4;

    for (;;) {
        unsigned char frame[MAX_FRAME_BYTES] = {0};
        char control[CMSG_SPACE(sizeof(int) * MAX_FDS)] = {0};
        struct iovec iov = {.iov_base = frame, .iov_len = sizeof(frame) - 1};
        struct msghdr message = {0};
        message.msg_iov = &iov;
        message.msg_iovlen = 1;
        message.msg_control = control;
        message.msg_controllen = sizeof(control);
        ssize_t received = recvmsg(socket_fd, &message, 0);
        if (received <= 0 || (message.msg_flags & (MSG_TRUNC | MSG_CTRUNC)) != 0) return 5;
        if ((size_t)received == sizeof(frame)) return 6;
        frame[received] = '\0';
        unsigned char *header_end = memchr(frame, '\n', (size_t)received);
        if (header_end == NULL) return 6;
        *header_end = '\0';
        unsigned char *payload = header_end + 1;
        size_t header_length = (size_t)(header_end - frame);

        char magic[16] = {0};
        char lease[512] = {0};
        char request[512] = {0};
        char declared_hash[32] = {0};
        char roles[1024] = {0};
        size_t sequence = 0;
        size_t payload_length = 0;
        size_t declared_count = 0;
        if (sscanf((char *)frame,
                   "%15[^\t]\t%511[^\t]\t%zu\t%511[^\t]\t%zu\t%31[^\t]\t%zu\t%1023s",
                   magic, lease, &sequence, request, &payload_length, declared_hash,
                   &declared_count, roles) != 8 ||
            strcmp(magic, "NUISPWU2") != 0 || declared_count > MAX_FDS ||
            role_count(roles) != declared_count ||
            header_length + 1 + payload_length != (size_t)received) return 6;
        char actual_hash[19];
        fnv1a64_hex(payload, payload_length, actual_hash);
        if (strcmp(actual_hash, declared_hash) != 0) return 6;

        int descriptors[MAX_FDS];
        size_t descriptor_count = 0;
        for (struct cmsghdr *header = CMSG_FIRSTHDR(&message); header != NULL;
             header = CMSG_NXTHDR(&message, header)) {
            if (header->cmsg_level != SOL_SOCKET || header->cmsg_type != SCM_RIGHTS) return 7;
            size_t count = (header->cmsg_len - CMSG_LEN(0)) / sizeof(int);
            if (descriptor_count + count > MAX_FDS) return 8;
            memcpy(descriptors + descriptor_count, CMSG_DATA(header), count * sizeof(int));
            descriptor_count += count;
        }
        if (descriptor_count != declared_count) return 9;

        unsigned int byte_sum = 0;
        for (size_t index = 0; index < descriptor_count; index++) {
            int flags = fcntl(descriptors[index], F_GETFD);
            if (flags < 0 || fcntl(descriptors[index], F_SETFD, flags | FD_CLOEXEC) < 0) return 10;
            unsigned char value = 0;
            if (pread(descriptors[index], &value, 1, 0) != 1) return 11;
            byte_sum += value;
            close(descriptors[index]);
        }
        char receipt[4096];
        snprintf(receipt, sizeof(receipt),
                 "NUISPWUR2\t%s\t%zu\t%s\t%d\t%zu\t%u\t%s\t%s",
                 lease, sequence, request, getpid(), descriptor_count, byte_sum,
                 actual_hash, roles);
        if (send_text(socket_fd, receipt) != 0) return 12;
        if (strcmp(request, "__close__") == 0) break;
    }
    close(socket_fd);
    return 0;
}
