use super::{
    reports::NsldObjectImageDryRunReport, toml::escape_toml_string, NSLD_LINK_INPUT_TABLE_PRODUCER,
    NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE,
};
use std::fmt::Write as _;

pub(crate) fn render_object_image_dry_run(report: &NsldObjectImageDryRunReport) -> String {
    let mut out = String::with_capacity(
        1536 + report.backend_capabilities.len() * 96
            + report.relocation_lowering_rules.len() * 192
            + report.relocation_records.len() * 256,
    );
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
    writeln!(out, "record_count = {}", report.record_count).unwrap();
    writeln!(
        out,
        "total_file_size_bytes = {}",
        report.total_file_size_bytes
    )
    .unwrap();
    writeln!(out, "image_constructed = {}", report.image_constructed).unwrap();
    writeln!(out, "image_ready = {}", report.image_ready).unwrap();
    push_optional_usize(&mut out, "image_size_bytes", report.image_size_bytes);
    push_optional_string(&mut out, "image_hash", report.image_hash.as_deref());
    writeln!(
        out,
        "relocation_lowering_valid = {}",
        report.relocation_lowering_valid
    )
    .unwrap();
    writeln!(
        out,
        "relocation_lowering_rule_count = {}",
        report.relocation_lowering_rule_count
    )
    .unwrap();
    writeln!(
        out,
        "relocation_lowering_issues = [{}]",
        super::toml::toml_string_array_literal(&report.relocation_lowering_issues)
    )
    .unwrap();
    writeln!(
        out,
        "relocation_record_count = {}",
        report.relocation_record_count
    )
    .unwrap();
    push_string(
        &mut out,
        "relocation_record_table_hash",
        &report.relocation_record_table_hash,
    );
    writeln!(
        out,
        "blockers = [{}]",
        super::toml::toml_string_array_literal(&report.blockers)
    )
    .unwrap();
    for capability in &report.backend_capabilities {
        out.push_str("\n[[backend_capability]]\n");
        push_string(&mut out, "capability_id", &capability.capability_id);
        push_string(&mut out, "status", &capability.status);
        writeln!(out, "required = {}", capability.required).unwrap();
    }
    for rule in &report.relocation_lowering_rules {
        out.push_str("\n[[relocation_lowering_rule]]\n");
        push_string(&mut out, "rule_id", &rule.rule_id);
        push_string(&mut out, "source_seed_kind", &rule.source_seed_kind);
        push_string(
            &mut out,
            "target_relocation_kind",
            &rule.target_relocation_kind,
        );
        writeln!(out, "pc_relative = {}", rule.pc_relative).unwrap();
        writeln!(out, "length_power = {}", rule.length_power).unwrap();
        writeln!(out, "external = {}", rule.external).unwrap();
        writeln!(out, "relocation_type = {}", rule.relocation_type).unwrap();
    }
    for record in &report.relocation_records {
        out.push_str("\n[[relocation_record]]\n");
        push_string(&mut out, "record_id", &record.record_id);
        push_string(&mut out, "relocation_seed_id", &record.relocation_seed_id);
        push_string(&mut out, "source_section_id", &record.source_section_id);
        writeln!(out, "source_offset = {}", record.source_offset).unwrap();
        push_string(&mut out, "source_seed_kind", &record.source_seed_kind);
        push_string(
            &mut out,
            "target_relocation_kind",
            &record.target_relocation_kind,
        );
        writeln!(out, "symbol_index = {}", record.symbol_index).unwrap();
        writeln!(out, "pc_relative = {}", record.pc_relative).unwrap();
        writeln!(out, "length_power = {}", record.length_power).unwrap();
        writeln!(out, "external = {}", record.external).unwrap();
        writeln!(out, "relocation_type = {}", record.relocation_type).unwrap();
    }
    out
}

fn push_string(out: &mut String, key: &str, value: &str) {
    writeln!(out, "{key} = \"{}\"", escape_toml_string(value)).unwrap();
}

fn push_optional_string(out: &mut String, key: &str, value: Option<&str>) {
    match value {
        Some(value) => push_string(out, key, value),
        None => writeln!(out, "{key} = \"\"").unwrap(),
    }
}

fn push_optional_usize(out: &mut String, key: &str, value: Option<usize>) {
    match value {
        Some(value) => writeln!(out, "{key} = {value}").unwrap(),
        None => writeln!(out, "{key} = 0").unwrap(),
    }
}
