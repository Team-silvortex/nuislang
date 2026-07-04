use super::{
    reports::NsldObjectEmitReport,
    toml::{escape_toml_string, toml_string_array_literal},
    NSLD_LINK_INPUT_TABLE_PRODUCER, NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE,
};

pub(crate) fn render_object_emit_blocked(report: &NsldObjectEmitReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-object-emit-blocked-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"object-emit-blocked\"\n");
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
    push_string(&mut out, "writer_input_path", &report.writer_input_path);
    push_string(&mut out, "blocked_report_path", &report.blocked_report_path);
    push_string(
        &mut out,
        "image_dry_run_report_path",
        &report.image_dry_run_report_path,
    );
    push_string(&mut out, "image_dry_run_path", &report.image_dry_run_path);
    push_string(
        &mut out,
        "image_dry_run_hash",
        report.image_dry_run_hash.as_deref().unwrap_or(""),
    );
    push_string(&mut out, "writer_target_id", &report.writer_target_id);
    push_string(&mut out, "writer_backend_kind", &report.writer_backend_kind);
    push_string(&mut out, "object_family", &report.object_family);
    push_string(&mut out, "object_plan_hash", &report.object_plan_hash);
    out.push_str(&format!("emitted = {}\n", report.emitted));
    out.push_str(&format!("can_emit_object = {}\n", report.can_emit_object));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    out
}

fn push_string(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(value)));
}
