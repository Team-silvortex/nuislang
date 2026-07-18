pub(crate) fn append_c_shim_owned_blob_runtime(out: &mut String) {
    out.push_str(
        r#"

#define NUIS_SCHEDULER_OWNED_BLOB_TAG_V1 UINT64_C(0x4e53424c4f423031)

typedef struct {
    uint64_t protocol_tag;
    uint64_t glm_token;
    int64_t byte_len;
    unsigned char bytes[];
} NuisSchedulerOwnedBlobV1;

typedef struct {
    int64_t kind;
    int64_t value;
} NuisSchedulerOwnedAggregateSlotV1;

typedef struct {
    uint64_t protocol_tag;
    int64_t slot_count;
    int64_t state;
    NuisSchedulerOwnedAggregateSlotV1 slots[];
} NuisSchedulerOwnedAggregateV1;

#define NUIS_SCHEDULER_OWNED_AGGREGATE_TAG_V1 UINT64_C(0x4e53414747523031)
#define NUIS_SCHEDULER_OWNED_SLOT_UNSET_V1 INT64_C(-1)
#define NUIS_SCHEDULER_OWNED_SLOT_SCALAR_V1 INT64_C(0)
#define NUIS_SCHEDULER_OWNED_SLOT_BLOB_V1 INT64_C(1)
#define NUIS_SCHEDULER_OWNED_AGGREGATE_BUILDING_V1 INT64_C(1)
#define NUIS_SCHEDULER_OWNED_AGGREGATE_FINALIZED_V1 INT64_C(2)
#define NUIS_SCHEDULER_OWNED_AGGREGATE_POISONED_V1 INT64_C(3)

static int64_t nuis_scheduler_owned_blob_live_count_v1 = 0;

void nuis_scheduler_owned_aggregate_drop_v1(void* data);

static int nuis_scheduler_owned_blob_valid_v1(const void* data) {
    if (data == NULL) return 0;
    const NuisSchedulerOwnedBlobV1* blob =
        (const NuisSchedulerOwnedBlobV1*)data;
    return blob->protocol_tag == NUIS_SCHEDULER_OWNED_BLOB_TAG_V1
        && blob->glm_token != 0
        && blob->byte_len >= 0;
}

void* nuis_scheduler_owned_blob_copy_v1(
    const void* source,
    int64_t byte_len,
    uint64_t glm_token
) {
    if (byte_len < 0 || glm_token == 0) return NULL;
    if (byte_len > 0 && source == NULL) return NULL;
    if ((uint64_t)byte_len > SIZE_MAX - sizeof(NuisSchedulerOwnedBlobV1)) {
        return NULL;
    }
    size_t allocation_size = sizeof(NuisSchedulerOwnedBlobV1) + (size_t)byte_len;
    NuisSchedulerOwnedBlobV1* blob =
        (NuisSchedulerOwnedBlobV1*)malloc(allocation_size);
    if (blob == NULL) return NULL;
    blob->protocol_tag = NUIS_SCHEDULER_OWNED_BLOB_TAG_V1;
    blob->glm_token = glm_token;
    blob->byte_len = byte_len;
    if (byte_len > 0) memcpy(blob->bytes, source, (size_t)byte_len);
    nuis_scheduler_owned_blob_live_count_v1 += 1;
    return blob;
}

void* nuis_scheduler_owned_blob_copy_text_v1(int64_t text_handle, uint64_t glm_token) {
    const char* text = nuis_host_text_lookup(text_handle);
    size_t len = nuis_host_text_lookup_len(text_handle);
    if (text == NULL || len == SIZE_MAX) return NULL;
    return nuis_scheduler_owned_blob_copy_v1(text, (int64_t)(len + 1), glm_token);
}

void* nuis_scheduler_owned_blob_move_v1(void* data) {
    return nuis_scheduler_owned_blob_valid_v1(data) ? data : NULL;
}

void nuis_scheduler_owned_blob_drop_v1(void* data) {
    if (data == NULL) return;
    NuisSchedulerOwnedBlobV1* blob = (NuisSchedulerOwnedBlobV1*)data;
    if (blob->protocol_tag == NUIS_SCHEDULER_OWNED_BLOB_TAG_V1) {
        blob->protocol_tag = 0;
        blob->glm_token = 0;
        blob->byte_len = 0;
        nuis_scheduler_owned_blob_live_count_v1 -= 1;
    }
    free(blob);
}

int64_t nuis_scheduler_owned_blob_len_v1(const void* data) {
    if (!nuis_scheduler_owned_blob_valid_v1(data)) return -1;
    return ((const NuisSchedulerOwnedBlobV1*)data)->byte_len;
}

uint64_t nuis_scheduler_owned_blob_glm_token_v1(const void* data) {
    if (!nuis_scheduler_owned_blob_valid_v1(data)) return 0;
    return ((const NuisSchedulerOwnedBlobV1*)data)->glm_token;
}

const void* nuis_scheduler_owned_blob_data_v1(const void* data) {
    if (!nuis_scheduler_owned_blob_valid_v1(data)) return NULL;
    return ((const NuisSchedulerOwnedBlobV1*)data)->bytes;
}

int64_t nuis_scheduler_owned_blob_text_lift_v1(const void* data) {
    if (!nuis_scheduler_owned_blob_valid_v1(data)) return 0;
    const NuisSchedulerOwnedBlobV1* blob =
        (const NuisSchedulerOwnedBlobV1*)data;
    if (blob->byte_len <= 0 || blob->bytes[blob->byte_len - 1] != 0) return 0;
    return nuis_host_text_register_sized(
        (const char*)blob->bytes,
        (size_t)(blob->byte_len - 1)
    );
}

int64_t nuis_scheduler_owned_blob_live_count_get_v1(void) {
    return nuis_scheduler_owned_blob_live_count_v1;
}

void* nuis_scheduler_owned_aggregate_alloc_v1(int64_t slot_count) {
    if (slot_count <= 0) return NULL;
    if ((uint64_t)slot_count
        > (SIZE_MAX - sizeof(NuisSchedulerOwnedAggregateV1))
            / sizeof(NuisSchedulerOwnedAggregateSlotV1)) return NULL;
    size_t size = sizeof(NuisSchedulerOwnedAggregateV1)
        + (size_t)slot_count * sizeof(NuisSchedulerOwnedAggregateSlotV1);
    NuisSchedulerOwnedAggregateV1* aggregate =
        (NuisSchedulerOwnedAggregateV1*)calloc(1, size);
    if (aggregate == NULL) return NULL;
    aggregate->protocol_tag = NUIS_SCHEDULER_OWNED_AGGREGATE_TAG_V1;
    aggregate->slot_count = slot_count;
    aggregate->state = NUIS_SCHEDULER_OWNED_AGGREGATE_BUILDING_V1;
    for (int64_t index = 0; index < slot_count; index += 1) {
        aggregate->slots[index].kind = NUIS_SCHEDULER_OWNED_SLOT_UNSET_V1;
    }
    return aggregate;
}

static int nuis_scheduler_owned_aggregate_valid_v1(
    const NuisSchedulerOwnedAggregateV1* aggregate
) {
    return aggregate != NULL
        && aggregate->protocol_tag == NUIS_SCHEDULER_OWNED_AGGREGATE_TAG_V1
        && aggregate->slot_count > 0;
}

static int nuis_scheduler_owned_aggregate_build_slot_valid_v1(
    const NuisSchedulerOwnedAggregateV1* aggregate,
    int64_t index
) {
    return nuis_scheduler_owned_aggregate_valid_v1(aggregate)
        && aggregate->state == NUIS_SCHEDULER_OWNED_AGGREGATE_BUILDING_V1
        && index >= 0
        && index < aggregate->slot_count
        && aggregate->slots[index].kind == NUIS_SCHEDULER_OWNED_SLOT_UNSET_V1;
}

int64_t nuis_scheduler_owned_aggregate_set_scalar_v1(
    void* data,
    int64_t index,
    int64_t value
) {
    NuisSchedulerOwnedAggregateV1* aggregate =
        (NuisSchedulerOwnedAggregateV1*)data;
    if (!nuis_scheduler_owned_aggregate_build_slot_valid_v1(aggregate, index)) {
        if (nuis_scheduler_owned_aggregate_valid_v1(aggregate)) {
            aggregate->state = NUIS_SCHEDULER_OWNED_AGGREGATE_POISONED_V1;
        }
        return 0;
    }
    aggregate->slots[index].kind = NUIS_SCHEDULER_OWNED_SLOT_SCALAR_V1;
    aggregate->slots[index].value = value;
    return 1;
}

int64_t nuis_scheduler_owned_aggregate_set_blob_v1(
    void* data,
    int64_t index,
    void* blob
) {
    NuisSchedulerOwnedAggregateV1* aggregate =
        (NuisSchedulerOwnedAggregateV1*)data;
    int blob_valid = nuis_scheduler_owned_blob_valid_v1(blob);
    if (!nuis_scheduler_owned_aggregate_build_slot_valid_v1(aggregate, index)
        || !blob_valid) {
        if (blob_valid) nuis_scheduler_owned_blob_drop_v1(blob);
        if (nuis_scheduler_owned_aggregate_valid_v1(aggregate)) {
            aggregate->state = NUIS_SCHEDULER_OWNED_AGGREGATE_POISONED_V1;
        }
        return 0;
    }
    aggregate->slots[index].kind = NUIS_SCHEDULER_OWNED_SLOT_BLOB_V1;
    aggregate->slots[index].value = (int64_t)(intptr_t)blob;
    return 1;
}

int64_t nuis_scheduler_owned_aggregate_get_v1(const void* data, int64_t index) {
    const NuisSchedulerOwnedAggregateV1* aggregate =
        (const NuisSchedulerOwnedAggregateV1*)data;
    if (!nuis_scheduler_owned_aggregate_valid_v1(aggregate)
        || aggregate->state != NUIS_SCHEDULER_OWNED_AGGREGATE_FINALIZED_V1
        || index < 0 || index >= aggregate->slot_count) return 0;
    return aggregate->slots[index].value;
}

void* nuis_scheduler_owned_aggregate_take_blob_v1(void* data, int64_t index) {
    NuisSchedulerOwnedAggregateV1* aggregate =
        (NuisSchedulerOwnedAggregateV1*)data;
    if (!nuis_scheduler_owned_aggregate_valid_v1(aggregate)
        || aggregate->state != NUIS_SCHEDULER_OWNED_AGGREGATE_FINALIZED_V1
        || index < 0 || index >= aggregate->slot_count
        || aggregate->slots[index].kind != NUIS_SCHEDULER_OWNED_SLOT_BLOB_V1) {
        return NULL;
    }
    void* blob = (void*)(intptr_t)aggregate->slots[index].value;
    aggregate->slots[index].kind = NUIS_SCHEDULER_OWNED_SLOT_SCALAR_V1;
    aggregate->slots[index].value = 0;
    return nuis_scheduler_owned_blob_move_v1(blob);
}

void* nuis_scheduler_owned_aggregate_finish_v1(void* data) {
    NuisSchedulerOwnedAggregateV1* aggregate =
        (NuisSchedulerOwnedAggregateV1*)data;
    int complete = nuis_scheduler_owned_aggregate_valid_v1(aggregate)
        && aggregate->state == NUIS_SCHEDULER_OWNED_AGGREGATE_BUILDING_V1;
    if (complete) {
        for (int64_t index = 0; index < aggregate->slot_count; index += 1) {
            if (aggregate->slots[index].kind == NUIS_SCHEDULER_OWNED_SLOT_UNSET_V1) {
                complete = 0;
                break;
            }
        }
    }
    if (complete) {
        aggregate->state = NUIS_SCHEDULER_OWNED_AGGREGATE_FINALIZED_V1;
        return aggregate;
    }
    nuis_scheduler_owned_aggregate_drop_v1(data);
    return NULL;
}

void nuis_scheduler_owned_aggregate_require_v1(const void* data) {
    const NuisSchedulerOwnedAggregateV1* aggregate =
        (const NuisSchedulerOwnedAggregateV1*)data;
    if (nuis_scheduler_owned_aggregate_valid_v1(aggregate)
        && aggregate->state == NUIS_SCHEDULER_OWNED_AGGREGATE_FINALIZED_V1) return;
    fprintf(stderr, "nuis: immediate owned aggregate construction failed\n");
    exit(71);
}

void nuis_scheduler_owned_aggregate_drop_v1(void* data) {
    if (data == NULL) return;
    NuisSchedulerOwnedAggregateV1* aggregate =
        (NuisSchedulerOwnedAggregateV1*)data;
    if (aggregate->protocol_tag == NUIS_SCHEDULER_OWNED_AGGREGATE_TAG_V1) {
        for (int64_t index = 0; index < aggregate->slot_count; index += 1) {
            if (aggregate->slots[index].kind == NUIS_SCHEDULER_OWNED_SLOT_BLOB_V1) {
                nuis_scheduler_owned_blob_drop_v1(
                    (void*)(intptr_t)aggregate->slots[index].value
                );
            }
        }
        aggregate->protocol_tag = 0;
        aggregate->slot_count = 0;
        aggregate->state = 0;
    }
    free(aggregate);
}
"#,
    );
}
