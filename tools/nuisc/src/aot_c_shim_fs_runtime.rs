pub(crate) fn append_c_shim_fs_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_path_rename(int64_t src_handle, int64_t dst_handle) {
    const char* src = nuis_host_text_lookup(src_handle);
    const char* dst = nuis_host_text_lookup(dst_handle);
    if (src == NULL || src[0] == '\0' || dst == NULL || dst[0] == '\0') return 0;
    return rename(src, dst) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_copy(int64_t src_handle, int64_t dst_handle) {
    const char* src = nuis_host_text_lookup(src_handle);
    const char* dst = nuis_host_text_lookup(dst_handle);
    if (src == NULL || src[0] == '\0' || dst == NULL || dst[0] == '\0') return 0;
    FILE* in = fopen(src, "rb");
    if (in == NULL) return 0;
    FILE* out = fopen(dst, "wb");
    if (out == NULL) {
        fclose(in);
        return 0;
    }
    char buffer[4096];
    int ok = 1;
    while (!feof(in)) {
        size_t got = fread(buffer, 1, sizeof(buffer), in);
        if (got > 0 && fwrite(buffer, 1, got, out) != got) {
            ok = 0;
            break;
        }
        if (ferror(in)) {
            ok = 0;
            break;
        }
    }
    fclose(in);
    if (fclose(out) != 0) ok = 0;
    return ok ? 1 : 0;
}

static int64_t nuis_host_path_remove(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return unlink(path) == 0 ? 1 : 0;
}

static int64_t nuis_host_fs_exists(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    return stat(path, &st) == 0 ? 1 : 0;
}

static int64_t nuis_host_fs_kind(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
    if (S_ISREG(st.st_mode)) return 1;
    if (S_ISDIR(st.st_mode)) return 2;
    return 3;
}

static int64_t nuis_host_fs_size(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
    return (int64_t)st.st_size;
}

static int64_t nuis_host_stat_mode(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
    return (int64_t)st.st_mode;
}

static int64_t nuis_host_stat_mtime_ns(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
#if defined(__APPLE__)
    return (int64_t)st.st_mtimespec.tv_sec * 1000000000LL + (int64_t)st.st_mtimespec.tv_nsec;
#else
    return (int64_t)st.st_mtim.tv_sec * 1000000000LL + (int64_t)st.st_mtim.tv_nsec;
#endif
}

static int64_t nuis_host_stat_ctime_ns(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
#if defined(__APPLE__)
    return (int64_t)st.st_ctimespec.tv_sec * 1000000000LL + (int64_t)st.st_ctimespec.tv_nsec;
#else
    return (int64_t)st.st_ctim.tv_sec * 1000000000LL + (int64_t)st.st_ctim.tv_nsec;
#endif
}
"#,
    );
}
