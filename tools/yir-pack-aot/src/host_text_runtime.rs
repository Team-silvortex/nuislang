pub(crate) fn host_text_runtime_source() -> &'static str {
    r#"
static char* nuis_host_text_slots[4096];
static size_t nuis_host_text_lengths[4096];
static int64_t nuis_host_text_len = 0;

static int64_t nuis_host_text_register_sized(const char* text, size_t len) {
    if (text == NULL || nuis_host_text_len >= 4096 || len == SIZE_MAX) return 0;
    char* copy = (char*)malloc(len + 1);
    if (copy == NULL) return 0;
    if (len > 0) memcpy(copy, text, len);
    copy[len] = 0;
    nuis_host_text_slots[nuis_host_text_len] = copy;
    nuis_host_text_lengths[nuis_host_text_len] = len;
    nuis_host_text_len += 1;
    return nuis_host_text_len;
}

static int64_t nuis_host_text_register(const char* text) {
    return text == NULL ? 0 : nuis_host_text_register_sized(text, strlen(text));
}

int64_t nuis_host_text_lift(const char* text) {
    return nuis_host_text_register(text);
}

static const char* nuis_host_text_lookup(int64_t handle) {
    static char fallback[64];
    if (handle > 0 && handle <= nuis_host_text_len
        && nuis_host_text_slots[handle - 1] != NULL) {
        return nuis_host_text_slots[handle - 1];
    }
    if (handle == 0) return "";
    snprintf(fallback, sizeof(fallback), "%lld", (long long)handle);
    return fallback;
}

static size_t nuis_host_text_lookup_len(int64_t handle) {
    if (handle > 0 && handle <= nuis_host_text_len
        && nuis_host_text_slots[handle - 1] != NULL) {
        return nuis_host_text_lengths[handle - 1];
    }
    return strlen(nuis_host_text_lookup(handle));
}
"#
}
