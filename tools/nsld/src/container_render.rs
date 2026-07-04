const CONTAINER_PLAN_SCHEMA: &str = "nuis-nsld-container-plan-v1";
const CONTAINER_PLAN_SCHEMA_VERSION: usize = 1;
const CONTAINER_PLAN_KIND: &str = "deterministic-container-layout-plan";
const CONTAINER_SCHEMA: &str = "nuis-nsld-container-v1";
const CONTAINER_SCHEMA_VERSION: usize = 1;
const CONTAINER_KIND: &str = "deterministic-hetero-container";
const PRODUCER: &str = "nsld";
const PRODUCER_PHASE: &str = "alpha-0.6.0";

use super::container_model::{NsldContainerPlanReport, NsldContainerReport};

pub(crate) fn render_container_plan_toml(report: &NsldContainerPlanReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(CONTAINER_PLAN_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {CONTAINER_PLAN_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "plan_kind = \"{}\"\n",
        escape_toml_string(CONTAINER_PLAN_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "container_magic = \"{}\"\n",
        escape_toml_string(&report.container_magic)
    ));
    out.push_str(&format!(
        "container_version = {}\n",
        report.container_version
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "section_table_hash = \"{}\"\n",
        escape_toml_string(&report.section_table_hash)
    ));
    out.push_str(&format!(
        "container_layout_hash = \"{}\"\n",
        escape_toml_string(&report.container_layout_hash)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        escape_toml_string(&report.output_path)
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

pub(crate) fn render_container_toml(report: &NsldContainerReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(CONTAINER_SCHEMA)
    ));
    out.push_str(&format!("schema_version = {CONTAINER_SCHEMA_VERSION}\n"));
    out.push_str(&format!(
        "container_kind = \"{}\"\n",
        escape_toml_string(CONTAINER_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "container_magic = \"{}\"\n",
        escape_toml_string(&report.container_magic)
    ));
    out.push_str(&format!(
        "container_version = {}\n",
        report.container_version
    ));
    out.push_str(&format!(
        "metadata_table_hash = \"{}\"\n",
        escape_toml_string(&report.metadata_table_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "container_section_table_hash = \"{}\"\n",
        escape_toml_string(&report.container_section_table_hash)
    ));
    out.push_str(&format!(
        "container_layout_hash = \"{}\"\n",
        escape_toml_string(&report.container_layout_hash)
    ));
    out.push_str(&format!(
        "container_hash = \"{}\"\n",
        escape_toml_string(&report.container_hash)
    ));
    out.push_str(&format!(
        "loader_readiness = \"{}\"\n",
        escape_toml_string(&report.loader_readiness)
    ));
    out.push_str(&format!(
        "loader_blockers = [{}]\n",
        toml_string_array_literal(&report.loader_blockers)
    ));
    out.push_str(&format!(
        "loader_entry_kind = \"{}\"\n",
        escape_toml_string(&report.loader_entry_kind)
    ));
    out.push_str(&format!(
        "loader_entry_symbol = \"{}\"\n",
        escape_toml_string(&report.loader_entry_symbol)
    ));
    out.push_str(&format!(
        "loader_entry_section_id = \"{}\"\n",
        escape_toml_string(&report.loader_entry_section_id)
    ));
    out.push_str(&format!(
        "loader_symbol_count = {}\n",
        report.loader_symbols.len()
    ));
    out.push_str(&format!(
        "loader_symbol_table_hash = \"{}\"\n",
        escape_toml_string(&report.loader_symbol_table_hash)
    ));
    out.push_str(&format!(
        "relocation_count = {}\n",
        report.relocations.len()
    ));
    out.push_str(&format!(
        "relocation_table_hash = \"{}\"\n",
        escape_toml_string(&report.relocation_table_hash)
    ));
    out.push_str(&format!(
        "external_import_count = {}\n",
        report.external_imports.len()
    ));
    out.push_str(&format!(
        "compatibility_domain_count = {}\n",
        report.compatibility_domains.len()
    ));
    out.push_str(&format!(
        "compatibility_domain_table_hash = \"{}\"\n",
        escape_toml_string(&report.compatibility_domain_table_hash)
    ));
    out.push_str(&format!(
        "external_import_table_hash = \"{}\"\n",
        escape_toml_string(&report.external_import_table_hash)
    ));
    out.push_str(&format!(
        "payload_size_bytes = {}\n",
        report.payload_size_bytes
    ));
    out.push_str(&format!(
        "payload_hash = \"{}\"\n",
        escape_toml_string(&report.payload_hash)
    ));
    out.push_str(&format!(
        "payload_path = \"{}\"\n",
        escape_toml_string(&report.payload_path)
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for symbol in &report.loader_symbols {
        out.push_str("\n[[loader_symbol]]\n");
        out.push_str(&format!(
            "symbol_id = \"{}\"\n",
            escape_toml_string(&symbol.symbol_id)
        ));
        out.push_str(&format!(
            "symbol_kind = \"{}\"\n",
            escape_toml_string(&symbol.symbol_kind)
        ));
        out.push_str(&format!(
            "symbol_name = \"{}\"\n",
            escape_toml_string(&symbol.symbol_name)
        ));
        out.push_str(&format!(
            "lifecycle_hook = \"{}\"\n",
            escape_toml_string(&symbol.lifecycle_hook)
        ));
        out.push_str(&format!(
            "section_id = \"{}\"\n",
            escape_toml_string(&symbol.section_id)
        ));
        out.push_str(&format!("offset = {}\n", symbol.offset));
        out.push_str(&format!("size_bytes = {}\n", symbol.size_bytes));
        out.push_str(&format!(
            "payload_hash = \"{}\"\n",
            escape_toml_string(&symbol.payload_hash)
        ));
    }
    for relocation in &report.relocations {
        out.push_str("\n[[relocation]]\n");
        out.push_str(&format!(
            "relocation_id = \"{}\"\n",
            escape_toml_string(&relocation.relocation_id)
        ));
        out.push_str(&format!(
            "relocation_kind = \"{}\"\n",
            escape_toml_string(&relocation.relocation_kind)
        ));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            escape_toml_string(&relocation.source_section_id)
        ));
        out.push_str(&format!("source_offset = {}\n", relocation.source_offset));
        out.push_str(&format!(
            "target_symbol_id = \"{}\"\n",
            escape_toml_string(&relocation.target_symbol_id)
        ));
        out.push_str(&format!("addend = {}\n", relocation.addend));
    }
    for domain in &report.compatibility_domains {
        out.push_str("\n[[compatibility_domain]]\n");
        out.push_str(&format!(
            "domain_id = \"{}\"\n",
            escape_toml_string(&domain.domain_id)
        ));
        out.push_str(&format!(
            "domain_kind = \"{}\"\n",
            escape_toml_string(&domain.domain_kind)
        ));
        out.push_str(&format!(
            "paradigm = \"{}\"\n",
            escape_toml_string(&domain.paradigm)
        ));
        out.push_str(&format!(
            "lifecycle_hook = \"{}\"\n",
            escape_toml_string(&domain.lifecycle_hook)
        ));
        out.push_str(&format!(
            "abi_family = \"{}\"\n",
            escape_toml_string(&domain.abi_family)
        ));
        out.push_str(&format!(
            "wrapper_policy = \"{}\"\n",
            escape_toml_string(&domain.wrapper_policy)
        ));
        out.push_str(&format!("required = {}\n", domain.required));
    }
    for external_import in &report.external_imports {
        out.push_str("\n[[external_import]]\n");
        out.push_str(&format!(
            "import_id = \"{}\"\n",
            escape_toml_string(&external_import.import_id)
        ));
        out.push_str(&format!(
            "import_kind = \"{}\"\n",
            escape_toml_string(&external_import.import_kind)
        ));
        out.push_str(&format!(
            "import_name = \"{}\"\n",
            escape_toml_string(&external_import.import_name)
        ));
        out.push_str(&format!(
            "provider = \"{}\"\n",
            escape_toml_string(&external_import.provider)
        ));
        out.push_str(&format!("required = {}\n", external_import.required));
    }
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
        out.push_str(&format!(
            "payload_hash = \"{}\"\n",
            escape_toml_string(&section.payload_hash)
        ));
        out.push_str(&format!("required = {}\n", section.required));
        out.push_str(&format!("offset = {}\n", section.offset));
        out.push_str(&format!("size_bytes = {}\n", section.size_bytes));
    }
    out
}

fn toml_string_array_literal(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml_string(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
