pub(crate) fn append_c_shim_env_io_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_dir_open(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    if (nuis_host_dir_len >= 256) return 0;
    DIR* dir = opendir(path);
    if (dir == NULL) return 0;
    int64_t count = 0;
    struct dirent* entry = NULL;
    while ((entry = readdir(dir)) != NULL) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) continue;
        count += 1;
    }
    rewinddir(dir);
    nuis_host_dir_slots[nuis_host_dir_len] = dir;
    nuis_host_dir_entry_counts[nuis_host_dir_len] = count;
    nuis_host_dir_len += 1;
    return nuis_host_dir_len;
}

static int64_t nuis_host_dir_entry_count(int64_t dir_handle) {
    if (dir_handle <= 0 || dir_handle > nuis_host_dir_len) return 0;
    return nuis_host_dir_entry_counts[dir_handle - 1];
}

static int64_t nuis_host_dir_close(int64_t dir_handle) {
    if (dir_handle <= 0 || dir_handle > nuis_host_dir_len) return 0;
    DIR* dir = nuis_host_dir_slots[dir_handle - 1];
    if (dir == NULL) return 0;
    nuis_host_dir_slots[dir_handle - 1] = NULL;
    return closedir(dir) == 0 ? 1 : 0;
}

static int64_t nuis_host_dir_create(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return mkdir(path, 0755) == 0 ? 1 : 0;
}

static int64_t nuis_host_dir_remove(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return rmdir(path) == 0 ? 1 : 0;
}

static int64_t nuis_host_stdin_read(int64_t buffer_handle, int64_t len) {
    (void)buffer_handle;
    if (len <= 0) return 0;
    char scratch[4096];
    size_t read_len = (size_t)len;
    if (read_len > sizeof(scratch)) read_len = sizeof(scratch);
    ssize_t got = read(STDIN_FILENO, scratch, read_len);
    return got > 0 ? (int64_t)got : 0;
}

static int64_t nuis_host_stdout_write(int64_t handle) {
    const char* text = nuis_host_text_lookup(handle);
    size_t len = strlen(text);
    if (len == 0) return 0;
    return (int64_t)fwrite(text, 1, len, stdout);
}

static int64_t nuis_host_stderr_write(int64_t handle) {
    const char* text = nuis_host_text_lookup(handle);
    size_t len = strlen(text);
    if (len == 0) return 0;
    return (int64_t)fwrite(text, 1, len, stderr);
}

static int64_t nuis_host_stdout_flush(void) {
    return fflush(stdout) == 0 ? 1 : 0;
}

static int64_t nuis_host_stderr_flush(void) {
    return fflush(stderr) == 0 ? 1 : 0;
}

static int64_t nuis_host_tty_isatty(int64_t fd) {
    return isatty((int)fd) ? 1 : 0;
}

static int64_t nuis_host_tty_width(int64_t fd) {
    struct winsize ws;
    if (ioctl((int)fd, TIOCGWINSZ, &ws) != 0) return 0;
    return (int64_t)ws.ws_col;
}

static int64_t nuis_host_tty_height(int64_t fd) {
    struct winsize ws;
    if (ioctl((int)fd, TIOCGWINSZ, &ws) != 0) return 0;
    return (int64_t)ws.ws_row;
}

static int64_t nuis_host_cwd_handle(void) {
    char buffer[PATH_MAX];
    if (getcwd(buffer, sizeof(buffer)) == NULL) return 0;
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_cwd_len_value(void) {
    return nuis_host_text_len_value(nuis_host_cwd_handle());
}

static int64_t nuis_host_temp_dir_handle(void) {
    const char* tmp = getenv("TMPDIR");
    if (tmp == NULL || tmp[0] == '\0') tmp = "/tmp";
    return nuis_host_text_register(tmp);
}

static int64_t nuis_host_temp_path_len(void) {
    return nuis_host_text_len_value(nuis_host_temp_dir_handle());
}

static int64_t nuis_host_temp_file_handle(int64_t prefix_handle) {
    const char* prefix = nuis_host_text_lookup(prefix_handle);
    const char* tmp = getenv("TMPDIR");
    if (tmp == NULL || tmp[0] == '\0') tmp = "/tmp";
    char buffer[PATH_MAX];
    snprintf(buffer, sizeof(buffer), "%s/%sXXXXXX", tmp, (prefix != NULL && prefix[0] != '\0') ? prefix : "nuis");
    int fd = mkstemp(buffer);
    if (fd < 0) return 0;
    close(fd);
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_chdir_value(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return chdir(path) == 0 ? 1 : 0;
}
"#,
    );
}
