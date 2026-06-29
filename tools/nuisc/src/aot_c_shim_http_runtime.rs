pub(crate) fn append_c_shim_http_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_find_status_line_reason(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t limit = offset + len;
    int64_t line_end = offset;
    while (line_end < limit) {
        int64_t value = buffer[line_end];
        if (value == 13 || value == 10) break;
        line_end += 1;
    }
    if (line_end <= offset) return 0;
    int64_t first_space = -1;
    for (int64_t index = offset; index < line_end; ++index) {
        if (buffer[index] == 32) {
            first_space = index;
            break;
        }
    }
    if (first_space < offset) return 0;
    int64_t second_space = -1;
    for (int64_t index = first_space + 1; index < line_end; ++index) {
        if (buffer[index] == 32) {
            second_space = index;
            break;
        }
    }
    if (second_space < first_space + 1) return 0;
    int64_t reason_offset = second_space + 1;
    while (reason_offset < line_end) {
        int64_t value = buffer[reason_offset];
        if (value != 32 && value != 9) break;
        reason_offset += 1;
    }
    int64_t reason_len = line_end - reason_offset;
    char* text = (char*)malloc((size_t)reason_len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < reason_len; ++index) {
        int64_t value = buffer[reason_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[reason_len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)reason_len);
}

static int64_t nuis_host_parse_http_response_summary(int64_t buffer_handle, int64_t offset, int64_t len) {
    static const char content_type_name[] = "Content-Type";
    static const char content_length_name[] = "Content-Length";
    static const char content_type_prefix[] = " | content-type=";
    static const char content_length_prefix[] = " | content-length=";
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t limit = offset + len;
    int64_t line_end = offset;
    while (line_end < limit) {
        int64_t value = buffer[line_end];
        if (value == 13 || value == 10) break;
        line_end += 1;
    }
    if (line_end <= offset) return 0;
    int64_t first_space = -1;
    for (int64_t index = offset; index < line_end; ++index) {
        if (buffer[index] == 32) {
            first_space = index;
            break;
        }
    }
    if (first_space < offset) return 0;
    int64_t second_space = -1;
    for (int64_t index = first_space + 1; index < line_end; ++index) {
        if (buffer[index] == 32) {
            second_space = index;
            break;
        }
    }
    if (second_space < first_space + 1) return 0;
    int64_t status_offset = first_space + 1;
    int64_t status_len = second_space - status_offset;

    int64_t reason_handle = nuis_host_find_status_line_reason(buffer_handle, offset, len);
    const char* reason = nuis_host_text_lookup(reason_handle);
    int64_t content_type_handle =
        nuis_host_find_header_value_named(
            buffer_handle,
            offset,
            len,
            content_type_name,
            sizeof(content_type_name) - 1
        );
    int64_t content_length_handle =
        nuis_host_find_header_value_named(
            buffer_handle,
            offset,
            len,
            content_length_name,
            sizeof(content_length_name) - 1
        );
    const char* content_type = nuis_host_text_lookup(content_type_handle);
    const char* content_length = nuis_host_text_lookup(content_length_handle);

    int has_reason = reason != NULL && reason[0] != '\0';
    int has_content_type = content_type != NULL && content_type[0] != '\0';
    int has_content_length = content_length != NULL && content_length[0] != '\0';
    size_t reason_len = has_reason ? nuis_host_text_lookup_len(reason_handle) : 0;
    size_t content_type_len = has_content_type ? nuis_host_text_lookup_len(content_type_handle) : 0;
    size_t content_length_len =
        has_content_length ? nuis_host_text_lookup_len(content_length_handle) : 0;
    size_t total = (size_t)status_len + 1;
    if (has_reason) total += 1 + reason_len;
    if (has_content_type) total += sizeof(content_type_prefix) - 1 + content_type_len;
    if (has_content_length) total += sizeof(content_length_prefix) - 1 + content_length_len;

    char* text = (char*)malloc(total);
    if (text == NULL) return 0;
    size_t cursor = 0;
    for (int64_t index = 0; index < status_len; ++index) {
        int64_t value = buffer[status_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[cursor++] = (char)value;
    }
    if (has_reason) {
        text[cursor++] = ' ';
        memcpy(text + cursor, reason, reason_len);
        cursor += reason_len;
    }
    if (has_content_type) {
        memcpy(text + cursor, content_type_prefix, sizeof(content_type_prefix) - 1);
        cursor += sizeof(content_type_prefix) - 1;
        memcpy(text + cursor, content_type, content_type_len);
        cursor += content_type_len;
    }
    if (has_content_length) {
        memcpy(text + cursor, content_length_prefix, sizeof(content_length_prefix) - 1);
        cursor += sizeof(content_length_prefix) - 1;
        memcpy(text + cursor, content_length, content_length_len);
        cursor += content_length_len;
    }
    text[cursor] = '\0';
    return nuis_host_text_register_owned_sized(text, cursor);
}

static int64_t nuis_host_parse_http_request_summary(int64_t buffer_handle, int64_t offset, int64_t len) {
    static const char host_name[] = "Host";
    static const char connection_name[] = "Connection";
    static const char host_prefix[] = " | host=";
    static const char connection_prefix[] = " | connection=";
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t limit = offset + len;
    int64_t line_end = offset;
    while (line_end < limit) {
        int64_t value = buffer[line_end];
        if (value == 13 || value == 10) break;
        line_end += 1;
    }
    if (line_end <= offset) return 0;
    int64_t first_space = -1;
    for (int64_t index = offset; index < line_end; ++index) {
        if (buffer[index] == 32) {
            first_space = index;
            break;
        }
    }
    if (first_space < offset) return 0;
    int64_t second_space = -1;
    for (int64_t index = first_space + 1; index < line_end; ++index) {
        if (buffer[index] == 32) {
            second_space = index;
            break;
        }
    }
    if (second_space < first_space + 1) return 0;
    int64_t method_len = first_space - offset;
    int64_t path_offset = first_space + 1;
    int64_t path_len = second_space - path_offset;

    int64_t host_handle = nuis_host_find_header_value_named(
        buffer_handle,
        offset,
        len,
        host_name,
        sizeof(host_name) - 1
    );
    int64_t connection_handle =
        nuis_host_find_header_value_named(
            buffer_handle,
            offset,
            len,
            connection_name,
            sizeof(connection_name) - 1
        );
    const char* host = nuis_host_text_lookup(host_handle);
    const char* connection = nuis_host_text_lookup(connection_handle);
    int has_host = host != NULL && host[0] != '\0';
    int has_connection = connection != NULL && connection[0] != '\0';
    size_t host_len = has_host ? nuis_host_text_lookup_len(host_handle) : 0;
    size_t connection_len = has_connection ? nuis_host_text_lookup_len(connection_handle) : 0;
    size_t total = (size_t)method_len + 1 + (size_t)path_len + 1;
    if (has_host) total += sizeof(host_prefix) - 1 + host_len;
    if (has_connection) total += sizeof(connection_prefix) - 1 + connection_len;

    char* text = (char*)malloc(total);
    if (text == NULL) return 0;
    size_t cursor = 0;
    for (int64_t index = 0; index < method_len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[cursor++] = (char)value;
    }
    text[cursor++] = ' ';
    for (int64_t index = 0; index < path_len; ++index) {
        int64_t value = buffer[path_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[cursor++] = (char)value;
    }
    if (has_host) {
        memcpy(text + cursor, host_prefix, sizeof(host_prefix) - 1);
        cursor += sizeof(host_prefix) - 1;
        memcpy(text + cursor, host, host_len);
        cursor += host_len;
    }
    if (has_connection) {
        memcpy(text + cursor, connection_prefix, sizeof(connection_prefix) - 1);
        cursor += sizeof(connection_prefix) - 1;
        memcpy(text + cursor, connection, connection_len);
        cursor += connection_len;
    }
    text[cursor] = '\0';
    return nuis_host_text_register_owned_sized(text, cursor);
}

static int64_t nuis_host_parse_http_roundtrip_summary(
    int64_t request_buffer_handle,
    int64_t request_offset,
    int64_t request_len,
    int64_t response_buffer_handle,
    int64_t response_offset,
    int64_t response_len
) {
    static const char arrow_separator[] = " -> ";
    int64_t request_handle =
        nuis_host_parse_http_request_summary(request_buffer_handle, request_offset, request_len);
    int64_t response_handle =
        nuis_host_parse_http_response_summary(response_buffer_handle, response_offset, response_len);
    const char* request = nuis_host_text_lookup(request_handle);
    const char* response = nuis_host_text_lookup(response_handle);
    if (request == NULL) request = "";
    if (response == NULL) response = "";
    size_t request_len_text = nuis_host_text_lookup_len(request_handle);
    size_t response_len_text = nuis_host_text_lookup_len(response_handle);
    size_t total = request_len_text + (sizeof(arrow_separator) - 1) + response_len_text + 1;
    char* text = (char*)malloc(total);
    if (text == NULL) return 0;
    memcpy(text, request, request_len_text);
    memcpy(text + request_len_text, arrow_separator, sizeof(arrow_separator) - 1);
    memcpy(
        text + request_len_text + (sizeof(arrow_separator) - 1),
        response,
        response_len_text
    );
    text[total - 1] = '\0';
    return nuis_host_text_register_owned_sized(text, total - 1);
}
"#,
    );
}
