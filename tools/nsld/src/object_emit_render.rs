use super::{
    reports::NsldObjectEmitReport,
    toml::{escape_toml_string, toml_string_array_literal},
    NSLD_LINK_INPUT_TABLE_PRODUCER, NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE,
};
use std::fmt::Write as _;

pub(crate) fn render_object_emit_blocked(report: &NsldObjectEmitReport) -> String {
    let mut out = String::with_capacity(1024 + report.blockers.len() * 96);
    out.push_str("schema = \"nuis-nsld-object-emit-blocked-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"object-emit-blocked\"\n");
    writeln!(
        out,
        "producer = \"{}\"",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    )
    .unwrap();
    writeln!(
        out,
        "producer_phase = \"{}\"",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    )
    .unwrap();
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
    writeln!(out, "emitted = {}", report.emitted).unwrap();
    writeln!(out, "can_emit_object = {}", report.can_emit_object).unwrap();
    writeln!(
        out,
        "blockers = [{}]",
        toml_string_array_literal(&report.blockers)
    )
    .unwrap();
    out
}

fn push_string(out: &mut String, key: &str, value: &str) {
    writeln!(out, "{key} = \"{}\"", escape_toml_string(value)).unwrap();
}
