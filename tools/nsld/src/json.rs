use super::{
    container::{
        NsldContainerEmitReport, NsldContainerExternalImport, NsldContainerLoaderSymbol,
        NsldContainerPlanEmitReport, NsldContainerPlanReport, NsldContainerPlanVerifyReport,
        NsldContainerRelocationEntry, NsldContainerReport, NsldContainerSectionEntry,
        NsldContainerVerifyReport,
    },
    reports::*,
};

pub(crate) fn check_report_json(report: &NsldCheckReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_check"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_usize_field("checks", report.checks),
        json_usize_field("failures", report.failures),
        json_bool_field(
            "artifact_lowering_alignment_consistent",
            report.artifact_lowering_alignment_consistent,
        ),
        json_usize_field(
            "artifact_lowering_alignment_mismatches",
            report.artifact_lowering_alignment_mismatches,
        ),
        json_bool_field("clock_protocol_valid", report.clock_protocol_valid),
        json_string_array_field("clock_protocol_issues", &report.clock_protocol_issues),
        json_bool_field("hetero_calculate_valid", report.hetero_calculate_valid),
        json_string_array_field("hetero_calculate_issues", &report.hetero_calculate_issues),
        json_bool_field("static_link", report.static_link),
        json_bool_field("lifecycle_driven", report.lifecycle_driven),
        json_bool_field("sidecar_capability_valid", report.sidecar_capability_valid),
        json_string_array_field(
            "sidecar_capability_issues",
            &report.sidecar_capability_issues,
        ),
        json_bool_field("link_input_table_present", report.link_input_table_present),
        json_optional_bool_field("link_input_table_valid", report.link_input_table_valid),
        json_string_array_field("link_input_table_issues", &report.link_input_table_issues),
        json_bool_field("link_unit_table_present", report.link_unit_table_present),
        json_optional_bool_field("link_unit_table_valid", report.link_unit_table_valid),
        json_string_array_field("link_unit_table_issues", &report.link_unit_table_issues),
        json_bool_field("link_bundle_present", report.link_bundle_present),
        json_optional_bool_field("link_bundle_valid", report.link_bundle_valid),
        json_string_array_field("link_bundle_issues", &report.link_bundle_issues),
        json_bool_field("assemble_plan_present", report.assemble_plan_present),
        json_optional_bool_field("assemble_plan_valid", report.assemble_plan_valid),
        json_string_array_field("assemble_plan_issues", &report.assemble_plan_issues),
        json_bool_field("section_manifest_present", report.section_manifest_present),
        json_optional_bool_field("section_manifest_valid", report.section_manifest_valid),
        json_string_array_field("section_manifest_issues", &report.section_manifest_issues),
        json_bool_field("container_plan_present", report.container_plan_present),
        json_optional_bool_field("container_plan_valid", report.container_plan_valid),
        json_string_array_field("container_plan_issues", &report.container_plan_issues),
        json_bool_field("container_present", report.container_present),
        json_optional_bool_field("container_valid", report.container_valid),
        json_string_array_field("container_issues", &report.container_issues),
        json_bool_field(
            "container_payload_present",
            report.container_payload_present,
        ),
        json_string_array_field("container_payload_issues", &report.container_payload_issues),
        json_optional_string_field(
            "container_loader_readiness",
            report.container_loader_readiness.as_deref(),
        ),
        json_string_array_field(
            "container_loader_blockers",
            &report.container_loader_blockers,
        ),
        json_optional_string_field(
            "container_metadata_table_hash",
            report.container_metadata_table_hash.as_deref(),
        ),
        json_optional_usize_field(
            "container_external_import_count",
            report.container_external_import_count,
        ),
        json_bool_field("artifact_chain_valid", report.artifact_chain_valid),
        json_string_array_field("artifact_chain_issues", &report.artifact_chain_issues),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        format!("\"domains\":[{}]", domains_json(&report.domains)),
        format!(
            "\"sidecar_capabilities\":[{}]",
            sidecar_capabilities_json(&report.sidecar_capabilities)
        ),
        format!(
            "\"clock_edges\":[{}]",
            clock_edges_json(&report.clock_edges)
        ),
        format!(
            "\"data_segments\":[{}]",
            data_segments_json(&report.data_segments)
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_units_emit_report_json(report: &NsldLinkUnitsEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_units_verify_report_json(report: &NsldLinkUnitsVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field("expected_unit_count", report.expected_unit_count),
        json_usize_field(
            "expected_hetero_unit_count",
            report.expected_hetero_unit_count,
        ),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_string_field("expected_unit_table_hash", &report.expected_unit_table_hash),
        json_optional_usize_field("actual_unit_count", report.actual_unit_count),
        json_optional_usize_field("actual_hetero_unit_count", report.actual_hetero_unit_count),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_string_field(
            "actual_unit_table_hash",
            report.actual_unit_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_report_json(report: &NsldLinkBundleReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle"),
        json_string_field("manifest", &report.manifest),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("compiled_artifact_path", &report.compiled_artifact_path),
        json_string_field("native_output_path", &report.native_output_path),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_emit_report_json(report: &NsldLinkBundleEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_verify_report_json(report: &NsldLinkBundleVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_bundle_id", &report.expected_bundle_id),
        json_string_field("expected_bundle_hash", &report.expected_bundle_hash),
        json_optional_string_field("actual_bundle_id", report.actual_bundle_id.as_deref()),
        json_optional_string_field("actual_bundle_hash", report.actual_bundle_hash.as_deref()),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_prepare_report_json(report: &NsldPrepareReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_prepare"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("link_input_table_path", &report.link_input_table_path),
        json_string_field("link_unit_table_path", &report.link_unit_table_path),
        json_string_field("link_bundle_path", &report.link_bundle_path),
        json_string_field("assemble_plan_path", &report.assemble_plan_path),
        json_string_field("section_manifest_path", &report.section_manifest_path),
        json_string_field("container_plan_path", &report.container_plan_path),
        json_string_field("container_path", &report.container_path),
        json_string_field("container_payload_path", &report.container_payload_path),
        json_usize_field("link_input_count", report.link_input_count),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_usize_field("unit_count", report.unit_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_string_field("section_table_hash", &report.section_table_hash),
        json_string_field("metadata_table_hash", &report.metadata_table_hash),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_assemble_plan_report_json(report: &NsldAssemblePlanReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_assemble_plan_emit_report_json(report: &NsldAssemblePlanEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_assemble_plan_verify_report_json(
    report: &NsldAssemblePlanVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_assemble_plan_hash",
            &report.expected_assemble_plan_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_optional_string_field(
            "actual_assemble_plan_hash",
            report.actual_assemble_plan_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_section_manifest_report_json(report: &NsldSectionManifestReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_section_manifest_emit_report_json(
    report: &NsldSectionManifestEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_section_manifest_verify_report_json(
    report: &NsldSectionManifestVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_string_field(
            "expected_section_table_hash",
            &report.expected_section_table_hash,
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_optional_string_field(
            "actual_section_table_hash",
            report.actual_section_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

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
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_string_field("loader_readiness", &report.loader_readiness),
        json_string_array_field("loader_blockers", &report.loader_blockers),
        json_string_field("loader_entry_kind", &report.loader_entry_kind),
        json_string_field("loader_entry_symbol", &report.loader_entry_symbol),
        json_string_field("loader_entry_section_id", &report.loader_entry_section_id),
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
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_field("output_path", &report.output_path),
        json_string_field("payload_path", &report.payload_path),
        json_usize_field("section_count", report.section_count),
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
        json_string_array_field("section_range_issues", &report.section_range_issues),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_inputs_emit_report_json(report: &NsldLinkInputsEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_inputs_verify_report_json(report: &NsldLinkInputsVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_usize_field(
            "expected_link_input_total_bytes",
            report.expected_link_input_total_bytes,
        ),
        json_string_field(
            "expected_link_input_table_hash",
            &report.expected_link_input_table_hash,
        ),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_usize_field(
            "actual_link_input_total_bytes",
            report.actual_link_input_total_bytes,
        ),
        json_optional_string_field(
            "actual_link_input_table_hash",
            report.actual_link_input_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_closure_report_json(report: &NsldClosureReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("closed", report.closed),
        json_string_array_field("internal_contracts", &report.internal_contracts),
        format!(
            "\"link_inputs\":[{}]",
            nsld_link_inputs_json(&report.link_inputs)
        ),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_bool_field("link_input_table_present", report.link_input_table_present),
        json_optional_bool_field("link_input_table_valid", report.link_input_table_valid),
        json_string_array_field("external_dependencies", &report.external_dependencies),
        json_string_array_field("unresolved", &report.unresolved),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("domain_count", report.domain_count),
        json_usize_field("hetero_domain_count", report.hetero_domain_count),
        json_usize_field("sidecar_capability_count", report.sidecar_capability_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_unit_report_json(report: &NsldLinkUnitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units"),
        json_string_field("manifest", &report.manifest),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        format!("\"units\":[{}]", nsld_link_units_json(&report.units)),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_inputs_json(inputs: &[NsldLinkInputDiagnostic]) -> String {
    inputs
        .iter()
        .map(|input| {
            let fields = vec![
                json_usize_field("order_index", input.order_index),
                json_string_field("input_id", &input.input_id),
                json_string_field("input_kind", &input.input_kind),
                json_string_field("domain_family", &input.domain_family),
                json_string_field("package_id", &input.package_id),
                json_string_field("path", &input.path),
                json_string_field("native_ir", &input.native_ir),
                json_string_field("dispatch_lowering", &input.dispatch_lowering),
                json_usize_field("contract_count", input.contract_count),
                json_usize_field("content_bytes", input.content_bytes),
                json_string_field("content_hash", &input.content_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_link_units_json(units: &[NsldLinkUnitDiagnostic]) -> String {
    units
        .iter()
        .map(|unit| {
            let fields = vec![
                json_usize_field("order_index", unit.order_index),
                json_string_field("unit_id", &unit.unit_id),
                json_string_field("unit_kind", &unit.unit_kind),
                json_string_field("domain_family", &unit.domain_family),
                json_string_field("package_id", &unit.package_id),
                json_string_field("backend_family", &unit.backend_family),
                json_string_field("lowering_target", &unit.lowering_target),
                json_string_field("packaging_role", &unit.packaging_role),
                json_string_array_field("link_input_ids", &unit.link_input_ids),
                json_usize_field("clock_edge_count", unit.clock_edge_count),
                json_usize_field("data_segment_count", unit.data_segment_count),
                json_bool_field("requires_host_wrapper", unit.requires_host_wrapper),
                json_string_field("deterministic_order_key", &unit.deterministic_order_key),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_assemble_sections_json(sections: &[NsldAssembleSectionDiagnostic]) -> String {
    sections
        .iter()
        .map(|section| {
            let fields = vec![
                json_usize_field("order_index", section.order_index),
                json_string_field("section_id", &section.section_id),
                json_string_field("section_kind", &section.section_kind),
                json_string_field("source_path", &section.source_path),
                json_string_field("source_hash", &section.source_hash),
                json_bool_field("required", section.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_container_sections_json(sections: &[NsldContainerSectionEntry]) -> String {
    sections
        .iter()
        .map(|section| {
            let fields = vec![
                json_usize_field("order_index", section.order_index),
                json_string_field("section_id", &section.section_id),
                json_string_field("section_kind", &section.section_kind),
                json_string_field("source_path", &section.source_path),
                json_string_field("source_hash", &section.source_hash),
                json_string_field("payload_hash", &section.payload_hash),
                json_bool_field("required", section.required),
                json_usize_field("offset", section.offset),
                json_usize_field("size_bytes", section.size_bytes),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn domains_json(domains: &[NsldDomainDiagnostic]) -> String {
    domains
        .iter()
        .map(|domain| {
            let fields = vec![
                json_string_field("domain_family", &domain.domain_family),
                json_string_field("package_id", &domain.package_id),
                json_string_field("kind", &domain.kind),
                json_string_field("packaging_role", &domain.packaging_role),
                json_string_field("lowering_target", &domain.lowering_target),
                json_string_field("backend_family", &domain.backend_family),
                json_bool_field("alignment_consistent", domain.alignment_consistent),
                json_string_array_field("alignment_issues", &domain.alignment_issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn sidecar_capabilities_json(capabilities: &[NsldSidecarCapabilityDiagnostic]) -> String {
    capabilities
        .iter()
        .map(|capability| {
            let fields = vec![
                json_string_field("domain_family", &capability.domain_family),
                json_string_field("package_id", &capability.package_id),
                json_string_field("path", &capability.path),
                json_bool_field("valid", capability.valid),
                json_string_field("capability_owner", &capability.capability_owner),
                json_string_field("frontend_ir", &capability.frontend_ir),
                json_string_field("native_ir", &capability.native_ir),
                json_string_field("dispatch_lowering", &capability.dispatch_lowering),
                json_usize_field("content_bytes", capability.content_bytes),
                json_string_field("content_hash", &capability.content_hash),
                json_string_array_field("validation_contracts", &capability.validation_contracts),
                json_string_array_field("issues", &capability.issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn clock_edges_json(edges: &[NsldClockEdgeDiagnostic]) -> String {
    edges
        .iter()
        .map(|edge| {
            let fields = vec![
                json_usize_field("index", edge.index),
                json_string_field("from", &edge.from),
                json_string_field("to", &edge.to),
                json_string_field("relation", &edge.relation),
                json_string_field("source", &edge.source),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn data_segments_json(segments: &[NsldDataSegmentDiagnostic]) -> String {
    segments
        .iter()
        .map(|segment| {
            let fields = vec![
                json_usize_field("index", segment.index),
                json_string_field("segment_id", &segment.segment_id),
                json_string_field("domain_family", &segment.domain_family),
                json_string_field("owner_package", &segment.owner_package),
                json_string_field("order_key", &segment.order_key),
                json_string_field("access_phase", &segment.access_phase),
                json_string_field("source_path", &segment.source_path),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{name}\":{value}")
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", json_escape(value))
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{name}\":{value}")
}

fn json_isize_field(name: &str, value: isize) -> String {
    format!("\"{name}\":{value}")
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => json_string_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{name}\":[{body}]")
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
