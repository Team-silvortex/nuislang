#define _POSIX_C_SOURCE 200809L

#include <dlfcn.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#define NUIS_VULKAN_MAX_OUTPUTS 8

typedef struct {
    int fd;
    size_t payload_offset;
    size_t payload_length;
    size_t hash_offset;
} OutputDescriptor;

typedef struct {
    size_t logical_length;
    size_t carrier_length;
    size_t row_length;
    size_t row_stride;
    size_t row_count;
} OutputLayout;

static uint64_t fnv1a64(const unsigned char* bytes, size_t length) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    for (size_t index = 0; index < length; index++) {
        hash ^= bytes[index];
        hash *= UINT64_C(0x100000001b3);
    }
    return hash;
}

static int parse_output_descriptor_text(const char* text, OutputDescriptor* output) {
    char tail = '\0';
    return text != NULL
        && sscanf(text, "fd:%d:%zu:%zu:%zu%c", &output->fd, &output->payload_offset, &output->payload_length, &output->hash_offset, &tail) == 4;
}

static int parse_output_descriptors(OutputDescriptor* outputs, size_t* output_count) {
    const char* manifest = getenv("NUIS_PROVIDER_OUTPUT_FDS");
    if (manifest == NULL || manifest[0] == '\0') {
        *output_count = 1;
        return parse_output_descriptor_text(getenv("NUIS_PROVIDER_OUTPUT_FD"), &outputs[0]);
    }
    char* copy = strdup(manifest);
    if (copy == NULL) return 0;
    size_t count = 0;
    char* state = NULL;
    for (char* item = strtok_r(copy, ",", &state); item != NULL; item = strtok_r(NULL, ",", &state)) {
        char* descriptor = strchr(item, '=');
        if (descriptor == NULL || descriptor == item || count == NUIS_VULKAN_MAX_OUTPUTS) {
            free(copy);
            return 0;
        }
        *descriptor++ = '\0';
        if (item[0] == '\0' || !parse_output_descriptor_text(descriptor, &outputs[count])) {
            free(copy);
            return 0;
        }
        count++;
    }
    free(copy);
    if (count == 0) return 0;
    *output_count = count;
    return 1;
}

static int parse_output_layouts(
    const char* manifest,
    OutputLayout* outputs,
    size_t expected_count) {
    if (manifest == NULL || manifest[0] == '\0') return 0;
    char* copy = strdup(manifest);
    if (copy == NULL) return 0;
    size_t count = 0;
    char* state = NULL;
    for (char* item = strtok_r(copy, ",", &state); item != NULL; item = strtok_r(NULL, ",", &state)) {
        char tail = '\0';
        if (count == expected_count
            || sscanf(item, "%zu:%zu:%zu:%zu:%zu%c",
                &outputs[count].logical_length,
                &outputs[count].carrier_length,
                &outputs[count].row_length,
                &outputs[count].row_stride,
                &outputs[count].row_count,
                &tail) != 5
            || outputs[count].logical_length == 0
            || outputs[count].row_length == 0
            || outputs[count].row_count == 0
            || outputs[count].row_stride < outputs[count].row_length
            || outputs[count].row_length > SIZE_MAX / outputs[count].row_count
            || outputs[count].row_length * outputs[count].row_count != outputs[count].logical_length
            || outputs[count].row_stride > SIZE_MAX / outputs[count].row_count
            || outputs[count].row_stride * outputs[count].row_count != outputs[count].carrier_length) {
            free(copy);
            return 0;
        }
        count++;
    }
    free(copy);
    return count == expected_count;
}

static int parse_u32(const char* text, uint32_t* value) {
    char* end = NULL;
    unsigned long parsed = strtoul(text, &end, 10);
    if (text == end || *end != '\0' || parsed == 0 || parsed > UINT32_MAX) return 0;
    *value = (uint32_t)parsed;
    return 1;
}

