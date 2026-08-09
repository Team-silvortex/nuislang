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
"#,
    );
}
