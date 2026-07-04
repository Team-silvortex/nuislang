use super::{
    container::{
        NsldContainerEmitReport, NsldContainerExternalImport, NsldContainerLoaderSymbol,
        NsldContainerPlanEmitReport, NsldContainerPlanReport, NsldContainerPlanVerifyReport,
        NsldContainerRelocationEntry, NsldContainerReport, NsldContainerVerifyReport,
    },
    json_fields::*,
    json_fragments::{nsld_assemble_sections_json, nsld_container_sections_json},
};

pub(crate) fn nsld_container_plan_report_json(report: &NsldContainerPlanReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("container_magic", &report.container_magic),
        json_usize_field("container_version", report.container_version),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("output_path", &report.output_path),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_container_plan_emit_report_json(report: &NsldContainerPlanEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_usize_field("section_count", report.section_count),
        json_string_field("container_layout_hash", &report.container_layout_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_container_plan_verify_report_json(
    report: &NsldContainerPlanVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_container_layout_hash",
            &report.expected_container_layout_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_optional_string_field(
            "actual_container_layout_hash",
            report.actual_container_layout_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_container_report_json(report: &NsldContainerReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("container_magic", &report.container_magic),
        json_usize_field("container_version", report.container_version),
        json_string_field("metadata_table_hash", &report.metadata_table_hash),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_string_field("loader_readiness", &report.loader_readiness),
        json_string_array_field("loader_blockers", &report.loader_blockers),
        json_string_field("loader_entry_kind", &report.loader_entry_kind),
        json_string_field("loader_entry_symbol", &report.loader_entry_symbol),
        json_string_field("loader_entry_section_id", &report.loader_entry_section_id),
        json_string_field("loader_symbol_table_hash", &report.loader_symbol_table_hash),
        json_string_field("relocation_table_hash", &report.relocation_table_hash),
        format!(
            "\"loader_symbols\":[{}]",
            nsld_container_loader_symbols_json(&report.loader_symbols)
        ),
        format!(
            "\"relocations\":[{}]",
            nsld_container_relocations_json(&report.relocations)
        ),
        format!(
            "\"external_imports\":[{}]",
            nsld_container_external_imports_json(&report.external_imports)
        ),
        json_string_field(
            "external_import_table_hash",
            &report.external_import_table_hash,
        ),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_field("output_path", &report.output_path),
        json_string_field("payload_path", &report.payload_path),
        json_usize_field("section_count", report.section_count),
        json_string_field(
            "container_section_table_hash",
            &report.container_section_table_hash,
        ),
        format!(
            "\"sections\":[{}]",
            nsld_container_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_container_loader_symbols_json(symbols: &[NsldContainerLoaderSymbol]) -> String {
    symbols
        .iter()
        .map(|symbol| {
            let fields = vec![
                json_string_field("symbol_id", &symbol.symbol_id),
                json_string_field("symbol_kind", &symbol.symbol_kind),
                json_string_field("symbol_name", &symbol.symbol_name),
                json_string_field("section_id", &symbol.section_id),
                json_usize_field("offset", symbol.offset),
                json_usize_field("size_bytes", symbol.size_bytes),
                json_string_field("payload_hash", &symbol.payload_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_container_relocations_json(relocations: &[NsldContainerRelocationEntry]) -> String {
    relocations
        .iter()
        .map(|relocation| {
            let fields = vec![
                json_string_field("relocation_id", &relocation.relocation_id),
                json_string_field("relocation_kind", &relocation.relocation_kind),
                json_string_field("source_section_id", &relocation.source_section_id),
                json_usize_field("source_offset", relocation.source_offset),
                json_string_field("target_symbol_id", &relocation.target_symbol_id),
                json_isize_field("addend", relocation.addend),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_container_external_imports_json(imports: &[NsldContainerExternalImport]) -> String {
    imports
        .iter()
        .map(|external_import| {
            let fields = vec![
                json_string_field("import_id", &external_import.import_id),
                json_string_field("import_kind", &external_import.import_kind),
                json_string_field("import_name", &external_import.import_name),
                json_string_field("provider", &external_import.provider),
                json_bool_field("required", external_import.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_container_emit_report_json(report: &NsldContainerEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("payload_path", &report.payload_path),
        json_bool_field("ready", report.ready),
        json_string_field("metadata_table_hash", &report.metadata_table_hash),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_usize_field("section_count", report.section_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_container_verify_report_json(report: &NsldContainerVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_container_layout_hash",
            &report.expected_container_layout_hash,
        ),
        json_string_field("expected_container_hash", &report.expected_container_hash),
        json_string_field(
            "expected_metadata_table_hash",
            &report.expected_metadata_table_hash,
        ),
        json_usize_field(
            "expected_payload_size_bytes",
            report.expected_payload_size_bytes,
        ),
        json_string_field("expected_payload_hash", &report.expected_payload_hash),
        json_string_field("expected_payload_path", &report.expected_payload_path),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_string_field(
            "expected_container_section_table_hash",
            &report.expected_container_section_table_hash,
        ),
        json_string_field(
            "expected_loader_readiness",
            &report.expected_loader_readiness,
        ),
        json_string_field(
            "expected_loader_entry_kind",
            &report.expected_loader_entry_kind,
        ),
        json_string_field(
            "expected_loader_entry_symbol",
            &report.expected_loader_entry_symbol,
        ),
        json_string_field(
            "expected_loader_entry_section_id",
            &report.expected_loader_entry_section_id,
        ),
        json_usize_field(
            "expected_loader_symbol_count",
            report.expected_loader_symbol_count,
        ),
        json_string_field(
            "expected_loader_symbol_table_hash",
            &report.expected_loader_symbol_table_hash,
        ),
        json_string_field(
            "expected_loader_symbol_id",
            &report.expected_loader_symbol_id,
        ),
        json_string_field(
            "expected_loader_symbol_kind",
            &report.expected_loader_symbol_kind,
        ),
        json_string_field(
            "expected_loader_symbol_name",
            &report.expected_loader_symbol_name,
        ),
        json_string_field(
            "expected_loader_symbol_section_id",
            &report.expected_loader_symbol_section_id,
        ),
        json_usize_field(
            "expected_relocation_count",
            report.expected_relocation_count,
        ),
        json_string_field(
            "expected_relocation_table_hash",
            &report.expected_relocation_table_hash,
        ),
        json_string_field("expected_relocation_id", &report.expected_relocation_id),
        json_string_field("expected_relocation_kind", &report.expected_relocation_kind),
        json_string_field(
            "expected_relocation_source_section_id",
            &report.expected_relocation_source_section_id,
        ),
        json_usize_field(
            "expected_relocation_source_offset",
            report.expected_relocation_source_offset,
        ),
        json_string_field(
            "expected_relocation_target_symbol_id",
            &report.expected_relocation_target_symbol_id,
        ),
        json_isize_field(
            "expected_relocation_addend",
            report.expected_relocation_addend,
        ),
        json_usize_field(
            "expected_external_import_count",
            report.expected_external_import_count,
        ),
        json_string_field(
            "expected_external_import_table_hash",
            &report.expected_external_import_table_hash,
        ),
        json_string_field(
            "expected_external_import_id",
            &report.expected_external_import_id,
        ),
        json_string_field(
            "expected_external_import_kind",
            &report.expected_external_import_kind,
        ),
        json_string_field(
            "expected_external_import_name",
            &report.expected_external_import_name,
        ),
        json_string_field(
            "expected_external_import_provider",
            &report.expected_external_import_provider,
        ),
        json_bool_field(
            "expected_external_import_required",
            report.expected_external_import_required,
        ),
        json_bool_field(
            "expected_native_object_section_present",
            report.expected_native_object_section_present,
        ),
        json_string_field(
            "expected_native_object_section_id",
            &report.expected_native_object_section_id,
        ),
        json_bool_field(
            "expected_native_object_loader_symbol_present",
            report.expected_native_object_loader_symbol_present,
        ),
        json_string_field(
            "expected_native_object_loader_symbol_id",
            &report.expected_native_object_loader_symbol_id,
        ),
        json_bool_field(
            "expected_native_object_relocation_present",
            report.expected_native_object_relocation_present,
        ),
        json_string_field(
            "expected_native_object_relocation_id",
            &report.expected_native_object_relocation_id,
        ),
        json_optional_string_field(
            "actual_container_layout_hash",
            report.actual_container_layout_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_container_hash",
            report.actual_container_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_metadata_table_hash",
            report.actual_metadata_table_hash.as_deref(),
        ),
        json_optional_usize_field(
            "actual_payload_size_bytes",
            report.actual_payload_size_bytes,
        ),
        json_optional_string_field("actual_payload_hash", report.actual_payload_hash.as_deref()),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_optional_string_field(
            "actual_container_section_table_hash",
            report.actual_container_section_table_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_readiness",
            report.actual_loader_readiness.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_entry_kind",
            report.actual_loader_entry_kind.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_entry_symbol",
            report.actual_loader_entry_symbol.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_entry_section_id",
            report.actual_loader_entry_section_id.as_deref(),
        ),
        json_optional_usize_field(
            "actual_loader_symbol_count",
            report.actual_loader_symbol_count,
        ),
        json_optional_string_field(
            "actual_loader_symbol_table_hash",
            report.actual_loader_symbol_table_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_symbol_id",
            report.actual_loader_symbol_id.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_symbol_kind",
            report.actual_loader_symbol_kind.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_symbol_name",
            report.actual_loader_symbol_name.as_deref(),
        ),
        json_optional_string_field(
            "actual_loader_symbol_section_id",
            report.actual_loader_symbol_section_id.as_deref(),
        ),
        json_optional_usize_field("actual_relocation_count", report.actual_relocation_count),
        json_optional_string_field(
            "actual_relocation_table_hash",
            report.actual_relocation_table_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_relocation_id",
            report.actual_relocation_id.as_deref(),
        ),
        json_optional_string_field(
            "actual_relocation_kind",
            report.actual_relocation_kind.as_deref(),
        ),
        json_optional_string_field(
            "actual_relocation_source_section_id",
            report.actual_relocation_source_section_id.as_deref(),
        ),
        json_optional_usize_field(
            "actual_relocation_source_offset",
            report.actual_relocation_source_offset,
        ),
        json_optional_string_field(
            "actual_relocation_target_symbol_id",
            report.actual_relocation_target_symbol_id.as_deref(),
        ),
        json_optional_isize_field("actual_relocation_addend", report.actual_relocation_addend),
        json_optional_usize_field(
            "actual_external_import_count",
            report.actual_external_import_count,
        ),
        json_optional_string_field(
            "actual_external_import_table_hash",
            report.actual_external_import_table_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_external_import_id",
            report.actual_external_import_id.as_deref(),
        ),
        json_optional_string_field(
            "actual_external_import_kind",
            report.actual_external_import_kind.as_deref(),
        ),
        json_optional_string_field(
            "actual_external_import_name",
            report.actual_external_import_name.as_deref(),
        ),
        json_optional_string_field(
            "actual_external_import_provider",
            report.actual_external_import_provider.as_deref(),
        ),
        json_optional_bool_field(
            "actual_external_import_required",
            report.actual_external_import_required,
        ),
        json_bool_field(
            "actual_native_object_section_present",
            report.actual_native_object_section_present,
        ),
        json_optional_string_field(
            "actual_native_object_section_id",
            report.actual_native_object_section_id.as_deref(),
        ),
        json_bool_field(
            "actual_native_object_loader_symbol_present",
            report.actual_native_object_loader_symbol_present,
        ),
        json_optional_string_field(
            "actual_native_object_loader_symbol_id",
            report.actual_native_object_loader_symbol_id.as_deref(),
        ),
        json_bool_field(
            "actual_native_object_relocation_present",
            report.actual_native_object_relocation_present,
        ),
        json_optional_string_field(
            "actual_native_object_relocation_id",
            report.actual_native_object_relocation_id.as_deref(),
        ),
        json_string_array_field("container_section_issues", &report.container_section_issues),
        json_string_array_field("section_range_issues", &report.section_range_issues),
        json_string_array_field("loader_symbol_issues", &report.loader_symbol_issues),
        json_string_array_field("relocation_issues", &report.relocation_issues),
        json_string_array_field("external_import_issues", &report.external_import_issues),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
