use super::{
    container::NsldContainerVerifyReport,
    display::{optional_bool_text, optional_string_text, optional_usize_text},
};

pub(crate) fn print_nsld_container_verify_report(report: &NsldContainerVerifyReport) {
    println!("Nsld container verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_container_layout_hash: {}",
        report.expected_container_layout_hash
    );
    println!(
        "  expected_container_hash: {}",
        report.expected_container_hash
    );
    println!(
        "  expected_metadata_table_hash: {}",
        report.expected_metadata_table_hash
    );
    println!(
        "  expected_payload_size_bytes: {}",
        report.expected_payload_size_bytes
    );
    println!("  expected_payload_hash: {}", report.expected_payload_hash);
    println!("  expected_payload_path: {}", report.expected_payload_path);
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  expected_container_section_table_hash: {}",
        report.expected_container_section_table_hash
    );
    println!(
        "  expected_loader_readiness: {}",
        report.expected_loader_readiness
    );
    println!(
        "  expected_loader_entry_kind: {}",
        report.expected_loader_entry_kind
    );
    println!(
        "  expected_loader_entry_symbol: {}",
        report.expected_loader_entry_symbol
    );
    println!(
        "  expected_loader_entry_section_id: {}",
        report.expected_loader_entry_section_id
    );
    println!(
        "  expected_loader_symbol_count: {}",
        report.expected_loader_symbol_count
    );
    println!(
        "  expected_loader_symbol_table_hash: {}",
        report.expected_loader_symbol_table_hash
    );
    println!(
        "  expected_loader_symbol_id: {}",
        report.expected_loader_symbol_id
    );
    println!(
        "  expected_loader_symbol_kind: {}",
        report.expected_loader_symbol_kind
    );
    println!(
        "  expected_loader_symbol_name: {}",
        report.expected_loader_symbol_name
    );
    println!(
        "  expected_loader_symbol_section_id: {}",
        report.expected_loader_symbol_section_id
    );
    println!(
        "  expected_relocation_count: {}",
        report.expected_relocation_count
    );
    println!(
        "  expected_relocation_table_hash: {}",
        report.expected_relocation_table_hash
    );
    println!(
        "  expected_relocation_id: {}",
        report.expected_relocation_id
    );
    println!(
        "  expected_relocation_kind: {}",
        report.expected_relocation_kind
    );
    println!(
        "  expected_relocation_source_section_id: {}",
        report.expected_relocation_source_section_id
    );
    println!(
        "  expected_relocation_source_offset: {}",
        report.expected_relocation_source_offset
    );
    println!(
        "  expected_relocation_target_symbol_id: {}",
        report.expected_relocation_target_symbol_id
    );
    println!(
        "  expected_relocation_addend: {}",
        report.expected_relocation_addend
    );
    println!(
        "  expected_external_import_count: {}",
        report.expected_external_import_count
    );
    println!(
        "  expected_external_import_table_hash: {}",
        report.expected_external_import_table_hash
    );
    println!(
        "  expected_external_import_id: {}",
        report.expected_external_import_id
    );
    println!(
        "  expected_external_import_kind: {}",
        report.expected_external_import_kind
    );
    println!(
        "  expected_external_import_name: {}",
        report.expected_external_import_name
    );
    println!(
        "  expected_external_import_provider: {}",
        report.expected_external_import_provider
    );
    println!(
        "  expected_external_import_required: {}",
        report.expected_external_import_required
    );
    println!(
        "  expected_native_object: section_present={} section_id={} loader_symbol_present={} loader_symbol_id={} relocation_present={} relocation_id={}",
        report.expected_native_object_section_present,
        report.expected_native_object_section_id,
        report.expected_native_object_loader_symbol_present,
        report.expected_native_object_loader_symbol_id,
        report.expected_native_object_relocation_present,
        report.expected_native_object_relocation_id
    );
    println!(
        "  actual_container_layout_hash: {}",
        report
            .actual_container_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_container_hash: {}",
        report.actual_container_hash.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_metadata_table_hash: {}",
        report
            .actual_metadata_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_payload_size_bytes: {}",
        optional_usize_text(report.actual_payload_size_bytes)
    );
    println!(
        "  actual_payload_hash: {}",
        report.actual_payload_hash.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    println!(
        "  actual_container_section_table_hash: {}",
        report
            .actual_container_section_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_readiness: {}",
        report
            .actual_loader_readiness
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_entry_kind: {}",
        report
            .actual_loader_entry_kind
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_entry_symbol: {}",
        report
            .actual_loader_entry_symbol
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_entry_section_id: {}",
        report
            .actual_loader_entry_section_id
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_symbol_count: {}",
        optional_usize_text(report.actual_loader_symbol_count)
    );
    println!(
        "  actual_loader_symbol_table_hash: {}",
        report
            .actual_loader_symbol_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_symbol_id: {}",
        report
            .actual_loader_symbol_id
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_symbol_kind: {}",
        report
            .actual_loader_symbol_kind
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_symbol_name: {}",
        report
            .actual_loader_symbol_name
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_loader_symbol_section_id: {}",
        report
            .actual_loader_symbol_section_id
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_relocation_count: {}",
        optional_usize_text(report.actual_relocation_count)
    );
    println!(
        "  actual_relocation_table_hash: {}",
        report
            .actual_relocation_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_relocation_id: {}",
        report.actual_relocation_id.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_relocation_kind: {}",
        report
            .actual_relocation_kind
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_relocation_source_section_id: {}",
        report
            .actual_relocation_source_section_id
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_relocation_source_offset: {}",
        optional_usize_text(report.actual_relocation_source_offset)
    );
    println!(
        "  actual_relocation_target_symbol_id: {}",
        report
            .actual_relocation_target_symbol_id
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_relocation_addend: {}",
        report
            .actual_relocation_addend
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  actual_external_import_count: {}",
        optional_usize_text(report.actual_external_import_count)
    );
    println!(
        "  actual_external_import_table_hash: {}",
        report
            .actual_external_import_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_external_import_id: {}",
        report
            .actual_external_import_id
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_external_import_kind: {}",
        report
            .actual_external_import_kind
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_external_import_name: {}",
        report
            .actual_external_import_name
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_external_import_provider: {}",
        report
            .actual_external_import_provider
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_external_import_required: {}",
        optional_bool_text(report.actual_external_import_required)
    );
    println!(
        "  actual_native_object: section_present={} section_id={} loader_symbol_present={} loader_symbol_id={} relocation_present={} relocation_id={}",
        report.actual_native_object_section_present,
        optional_string_text(report.actual_native_object_section_id.as_deref()),
        report.actual_native_object_loader_symbol_present,
        optional_string_text(report.actual_native_object_loader_symbol_id.as_deref()),
        report.actual_native_object_relocation_present,
        optional_string_text(report.actual_native_object_relocation_id.as_deref())
    );
    for issue in &report.container_section_issues {
        println!("  container_section_issue: {issue}");
    }
    for issue in &report.loader_symbol_issues {
        println!("  loader_symbol_issue: {issue}");
    }
    for issue in &report.relocation_issues {
        println!("  relocation_issue: {issue}");
    }
    for issue in &report.external_import_issues {
        println!("  external_import_issue: {issue}");
    }
    for issue in &report.section_range_issues {
        println!("  section_range_issue: {issue}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
