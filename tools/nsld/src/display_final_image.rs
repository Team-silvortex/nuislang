use super::{display_text::*, reports::*};

pub(crate) fn print_nsld_final_executable_layout_plan_report(
    report: &NsldFinalExecutableLayoutPlanReport,
) {
    println!("Nsld final executable layout plan");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  layout_hash: {}", report.layout_hash);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!(
        "  platform_envelope_family: {}",
        report.platform_envelope_family
    );
    println!(
        "  platform_envelope_policy: {}",
        report.platform_envelope_policy
    );
    println!(
        "  internal_binary_format: {}",
        report.internal_binary_format
    );
    println!("  lifecycle_entry_hook: {}", report.lifecycle_entry_hook);
    println!("  scheduler_contract: {}", report.scheduler_contract);
    println!("  data_segment_ordering: {}", report.data_segment_ordering);
    println!("  native_object_path: {}", report.native_object_path);
    println!(
        "  native_object_required: {}",
        report.native_object_required
    );
    println!("  native_object_present: {}", report.native_object_present);
    println!("  compatibility_domain: {}", report.compatibility_domain);
    println!(
        "  compatibility_lifecycle_hook: {}",
        report.compatibility_lifecycle_hook
    );
    println!("  payload_count: {}", report.payload_count);
    println!("  byte_alignment: {}", report.byte_alignment);
    println!("  byte_span: {}", report.byte_span);
    println!("  byte_map_hash: {}", report.byte_map_hash);
    println!(
        "  relocation_application_strategy: {}",
        report.relocation_application_strategy
    );
    println!(
        "  relocation_application_table_source: {}",
        report.relocation_application_table_source
    );
    println!(
        "  relocation_application_count: {}",
        report.relocation_application_count
    );
    println!(
        "  relocation_application_table_hash: {}",
        report.relocation_application_table_hash
    );
    for payload in &report.payload_names {
        println!("  payload: {payload}");
    }
    for payload in &report.payloads {
        println!(
            "  payload_diagnostic: order={} id={} kind={} hook={} required={} present={} hash={} path={}",
            payload.order_index,
            payload.payload_id,
            payload.payload_kind,
            payload.lifecycle_hook,
            payload.required,
            payload.present,
            payload.content_hash,
            payload.path
        );
    }
    for entry in &report.byte_map_entries {
        println!(
            "  byte_map_entry: order={} payload={} kind={} offset={} size={} align={} hash={}",
            entry.order_index,
            entry.payload_id,
            entry.payload_kind,
            entry.offset,
            entry.size_bytes,
            entry.alignment,
            entry.content_hash
        );
    }
    for record in &report.relocation_applications {
        println!(
            "  relocation_application: order={} id={} kind={} payload={} section={} source_offset={} image_offset={} target={} addend={} status={}",
            record.order_index,
            record.relocation_id,
            record.relocation_kind,
            record.source_payload_id,
            record.source_section_id,
            record.source_offset,
            record.image_offset,
            record.target_symbol_id,
            record.addend,
            record.application_status
        );
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_layout_plan_emit_report(
    report: &NsldFinalExecutableLayoutPlanEmitReport,
) {
    println!("Nsld final executable layout plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  layout_hash: {}", report.layout_hash);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  payload_count: {}", report.payload_count);
    println!("  native_object_present: {}", report.native_object_present);
}

pub(crate) fn print_nsld_final_executable_layout_plan_verify_report(
    report: &NsldFinalExecutableLayoutPlanVerifyReport,
) {
    println!("Nsld final executable layout plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_layout_hash: {}", report.expected_layout_hash);
    println!(
        "  actual_layout_hash: {}",
        optional_string_text(report.actual_layout_hash.as_deref())
    );
    println!(
        "  expected_payload_count: {}",
        report.expected_payload_count
    );
    println!(
        "  actual_payload_count: {}",
        optional_usize_text(report.actual_payload_count)
    );
    for payload in &report.expected_payloads {
        println!("  expected_payload: {payload}");
    }
    for payload in &report.actual_payloads {
        println!("  actual_payload: {payload}");
    }
    println!(
        "  expected_payload_entry_count: {}",
        report.expected_payload_entry_count
    );
    println!(
        "  actual_payload_entry_count: {}",
        report.actual_payload_entry_count
    );
    println!(
        "  expected_byte_map_entry_count: {}",
        report.expected_byte_map_entry_count
    );
    println!(
        "  actual_byte_map_entry_count: {}",
        report.actual_byte_map_entry_count
    );
    println!("  expected_byte_span: {}", report.expected_byte_span);
    println!(
        "  actual_byte_span: {}",
        optional_usize_text(report.actual_byte_span)
    );
    println!(
        "  expected_byte_map_hash: {}",
        report.expected_byte_map_hash
    );
    println!(
        "  actual_byte_map_hash: {}",
        optional_string_text(report.actual_byte_map_hash.as_deref())
    );
    println!(
        "  expected_lifecycle_entry_hook: {}",
        report.expected_lifecycle_entry_hook
    );
    println!(
        "  actual_lifecycle_entry_hook: {}",
        optional_string_text(report.actual_lifecycle_entry_hook.as_deref())
    );
    println!(
        "  expected_platform_envelope_family: {}",
        report.expected_platform_envelope_family
    );
    println!(
        "  actual_platform_envelope_family: {}",
        optional_string_text(report.actual_platform_envelope_family.as_deref())
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_image_dry_run_report(
    report: &NsldFinalExecutableImageDryRunReport,
) {
    println!("Nsld final executable image dry run");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  image_format: {}", report.image_format);
    println!("  image_magic: {}", report.image_magic);
    println!("  image_header_size: {}", report.image_header_size);
    println!("  payload_byte_offset: {}", report.payload_byte_offset);
    println!("  payload_byte_span: {}", report.payload_byte_span);
    println!("  layout_hash: {}", report.layout_hash);
    println!("  byte_map_hash: {}", report.byte_map_hash);
    println!("  payload_count: {}", report.payload_count);
    println!("  byte_span: {}", report.byte_span);
    println!(
        "  scheduler_metadata_payload_id: {}",
        report.scheduler_metadata_payload_id
    );
    println!(
        "  scheduler_metadata_present: {}",
        report.scheduler_metadata_present
    );
    println!(
        "  scheduler_metadata_offset: {}",
        optional_usize_text(report.scheduler_metadata_offset)
    );
    println!(
        "  scheduler_metadata_hash: {}",
        optional_string_text(report.scheduler_metadata_hash.as_deref())
    );
    println!(
        "  backend_artifact_payload_count: {}",
        report.backend_artifact_payload_count
    );
    println!(
        "  backend_artifact_payload_present_count: {}",
        report.backend_artifact_payload_present_count
    );
    println!(
        "  backend_artifact_payload_role_status: {}",
        report.backend_artifact_payload_role_status
    );
    for payload_id in &report.backend_artifact_payload_ids {
        println!("  backend_artifact_payload_id: {payload_id}");
    }
    for payload_kind in &report.backend_artifact_payload_kinds {
        println!("  backend_artifact_payload_kind: {payload_kind}");
    }
    println!(
        "  backend_artifact_payload_first_missing: {}",
        optional_string_text(report.backend_artifact_payload_first_missing.as_deref())
    );
    println!(
        "  relocation_application_strategy: {}",
        report.relocation_application_strategy
    );
    println!(
        "  relocation_application_count: {}",
        report.relocation_application_count
    );
    println!(
        "  relocation_application_table_hash: {}",
        report.relocation_application_table_hash
    );
    println!(
        "  relocation_application_audit_status: {}",
        report.relocation_application_audit_status
    );
    println!(
        "  relocation_application_audit_count: {}",
        report.relocation_application_audit_count
    );
    println!(
        "  relocation_application_audit_table_hash: {}",
        report.relocation_application_audit_table_hash
    );
    for blocker in &report.relocation_application_audit_blockers {
        println!("  relocation_application_audit_blocker: {blocker}");
    }
    println!(
        "  relocation_patch_preview_status: {}",
        report.relocation_patch_preview_status
    );
    println!(
        "  relocation_patch_preview_count: {}",
        report.relocation_patch_preview_count
    );
    println!(
        "  relocation_patch_preview_table_hash: {}",
        report.relocation_patch_preview_table_hash
    );
    println!(
        "  relocation_patch_application_status: {}",
        report.relocation_patch_application_status
    );
    println!(
        "  relocation_patch_application_count: {}",
        report.relocation_patch_application_count
    );
    println!(
        "  relocation_patch_application_table_hash: {}",
        report.relocation_patch_application_table_hash
    );
    for blocker in &report.relocation_patch_application_blockers {
        println!("  relocation_patch_application_blocker: {blocker}");
    }
    for record in &report.relocation_patch_previews {
        println!(
            "  relocation_patch_preview: order={} relocation={} kind={} offset={} width={} resolved_value={} value_hash={} target={} target_image_offset={} status={} resolver={}",
            record.order_index,
            record.relocation_id,
            record.patch_kind,
            record.patch_offset,
            record.patch_width_bytes,
            record
                .resolved_patch_value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            record.patch_value_hash,
            record.target_symbol_id,
            record
                .target_symbol_image_offset
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            record.preview_status,
            record.resolver_status
        );
    }
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        optional_string_text(report.image_hash.as_deref())
    );
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_final_executable_image_dry_run_emit_report(
    report: &NsldFinalExecutableImageDryRunEmitReport,
) {
    println!("Nsld final executable image dry run emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  image_emitted: {}", report.image_emitted);
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!("  image_format: {}", report.image_format);
    println!("  image_header_size: {}", report.image_header_size);
    println!("  payload_byte_offset: {}", report.payload_byte_offset);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        optional_string_text(report.image_hash.as_deref())
    );
}

pub(crate) fn print_nsld_final_executable_image_dry_run_verify_report(
    report: &NsldFinalExecutableImageDryRunVerifyReport,
) {
    println!("Nsld final executable image dry run verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  image_path: {}", report.image_path);
    println!("  valid: {}", report.valid);
    println!("  expected_layout_hash: {}", report.expected_layout_hash);
    println!(
        "  actual_layout_hash: {}",
        optional_string_text(report.actual_layout_hash.as_deref())
    );
    println!(
        "  expected_byte_map_hash: {}",
        report.expected_byte_map_hash
    );
    println!(
        "  actual_byte_map_hash: {}",
        optional_string_text(report.actual_byte_map_hash.as_deref())
    );
    println!("  expected_image_magic: {}", report.expected_image_magic);
    println!(
        "  actual_image_magic: {}",
        optional_string_text(report.actual_image_magic.as_deref())
    );
    println!(
        "  expected_image_version: {}",
        report.expected_image_version
    );
    println!(
        "  actual_image_version: {}",
        report
            .actual_image_version
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  expected_image_header_size: {}",
        report.expected_image_header_size
    );
    println!(
        "  actual_image_header_size: {}",
        optional_usize_text(report.actual_image_header_size)
    );
    println!(
        "  expected_payload_byte_offset: {}",
        report.expected_payload_byte_offset
    );
    println!(
        "  actual_payload_byte_offset: {}",
        optional_usize_text(report.actual_payload_byte_offset)
    );
    println!(
        "  expected_payload_byte_span: {}",
        report.expected_payload_byte_span
    );
    println!(
        "  actual_payload_byte_span: {}",
        optional_usize_text(report.actual_payload_byte_span)
    );
    println!(
        "  actual_header_layout_hash: {}",
        optional_string_text(report.actual_header_layout_hash.as_deref())
    );
    println!(
        "  actual_header_byte_map_hash: {}",
        optional_string_text(report.actual_header_byte_map_hash.as_deref())
    );
    println!(
        "  expected_payload_region_count: {}",
        report.expected_payload_region_count
    );
    println!(
        "  actual_payload_region_count: {}",
        optional_usize_text(report.actual_payload_region_count)
    );
    println!(
        "  expected_payload_region_hash: {}",
        optional_string_text(report.expected_payload_region_hash.as_deref())
    );
    println!(
        "  actual_payload_region_hash: {}",
        optional_string_text(report.actual_payload_region_hash.as_deref())
    );
    println!(
        "  expected_scheduler_metadata_payload_id: {}",
        report.expected_scheduler_metadata_payload_id
    );
    println!(
        "  actual_scheduler_metadata_payload_id: {}",
        optional_string_text(report.actual_scheduler_metadata_payload_id.as_deref())
    );
    println!(
        "  expected_scheduler_metadata_present: {}",
        report.expected_scheduler_metadata_present
    );
    println!(
        "  actual_scheduler_metadata_present: {}",
        optional_bool_text(report.actual_scheduler_metadata_present)
    );
    println!(
        "  expected_scheduler_metadata_offset: {}",
        optional_usize_text(report.expected_scheduler_metadata_offset)
    );
    println!(
        "  actual_scheduler_metadata_offset: {}",
        optional_usize_text(report.actual_scheduler_metadata_offset)
    );
    println!(
        "  expected_scheduler_metadata_hash: {}",
        optional_string_text(report.expected_scheduler_metadata_hash.as_deref())
    );
    println!(
        "  actual_scheduler_metadata_hash: {}",
        optional_string_text(report.actual_scheduler_metadata_hash.as_deref())
    );
    println!(
        "  expected_image_constructed: {}",
        report.expected_image_constructed
    );
    println!(
        "  actual_image_constructed: {}",
        optional_bool_text(report.actual_image_constructed)
    );
    println!("  expected_image_ready: {}", report.expected_image_ready);
    println!(
        "  actual_image_ready: {}",
        optional_bool_text(report.actual_image_ready)
    );
    println!(
        "  expected_image_size_bytes: {}",
        optional_usize_text(report.expected_image_size_bytes)
    );
    println!(
        "  actual_image_size_bytes: {}",
        optional_usize_text(report.actual_image_size_bytes)
    );
    println!(
        "  expected_image_hash: {}",
        optional_string_text(report.expected_image_hash.as_deref())
    );
    println!(
        "  actual_image_hash: {}",
        optional_string_text(report.actual_image_hash.as_deref())
    );
    for blocker in &report.expected_blockers {
        println!("  expected_blocker: {blocker}");
    }
    for blocker in &report.actual_blockers {
        println!("  actual_blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
