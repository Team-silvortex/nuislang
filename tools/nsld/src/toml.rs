use super::{
    reports::{
        NsldAssemblePlanReport, NsldLinkBundleReport, NsldLinkInputDiagnostic, NsldLinkUnitReport,
        NsldSectionManifestReport,
    },
    NSLD_ASSEMBLE_PLAN_KIND, NSLD_ASSEMBLE_PLAN_SCHEMA, NSLD_ASSEMBLE_PLAN_SCHEMA_VERSION,
    NSLD_LINK_BUNDLE_KIND, NSLD_LINK_BUNDLE_SCHEMA, NSLD_LINK_BUNDLE_SCHEMA_VERSION,
    NSLD_LINK_INPUT_TABLE_KIND, NSLD_LINK_INPUT_TABLE_PRODUCER,
    NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE, NSLD_LINK_INPUT_TABLE_SCHEMA,
    NSLD_LINK_INPUT_TABLE_SCHEMA_VERSION, NSLD_LINK_UNIT_TABLE_KIND, NSLD_LINK_UNIT_TABLE_SCHEMA,
    NSLD_LINK_UNIT_TABLE_SCHEMA_VERSION, NSLD_SECTION_MANIFEST_KIND, NSLD_SECTION_MANIFEST_SCHEMA,
    NSLD_SECTION_MANIFEST_SCHEMA_VERSION,
};
use std::fmt::Write as _;

pub(crate) use super::object_emit_render::render_object_emit_blocked;
pub(crate) use super::object_image_render::render_object_image_dry_run;
pub(crate) use super::object_render::{
    render_object_byte_layout, render_object_file_layout, render_object_plan,
    render_object_writer_dry_run, render_object_writer_input,
};
pub(crate) use super::toml_read::{
    bool_value, first_table_bool_value, first_table_isize_value, first_table_string_value,
    first_table_usize_value, string_array_value, string_value, usize_value,
};

pub(crate) fn render_link_input_table(
    inputs: &[NsldLinkInputDiagnostic],
    total_bytes: usize,
    table_hash: &str,
) -> String {
    let mut out = String::with_capacity(512 + inputs.len() * 448);
    writeln!(
        out,
        "schema = \"{}\"",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_SCHEMA)
    )
    .unwrap();
    writeln!(
        out,
        "schema_version = {NSLD_LINK_INPUT_TABLE_SCHEMA_VERSION}"
    )
    .unwrap();
    writeln!(
        out,
        "table_kind = \"{}\"",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_KIND)
    )
    .unwrap();
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
    writeln!(out, "link_input_count = {}", inputs.len()).unwrap();
    writeln!(out, "link_input_total_bytes = {total_bytes}").unwrap();
    writeln!(
        out,
        "link_input_table_hash = \"{}\"",
        escape_toml_string(table_hash)
    )
    .unwrap();
    for input in inputs {
        out.push_str("\n[[link_input]]\n");
        writeln!(out, "order_index = {}", input.order_index).unwrap();
        writeln!(
            out,
            "input_id = \"{}\"",
            escape_toml_string(&input.input_id)
        )
        .unwrap();
        writeln!(
            out,
            "input_kind = \"{}\"",
            escape_toml_string(&input.input_kind)
        )
        .unwrap();
        writeln!(
            out,
            "domain_family = \"{}\"",
            escape_toml_string(&input.domain_family)
        )
        .unwrap();
        writeln!(
            out,
            "package_id = \"{}\"",
            escape_toml_string(&input.package_id)
        )
        .unwrap();
        writeln!(out, "path = \"{}\"", escape_toml_string(&input.path)).unwrap();
        writeln!(
            out,
            "native_ir = \"{}\"",
            escape_toml_string(&input.native_ir)
        )
        .unwrap();
        writeln!(
            out,
            "dispatch_lowering = \"{}\"",
            escape_toml_string(&input.dispatch_lowering)
        )
        .unwrap();
        writeln!(out, "contract_count = {}", input.contract_count).unwrap();
        writeln!(out, "content_bytes = {}", input.content_bytes).unwrap();
        writeln!(
            out,
            "content_hash = \"{}\"",
            escape_toml_string(&input.content_hash)
        )
        .unwrap();
    }
    out
}

