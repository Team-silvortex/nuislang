use super::{json_fields::*, json_fragments::*, reports::*};

pub(crate) fn nsld_object_plan_report_json(report: &NsldObjectPlanReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("target_arch", &report.target_arch),
        json_string_field("target_os", &report.target_os),
        json_string_field("object_format", &report.object_format),
        json_string_field("calling_abi", &report.calling_abi),
        json_string_field("clang_target", &report.clang_target),
        json_string_field("output_path", &report.output_path),
        json_string_field("source_container_path", &report.source_container_path),
        json_string_field("source_payload_path", &report.source_payload_path),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_string_field("object_layout_hash", &report.object_layout_hash),
        json_usize_field("relocation_seed_count", report.relocation_seed_count),
        json_string_field(
            "relocation_seed_table_hash",
            &report.relocation_seed_table_hash,
        ),
        json_string_field("writer_target_id", &report.writer_target_id),
        json_string_field("writer_backend_kind", &report.writer_backend_kind),
        json_string_field("writer_status", &report.writer_status),
        json_string_field("object_family", &report.object_family),
        json_string_array_field("unsupported_features", &report.unsupported_features),
        json_string_field("emission_status", &report.emission_status),
        format!(
            "\"object_sections\":[{}]",
            nsld_object_sections_json(&report.object_sections)
        ),
        format!(
            "\"relocation_seeds\":[{}]",
            nsld_object_relocation_seeds_json(&report.relocation_seeds)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_plan_emit_report_json(report: &NsldObjectPlanEmitReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_usize_field("section_count", report.section_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_plan_verify_report_json(report: &NsldObjectPlanVerifyReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_object_plan_hash",
            &report.expected_object_plan_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_optional_string_field(
            "actual_object_plan_hash",
            report.actual_object_plan_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_writer_readiness_report_json(
    report: &NsldObjectWriterReadinessReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_writer_readiness"),
        json_string_field("manifest", &report.manifest),
        json_string_field("writer_target_id", &report.writer_target_id),
        json_string_field("writer_status", &report.writer_status),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_usize_field("section_count", report.section_count),
        json_bool_field("can_emit_object", report.can_emit_object),
        format!(
            "\"writer_stages\":[{}]",
            nsld_object_writer_stages_json(&report.writer_stages)
        ),
        json_string_array_field("unsupported_features", &report.unsupported_features),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_object_writer_stages_json(stages: &[NsldObjectWriterStageDiagnostic]) -> String {
    stages
        .iter()
        .map(|stage| {
            let fields = [
                json_string_field("stage_id", &stage.stage_id),
                json_string_field("status", &stage.status),
                json_bool_field("required", stage.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_object_writer_input_verify_report_json(
    report: &NsldObjectWriterInputVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_writer_input_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_object_plan_hash",
            &report.expected_object_plan_hash,
        ),
        json_string_field(
            "expected_object_layout_hash",
            &report.expected_object_layout_hash,
        ),
        json_string_field(
            "expected_relocation_seed_table_hash",
            &report.expected_relocation_seed_table_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_usize_field(
            "expected_relocation_seed_count",
            report.expected_relocation_seed_count,
        ),
        json_optional_string_field(
            "actual_object_plan_hash",
            report.actual_object_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_object_layout_hash",
            report.actual_object_layout_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_relocation_seed_table_hash",
            report.actual_relocation_seed_table_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_optional_usize_field(
            "actual_relocation_seed_count",
            report.actual_relocation_seed_count,
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_writer_dry_run_report_json(
    report: &NsldObjectWriterDryRunReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_writer_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_string_field("planned_output_path", &report.planned_output_path),
        json_string_field("writer_target_id", &report.writer_target_id),
        json_string_field("writer_backend_kind", &report.writer_backend_kind),
        json_string_field("object_family", &report.object_family),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_string_field("object_layout_hash", &report.object_layout_hash),
        json_string_field(
            "relocation_seed_table_hash",
            &report.relocation_seed_table_hash,
        ),
        json_usize_field("section_count", report.section_count),
        json_usize_field("relocation_seed_count", report.relocation_seed_count),
        json_bool_field("writer_input_valid", report.writer_input_valid),
        json_bool_field("can_emit_object", report.can_emit_object),
        json_bool_field("dry_run_ready", report.dry_run_ready),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_writer_dry_run_emit_report_json(
    report: &NsldObjectWriterDryRunEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_writer_dry_run_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("dry_run_ready", report.dry_run_ready),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_usize_field("section_count", report.section_count),
        json_usize_field("relocation_seed_count", report.relocation_seed_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_writer_dry_run_verify_report_json(
    report: &NsldObjectWriterDryRunVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_writer_dry_run_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_object_plan_hash",
            &report.expected_object_plan_hash,
        ),
        json_string_field(
            "expected_writer_backend_kind",
            &report.expected_writer_backend_kind,
        ),
        json_string_field("expected_object_family", &report.expected_object_family),
        json_string_field(
            "expected_object_layout_hash",
            &report.expected_object_layout_hash,
        ),
        json_string_field(
            "expected_relocation_seed_table_hash",
            &report.expected_relocation_seed_table_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_usize_field(
            "expected_relocation_seed_count",
            report.expected_relocation_seed_count,
        ),
        json_bool_field("expected_dry_run_ready", report.expected_dry_run_ready),
        json_optional_string_field(
            "actual_object_plan_hash",
            report.actual_object_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_writer_backend_kind",
            report.actual_writer_backend_kind.as_deref(),
        ),
        json_optional_string_field(
            "actual_object_family",
            report.actual_object_family.as_deref(),
        ),
        json_optional_string_field(
            "actual_object_layout_hash",
            report.actual_object_layout_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_relocation_seed_table_hash",
            report.actual_relocation_seed_table_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_optional_usize_field(
            "actual_relocation_seed_count",
            report.actual_relocation_seed_count,
        ),
        json_optional_bool_field("actual_dry_run_ready", report.actual_dry_run_ready),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_byte_layout_report_json(report: &NsldObjectByteLayoutReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_byte_layout"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_target_id", &report.writer_target_id),
        json_string_field("writer_backend_kind", &report.writer_backend_kind),
        json_string_field("object_family", &report.object_family),
        json_string_field("object_format", &report.object_format),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_string_field("object_layout_hash", &report.object_layout_hash),
        json_string_field("byte_layout_hash", &report.byte_layout_hash),
        json_usize_field("section_count", report.section_count),
        json_usize_field("total_size_bytes", report.total_size_bytes),
        json_bool_field("layout_ready", report.layout_ready),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_byte_layout_emit_report_json(
    report: &NsldObjectByteLayoutEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_byte_layout_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("layout_ready", report.layout_ready),
        json_string_field("byte_layout_hash", &report.byte_layout_hash),
        json_usize_field("section_count", report.section_count),
        json_usize_field("total_size_bytes", report.total_size_bytes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_byte_layout_verify_report_json(
    report: &NsldObjectByteLayoutVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_byte_layout_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_byte_layout_hash",
            &report.expected_byte_layout_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_usize_field(
            "expected_total_size_bytes",
            report.expected_total_size_bytes,
        ),
        json_optional_string_field(
            "actual_byte_layout_hash",
            report.actual_byte_layout_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_optional_usize_field("actual_total_size_bytes", report.actual_total_size_bytes),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_file_layout_report_json(report: &NsldObjectFileLayoutReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_file_layout"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_target_id", &report.writer_target_id),
        json_string_field("writer_backend_kind", &report.writer_backend_kind),
        json_string_field("object_family", &report.object_family),
        json_string_field("object_format", &report.object_format),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_string_field("byte_layout_hash", &report.byte_layout_hash),
        json_string_field("file_layout_hash", &report.file_layout_hash),
        json_usize_field("record_count", report.record_count),
        json_usize_field("total_file_size_bytes", report.total_file_size_bytes),
        json_bool_field("layout_ready", report.layout_ready),
        format!(
            "\"records\":[{}]",
            nsld_object_file_layout_records_json(&report.records)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_file_layout_emit_report_json(
    report: &NsldObjectFileLayoutEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_file_layout_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("layout_ready", report.layout_ready),
        json_string_field("file_layout_hash", &report.file_layout_hash),
        json_usize_field("record_count", report.record_count),
        json_usize_field("total_file_size_bytes", report.total_file_size_bytes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_file_layout_verify_report_json(
    report: &NsldObjectFileLayoutVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_file_layout_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_file_layout_hash",
            &report.expected_file_layout_hash,
        ),
        json_usize_field("expected_record_count", report.expected_record_count),
        json_usize_field(
            "expected_total_file_size_bytes",
            report.expected_total_file_size_bytes,
        ),
        json_optional_string_field(
            "actual_file_layout_hash",
            report.actual_file_layout_hash.as_deref(),
        ),
        json_optional_usize_field("actual_record_count", report.actual_record_count),
        json_optional_usize_field(
            "actual_total_file_size_bytes",
            report.actual_total_file_size_bytes,
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
