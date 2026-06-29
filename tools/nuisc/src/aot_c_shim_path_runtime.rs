pub(crate) fn append_c_shim_path_runtime(out: &mut String) {
    out.push_str(
        r#"

static int64_t nuis_host_path_join_len(int64_t lhs_handle, int64_t rhs_handle) {
    const char* lhs = nuis_host_text_lookup(lhs_handle);
    const char* rhs = nuis_host_text_lookup(rhs_handle);
    size_t lhs_len = strlen(lhs);
    size_t rhs_len = strlen(rhs);
    size_t needs_sep = (lhs_len > 0 && rhs_len > 0 && lhs[lhs_len - 1] != '/') ? 1 : 0;
    return (int64_t)(lhs_len + needs_sep + rhs_len);
}

static int64_t nuis_host_path_is_absolute(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path != NULL && path[0] == '/') ? 1 : 0;
}

static int64_t nuis_host_path_is_empty(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path == NULL || path[0] == '\0') ? 1 : 0;
}

static int64_t nuis_host_path_is_dot(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    return (len == 1 && path[0] == '.') ? 1 : 0;
}

static int64_t nuis_host_path_is_dotdot(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 2 && path[len - 1] == '/') {
        len -= 1;
    }
    return (len == 2 && path[0] == '.' && path[1] == '.') ? 1 : 0;
}

static int64_t nuis_host_path_is_relative(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path != NULL && path[0] != '/') ? 1 : 0;
}

static int64_t nuis_host_path_is_root(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] != '/') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    return len == 1 ? 1 : 0;
}

static int64_t nuis_host_path_basename(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t slice_len = len - start;
    char buffer[PATH_MAX];
    if (slice_len >= sizeof(buffer)) slice_len = sizeof(buffer) - 1;
    memcpy(buffer, path + start, slice_len);
    buffer[slice_len] = '\0';
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_filename(int64_t path_handle) {
    return nuis_host_path_basename(path_handle);
}

static int64_t nuis_host_path_basename_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* name = nuis_host_text_lookup(name_handle);
    if (path == NULL || name == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t slice_len = len - start;
    size_t name_len = strlen(name);
    if (slice_len != name_len) return 0;
    return memcmp(path + start, name, slice_len) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_filename_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    return nuis_host_path_basename_matches(path_handle, name_handle);
}

static int64_t nuis_host_path_parent_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* name = nuis_host_text_lookup(name_handle);
    if (path == NULL || name == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] != '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t name_len = strlen(name);
    if (len != name_len) return 0;
    return memcmp(path, name, len) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_stem_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* name = nuis_host_text_lookup(name_handle);
    if (path == NULL || name == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t end = len;
    size_t dot = end;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    if (dot > start + 1 && dot < end) {
        end = dot - 1;
    }
    size_t stem_len = end - start;
    size_t name_len = strlen(name);
    if (stem_len != name_len) return 0;
    return memcmp(path + start, name, stem_len) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_parent(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] != '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    char buffer[PATH_MAX];
    if (len >= sizeof(buffer)) len = sizeof(buffer) - 1;
    memcpy(buffer, path, len);
    buffer[len] = '\0';
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_has_parent(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    if (len == 1 && (path[0] == '.' || path[0] == '/')) return 0;
    if (len == 2 && path[0] == '.' && path[1] == '.') return 0;
    size_t i = len;
    while (i > 0) {
        if (path[i - 1] == '/') return 1;
        i -= 1;
    }
    return 0;
}

static int64_t nuis_host_path_is_basename_only(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    if (len == 1 && (path[0] == '.' || path[0] == '/')) return 0;
    if (len == 2 && path[0] == '.' && path[1] == '.') return 0;
    size_t i = 0;
    while (i < len) {
        if (path[i] == '/') return 0;
        i += 1;
    }
    return 1;
}

static int64_t nuis_host_path_depth(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    int64_t depth = 0;
    size_t i = 0;
    while (i < len) {
        while (i < len && path[i] == '/') {
            i += 1;
        }
        if (i >= len) break;
        depth += 1;
        while (i < len && path[i] != '/') {
            i += 1;
        }
    }
    return depth;
}

static int64_t nuis_host_path_stem(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t end = len;
    size_t dot = end;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    if (dot > start + 1 && dot < end) {
        end = dot - 1;
    }
    size_t slice_len = end - start;
    char buffer[PATH_MAX];
    if (slice_len >= sizeof(buffer)) slice_len = sizeof(buffer) - 1;
    memcpy(buffer, path + start, slice_len);
    buffer[slice_len] = '\0';
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_extension(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t dot = len;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    char buffer[PATH_MAX];
    if (dot > start + 1 && dot < len) {
        size_t slice_len = len - dot;
        if (slice_len >= sizeof(buffer)) slice_len = sizeof(buffer) - 1;
        memcpy(buffer, path + dot, slice_len);
        buffer[slice_len] = '\0';
    } else {
        buffer[0] = '\0';
    }
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_has_extension(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t dot = len;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    return (dot > start + 1 && dot < len) ? 1 : 0;
}

static int64_t nuis_host_path_matches_extension(int64_t path_handle, int64_t ext_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* ext = nuis_host_text_lookup(ext_handle);
    if (path == NULL || ext == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t dot = len;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    if (!(dot > start + 1 && dot < len)) return 0;
    const char* actual = path + dot;
    if (ext[0] == '.') ext += 1;
    return strcmp(actual, ext) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_extension_is(int64_t path_handle, int64_t ext_handle) {
    return nuis_host_path_matches_extension(path_handle, ext_handle);
}

static int64_t nuis_host_path_starts_with_dot(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path != NULL && path[0] == '.') ? 1 : 0;
}

static int64_t nuis_host_path_ends_with_slash(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    return (len > 1 && path[len - 1] == '/') ? 1 : 0;
}

static int64_t nuis_host_path_is_hidden(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t slice_len = len - start;
    return (slice_len > 1 && path[start] == '.') ? 1 : 0;
}
"#,
    );
}
