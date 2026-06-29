pub(crate) fn append_c_shim_file_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_file_open(int64_t path_handle, int64_t flags) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    int fd = open(path, (int)flags, 0644);
    return fd >= 0 ? (int64_t)fd : 0;
}

static int64_t nuis_host_file_read(int64_t file_handle, int64_t buffer_handle, int64_t len) {
    if (file_handle < 0 || buffer_handle == 0 || len <= 0) return 0;
    char scratch[4096];
    size_t read_len = (size_t)len;
    if (read_len > sizeof(scratch)) read_len = sizeof(scratch);
    ssize_t got = read((int)file_handle, scratch, read_len);
    if (got <= 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (ssize_t i = 0; i < got; ++i) {
        buffer[i] = (unsigned char)scratch[i];
    }
    return (int64_t)got;
}

static int64_t nuis_host_file_write(int64_t file_handle, int64_t text_handle) {
    if (file_handle < 0) return 0;
    const char* text = nuis_host_text_lookup(text_handle);
    size_t len = strlen(text);
    if (len == 0) return 0;
    ssize_t wrote = write((int)file_handle, text, len);
    return wrote > 0 ? (int64_t)wrote : 0;
}

static int64_t nuis_host_file_close(int64_t file_handle) {
    if (file_handle < 0) return 0;
    return close((int)file_handle) == 0 ? 1 : 0;
}
"#,
    );
}
