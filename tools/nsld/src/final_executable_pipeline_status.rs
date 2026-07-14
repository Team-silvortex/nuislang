pub(crate) fn nsld_pipeline_self_owned_image_status(
    launcher_manifest_ready: bool,
    nsb_path: &str,
    nsb_present: bool,
    nsb_hash: Option<&str>,
    image_header_valid: bool,
) -> &'static str {
    if launcher_manifest_ready && nsb_present && image_header_valid {
        return "ready";
    }
    if nsb_path.is_empty() {
        return "path-missing";
    }
    if !nsb_present {
        return "missing";
    }
    if !image_header_valid {
        return "header-invalid";
    }
    if nsb_hash.is_none() {
        return "hash-missing";
    }
    "blocked"
}