pub(crate) fn render_link_unit_table(report: &NsldLinkUnitReport) -> String {
    let mut out = String::with_capacity(512 + report.units.len() * 512);
    writeln!(
        out,
        "schema = \"{}\"",
        escape_toml_string(NSLD_LINK_UNIT_TABLE_SCHEMA)
    )
    .unwrap();
    writeln!(
        out,
        "schema_version = {NSLD_LINK_UNIT_TABLE_SCHEMA_VERSION}"
    )
    .unwrap();
    writeln!(
        out,
        "table_kind = \"{}\"",
        escape_toml_string(NSLD_LINK_UNIT_TABLE_KIND)
    )
    .unwrap();
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
    writeln!(out, "unit_count = {}", report.unit_count).unwrap();
    writeln!(out, "hetero_unit_count = {}", report.hetero_unit_count).unwrap();
    writeln!(out, "link_input_count = {}", report.link_input_count).unwrap();
    writeln!(out, "hetero_node_count = {}", report.hetero_node_count).unwrap();
    writeln!(out, "clock_edge_count = {}", report.clock_edge_count).unwrap();
    writeln!(out, "data_segment_count = {}", report.data_segment_count).unwrap();
    writeln!(
        out,
        "unit_table_hash = \"{}\"",
        escape_toml_string(&report.unit_table_hash)
    )
    .unwrap();
    for unit in &report.units {
        out.push_str("\n[[link_unit]]\n");
        writeln!(out, "order_index = {}", unit.order_index).unwrap();
        writeln!(out, "unit_id = \"{}\"", escape_toml_string(&unit.unit_id)).unwrap();
        writeln!(
            out,
            "unit_kind = \"{}\"",
            escape_toml_string(&unit.unit_kind)
        )
        .unwrap();
        writeln!(
            out,
            "domain_family = \"{}\"",
            escape_toml_string(&unit.domain_family)
        )
        .unwrap();
        writeln!(
            out,
            "package_id = \"{}\"",
            escape_toml_string(&unit.package_id)
        )
        .unwrap();
        writeln!(
            out,
            "backend_family = \"{}\"",
            escape_toml_string(&unit.backend_family)
        )
        .unwrap();
        writeln!(
            out,
            "lowering_target = \"{}\"",
            escape_toml_string(&unit.lowering_target)
        )
        .unwrap();
        writeln!(
            out,
            "packaging_role = \"{}\"",
            escape_toml_string(&unit.packaging_role)
        )
        .unwrap();
        writeln!(
            out,
            "link_input_ids = [{}]",
            toml_string_array_literal(&unit.link_input_ids)
        )
        .unwrap();
        writeln!(out, "hetero_node_count = {}", unit.hetero_node_count).unwrap();
        writeln!(
            out,
            "hetero_timestamps = [{}]",
            toml_string_array_literal(&unit.hetero_timestamps)
        )
        .unwrap();
        writeln!(
            out,
            "lifecycle_hooks = [{}]",
            toml_string_array_literal(&unit.lifecycle_hooks)
        )
        .unwrap();
        writeln!(out, "wait_event_count = {}", unit.wait_event_count).unwrap();
        writeln!(out, "emit_event_count = {}", unit.emit_event_count).unwrap();
        writeln!(out, "clock_edge_count = {}", unit.clock_edge_count).unwrap();
        writeln!(out, "data_segment_count = {}", unit.data_segment_count).unwrap();
        writeln!(
            out,
            "requires_host_wrapper = {}",
            unit.requires_host_wrapper
        )
        .unwrap();
        writeln!(
            out,
            "deterministic_order_key = \"{}\"",
            escape_toml_string(&unit.deterministic_order_key)
        )
        .unwrap();
    }
    out
}

pub(crate) fn render_link_bundle(report: &NsldLinkBundleReport) -> String {
    let mut out = String::with_capacity(1024 + report.issues.len() * 64);
    writeln!(
        out,
        "schema = \"{}\"",
        escape_toml_string(NSLD_LINK_BUNDLE_SCHEMA)
    )
    .unwrap();
    writeln!(out, "schema_version = {NSLD_LINK_BUNDLE_SCHEMA_VERSION}").unwrap();
    writeln!(
        out,
        "bundle_kind = \"{}\"",
        escape_toml_string(NSLD_LINK_BUNDLE_KIND)
    )
    .unwrap();
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
    writeln!(
        out,
        "bundle_id = \"{}\"",
        escape_toml_string(&report.bundle_id)
    )
    .unwrap();
    writeln!(
        out,
        "bundle_hash = \"{}\"",
        escape_toml_string(&report.bundle_hash)
    )
    .unwrap();
    writeln!(out, "bundle_ready = {}", report.bundle_ready).unwrap();
    writeln!(out, "unit_count = {}", report.unit_count).unwrap();
    writeln!(out, "hetero_unit_count = {}", report.hetero_unit_count).unwrap();
    writeln!(out, "link_input_count = {}", report.link_input_count).unwrap();
    writeln!(
        out,
        "link_input_total_bytes = {}",
        report.link_input_total_bytes
    )
    .unwrap();
    writeln!(
        out,
        "link_input_table_hash = \"{}\"",
        escape_toml_string(&report.link_input_table_hash)
    )
    .unwrap();
    writeln!(
        out,
        "unit_table_hash = \"{}\"",
        escape_toml_string(&report.unit_table_hash)
    )
    .unwrap();
    writeln!(out, "clock_edge_count = {}", report.clock_edge_count).unwrap();
    writeln!(out, "data_segment_count = {}", report.data_segment_count).unwrap();
    writeln!(
        out,
        "final_stage_link_mode = \"{}\"",
        escape_toml_string(&report.final_stage_link_mode)
    )
    .unwrap();
    writeln!(
        out,
        "host_wrapper_required = {}",
        report.host_wrapper_required
    )
    .unwrap();
    writeln!(
        out,
        "compiled_artifact_path = \"{}\"",
        escape_toml_string(&report.compiled_artifact_path)
    )
    .unwrap();
    writeln!(
        out,
        "native_output_path = \"{}\"",
        escape_toml_string(&report.native_output_path)
    )
    .unwrap();
    writeln!(
        out,
        "issues = [{}]",
        toml_string_array_literal(&report.issues)
    )
    .unwrap();
    out
}

