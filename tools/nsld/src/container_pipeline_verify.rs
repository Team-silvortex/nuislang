use super::{
    container,
    container_pipeline::nsld_container_report,
    container_pipeline_actual::{actual_container_fields, ActualContainerFields},
    container_pipeline_mismatch::container_metadata_issues,
    container_pipeline_tables::container_table_issue_sets,
    fnv1a64_hex,
};
use std::path::Path;
use std::{fs, path::PathBuf};

pub(crate) fn nsld_verify_container_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> container::NsldContainerVerifyReport {
    let expected_report = nsld_container_report(manifest, plan);
    let expected = container::render_container_toml(&expected_report);
    let input_path = PathBuf::from(&expected_report.output_path);
    let payload_path = PathBuf::from(&expected_report.payload_path);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_container `{}`: {error}",
            input_path.display()
        )
    });
    let actual_fields = actual_container_fields(actual.as_ref(), &mut issues);
    if let Ok(ref actual) = actual {
        issues.extend(container_metadata_issues(
            &expected_report,
            &expected,
            actual,
            &actual_fields,
        ));
    }
    let mut container_section_issues = Vec::new();
    let mut section_range_issues = Vec::new();
    let mut loader_symbol_issues = Vec::new();
    let mut relocation_issues = Vec::new();
    let mut external_import_issues = Vec::new();
    if let Ok(source) = actual.as_ref() {
        let table_issue_sets = container_table_issue_sets(source, &expected_report);
        container_section_issues = table_issue_sets.container_section_issues;
        issues.extend(container_section_issues.iter().cloned());

        loader_symbol_issues = table_issue_sets.loader_symbol_issues;
        issues.extend(loader_symbol_issues.iter().cloned());

        relocation_issues = table_issue_sets.relocation_issues;
        issues.extend(relocation_issues.iter().cloned());

        external_import_issues = table_issue_sets.external_import_issues;
        issues.extend(external_import_issues.iter().cloned());
    }
    let (actual_payload_file_size, actual_payload_file_hash) = match fs::read(&payload_path)
        .map_err(|error| {
            format!(
                "missing_or_unreadable_container_payload `{}`: {error}",
                payload_path.display()
            )
        }) {
        Ok(bytes) => {
            section_range_issues =
                container::payload_range_issues(&expected_report, &bytes, fnv1a64_hex);
            (Some(bytes.len()), Some(fnv1a64_hex(&bytes)))
        }
        Err(error) => {
            issues.push(error);
            (None, None)
        }
    };
    issues.extend(section_range_issues.iter().cloned());
    if actual_payload_file_size != Some(expected_report.payload_size_bytes) {
        issues.push(format!(
            "payload_file_size mismatch: expected {}, found {}",
            expected_report.payload_size_bytes,
            actual_payload_file_size
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if actual_payload_file_hash.as_deref() != Some(expected_report.payload_hash.as_str()) {
        issues.push(format!(
            "payload_file_hash mismatch: expected {}, found {}",
            expected_report.payload_hash,
            actual_payload_file_hash
                .clone()
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }

    let ActualContainerFields {
        actual_container_layout_hash,
        actual_container_hash,
        actual_metadata_table_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_section_count,
        actual_container_section_table_hash,
        actual_loader_readiness,
        actual_loader_entry_kind,
        actual_loader_entry_symbol,
        actual_loader_entry_section_id,
        actual_loader_symbol_count,
        actual_loader_symbol_id,
        actual_loader_symbol_kind,
        actual_loader_symbol_name,
        actual_loader_symbol_section_id,
        actual_loader_symbol_table_hash,
        actual_relocation_count,
        actual_relocation_table_hash,
        actual_relocation_id,
        actual_relocation_kind,
        actual_relocation_source_section_id,
        actual_relocation_source_offset,
        actual_relocation_target_symbol_id,
        actual_relocation_addend,
        actual_external_import_count,
        actual_external_import_table_hash,
        actual_external_import_id,
        actual_external_import_kind,
        actual_external_import_name,
        actual_external_import_provider,
        actual_external_import_required,
    } = actual_fields;
    container::NsldContainerVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_container_layout_hash: expected_report.container_layout_hash,
        expected_container_hash: expected_report.container_hash,
        expected_metadata_table_hash: expected_report.metadata_table_hash,
        expected_payload_size_bytes: expected_report.payload_size_bytes,
        expected_payload_hash: expected_report.payload_hash,
        expected_payload_path: expected_report.payload_path,
        expected_section_count: expected_report.section_count,
        expected_container_section_table_hash: expected_report.container_section_table_hash,
        expected_loader_readiness: expected_report.loader_readiness,
        expected_loader_entry_kind: expected_report.loader_entry_kind,
        expected_loader_entry_symbol: expected_report.loader_entry_symbol,
        expected_loader_entry_section_id: expected_report.loader_entry_section_id,
        expected_loader_symbol_count: expected_report.loader_symbols.len(),
        expected_loader_symbol_id: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.symbol_id.clone())
            .unwrap_or_default(),
        expected_loader_symbol_kind: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.symbol_kind.clone())
            .unwrap_or_default(),
        expected_loader_symbol_name: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.symbol_name.clone())
            .unwrap_or_default(),
        expected_loader_symbol_section_id: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.section_id.clone())
            .unwrap_or_default(),
        expected_loader_symbol_table_hash: expected_report.loader_symbol_table_hash,
        expected_relocation_count: expected_report.relocations.len(),
        expected_relocation_table_hash: expected_report.relocation_table_hash,
        expected_relocation_id: expected_report
            .relocations
            .first()
            .map(|relocation| relocation.relocation_id.clone())
            .unwrap_or_default(),
        expected_relocation_kind: expected_report
            .relocations
            .first()
            .map(|relocation| relocation.relocation_kind.clone())
            .unwrap_or_default(),
        expected_relocation_source_section_id: expected_report
            .relocations
            .first()
            .map(|relocation| relocation.source_section_id.clone())
            .unwrap_or_default(),
        expected_relocation_source_offset: expected_report
            .relocations
            .first()
            .map(|relocation| relocation.source_offset)
            .unwrap_or_default(),
        expected_relocation_target_symbol_id: expected_report
            .relocations
            .first()
            .map(|relocation| relocation.target_symbol_id.clone())
            .unwrap_or_default(),
        expected_relocation_addend: expected_report
            .relocations
            .first()
            .map(|relocation| relocation.addend)
            .unwrap_or_default(),
        expected_external_import_count: expected_report.external_imports.len(),
        expected_external_import_table_hash: expected_report.external_import_table_hash,
        expected_external_import_id: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.import_id.clone())
            .unwrap_or_default(),
        expected_external_import_kind: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.import_kind.clone())
            .unwrap_or_default(),
        expected_external_import_name: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.import_name.clone())
            .unwrap_or_default(),
        expected_external_import_provider: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.provider.clone())
            .unwrap_or_default(),
        expected_external_import_required: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.required)
            .unwrap_or(false),
        actual_container_layout_hash,
        actual_container_hash,
        actual_metadata_table_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_section_count,
        actual_container_section_table_hash,
        actual_loader_readiness,
        actual_loader_entry_kind,
        actual_loader_entry_symbol,
        actual_loader_entry_section_id,
        actual_loader_symbol_count,
        actual_loader_symbol_id,
        actual_loader_symbol_kind,
        actual_loader_symbol_name,
        actual_loader_symbol_section_id,
        actual_loader_symbol_table_hash,
        actual_relocation_count,
        actual_relocation_table_hash,
        actual_relocation_id,
        actual_relocation_kind,
        actual_relocation_source_section_id,
        actual_relocation_source_offset,
        actual_relocation_target_symbol_id,
        actual_relocation_addend,
        actual_external_import_count,
        actual_external_import_table_hash,
        actual_external_import_id,
        actual_external_import_kind,
        actual_external_import_name,
        actual_external_import_provider,
        actual_external_import_required,
        container_section_issues,
        section_range_issues,
        loader_symbol_issues,
        relocation_issues,
        external_import_issues,
        issues,
    }
}
