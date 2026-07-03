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

pub(crate) use super::object_render::{
    render_object_byte_layout, render_object_emit_blocked, render_object_plan,
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
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_LINK_INPUT_TABLE_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "table_kind = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("link_input_count = {}\n", inputs.len()));
    out.push_str(&format!("link_input_total_bytes = {total_bytes}\n"));
    out.push_str(&format!(
        "link_input_table_hash = \"{}\"\n",
        escape_toml_string(table_hash)
    ));
    for input in inputs {
        out.push_str("\n[[link_input]]\n");
        out.push_str(&format!("order_index = {}\n", input.order_index));
        out.push_str(&format!(
            "input_id = \"{}\"\n",
            escape_toml_string(&input.input_id)
        ));
        out.push_str(&format!(
            "input_kind = \"{}\"\n",
            escape_toml_string(&input.input_kind)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&input.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&input.package_id)
        ));
        out.push_str(&format!("path = \"{}\"\n", escape_toml_string(&input.path)));
        out.push_str(&format!(
            "native_ir = \"{}\"\n",
            escape_toml_string(&input.native_ir)
        ));
        out.push_str(&format!(
            "dispatch_lowering = \"{}\"\n",
            escape_toml_string(&input.dispatch_lowering)
        ));
        out.push_str(&format!("contract_count = {}\n", input.contract_count));
        out.push_str(&format!("content_bytes = {}\n", input.content_bytes));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            escape_toml_string(&input.content_hash)
        ));
    }
    out
}

pub(crate) fn render_link_unit_table(report: &NsldLinkUnitReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_LINK_UNIT_TABLE_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_LINK_UNIT_TABLE_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "table_kind = \"{}\"\n",
        escape_toml_string(NSLD_LINK_UNIT_TABLE_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("unit_count = {}\n", report.unit_count));
    out.push_str(&format!(
        "hetero_unit_count = {}\n",
        report.hetero_unit_count
    ));
    out.push_str(&format!("link_input_count = {}\n", report.link_input_count));
    out.push_str(&format!("clock_edge_count = {}\n", report.clock_edge_count));
    out.push_str(&format!(
        "data_segment_count = {}\n",
        report.data_segment_count
    ));
    out.push_str(&format!(
        "unit_table_hash = \"{}\"\n",
        escape_toml_string(&report.unit_table_hash)
    ));
    for unit in &report.units {
        out.push_str("\n[[link_unit]]\n");
        out.push_str(&format!("order_index = {}\n", unit.order_index));
        out.push_str(&format!(
            "unit_id = \"{}\"\n",
            escape_toml_string(&unit.unit_id)
        ));
        out.push_str(&format!(
            "unit_kind = \"{}\"\n",
            escape_toml_string(&unit.unit_kind)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(&unit.backend_family)
        ));
        out.push_str(&format!(
            "lowering_target = \"{}\"\n",
            escape_toml_string(&unit.lowering_target)
        ));
        out.push_str(&format!(
            "packaging_role = \"{}\"\n",
            escape_toml_string(&unit.packaging_role)
        ));
        out.push_str(&format!(
            "link_input_ids = [{}]\n",
            toml_string_array_literal(&unit.link_input_ids)
        ));
        out.push_str(&format!("clock_edge_count = {}\n", unit.clock_edge_count));
        out.push_str(&format!(
            "data_segment_count = {}\n",
            unit.data_segment_count
        ));
        out.push_str(&format!(
            "requires_host_wrapper = {}\n",
            unit.requires_host_wrapper
        ));
        out.push_str(&format!(
            "deterministic_order_key = \"{}\"\n",
            escape_toml_string(&unit.deterministic_order_key)
        ));
    }
    out
}

pub(crate) fn render_link_bundle(report: &NsldLinkBundleReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_LINK_BUNDLE_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_LINK_BUNDLE_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "bundle_kind = \"{}\"\n",
        escape_toml_string(NSLD_LINK_BUNDLE_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!(
        "bundle_id = \"{}\"\n",
        escape_toml_string(&report.bundle_id)
    ));
    out.push_str(&format!(
        "bundle_hash = \"{}\"\n",
        escape_toml_string(&report.bundle_hash)
    ));
    out.push_str(&format!("bundle_ready = {}\n", report.bundle_ready));
    out.push_str(&format!("unit_count = {}\n", report.unit_count));
    out.push_str(&format!(
        "hetero_unit_count = {}\n",
        report.hetero_unit_count
    ));
    out.push_str(&format!("link_input_count = {}\n", report.link_input_count));
    out.push_str(&format!(
        "link_input_total_bytes = {}\n",
        report.link_input_total_bytes
    ));
    out.push_str(&format!(
        "link_input_table_hash = \"{}\"\n",
        escape_toml_string(&report.link_input_table_hash)
    ));
    out.push_str(&format!(
        "unit_table_hash = \"{}\"\n",
        escape_toml_string(&report.unit_table_hash)
    ));
    out.push_str(&format!("clock_edge_count = {}\n", report.clock_edge_count));
    out.push_str(&format!(
        "data_segment_count = {}\n",
        report.data_segment_count
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "host_wrapper_required = {}\n",
        report.host_wrapper_required
    ));
    out.push_str(&format!(
        "compiled_artifact_path = \"{}\"\n",
        escape_toml_string(&report.compiled_artifact_path)
    ));
    out.push_str(&format!(
        "native_output_path = \"{}\"\n",
        escape_toml_string(&report.native_output_path)
    ));
    out.push_str(&format!(
        "issues = [{}]\n",
        toml_string_array_literal(&report.issues)
    ));
    out
}

pub(crate) fn render_assemble_plan(report: &NsldAssemblePlanReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_ASSEMBLE_PLAN_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_ASSEMBLE_PLAN_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "plan_kind = \"{}\"\n",
        escape_toml_string(NSLD_ASSEMBLE_PLAN_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "bundle_id = \"{}\"\n",
        escape_toml_string(&report.bundle_id)
    ));
    out.push_str(&format!(
        "bundle_hash = \"{}\"\n",
        escape_toml_string(&report.bundle_hash)
    ));
    out.push_str(&format!(
        "assemble_plan_hash = \"{}\"\n",
        escape_toml_string(&report.assemble_plan_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "section_id = \"{}\"\n",
            escape_toml_string(&section.section_id)
        ));
        out.push_str(&format!(
            "section_kind = \"{}\"\n",
            escape_toml_string(&section.section_kind)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!("required = {}\n", section.required));
    }
    out
}

pub(crate) fn render_section_manifest(report: &NsldSectionManifestReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_SECTION_MANIFEST_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_SECTION_MANIFEST_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "manifest_kind = \"{}\"\n",
        escape_toml_string(NSLD_SECTION_MANIFEST_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "assemble_plan_hash = \"{}\"\n",
        escape_toml_string(&report.assemble_plan_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "section_table_hash = \"{}\"\n",
        escape_toml_string(&report.section_table_hash)
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "section_id = \"{}\"\n",
            escape_toml_string(&section.section_id)
        ));
        out.push_str(&format!(
            "section_kind = \"{}\"\n",
            escape_toml_string(&section.section_kind)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!("required = {}\n", section.required));
    }
    out
}

pub(crate) fn toml_string_array_literal(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml_string(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
