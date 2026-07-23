pub(crate) fn append_provider_worker_control_helpers(out: &mut String) {
    out.push_str(
        r#"

static int nuis_provider_worker_payload_has(const char* key) {
    if (nuis_provider_worker_payload == NULL) return 0;
    size_t key_length = strlen(key);
    const unsigned char* cursor = nuis_provider_worker_payload;
    const unsigned char* end = cursor + nuis_provider_worker_payload_length;
    while (cursor < end) {
        const unsigned char* newline = memchr(cursor, '\n', (size_t)(end - cursor));
        const unsigned char* line_end = newline == NULL ? end : newline;
        size_t line_length = (size_t)(line_end - cursor);
        if (line_length > key_length
            && memcmp(cursor, key, key_length) == 0
            && cursor[key_length] == '=') {
            return 1;
        }
        if (newline == NULL) break;
        cursor = newline + 1;
    }
    return 0;
}

static size_t nuis_provider_worker_split_tabs(
    char* text,
    char** fields,
    size_t field_capacity) {
    if (*text == '\0' || field_capacity == 0) return 0;
    size_t count = 0;
    char* cursor = text;
    while (1) {
        if (count == field_capacity || *cursor == '\0') return 0;
        fields[count++] = cursor;
        char* separator = strchr(cursor, '\t');
        if (separator == NULL) return count;
        *separator = '\0';
        cursor = separator + 1;
    }
}
"#,
    );
}
