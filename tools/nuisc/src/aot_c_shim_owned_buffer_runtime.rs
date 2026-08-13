pub(crate) fn append_c_shim_owned_buffer_runtime(out: &mut String) {
    out.push_str(
        r#"

#define NUIS_OWNED_BUFFER_MAGIC_V1 UINT64_C(0x4e55495342554631)

typedef struct {
    uint64_t magic;
    int64_t len;
} NuisOwnedBufferHeaderV1;

void* host_owned_buffer_make(int64_t seed) {
    const int64_t len = 4;
    NuisOwnedBufferHeaderV1* header =
        (NuisOwnedBufferHeaderV1*)malloc(sizeof(NuisOwnedBufferHeaderV1) + (size_t)len * sizeof(int64_t));
    if (header == NULL) {
        return NULL;
    }
    header->magic = NUIS_OWNED_BUFFER_MAGIC_V1;
    header->len = len;
    int64_t* payload = (int64_t*)(header + 1);
    for (int64_t index = 0; index < len; index += 1) {
        payload[index] = seed + index;
    }
    return payload;
}

int64_t host_owned_buffer_destroy(void* payload) {
    if (payload == NULL) {
        return -1;
    }
    NuisOwnedBufferHeaderV1* header = ((NuisOwnedBufferHeaderV1*)payload) - 1;
    if (header->magic != NUIS_OWNED_BUFFER_MAGIC_V1 || header->len < 0) {
        return -2;
    }
    header->magic = 0;
    free(header);
    return 0;
}

#define NUIS_OWNED_UTF8_MAGIC_V1 UINT64_C(0x4e55495355544631)

typedef struct {
    uint64_t magic;
    int64_t len;
} NuisOwnedUtf8HeaderV1;

static int64_t nuis_owned_utf8_live_v1 = 0;

void* host_owned_utf8_make(int64_t seed) {
    const int64_t len = 7;
    NuisOwnedUtf8HeaderV1* header =
        (NuisOwnedUtf8HeaderV1*)malloc(sizeof(NuisOwnedUtf8HeaderV1) + (size_t)len + 1);
    if (header == NULL) {
        return NULL;
    }
    header->magic = NUIS_OWNED_UTF8_MAGIC_V1;
    header->len = len;
    unsigned char* payload = (unsigned char*)(header + 1);
    payload[0] = (unsigned char)('A' + (seed % 26 + 26) % 26);
    payload[1] = 'u';
    payload[2] = 'i';
    payload[3] = 's';
    payload[4] = '-';
    payload[5] = 0xce;
    payload[6] = 0xb2;
    payload[7] = 0;
    nuis_owned_utf8_live_v1 += 1;
    return payload;
}

int64_t nuis_host_owned_utf8_validate_v1(void* payload) {
    if (payload == NULL) {
        return -1;
    }
    NuisOwnedUtf8HeaderV1* header = ((NuisOwnedUtf8HeaderV1*)payload) - 1;
    if (header->magic != NUIS_OWNED_UTF8_MAGIC_V1
        || header->len < 0
        || header->len > INT64_C(1048576)) {
        return -2;
    }
    char* text = (char*)payload;
    size_t len = (size_t)header->len;
    if (text[len] != '\0' || !nuis_host_text_is_valid_utf8(text, len)) {
        return -3;
    }
    return header->len;
}

int64_t host_owned_utf8_destroy(void* payload) {
    if (nuis_host_owned_utf8_validate_v1(payload) < 0) {
        return -1;
    }
    NuisOwnedUtf8HeaderV1* header = ((NuisOwnedUtf8HeaderV1*)payload) - 1;
    header->magic = 0;
    free(header);
    nuis_owned_utf8_live_v1 -= 1;
    return 0;
}

int64_t host_owned_utf8_live_count(void) {
    return nuis_owned_utf8_live_v1;
}
"#,
    );
}
