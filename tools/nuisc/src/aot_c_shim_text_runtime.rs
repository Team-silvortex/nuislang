pub(crate) fn append_c_shim_text_runtime(out: &mut String) {
    out.push_str(
        r#"

static uint64_t nuis_host_text_hash_bytes(const char* text, size_t len) {
    uint64_t hash = 1469598103934665603ULL;
    for (size_t index = 0; index < len; ++index) {
        hash ^= (unsigned char)text[index];
        hash *= 1099511628211ULL;
    }
    return hash;
}

static int64_t nuis_host_text_find_interned(const char* text, size_t len, uint64_t hash) {
    if (text == NULL) return 0;
    size_t mask = (sizeof(nuis_host_text_intern_table) / sizeof(nuis_host_text_intern_table[0])) - 1;
    size_t slot = (size_t)hash & mask;
    for (size_t probe = 0; probe <= mask; ++probe) {
        int64_t handle = nuis_host_text_intern_table[slot];
        if (handle == 0) return 0;
        if (handle <= nuis_host_text_len && nuis_host_text_slots[handle - 1] != NULL) {
            if (nuis_host_text_slot_hashes[handle - 1] == hash
                && nuis_host_text_slot_lens[handle - 1] == len
                && memcmp(nuis_host_text_slots[handle - 1], text, len) == 0) {
                return handle;
            }
        }
        slot = (slot + 1) & mask;
    }
    return 0;
}

static void nuis_host_text_intern_insert(int64_t handle, uint64_t hash) {
    if (handle <= 0) return;
    size_t mask = (sizeof(nuis_host_text_intern_table) / sizeof(nuis_host_text_intern_table[0])) - 1;
    size_t slot = (size_t)hash & mask;
    for (size_t probe = 0; probe <= mask; ++probe) {
        if (nuis_host_text_intern_table[slot] == 0) {
            nuis_host_text_intern_table[slot] = handle;
            return;
        }
        slot = (slot + 1) & mask;
    }
}

static int64_t nuis_host_text_register_sized(const char* text, size_t len) {
    if (text == NULL) return 0;
    if (nuis_host_text_len >= 4096) return 0;
    uint64_t hash = nuis_host_text_hash_bytes(text, len);
    int64_t interned = nuis_host_text_find_interned(text, len, hash);
    if (interned != 0) return interned;
    size_t size = len + 1;
    char* copy = (char*)malloc(size);
    if (copy == NULL) return 0;
    memcpy(copy, text, size);
    nuis_host_text_slots[nuis_host_text_len] = copy;
    nuis_host_text_slot_lens[nuis_host_text_len] = len;
    nuis_host_text_slot_hashes[nuis_host_text_len] = hash;
    nuis_host_text_len += 1;
    nuis_host_text_intern_insert(nuis_host_text_len, hash);
    return nuis_host_text_len;
}

static int64_t nuis_host_text_register(const char* text) {
    if (text == NULL) return 0;
    return nuis_host_text_register_sized(text, strlen(text));
}

static int64_t nuis_host_text_register_owned_sized(char* text, size_t len) {
    if (text == NULL) return 0;
    uint64_t hash = nuis_host_text_hash_bytes(text, len);
    int64_t interned = nuis_host_text_find_interned(text, len, hash);
    if (interned != 0) {
        free(text);
        return interned;
    }
    if (nuis_host_text_len >= 4096) {
        free(text);
        return 0;
    }
    nuis_host_text_slots[nuis_host_text_len] = text;
    nuis_host_text_slot_lens[nuis_host_text_len] = len;
    nuis_host_text_slot_hashes[nuis_host_text_len] = hash;
    nuis_host_text_len += 1;
    nuis_host_text_intern_insert(nuis_host_text_len, hash);
    return nuis_host_text_len;
}

static int64_t nuis_host_text_register_owned(char* text) {
    if (text == NULL) return 0;
    return nuis_host_text_register_owned_sized(text, strlen(text));
}

int64_t nuis_host_text_lift(const char* text) {
    return nuis_host_text_register(text);
}

static int64_t nuis_host_text_handle(int64_t text_handle) {
    return text_handle;
}

static const char* nuis_host_text_lookup(int64_t handle) {
    static char fallback[64];
    if (handle > 0 && handle <= nuis_host_text_len && nuis_host_text_slots[handle - 1] != NULL) {
        return nuis_host_text_slots[handle - 1];
    }
    if (handle == 0) return "";
    snprintf(fallback, sizeof(fallback), "%lld", (long long)handle);
    return fallback;
}

const char* nuis_host_text_ptr(int64_t handle) {
    return nuis_host_text_lookup(handle);
}

static size_t nuis_host_text_lookup_len(int64_t handle) {
    if (handle > 0 && handle <= nuis_host_text_len && nuis_host_text_slots[handle - 1] != NULL) {
        return nuis_host_text_slot_lens[handle - 1];
    }
    if (handle == 0) return 0;
    return strlen(nuis_host_text_lookup(handle));
}

static int64_t nuis_host_argv_count(void) {
    return (int64_t)nuis_argc;
}

static int64_t nuis_host_argv_at(int64_t index) {
    if (index < 0 || index >= nuis_argc || nuis_argv == NULL) return 0;
    return nuis_host_text_register(nuis_argv[index]);
}

static int64_t nuis_host_env_has(int64_t key_handle) {
    const char* key = nuis_host_text_lookup(key_handle);
    const char* value = getenv(key);
    return value != NULL ? 1 : 0;
}

static int64_t nuis_host_env_get(int64_t key_handle) {
    const char* key = nuis_host_text_lookup(key_handle);
    const char* value = getenv(key);
    if (value == NULL) return 0;
    return nuis_host_text_register(value);
}

static int64_t nuis_host_text_len_value(int64_t handle) {
    return (int64_t)nuis_host_text_lookup_len(handle);
}

static int64_t nuis_host_text_concat(int64_t lhs_handle, int64_t rhs_handle) {
    const char* lhs = nuis_host_text_lookup(lhs_handle);
    const char* rhs = nuis_host_text_lookup(rhs_handle);
    size_t lhs_len = lhs != NULL ? nuis_host_text_lookup_len(lhs_handle) : 0;
    size_t rhs_len = rhs != NULL ? nuis_host_text_lookup_len(rhs_handle) : 0;
    size_t total = lhs_len + rhs_len + 1;
    char* buffer = (char*)malloc(total);
    if (buffer == NULL) return 0;
    if (lhs_len > 0) {
        memcpy(buffer, lhs, lhs_len);
    }
    if (rhs_len > 0) {
        memcpy(buffer + lhs_len, rhs, rhs_len);
    }
    buffer[lhs_len + rhs_len] = '\0';
    return nuis_host_text_register_owned_sized(buffer, lhs_len + rhs_len);
}
"#,
    );
}