static int valid_spirv_entry(const char* entry) {
    if (entry == NULL || entry[0] == '\0') return 0;
    size_t length = strlen(entry);
    if (length > 127 || !((entry[0] >= 'A' && entry[0] <= 'Z') || (entry[0] >= 'a' && entry[0] <= 'z') || entry[0] == '_')) return 0;
    for (size_t index = 1; index < length; index++) {
        char byte = entry[index];
        if (!((byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z') || (byte >= '0' && byte <= '9') || byte == '_')) return 0;
    }
    return 1;
}

static uint32_t read_u32_le(const unsigned char* bytes) {
    uint32_t value = 0;
    for (size_t index = 0; index < 4; index++) value |= (uint32_t)bytes[index] << (index * 8);
    return value;
}

static uint64_t read_u64_le(const unsigned char* bytes) {
    uint64_t value = 0;
    for (size_t index = 0; index < 8; index++) value |= (uint64_t)bytes[index] << (index * 8);
    return value;
}

static int pread_exact(int fd, unsigned char* output, size_t length, size_t offset) {
    size_t read = 0;
    while (read < length) {
        ssize_t count = pread(fd, output + read, length - read, (off_t)(offset + read));
        if (count <= 0) return 0;
        read += (size_t)count;
    }
    return 1;
}

static int read_carrier_frame(const char* descriptor, unsigned char* output, size_t output_length) {
    int fd = -1;
    unsigned long long frame = 0;
    unsigned long long packet_length_raw = 0;
    unsigned long long packet_hash = 0;
    char tail = '\0';
    if (sscanf(descriptor, "fd:%d:%llu:%llu:%llu%c", &fd, &frame, &packet_length_raw, &packet_hash, &tail) != 4
        || fd < 0 || frame > UINT32_MAX || packet_length_raw > SIZE_MAX) return 0;
    size_t packet_length = (size_t)packet_length_raw;
    if (packet_length < 56) return 0;
    unsigned char* packet = malloc(packet_length);
    if (packet == NULL || !pread_exact(fd, packet, packet_length, 0)) {
        free(packet);
        return 0;
    }
    uint32_t frame_count = read_u32_le(packet + 8);
    uint32_t page_size = read_u32_le(packet + 12);
    uint32_t frame_index = read_u32_le(packet + 16);
    uint64_t payload_offset_raw = read_u64_le(packet + 24);
    uint64_t payload_length_raw = read_u64_le(packet + 32);
    uint64_t mapped_length_raw = read_u64_le(packet + 40);
    uint64_t payload_hash = read_u64_le(packet + 48);
    int valid = memcmp(packet, "NUISPFD1", 8) == 0
        && fnv1a64(packet, packet_length) == (uint64_t)packet_hash
        && frame_count == 1 && page_size != 0 && (page_size & (page_size - 1)) == 0
        && frame_index == (uint32_t)frame && payload_offset_raw <= SIZE_MAX
        && payload_length_raw == output_length && mapped_length_raw <= SIZE_MAX;
    size_t payload_offset = valid ? (size_t)payload_offset_raw : 0;
    size_t mapped_length = valid ? (size_t)mapped_length_raw : 0;
    valid = valid && payload_offset >= 56 && payload_offset % page_size == 0
        && mapped_length % page_size == 0 && mapped_length >= output_length
        && payload_offset <= packet_length && mapped_length <= packet_length - payload_offset
        && fnv1a64(packet + payload_offset, output_length) == payload_hash;
    if (valid) memcpy(output, packet + payload_offset, output_length);
    free(packet);
    return valid;
}

static int read_exact_file(const char* path, unsigned char* output, size_t length) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 0;
    size_t read = fread(output, 1, length, file);
    int trailing = fgetc(file);
    int closed = fclose(file);
    return read == length && trailing == EOF && closed == 0;
}

static int read_input(const char* source, unsigned char* output, size_t length) {
    return strncmp(source, "fd:", 3) == 0 ? read_carrier_frame(source, output, length) : read_exact_file(source, output, length);
}

static int read_file_alloc(const char* path, unsigned char** output, size_t* length) {
    FILE* file = fopen(path, "rb");
    if (file == NULL || fseek(file, 0, SEEK_END) != 0) return 0;
    long end = ftell(file);
    if (end <= 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return 0;
    }
    unsigned char* bytes = malloc((size_t)end);
    if (bytes == NULL) {
        fclose(file);
        return 0;
    }
    size_t read = fread(bytes, 1, (size_t)end, file);
    int closed = fclose(file);
    if (read != (size_t)end || closed != 0) {
        free(bytes);
        return 0;
    }
    *output = bytes;
    *length = (size_t)end;
    return 1;
}
