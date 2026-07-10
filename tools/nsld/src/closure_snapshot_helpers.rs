use super::{fnv1a64_hex, reports::NsldClosureReport, toml};
use std::path::PathBuf;

pub(crate) fn closure_snapshot_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    PathBuf::from(&plan.output_dir).join("nuis.nsld.closure.toml")
}

pub(crate) fn render_closure_snapshot(report: &NsldClosureReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-closure-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"linker-closure\"\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.6.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!("closed = {}\n", report.closed));
    out.push_str(&format!(
        "linker_contract_hash = \"{}\"\n",
        toml::escape_toml_string(&report.linker_contract_hash)
    ));
    out.push_str(&format!(
        "internal_contract_count = {}\n",
        report.internal_contracts.len()
    ));
    out.push_str(&format!(
        "link_input_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.link_input_table_hash)
    ));
    out.push_str(&format!(
        "container_metadata_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.container_metadata_table_hash)
    ));
    out.push_str(&format!(
        "container_layout_hash = \"{}\"\n",
        toml::escape_toml_string(&report.container_layout_hash)
    ));
    out.push_str(&format!(
        "container_hash = \"{}\"\n",
        toml::escape_toml_string(&report.container_hash)
    ));
    out.push_str(&format!(
        "payload_size_bytes = {}\n",
        report.payload_size_bytes
    ));
    out.push_str(&format!(
        "payload_hash = \"{}\"\n",
        toml::escape_toml_string(&report.payload_hash)
    ));
    out.push_str(&format!(
        "object_image_relocation_record_table_hash = \"{}\"\n",
        toml::escape_toml_string(
            report
                .object_image_relocation_record_table_hash
                .as_deref()
                .unwrap_or("missing")
        )
    ));
    out.push_str(&format!("unresolved_count = {}\n", report.unresolved.len()));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_link_mode)
    ));
    out
}

pub(crate) fn push_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &str,
    actual: Option<&str>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual.unwrap_or("missing")
        ));
    }
}

pub(crate) fn push_bool_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: bool,
    actual: Option<bool>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

pub(crate) fn push_usize_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: usize,
    actual: Option<usize>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn nsld_linker_contract_hash(
    internal_contracts: &[String],
    link_input_table_hash: &str,
    container_metadata_table_hash: &str,
    container_hash: &str,
    payload_hash: &str,
    container_loader_readiness: &str,
    object_image_relocation_record_table_hash: Option<&str>,
    external_dependencies: &[String],
    unresolved: &[String],
    final_stage_link_mode: &str,
) -> String {
    let mut material = String::new();
    material.push_str("internal_contracts\n");
    for contract in internal_contracts {
        material.push_str(contract);
        material.push('\n');
    }
    material.push_str("link_input_table_hash\t");
    material.push_str(link_input_table_hash);
    material.push('\n');
    material.push_str("container_metadata_table_hash\t");
    material.push_str(container_metadata_table_hash);
    material.push('\n');
    material.push_str("container_hash\t");
    material.push_str(container_hash);
    material.push('\n');
    material.push_str("payload_hash\t");
    material.push_str(payload_hash);
    material.push('\n');
    material.push_str("container_loader_readiness\t");
    material.push_str(container_loader_readiness);
    material.push('\n');
    material.push_str("object_image_relocation_record_table_hash\t");
    material.push_str(object_image_relocation_record_table_hash.unwrap_or("missing"));
    material.push('\n');
    material.push_str("external_dependencies\n");
    for dependency in external_dependencies {
        material.push_str(dependency);
        material.push('\n');
    }
    material.push_str("unresolved\n");
    for issue in unresolved {
        material.push_str(issue);
        material.push('\n');
    }
    material.push_str("final_stage_link_mode\t");
    material.push_str(final_stage_link_mode);
    material.push('\n');
    fnv1a64_hex(material.as_bytes())
}
