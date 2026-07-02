use super::{container, container_pipeline_actual::ActualContainerFields};

pub(crate) fn container_metadata_issues(
    expected_report: &container::NsldContainerReport,
    expected_toml: &str,
    actual_toml: &str,
    actual: &ActualContainerFields,
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual_toml != expected_toml {
        issues.push("container-content-mismatch".to_owned());
    }

    push_string_mismatch(
        &mut issues,
        "container_layout_hash",
        &expected_report.container_layout_hash,
        &actual.actual_container_layout_hash,
    );
    push_string_mismatch(
        &mut issues,
        "container_hash",
        &expected_report.container_hash,
        &actual.actual_container_hash,
    );
    push_string_mismatch(
        &mut issues,
        "metadata_table_hash",
        &expected_report.metadata_table_hash,
        &actual.actual_metadata_table_hash,
    );
    push_usize_mismatch(
        &mut issues,
        "payload_size_bytes",
        expected_report.payload_size_bytes,
        actual.actual_payload_size_bytes,
    );
    push_string_mismatch(
        &mut issues,
        "payload_hash",
        &expected_report.payload_hash,
        &actual.actual_payload_hash,
    );
    push_usize_mismatch(
        &mut issues,
        "section_count",
        expected_report.section_count,
        actual.actual_section_count,
    );
    push_string_mismatch(
        &mut issues,
        "container_section_table_hash",
        &expected_report.container_section_table_hash,
        &actual.actual_container_section_table_hash,
    );
    push_string_mismatch(
        &mut issues,
        "loader_readiness",
        &expected_report.loader_readiness,
        &actual.actual_loader_readiness,
    );
    push_string_mismatch(
        &mut issues,
        "loader_entry_kind",
        &expected_report.loader_entry_kind,
        &actual.actual_loader_entry_kind,
    );
    push_string_mismatch(
        &mut issues,
        "loader_entry_symbol",
        &expected_report.loader_entry_symbol,
        &actual.actual_loader_entry_symbol,
    );
    push_string_mismatch(
        &mut issues,
        "loader_entry_section_id",
        &expected_report.loader_entry_section_id,
        &actual.actual_loader_entry_section_id,
    );
    push_usize_mismatch(
        &mut issues,
        "loader_symbol_count",
        expected_report.loader_symbols.len(),
        actual.actual_loader_symbol_count,
    );
    if let Some(expected_symbol) = expected_report.loader_symbols.first() {
        push_string_mismatch(
            &mut issues,
            "loader_symbol_id",
            &expected_symbol.symbol_id,
            &actual.actual_loader_symbol_id,
        );
        push_string_mismatch(
            &mut issues,
            "loader_symbol_kind",
            &expected_symbol.symbol_kind,
            &actual.actual_loader_symbol_kind,
        );
        push_string_mismatch(
            &mut issues,
            "loader_symbol_name",
            &expected_symbol.symbol_name,
            &actual.actual_loader_symbol_name,
        );
        push_string_mismatch(
            &mut issues,
            "loader_symbol_section_id",
            &expected_symbol.section_id,
            &actual.actual_loader_symbol_section_id,
        );
    }
    push_string_mismatch(
        &mut issues,
        "loader_symbol_table_hash",
        &expected_report.loader_symbol_table_hash,
        &actual.actual_loader_symbol_table_hash,
    );
    push_usize_mismatch(
        &mut issues,
        "relocation_count",
        expected_report.relocations.len(),
        actual.actual_relocation_count,
    );
    push_string_mismatch(
        &mut issues,
        "relocation_table_hash",
        &expected_report.relocation_table_hash,
        &actual.actual_relocation_table_hash,
    );
    if let Some(expected_relocation) = expected_report.relocations.first() {
        push_string_mismatch(
            &mut issues,
            "relocation_id",
            &expected_relocation.relocation_id,
            &actual.actual_relocation_id,
        );
        push_string_mismatch(
            &mut issues,
            "relocation_kind",
            &expected_relocation.relocation_kind,
            &actual.actual_relocation_kind,
        );
        push_string_mismatch(
            &mut issues,
            "relocation_source_section_id",
            &expected_relocation.source_section_id,
            &actual.actual_relocation_source_section_id,
        );
        push_usize_mismatch(
            &mut issues,
            "relocation_source_offset",
            expected_relocation.source_offset,
            actual.actual_relocation_source_offset,
        );
        push_string_mismatch(
            &mut issues,
            "relocation_target_symbol_id",
            &expected_relocation.target_symbol_id,
            &actual.actual_relocation_target_symbol_id,
        );
        push_isize_mismatch(
            &mut issues,
            "relocation_addend",
            expected_relocation.addend,
            actual.actual_relocation_addend,
        );
    }
    push_usize_mismatch(
        &mut issues,
        "external_import_count",
        expected_report.external_imports.len(),
        actual.actual_external_import_count,
    );
    push_string_mismatch(
        &mut issues,
        "external_import_table_hash",
        &expected_report.external_import_table_hash,
        &actual.actual_external_import_table_hash,
    );
    if let Some(expected_import) = expected_report.external_imports.first() {
        push_string_mismatch(
            &mut issues,
            "external_import_id",
            &expected_import.import_id,
            &actual.actual_external_import_id,
        );
        push_string_mismatch(
            &mut issues,
            "external_import_kind",
            &expected_import.import_kind,
            &actual.actual_external_import_kind,
        );
        push_string_mismatch(
            &mut issues,
            "external_import_name",
            &expected_import.import_name,
            &actual.actual_external_import_name,
        );
        push_string_mismatch(
            &mut issues,
            "external_import_provider",
            &expected_import.provider,
            &actual.actual_external_import_provider,
        );
        push_bool_mismatch(
            &mut issues,
            "external_import_required",
            expected_import.required,
            actual.actual_external_import_required,
        );
    }

    issues
}

fn push_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &str,
    actual: &Option<String>,
) {
    if actual.as_deref() != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual.clone().unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn push_usize_mismatch(
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

fn push_isize_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: isize,
    actual: Option<isize>,
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

fn push_bool_mismatch(issues: &mut Vec<String>, field: &str, expected: bool, actual: Option<bool>) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}
