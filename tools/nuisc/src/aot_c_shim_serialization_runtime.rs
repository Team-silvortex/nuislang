pub(crate) fn append_c_shim_serialization_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_serialize_text_into(int64_t text_handle, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    const char* text = nuis_host_text_lookup(text_handle);
    if (text == NULL) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    size_t len = nuis_host_text_lookup_len(text_handle);
    for (size_t index = 0; index < len; ++index) {
        buffer[offset + (int64_t)index] = (unsigned char)text[index];
    }
    return (int64_t)len;
}

static int64_t nuis_host_serialize_i64_into(int64_t value, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    char text[64];
    int written = snprintf(text, sizeof(text), "%lld", (long long)value);
    if (written < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int index = 0; index < written; ++index) {
        buffer[offset + index] = (unsigned char)text[index];
    }
    return (int64_t)written;
}

static int64_t nuis_host_serialize_bool_into(int64_t value, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    const char* text = value != 0 ? "true" : "false";
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    size_t len = strlen(text);
    for (size_t index = 0; index < len; ++index) {
        buffer[offset + (int64_t)index] = (unsigned char)text[index];
    }
    return (int64_t)len;
}

static int64_t nuis_host_serialize_byte_into(int64_t value, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    if (value < 0 || value > 255) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    buffer[offset] = value;
    return 1;
}

static int64_t nuis_host_deserialize_i64_from(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len <= 0) return 0;
    if (len > 63) len = 63;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    char text[64];
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        text[index] = (char)value;
    }
    text[len] = '\0';
    char* end = NULL;
    long long parsed = strtoll(text, &end, 10);
    if (end == text) return 0;
    return (int64_t)parsed;
}

static int64_t nuis_host_deserialize_bool_from(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len <= 0) return 0;
    if (len > 5) len = 5;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    char text[6];
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        text[index] = (char)value;
    }
    text[len] = '\0';
    if (strcmp(text, "true") == 0 || strcmp(text, "1") == 0) return 1;
    if (strcmp(text, "false") == 0 || strcmp(text, "0") == 0) return 0;
    return 0;
}

static int64_t nuis_host_deserialize_byte_from(int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t value = buffer[offset];
    if (value < 0 || value > 255) return 0;
    return value;
}

static int64_t nuis_host_deserialize_text_from(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    char* text = (char*)malloc((size_t)len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)len);
}
"#,
    );
}
