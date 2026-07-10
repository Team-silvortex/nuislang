use super::{json_fields::*, json_fragments::*, json_object_image::*, reports::*};

pub(crate) fn nsld_closure_report_json(report: &NsldClosureReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("closed", report.closed),
        json_string_array_field("internal_contracts", &report.internal_contracts),
        json_string_field("linker_contract_hash", &report.linker_contract_hash),
        format!(
            "\"link_inputs\":[{}]",
            nsld_link_inputs_json(&report.link_inputs)
        ),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_bool_field("link_input_table_present", report.link_input_table_present),
        json_optional_bool_field("link_input_table_valid", report.link_input_table_valid),
        json_bool_field(
            "prepared_artifact_chain_valid",
            report.prepared_artifact_chain_valid,
        ),
        json_string_array_field(
            "prepared_artifact_chain_issues",
            &report.prepared_artifact_chain_issues,
        ),
        json_string_field(
            "container_metadata_table_hash",
            &report.container_metadata_table_hash,
        ),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_field(
            "container_loader_readiness",
            &report.container_loader_readiness,
        ),
        json_usize_field(
            "compatibility_domain_count",
            report.compatibility_domain_count,
        ),
        json_string_field(
            "compatibility_domain_table_hash",
            &report.compatibility_domain_table_hash,
        ),
        json_optional_string_field(
            "compatibility_domain_id",
            report.compatibility_domain_id.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_kind",
            report.compatibility_domain_kind.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_paradigm",
            report.compatibility_domain_paradigm.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_lifecycle_hook",
            report.compatibility_domain_lifecycle_hook.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_abi_family",
            report.compatibility_domain_abi_family.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_wrapper_policy",
            report.compatibility_domain_wrapper_policy.as_deref(),
        ),
        json_optional_bool_field(
            "compatibility_domain_required",
            report.compatibility_domain_required,
        ),
        format!(
            "\"compatibility_domain_summary\":{}",
            compatibility_domain_summary_json(
                Some(report.compatibility_domain_count),
                Some(&report.compatibility_domain_table_hash),
                report.compatibility_domain_id.as_deref(),
                report.compatibility_domain_kind.as_deref(),
                report.compatibility_domain_paradigm.as_deref(),
                report.compatibility_domain_lifecycle_hook.as_deref(),
                report.compatibility_domain_abi_family.as_deref(),
                report.compatibility_domain_wrapper_policy.as_deref(),
                report.compatibility_domain_required,
            )
        ),
        json_optional_bool_field(
            "object_image_relocation_lowering_valid",
            report.object_image_relocation_lowering_valid,
        ),
        json_optional_usize_field(
            "object_image_relocation_lowering_rule_count",
            report.object_image_relocation_lowering_rule_count,
        ),
        format!(
            "\"object_image_relocation_lowering_rules\":[{}]",
            relocation_lowering_rules_json(&report.object_image_relocation_lowering_rules)
        ),
        json_string_array_field(
            "object_image_relocation_lowering_issues",
            &report.object_image_relocation_lowering_issues,
        ),
        json_optional_usize_field(
            "object_image_relocation_record_count",
            report.object_image_relocation_record_count,
        ),
        json_optional_string_field(
            "object_image_relocation_record_table_hash",
            report.object_image_relocation_record_table_hash.as_deref(),
        ),
        format!(
            "\"object_image_relocation_records\":[{}]",
            relocation_records_json(&report.object_image_relocation_records)
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

pub(crate) fn nsld_closure_emit_report_json(report: &NsldClosureEmitReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("linker_contract_hash", &report.linker_contract_hash),
        json_bool_field("closed", report.closed),
        json_usize_field("internal_contract_count", report.internal_contract_count),
        json_usize_field("unresolved_count", report.unresolved_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_closure_verify_report_json(report: &NsldClosureVerifyReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_linker_contract_hash",
            &report.expected_linker_contract_hash,
        ),
        json_optional_string_field(
            "actual_linker_contract_hash",
            report.actual_linker_contract_hash.as_deref(),
        ),
        json_string_field("expected_container_hash", &report.expected_container_hash),
        json_optional_string_field(
            "actual_container_hash",
            report.actual_container_hash.as_deref(),
        ),
        json_usize_field(
            "expected_payload_size_bytes",
            report.expected_payload_size_bytes,
        ),
        json_optional_usize_field(
            "actual_payload_size_bytes",
            report.actual_payload_size_bytes,
        ),
        json_string_field("expected_payload_hash", &report.expected_payload_hash),
        json_optional_string_field("actual_payload_hash", report.actual_payload_hash.as_deref()),
        json_bool_field("expected_closed", report.expected_closed),
        json_optional_bool_field("actual_closed", report.actual_closed),
        json_usize_field(
            "expected_internal_contract_count",
            report.expected_internal_contract_count,
        ),
        json_optional_usize_field(
            "actual_internal_contract_count",
            report.actual_internal_contract_count,
        ),
        json_usize_field(
            "expected_unresolved_count",
            report.expected_unresolved_count,
        ),
        json_optional_usize_field("actual_unresolved_count", report.actual_unresolved_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_unit_report_json(report: &NsldLinkUnitReport) -> String {
    let fields = [
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
