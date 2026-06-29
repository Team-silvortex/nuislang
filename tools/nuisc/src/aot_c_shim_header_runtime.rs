pub(crate) fn append_c_shim_header_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_parse_header_line(int64_t buffer_handle, int64_t offset, int64_t len, int64_t expected_name_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t trimmed_len = len;
    if (trimmed_len > 0) {
        int64_t end = offset + trimmed_len - 1;
        int64_t last = buffer[end];
        if (last == 10) {
            if (trimmed_len >= 2 && buffer[end - 1] == 13) {
                trimmed_len -= 2;
            } else {
                trimmed_len -= 1;
            }
        } else if (last == 13) {
            trimmed_len -= 1;
        }
    }
    if (trimmed_len <= 0) return 0;
    int64_t colon = -1;
    for (int64_t index = 0; index < trimmed_len; ++index) {
        if (buffer[offset + index] == 58) {
            colon = offset + index;
            break;
        }
    }
    if (colon < offset) return 0;
    int64_t name_len = colon - offset;
    const char* expected_name = nuis_host_text_lookup(expected_name_handle);
    if (expected_name == NULL) return 0;
    size_t expected_len = strlen(expected_name);
    if ((int64_t)expected_len != name_len) return 0;
    for (int64_t index = 0; index < name_len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)expected_name[index]) return 0;
    }
    int64_t value_offset = colon + 1;
    int64_t line_end = offset + trimmed_len;
    while (value_offset < line_end) {
        int64_t value = buffer[value_offset];
        if (value != 32 && value != 9) break;
        value_offset += 1;
    }
    int64_t value_len = line_end - value_offset;
    char* text = (char*)malloc((size_t)value_len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < value_len; ++index) {
        int64_t value = buffer[value_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[value_len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)value_len);
}

static int64_t nuis_host_parse_header_line_named(
    int64_t buffer_handle,
    int64_t offset,
    int64_t len,
    const char* expected_name,
    size_t expected_len
) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || expected_name == NULL) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t trimmed_len = len;
    if (trimmed_len > 0) {
        int64_t end = offset + trimmed_len - 1;
        int64_t last = buffer[end];
        if (last == 10) {
            if (trimmed_len >= 2 && buffer[end - 1] == 13) {
                trimmed_len -= 2;
            } else {
                trimmed_len -= 1;
            }
        } else if (last == 13) {
            trimmed_len -= 1;
        }
    }
    if (trimmed_len <= 0) return 0;
    int64_t colon = -1;
    for (int64_t index = 0; index < trimmed_len; ++index) {
        if (buffer[offset + index] == 58) {
            colon = offset + index;
            break;
        }
    }
    if (colon < offset) return 0;
    int64_t name_len = colon - offset;
    if ((int64_t)expected_len != name_len) return 0;
    for (int64_t index = 0; index < name_len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)expected_name[index]) return 0;
    }
    int64_t value_offset = colon + 1;
    int64_t line_end = offset + trimmed_len;
    while (value_offset < line_end) {
        int64_t value = buffer[value_offset];
        if (value != 32 && value != 9) break;
        value_offset += 1;
    }
    int64_t value_len = line_end - value_offset;
    char* text = (char*)malloc((size_t)value_len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < value_len; ++index) {
        int64_t value = buffer[value_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[value_len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)value_len);
}

static int64_t nuis_host_find_header_value(int64_t buffer_handle, int64_t offset, int64_t len, int64_t expected_name_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t cursor = offset;
    int64_t limit = offset + len;
    while (cursor < limit) {
        int64_t line_end = cursor;
        while (line_end < limit) {
            int64_t value = ((int64_t*)(intptr_t)buffer_handle)[line_end];
            if (value == 13 || value == 10) break;
            line_end += 1;
        }
        int64_t line_len = line_end - cursor;
        if (line_len == 0) return 0;
        int64_t parsed = nuis_host_parse_header_line(
            buffer_handle,
            cursor,
            line_end < limit ? (line_end - cursor + 1) : line_len,
            expected_name_handle
        );
        if (parsed != 0) return parsed;
        if (line_end >= limit) break;
        if (((int64_t*)(intptr_t)buffer_handle)[line_end] == 13
            && line_end + 1 < limit
            && ((int64_t*)(intptr_t)buffer_handle)[line_end + 1] == 10) {
            cursor = line_end + 2;
        } else {
            cursor = line_end + 1;
        }
    }
    return 0;
}

static int64_t nuis_host_find_header_value_named(
    int64_t buffer_handle,
    int64_t offset,
    int64_t len,
    const char* expected_name,
    size_t expected_len
) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || expected_name == NULL) return 0;
    int64_t cursor = offset;
    int64_t limit = offset + len;
    while (cursor < limit) {
        int64_t line_end = cursor;
        while (line_end < limit) {
            int64_t value = ((int64_t*)(intptr_t)buffer_handle)[line_end];
            if (value == 13 || value == 10) break;
            line_end += 1;
        }
        int64_t line_len = line_end - cursor;
        if (line_len == 0) return 0;
        int64_t parsed = nuis_host_parse_header_line_named(
            buffer_handle,
            cursor,
            line_end < limit ? (line_end - cursor + 1) : line_len,
            expected_name,
            expected_len
        );
        if (parsed != 0) return parsed;
        if (line_end >= limit) break;
        if (((int64_t*)(intptr_t)buffer_handle)[line_end] == 13
            && line_end + 1 < limit
            && ((int64_t*)(intptr_t)buffer_handle)[line_end + 1] == 10) {
            cursor = line_end + 2;
        } else {
            cursor = line_end + 1;
        }
    }
    return 0;
}
"#,
    );
}
