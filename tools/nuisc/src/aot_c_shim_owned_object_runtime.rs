pub(crate) fn append_c_shim_owned_object_runtime(out: &mut String) {
    out.push_str(
        r#"

#define NUIS_OWNED_OBJECT_MAGIC_V1 UINT64_C(0x4e5549534f424a31)

typedef struct {
    uint64_t magic;
    int64_t values[2];
} NuisOwnedObjectV1;

static int64_t nuis_owned_object_live_v1 = 0;

void* host_owned_object_make(int64_t seed) {
    NuisOwnedObjectV1* object = (NuisOwnedObjectV1*)malloc(sizeof(NuisOwnedObjectV1));
    if (object == NULL) {
        return NULL;
    }
    object->magic = NUIS_OWNED_OBJECT_MAGIC_V1;
    object->values[0] = seed;
    object->values[1] = seed * INT64_C(3) + INT64_C(1);
    nuis_owned_object_live_v1 += 1;
    return object->values;
}

int64_t nuis_host_owned_object_validate_v1(void* payload) {
    if (payload == NULL) {
        return -1;
    }
    NuisOwnedObjectV1* object = (NuisOwnedObjectV1*)((uint64_t*)payload - 1);
    if (object->magic != NUIS_OWNED_OBJECT_MAGIC_V1) {
        return -2;
    }
    return INT64_C(16);
}

int64_t host_owned_object_destroy(void* payload) {
    if (nuis_host_owned_object_validate_v1(payload) != INT64_C(16)) {
        return -1;
    }
    NuisOwnedObjectV1* object = (NuisOwnedObjectV1*)((uint64_t*)payload - 1);
    object->magic = 0;
    free(object);
    nuis_owned_object_live_v1 -= 1;
    return 0;
}

int64_t host_owned_object_live_count(void) {
    return nuis_owned_object_live_v1;
}
"#,
    );
}
