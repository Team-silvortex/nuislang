#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static uint64_t fnv1a64(const unsigned char* bytes, size_t length) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    for (size_t index = 0; index < length; index++) {
        hash ^= bytes[index];
        hash *= UINT64_C(0x100000001b3);
    }
    return hash;
}

static int write_output(
    int fd,
    size_t payload_offset,
    size_t payload_length,
    size_t hash_offset,
    const unsigned char payload[24],
    uint64_t* hash) {
    if (payload_length != 24) return 0;
    *hash = fnv1a64(payload, payload_length);
    return pwrite(fd, payload, payload_length, (off_t)payload_offset)
            == (ssize_t)payload_length
        && pwrite(fd, hash, sizeof(*hash), (off_t)hash_offset)
            == (ssize_t)sizeof(*hash);
}

int main(void) {
    const char* manifest = getenv("NUIS_PROVIDER_OUTPUT_FDS");
    int primary_fd = -1;
    int audit_fd = -1;
    size_t primary_offset = 0;
    size_t primary_length = 0;
    size_t primary_hash_offset = 0;
    size_t audit_offset = 0;
    size_t audit_length = 0;
    size_t audit_hash_offset = 0;
    if (manifest == NULL
        || sscanf(
            manifest,
            "output.primary=fd:%d:%zu:%zu:%zu,"
            "output.audit=fd:%d:%zu:%zu:%zu",
            &primary_fd,
            &primary_offset,
            &primary_length,
            &primary_hash_offset,
            &audit_fd,
            &audit_offset,
            &audit_length,
            &audit_hash_offset) != 8) return 2;
    const unsigned char primary[24] = {
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
        13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24
    };
    const unsigned char audit[24] = {
        31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42,
        43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54
    };
    uint64_t primary_hash = 0;
    uint64_t audit_hash = 0;
    if (!write_output(
            primary_fd,
            primary_offset,
            primary_length,
            primary_hash_offset,
            primary,
            &primary_hash)
        || !write_output(
            audit_fd,
            audit_offset,
            audit_length,
            audit_hash_offset,
            audit,
            &audit_hash)) return 3;
    printf(
        "output_channel=inherited-fds\noutput_hashes=%" PRIu64 ",%" PRIu64 "\n",
        primary_hash,
        audit_hash);
    return fflush(stdout) == 0 ? 0 : 4;
}