pub(crate) fn render_assemble_plan(report: &NsldAssemblePlanReport) -> String {
    let mut out = String::with_capacity(768 + report.sections.len() * 256);
    writeln!(
        out,
        "schema = \"{}\"",
        escape_toml_string(NSLD_ASSEMBLE_PLAN_SCHEMA)
    )
    .unwrap();
    writeln!(out, "schema_version = {NSLD_ASSEMBLE_PLAN_SCHEMA_VERSION}").unwrap();
    writeln!(
        out,
        "plan_kind = \"{}\"",
        escape_toml_string(NSLD_ASSEMBLE_PLAN_KIND)
    )
    .unwrap();
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
    writeln!(out, "ready = {}", report.ready).unwrap();
    writeln!(
        out,
        "bundle_id = \"{}\"",
        escape_toml_string(&report.bundle_id)
    )
    .unwrap();
    writeln!(
        out,
        "bundle_hash = \"{}\"",
        escape_toml_string(&report.bundle_hash)
    )
    .unwrap();
    writeln!(
        out,
        "assemble_plan_hash = \"{}\"",
        escape_toml_string(&report.assemble_plan_hash)
    )
    .unwrap();
    writeln!(out, "section_count = {}", report.section_count).unwrap();
    writeln!(
        out,
        "blockers = [{}]",
        toml_string_array_literal(&report.blockers)
    )
    .unwrap();
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        writeln!(out, "order_index = {}", section.order_index).unwrap();
        writeln!(
            out,
            "section_id = \"{}\"",
            escape_toml_string(&section.section_id)
        )
        .unwrap();
        writeln!(
            out,
            "section_kind = \"{}\"",
            escape_toml_string(&section.section_kind)
        )
        .unwrap();
        writeln!(
            out,
            "source_path = \"{}\"",
            escape_toml_string(&section.source_path)
        )
        .unwrap();
        writeln!(
            out,
            "source_hash = \"{}\"",
            escape_toml_string(&section.source_hash)
        )
        .unwrap();
        writeln!(out, "required = {}", section.required).unwrap();
    }
    out
}

pub(crate) fn render_section_manifest(report: &NsldSectionManifestReport) -> String {
    let mut out = String::with_capacity(768 + report.sections.len() * 256);
    writeln!(
        out,
        "schema = \"{}\"",
        escape_toml_string(NSLD_SECTION_MANIFEST_SCHEMA)
    )
    .unwrap();
    writeln!(
        out,
        "schema_version = {NSLD_SECTION_MANIFEST_SCHEMA_VERSION}"
    )
    .unwrap();
    writeln!(
        out,
        "manifest_kind = \"{}\"",
        escape_toml_string(NSLD_SECTION_MANIFEST_KIND)
    )
    .unwrap();
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
    writeln!(out, "ready = {}", report.ready).unwrap();
    writeln!(
        out,
        "assemble_plan_hash = \"{}\"",
        escape_toml_string(&report.assemble_plan_hash)
    )
    .unwrap();
    writeln!(out, "section_count = {}", report.section_count).unwrap();
    writeln!(
        out,
        "section_table_hash = \"{}\"",
        escape_toml_string(&report.section_table_hash)
    )
    .unwrap();
    writeln!(
        out,
        "blockers = [{}]",
        toml_string_array_literal(&report.blockers)
    )
    .unwrap();
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        writeln!(out, "order_index = {}", section.order_index).unwrap();
        writeln!(
            out,
            "section_id = \"{}\"",
            escape_toml_string(&section.section_id)
        )
        .unwrap();
        writeln!(
            out,
            "section_kind = \"{}\"",
            escape_toml_string(&section.section_kind)
        )
        .unwrap();
        writeln!(
            out,
            "source_path = \"{}\"",
            escape_toml_string(&section.source_path)
        )
        .unwrap();
        writeln!(
            out,
            "source_hash = \"{}\"",
            escape_toml_string(&section.source_hash)
        )
        .unwrap();
        writeln!(out, "required = {}", section.required).unwrap();
    }
    out
}

pub(crate) fn toml_string_array_literal(values: &[String]) -> String {
    let mut out = String::with_capacity(values.len() * 24);
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        write!(out, "\"{}\"", escape_toml_string(value)).unwrap();
    }
    out
}

pub(crate) fn escape_toml_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}
