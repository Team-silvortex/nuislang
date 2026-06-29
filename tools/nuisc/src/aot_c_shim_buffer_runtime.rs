pub(crate) fn append_c_shim_buffer_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_deserialize_text_equals(int64_t buffer_handle, int64_t offset, int64_t len, int64_t expected_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* expected = nuis_host_text_lookup(expected_handle);
    if (expected == NULL) return 0;
    size_t expected_len = strlen(expected);
    if ((int64_t)expected_len != len) return 0;
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)expected[index]) return 0;
    }
    return 1;
}

static int64_t nuis_host_deserialize_text_starts_with(int64_t buffer_handle, int64_t offset, int64_t len, int64_t prefix_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* prefix = nuis_host_text_lookup(prefix_handle);
    if (prefix == NULL) return 0;
    size_t prefix_len = strlen(prefix);
    if ((int64_t)prefix_len > len) return 0;
    for (size_t index = 0; index < prefix_len; ++index) {
        int64_t value = buffer[offset + (int64_t)index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)prefix[index]) return 0;
    }
    return 1;
}

static int64_t nuis_host_deserialize_text_contains(int64_t buffer_handle, int64_t offset, int64_t len, int64_t needle_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* needle = nuis_host_text_lookup(needle_handle);
    if (needle == NULL) return 0;
    size_t needle_len = strlen(needle);
    if (needle_len == 0) return 1;
    if ((int64_t)needle_len > len) return 0;
    for (int64_t start = 0; start <= len - (int64_t)needle_len; ++start) {
        int matched = 1;
        for (size_t index = 0; index < needle_len; ++index) {
            int64_t value = buffer[offset + start + (int64_t)index];
            if (value < 0 || value > 255 || (unsigned char)value != (unsigned char)needle[index]) {
                matched = 0;
                break;
            }
        }
        if (matched) return 1;
    }
    return 0;
}

static int64_t nuis_host_deserialize_text_ends_with(int64_t buffer_handle, int64_t offset, int64_t len, int64_t suffix_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* suffix = nuis_host_text_lookup(suffix_handle);
    if (suffix == NULL) return 0;
    size_t suffix_len = strlen(suffix);
    if ((int64_t)suffix_len > len) return 0;
    int64_t start = offset + len - (int64_t)suffix_len;
    for (size_t index = 0; index < suffix_len; ++index) {
        int64_t value = buffer[start + (int64_t)index];
        if (value < 0 || value > 255 || (unsigned char)value != (unsigned char)suffix[index]) {
            return 0;
        }
    }
    return 1;
}

static int64_t nuis_host_buffer_find_byte(int64_t buffer_handle, int64_t offset, int64_t len, int64_t needle) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || needle < 0 || needle > 255) return -1;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int64_t index = 0; index < len; ++index) {
        if (buffer[offset + index] == needle) {
            return offset + index;
        }
    }
    return -1;
}

static int64_t nuis_host_fill_bytes(int64_t buffer_handle, int64_t offset, int64_t len, int64_t value) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || value < 0 || value > 255) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int64_t index = 0; index < len; ++index) {
        buffer[offset + index] = value;
    }
    return len;
}

static int64_t nuis_host_copy_bytes(int64_t dst_handle, int64_t dst_offset, int64_t dst_len, int64_t src_handle, int64_t src_offset, int64_t src_len) {
    if (dst_handle == 0 || src_handle == 0 || dst_offset < 0 || src_offset < 0 || dst_len < 0 || src_len < 0) return 0;
    int64_t copy_len = dst_len < src_len ? dst_len : src_len;
    int64_t* dst = (int64_t*)(intptr_t)dst_handle;
    int64_t* src = (int64_t*)(intptr_t)src_handle;
    if (copy_len <= 0) return 0;
    if (dst == src && dst_offset > src_offset && dst_offset < src_offset + copy_len) {
        for (int64_t index = copy_len; index > 0; --index) {
            int64_t value = src[src_offset + index - 1];
            if (value < 0 || value > 255) return 0;
            dst[dst_offset + index - 1] = value;
        }
    } else {
        for (int64_t index = 0; index < copy_len; ++index) {
            int64_t value = src[src_offset + index];
            if (value < 0 || value > 255) return 0;
            dst[dst_offset + index] = value;
        }
    }
    return copy_len;
}

static int64_t nuis_host_compare_bytes(int64_t lhs_handle, int64_t lhs_offset, int64_t lhs_len, int64_t rhs_handle, int64_t rhs_offset, int64_t rhs_len) {
    if (lhs_handle == 0 || rhs_handle == 0 || lhs_offset < 0 || rhs_offset < 0 || lhs_len < 0 || rhs_len < 0) return 0;
    int64_t* lhs = (int64_t*)(intptr_t)lhs_handle;
    int64_t* rhs = (int64_t*)(intptr_t)rhs_handle;
    int64_t shared_len = lhs_len < rhs_len ? lhs_len : rhs_len;
    for (int64_t index = 0; index < shared_len; ++index) {
        int64_t lhs_value = lhs[lhs_offset + index];
        int64_t rhs_value = rhs[rhs_offset + index];
        if (lhs_value < 0 || lhs_value > 255 || rhs_value < 0 || rhs_value > 255) return 0;
        if (lhs_value < rhs_value) return -1;
        if (lhs_value > rhs_value) return 1;
    }
    if (lhs_len < rhs_len) return -1;
    if (lhs_len > rhs_len) return 1;
    return 0;
}

static int64_t nuis_host_buffer_find_text(int64_t buffer_handle, int64_t offset, int64_t len, int64_t needle_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return -1;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* needle = nuis_host_text_lookup(needle_handle);
    if (needle == NULL) return -1;
    size_t needle_len = strlen(needle);
    if (needle_len == 0) return offset;
    if ((int64_t)needle_len > len) return -1;
    for (int64_t start = 0; start <= len - (int64_t)needle_len; ++start) {
        int matched = 1;
        for (size_t index = 0; index < needle_len; ++index) {
            int64_t value = buffer[offset + start + (int64_t)index];
            if (value < 0 || value > 255 || (unsigned char)value != (unsigned char)needle[index]) {
                matched = 0;
                break;
            }
        }
        if (matched) return offset + start;
    }
    return -1;
}

static int64_t nuis_host_buffer_find_line_end(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return -1;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value == 13 || value == 10) {
            return offset + index;
        }
    }
    return -1;
}

static int64_t nuis_host_buffer_trim_line_end(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    if (len == 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t end = offset + len - 1;
    int64_t last = buffer[end];
    if (last == 10) {
        if (len >= 2 && buffer[end - 1] == 13) {
            return len - 2;
        }
        return len - 1;
    }
    if (last == 13) {
        return len - 1;
    }
    return len;
}
"#,
    );
}
