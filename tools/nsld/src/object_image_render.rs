use super::{
    reports::NsldObjectImageDryRunReport, toml::escape_toml_string, NSLD_LINK_INPUT_TABLE_PRODUCER,
    NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE,
};

pub(crate) fn render_object_image_dry_run(report: &NsldObjectImageDryRunReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-object-image-dry-run-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"object-image-dry-run\"\n");
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    push_string(&mut out, "manifest", &report.manifest);
    push_string(&mut out, "output_path", &report.output_path);
    push_string(&mut out, "image_path", &report.image_path);
    push_string(&mut out, "writer_target_id", &report.writer_target_id);
    push_string(&mut out, "writer_backend_kind", &report.writer_backend_kind);
    push_string(&mut out, "object_family", &report.object_family);
    push_string(&mut out, "backend_kind", &report.backend_kind);
    push_string(&mut out, "backend_family", &report.backend_family);
    push_string(&mut out, "backend_status", &report.backend_status);
    push_string(&mut out, "object_format", &report.object_format);
    push_string(&mut out, "file_layout_hash", &report.file_layout_hash);
    out.push_str(&format!("record_count = {}\n", report.record_count));
    out.push_str(&format!(
        "total_file_size_bytes = {}\n",
        report.total_file_size_bytes
    ));
    out.push_str(&format!(
        "image_constructed = {}\n",
        report.image_constructed
    ));
    out.push_str(&format!("image_ready = {}\n", report.image_ready));
    push_optional_usize(&mut out, "image_size_bytes", report.image_size_bytes);
    push_optional_string(&mut out, "image_hash", report.image_hash.as_deref());
    out.push_str(&format!(
        "blockers = [{}]\n",
        super::toml::toml_string_array_literal(&report.blockers)
    ));
    out
}

fn push_string(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(value)));
}

fn push_optional_string(out: &mut String, key: &str, value: Option<&str>) {
    match value {
        Some(value) => push_string(out, key, value),
        None => out.push_str(&format!("{key} = \"\"\n")),
    }
}

fn push_optional_usize(out: &mut String, key: &str, value: Option<usize>) {
    match value {
        Some(value) => out.push_str(&format!("{key} = {value}\n")),
        None => out.push_str(&format!("{key} = 0\n")),
    }
}
