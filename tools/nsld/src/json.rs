pub(crate) use super::json_container::*;
pub(crate) use super::json_object::*;

use super::{json_fields::*, json_fragments::*, reports::*};

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
        json_bool_field("object_plan_present", report.object_plan_present),
        json_optional_bool_field("object_plan_valid", report.object_plan_valid),
        json_string_array_field("object_plan_issues", &report.object_plan_issues),
        json_bool_field("container_plan_present", report.container_plan_present),
        json_optional_bool_field("container_plan_valid", report.container_plan_valid),
        json_string_array_field("container_plan_issues", &report.container_plan_issues),
        json_bool_field("container_present", report.container_present),
        json_optional_bool_field("container_valid", report.container_valid),
        json_string_array_field("container_issues", &report.container_issues),
        json_string_array_field("container_section_issues", &report.container_section_issues),
        json_string_array_field(
            "container_loader_symbol_issues",
            &report.container_loader_symbol_issues,
        ),
        json_string_array_field(
            "container_relocation_issues",
            &report.container_relocation_issues,
        ),
        json_string_array_field(
            "container_external_import_issues",
            &report.container_external_import_issues,
        ),
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
        json_string_field("object_plan_path", &report.object_plan_path),
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
        json_string_field("object_plan_hash", &report.object_plan_hash),
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
        json_string_field(
            "container_metadata_table_hash",
            &report.container_metadata_table_hash,
        ),
        json_string_field(
            "container_loader_readiness",
            &report.container_loader_readiness,
        ),
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
