use std::fs;

use super::reports::NsldAssembleSectionDiagnostic;

const CONTAINER_PLAN_SCHEMA: &str = "nuis-nsld-container-plan-v1";
const CONTAINER_PLAN_SCHEMA_VERSION: usize = 1;
const CONTAINER_PLAN_KIND: &str = "deterministic-container-layout-plan";
const CONTAINER_SCHEMA: &str = "nuis-nsld-container-v1";
const CONTAINER_SCHEMA_VERSION: usize = 1;
const CONTAINER_KIND: &str = "deterministic-hetero-container";
const PRODUCER: &str = "nsld";
const PRODUCER_PHASE: &str = "alpha-0.6.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerPlanReport {
    pub(crate) manifest: String,
    pub(crate) ready: bool,
    pub(crate) container_magic: String,
    pub(crate) container_version: usize,
    pub(crate) section_count: usize,
    pub(crate) section_table_hash: String,
    pub(crate) container_layout_hash: String,
    pub(crate) output_path: String,
    pub(crate) sections: Vec<NsldAssembleSectionDiagnostic>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerPlanEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) ready: bool,
    pub(crate) container_layout_hash: String,
    pub(crate) section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerPlanVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_container_layout_hash: String,
    pub(crate) expected_section_count: usize,
    pub(crate) actual_container_layout_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerReport {
    pub(crate) manifest: String,
    pub(crate) ready: bool,
    pub(crate) container_magic: String,
    pub(crate) container_version: usize,
    pub(crate) container_layout_hash: String,
    pub(crate) container_hash: String,
    pub(crate) loader_readiness: String,
    pub(crate) loader_blockers: Vec<String>,
    pub(crate) loader_entry_kind: String,
    pub(crate) loader_entry_symbol: String,
    pub(crate) loader_entry_section_id: String,
    pub(crate) loader_symbols: Vec<NsldContainerLoaderSymbol>,
    pub(crate) relocations: Vec<NsldContainerRelocationEntry>,
    pub(crate) external_imports: Vec<NsldContainerExternalImport>,
    pub(crate) payload_size_bytes: usize,
    pub(crate) payload_hash: String,
    pub(crate) output_path: String,
    pub(crate) payload_path: String,
    pub(crate) section_count: usize,
    pub(crate) sections: Vec<NsldContainerSectionEntry>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerExternalImport {
    pub(crate) import_id: String,
    pub(crate) import_kind: String,
    pub(crate) import_name: String,
    pub(crate) provider: String,
    pub(crate) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerLoaderSymbol {
    pub(crate) symbol_id: String,
    pub(crate) symbol_kind: String,
    pub(crate) symbol_name: String,
    pub(crate) section_id: String,
    pub(crate) offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerRelocationEntry {
    pub(crate) relocation_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) source_section_id: String,
    pub(crate) source_offset: usize,
    pub(crate) target_symbol_id: String,
    pub(crate) addend: isize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerSectionEntry {
    pub(crate) order_index: usize,
    pub(crate) section_id: String,
    pub(crate) section_kind: String,
    pub(crate) source_path: String,
    pub(crate) source_hash: String,
    pub(crate) payload_hash: String,
    pub(crate) required: bool,
    pub(crate) offset: usize,
    pub(crate) size_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) payload_path: String,
    pub(crate) ready: bool,
    pub(crate) container_layout_hash: String,
    pub(crate) container_hash: String,
    pub(crate) payload_size_bytes: usize,
    pub(crate) payload_hash: String,
    pub(crate) section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_container_layout_hash: String,
    pub(crate) expected_container_hash: String,
    pub(crate) expected_payload_size_bytes: usize,
    pub(crate) expected_payload_hash: String,
    pub(crate) expected_payload_path: String,
    pub(crate) expected_section_count: usize,
    pub(crate) expected_loader_readiness: String,
    pub(crate) expected_external_import_count: usize,
    pub(crate) actual_container_layout_hash: Option<String>,
    pub(crate) actual_container_hash: Option<String>,
    pub(crate) actual_payload_size_bytes: Option<usize>,
    pub(crate) actual_payload_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) actual_loader_readiness: Option<String>,
    pub(crate) actual_external_import_count: Option<usize>,
    pub(crate) section_range_issues: Vec<String>,
    pub(crate) issues: Vec<String>,
}

pub(crate) fn payload_size(sections: &[NsldContainerSectionEntry]) -> usize {
    sections
        .iter()
        .map(|section| section.size_bytes)
        .fold(0usize, usize::saturating_add)
}

pub(crate) fn layout_hash(
    container_magic: &str,
    container_version: usize,
    section_count: usize,
    section_table_hash: &str,
    output_path: &str,
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let material = format!(
        "{container_magic}\t{container_version}\t{section_count}\t{section_table_hash}\t{output_path}\n"
    );
    hash_bytes(material.as_bytes())
}

pub(crate) fn section_entries(
    sections: &[NsldAssembleSectionDiagnostic],
    hash_bytes: fn(&[u8]) -> String,
) -> Vec<NsldContainerSectionEntry> {
    let mut offset = 0usize;
    sections
        .iter()
        .map(|section| {
            let size_bytes = fs::metadata(&section.source_path)
                .map(|metadata| metadata.len() as usize)
                .unwrap_or(0);
            let payload_hash = fs::read(&section.source_path)
                .map(|bytes| hash_bytes(&bytes))
                .unwrap_or_else(|_| "missing".to_owned());
            let entry = NsldContainerSectionEntry {
                order_index: section.order_index,
                section_id: section.section_id.clone(),
                section_kind: section.section_kind.clone(),
                source_path: section.source_path.clone(),
                source_hash: section.source_hash.clone(),
                payload_hash,
                required: section.required,
                offset,
                size_bytes,
            };
            offset = offset.saturating_add(size_bytes);
            entry
        })
        .collect()
}

pub(crate) fn payload_bytes(sections: &[NsldContainerSectionEntry]) -> Vec<u8> {
    let mut payload = Vec::new();
    for section in sections {
        if let Ok(bytes) = fs::read(&section.source_path) {
            payload.extend_from_slice(&bytes);
        }
    }
    payload
}

pub(crate) fn payload_hash(
    sections: &[NsldContainerSectionEntry],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    hash_bytes(&payload_bytes(sections))
}

pub(crate) fn file_hash(
    container_plan: &NsldContainerPlanReport,
    sections: &[NsldContainerSectionEntry],
    loader_entry_kind: &str,
    loader_entry_symbol: &str,
    loader_entry_section_id: &str,
    loader_symbols: &[NsldContainerLoaderSymbol],
    relocations: &[NsldContainerRelocationEntry],
    external_imports: &[NsldContainerExternalImport],
    loader_readiness: &str,
    loader_blockers: &[String],
    payload_size_bytes: usize,
    payload_hash: &str,
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    material.push_str(&container_plan.container_magic);
    material.push('\t');
    material.push_str(&container_plan.container_version.to_string());
    material.push('\t');
    material.push_str(&container_plan.container_layout_hash);
    material.push('\t');
    material.push_str(loader_readiness);
    material.push('\t');
    material.push_str(loader_entry_kind);
    material.push('\t');
    material.push_str(loader_entry_symbol);
    material.push('\t');
    material.push_str(loader_entry_section_id);
    material.push('\t');
    material.push_str(&payload_size_bytes.to_string());
    material.push('\t');
    material.push_str(payload_hash);
    material.push('\n');
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\t');
        material.push_str(&section.payload_hash);
        material.push('\t');
        material.push_str(&section.source_path);
        material.push('\t');
        material.push_str(&section.offset.to_string());
        material.push('\t');
        material.push_str(&section.size_bytes.to_string());
        material.push('\n');
    }
    for symbol in loader_symbols {
        material.push_str("loader_symbol\t");
        material.push_str(&symbol.symbol_id);
        material.push('\t');
        material.push_str(&symbol.symbol_kind);
        material.push('\t');
        material.push_str(&symbol.symbol_name);
        material.push('\t');
        material.push_str(&symbol.section_id);
        material.push('\t');
        material.push_str(&symbol.offset.to_string());
        material.push('\t');
        material.push_str(&symbol.size_bytes.to_string());
        material.push('\t');
        material.push_str(&symbol.payload_hash);
        material.push('\n');
    }
    for relocation in relocations {
        material.push_str("relocation\t");
        material.push_str(&relocation.relocation_id);
        material.push('\t');
        material.push_str(&relocation.relocation_kind);
        material.push('\t');
        material.push_str(&relocation.source_section_id);
        material.push('\t');
        material.push_str(&relocation.source_offset.to_string());
        material.push('\t');
        material.push_str(&relocation.target_symbol_id);
        material.push('\t');
        material.push_str(&relocation.addend.to_string());
        material.push('\n');
    }
    for external_import in external_imports {
        material.push_str("external_import\t");
        material.push_str(&external_import.import_id);
        material.push('\t');
        material.push_str(&external_import.import_kind);
        material.push('\t');
        material.push_str(&external_import.import_name);
        material.push('\t');
        material.push_str(&external_import.provider);
        material.push('\t');
        material.push_str(if external_import.required {
            "required"
        } else {
            "optional"
        });
        material.push('\n');
    }
    for blocker in loader_blockers {
        material.push_str("loader_blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    for blocker in &container_plan.blockers {
        material.push_str("blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn payload_range_issues(
    report: &NsldContainerReport,
    payload: &[u8],
    hash_bytes: fn(&[u8]) -> String,
) -> Vec<String> {
    let mut issues = Vec::new();
    for section in &report.sections {
        let end = section.offset.saturating_add(section.size_bytes);
        if end > payload.len() {
            issues.push(format!(
                "section_range_out_of_bounds: {} offset={} size={} payload_size={}",
                section.section_id,
                section.offset,
                section.size_bytes,
                payload.len()
            ));
            continue;
        }
        let actual_hash = hash_bytes(&payload[section.offset..end]);
        if actual_hash != section.payload_hash {
            issues.push(format!(
                "section_payload_hash mismatch: {} expected {}, found {}",
                section.section_id, section.payload_hash, actual_hash
            ));
        }
    }
    issues
}

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
    out.push_str(&format!("section_count = {}\n", report.section_count));
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
        "relocation_count = {}\n",
        report.relocations.len()
    ));
    out.push_str(&format!(
        "external_import_count = {}\n",
        report.external_imports.len()
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
