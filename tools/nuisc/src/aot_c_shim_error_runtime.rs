pub(crate) fn append_c_shim_error_runtime(out: &mut String) {
    out.push_str(
        r#"

// Bootstrap error facades preserve the stable numeric kind token until the
// native std error catalog supplies richer code, message, and severity tables.
static int64_t nuis_host_error_code(int64_t kind_handle) {
    return kind_handle;
}

static int64_t nuis_host_error_message(int64_t kind_handle) {
    return kind_handle;
}

static int64_t nuis_host_error_severity(int64_t kind_handle) {
    return kind_handle;
}
"#,
    );
}
