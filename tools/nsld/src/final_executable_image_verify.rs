use super::{
    final_executable_image::{
        parse_final_executable_image_header, verify_final_executable_image_payload_region,
        FINAL_EXECUTABLE_IMAGE_VERSION,
    },
    final_executable_image_actual::read_final_executable_image_dry_run_actual,
    final_executable_image_relocation::relocation_patch_byte_audit,
    final_executable_image_stage::{
        nsld_final_executable_image_dry_run_report, patched_final_executable_image_bytes,
    },
    final_executable_layout_stage::nsld_final_executable_layout_plan_report,
    final_executable_paths::{
        nsld_final_executable_image_dry_run_bytes_path, nsld_final_executable_image_dry_run_path,
    },
    final_executable_render::{optional_usize_toml, render_final_executable_image_dry_run},
    final_executable_verify_helpers::push_optional_string_mismatch,
    fnv1a64_hex,
    reports::NsldFinalExecutableImageDryRunVerifyReport,
};
use std::{fs, path::Path};

pub(crate) fn nsld_verify_final_executable_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableImageDryRunVerifyReport {
    let expected = nsld_final_executable_image_dry_run_report(manifest, plan);
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let expected_source = render_final_executable_image_dry_run(&expected);
    let input_path = nsld_final_executable_image_dry_run_path(plan);
    let image_path = nsld_final_executable_image_dry_run_bytes_path(plan);
    let mut issues = Vec::new();
    let actual = read_final_executable_image_dry_run_actual(&input_path);
    if let Err(error) = actual.as_ref() {
        issues.push(error.clone());
    }
    let actual_layout_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.layout_hash.clone());
    let actual_byte_map_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.byte_map_hash.clone());
    let actual_image_header_size = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.image_header_size);
    let actual_payload_byte_offset = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.payload_byte_offset);
    let actual_image_constructed = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.image_constructed);
    let actual_image_ready = actual.as_ref().ok().and_then(|actual| actual.image_ready);
    let actual_image_size_bytes = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.image_size_bytes);
    let actual_image_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.image_hash.clone());
    let actual_scheduler_metadata_payload_id = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.scheduler_metadata_payload_id.clone());
    let actual_scheduler_metadata_present = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.scheduler_metadata_present);
    let actual_scheduler_metadata_offset = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.scheduler_metadata_offset);
    let actual_scheduler_metadata_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.scheduler_metadata_hash.clone());
    let actual_relocation_application_strategy = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_application_strategy.clone());
    let actual_relocation_application_count = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_application_count);
    let actual_relocation_application_table_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_application_table_hash.clone());
    let actual_relocation_application_audit_status = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_application_audit_status.clone());
    let actual_relocation_application_audit_count = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_application_audit_count);
    let actual_relocation_application_audit_table_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_application_audit_table_hash.clone());
    let actual_relocation_application_audit_blockers = actual
        .as_ref()
        .ok()
        .map(|actual| actual.relocation_application_audit_blockers.clone())
        .unwrap_or_default();
    let actual_relocation_patch_preview_status = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_preview_status.clone());
    let actual_relocation_patch_preview_count = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_preview_count);
    let actual_relocation_patch_preview_table_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_preview_table_hash.clone());
    let actual_relocation_patch_preview_entry_count = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_preview_entry_count);
    let actual_relocation_patch_preview_record_table_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_preview_record_table_hash.clone());
    let actual_relocation_patch_application_status = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_application_status.clone());
    let actual_relocation_patch_application_count = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_application_count);
    let actual_relocation_patch_application_table_hash = actual
        .as_ref()
        .ok()
        .and_then(|actual| actual.relocation_patch_application_table_hash.clone());
    let actual_relocation_patch_application_blockers = actual
        .as_ref()
        .ok()
        .map(|actual| actual.relocation_patch_application_blockers.clone())
        .unwrap_or_default();
    let actual_blockers = actual
        .as_ref()
        .ok()
        .map(|actual| actual.blockers.clone())
        .unwrap_or_default();
    let actual_relocation_patch_byte_audit = actual.as_ref().ok().map(|actual| {
        relocation_patch_byte_audit(&image_path, &actual.relocation_patch_preview_records)
    });
    if let Ok(actual) = actual.as_ref() {
        if actual.source != expected_source {
            issues.push("final-executable-image-dry-run-content-mismatch".to_owned());
        }
        push_optional_string_mismatch(
            &mut issues,
            "layout_hash",
            Some(expected.layout_hash.as_str()),
            actual_layout_hash.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "byte_map_hash",
            Some(expected.byte_map_hash.as_str()),
            actual_byte_map_hash.as_deref(),
        );
        if actual_image_header_size != Some(expected.image_header_size) {
            issues.push(format!(
                "image_header_size mismatch: expected {}, found {}",
                expected.image_header_size,
                actual_image_header_size
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_payload_byte_offset != Some(expected.payload_byte_offset) {
            issues.push(format!(
                "payload_byte_offset mismatch: expected {}, found {}",
                expected.payload_byte_offset,
                actual_payload_byte_offset
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_constructed != Some(expected.image_constructed) {
            issues.push(format!(
                "image_constructed mismatch: expected {}, found {}",
                expected.image_constructed,
                actual_image_constructed
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_ready != Some(expected.image_ready) {
            issues.push(format!(
                "image_ready mismatch: expected {}, found {}",
                expected.image_ready,
                actual_image_ready
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_size_bytes != expected.image_size_bytes {
            issues.push(format!(
                "image_size_bytes mismatch: expected {}, found {}",
                optional_usize_toml(expected.image_size_bytes),
                optional_usize_toml(actual_image_size_bytes)
            ));
        }
        if actual_image_hash != expected.image_hash {
            issues.push(format!(
                "image_hash mismatch: expected {}, found {}",
                expected
                    .image_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_image_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "scheduler_metadata_payload_id",
            Some(expected.scheduler_metadata_payload_id.as_str()),
            actual_scheduler_metadata_payload_id.as_deref(),
        );
        if actual_scheduler_metadata_present != Some(expected.scheduler_metadata_present) {
            issues.push(format!(
                "scheduler_metadata_present mismatch: expected {}, found {}",
                expected.scheduler_metadata_present,
                actual_scheduler_metadata_present
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_scheduler_metadata_offset != expected.scheduler_metadata_offset {
            issues.push(format!(
                "scheduler_metadata_offset mismatch: expected {}, found {}",
                optional_usize_toml(expected.scheduler_metadata_offset),
                optional_usize_toml(actual_scheduler_metadata_offset)
            ));
        }
        if actual_scheduler_metadata_hash != expected.scheduler_metadata_hash {
            issues.push(format!(
                "scheduler_metadata_hash mismatch: expected {}, found {}",
                expected
                    .scheduler_metadata_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_scheduler_metadata_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "relocation_application_strategy",
            Some(expected.relocation_application_strategy.as_str()),
            actual_relocation_application_strategy.as_deref(),
        );
        if actual_relocation_application_count != Some(expected.relocation_application_count) {
            issues.push(format!(
                "relocation_application_count mismatch: expected {}, found {}",
                expected.relocation_application_count,
                actual_relocation_application_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "relocation_application_table_hash",
            Some(expected.relocation_application_table_hash.as_str()),
            actual_relocation_application_table_hash.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "relocation_application_audit_status",
            Some(expected.relocation_application_audit_status.as_str()),
            actual_relocation_application_audit_status.as_deref(),
        );
        if actual_relocation_application_audit_count
            != Some(expected.relocation_application_audit_count)
        {
            issues.push(format!(
                "relocation_application_audit_count mismatch: expected {}, found {}",
                expected.relocation_application_audit_count,
                actual_relocation_application_audit_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "relocation_application_audit_table_hash",
            Some(expected.relocation_application_audit_table_hash.as_str()),
            actual_relocation_application_audit_table_hash.as_deref(),
        );
        if actual_relocation_application_audit_blockers
            != expected.relocation_application_audit_blockers
        {
            issues.push(format!(
                "relocation_application_audit_blockers mismatch: expected [{}], found [{}]",
                expected.relocation_application_audit_blockers.join(", "),
                actual_relocation_application_audit_blockers.join(", ")
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "relocation_patch_preview_status",
            Some(expected.relocation_patch_preview_status.as_str()),
            actual_relocation_patch_preview_status.as_deref(),
        );
        if actual_relocation_patch_preview_count != Some(expected.relocation_patch_preview_count) {
            issues.push(format!(
                "relocation_patch_preview_count mismatch: expected {}, found {}",
                expected.relocation_patch_preview_count,
                actual_relocation_patch_preview_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "relocation_patch_preview_table_hash",
            Some(expected.relocation_patch_preview_table_hash.as_str()),
            actual_relocation_patch_preview_table_hash.as_deref(),
        );
        if actual_relocation_patch_preview_entry_count
            != Some(expected.relocation_patch_previews.len())
        {
            issues.push(format!(
                "relocation_patch_preview_entry_count mismatch: expected {}, found {}",
                expected.relocation_patch_previews.len(),
                actual_relocation_patch_preview_entry_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "relocation_patch_preview_record_table_hash",
            Some(expected.relocation_patch_preview_table_hash.as_str()),
            actual_relocation_patch_preview_record_table_hash.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "relocation_patch_application_status",
            Some(expected.relocation_patch_application_status.as_str()),
            actual_relocation_patch_application_status.as_deref(),
        );
        if actual_relocation_patch_application_count
            != Some(expected.relocation_patch_application_count)
        {
            issues.push(format!(
                "relocation_patch_application_count mismatch: expected {}, found {}",
                expected.relocation_patch_application_count,
                actual_relocation_patch_application_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        push_optional_string_mismatch(
            &mut issues,
            "relocation_patch_application_table_hash",
            Some(expected.relocation_patch_application_table_hash.as_str()),
            actual_relocation_patch_application_table_hash.as_deref(),
        );
        if actual_relocation_patch_application_blockers
            != expected.relocation_patch_application_blockers
        {
            issues.push(format!(
                "relocation_patch_application_blockers mismatch: expected [{}], found [{}]",
                expected.relocation_patch_application_blockers.join(", "),
                actual_relocation_patch_application_blockers.join(", ")
            ));
        }
        if let Some(audit) = actual_relocation_patch_byte_audit.as_ref() {
            if audit.status != "verified" {
                issues.push(format!(
                    "relocation_patch_byte_audit_status mismatch: expected verified, found {}",
                    audit.status
                ));
            }
            if audit.count != expected.relocation_patch_application_count {
                issues.push(format!(
                    "relocation_patch_byte_audit_count mismatch: expected {}, found {}",
                    expected.relocation_patch_application_count, audit.count
                ));
            }
            if !audit.blockers.is_empty() {
                issues.push(format!(
                    "relocation_patch_byte_audit_blockers mismatch: expected [], found [{}]",
                    audit.blockers.join(", ")
                ));
            }
        }
        if actual_blockers != expected.blockers {
            issues.push(format!(
                "blockers mismatch: expected [{}], found [{}]",
                expected.blockers.join(", "),
                actual_blockers.join(", ")
            ));
        }
    }
    if let Some(expected_hash) = expected.image_hash.as_deref() {
        match fs::read(&image_path) {
            Ok(bytes) => {
                let actual_hash = fnv1a64_hex(&bytes);
                if actual_hash != expected_hash {
                    issues.push(format!(
                        "image_bytes_hash mismatch: expected {expected_hash}, found {actual_hash}"
                    ));
                }
            }
            Err(error) => issues.push(format!(
                "missing_or_unreadable_final_executable_image_dry_run_bytes `{}`: {error}",
                image_path.display()
            )),
        }
    }
    let (
        actual_image_magic,
        actual_image_version,
        actual_header_image_size,
        actual_header_payload_offset,
        actual_header_payload_span,
        actual_header_layout_hash,
        actual_header_byte_map_hash,
    ) = match fs::read(&image_path) {
        Ok(bytes) => match parse_final_executable_image_header(&bytes) {
            Some(header) => {
                if header.magic != expected.image_magic {
                    issues.push(format!(
                        "image_header_magic mismatch: expected {}, found {}",
                        expected.image_magic, header.magic
                    ));
                }
                if header.version != FINAL_EXECUTABLE_IMAGE_VERSION {
                    issues.push(format!(
                        "image_header_version mismatch: expected {}, found {}",
                        FINAL_EXECUTABLE_IMAGE_VERSION, header.version
                    ));
                }
                if header.header_size != expected.image_header_size {
                    issues.push(format!(
                        "image_header_size_bytes mismatch: expected {}, found {}",
                        expected.image_header_size, header.header_size
                    ));
                }
                if header.payload_offset != expected.payload_byte_offset {
                    issues.push(format!(
                        "image_header_payload_offset mismatch: expected {}, found {}",
                        expected.payload_byte_offset, header.payload_offset
                    ));
                }
                if header.payload_span != expected.payload_byte_span {
                    issues.push(format!(
                        "image_header_payload_span mismatch: expected {}, found {}",
                        expected.payload_byte_span, header.payload_span
                    ));
                }
                if header.layout_hash != expected.layout_hash {
                    issues.push(format!(
                        "image_header_layout_hash mismatch: expected {}, found {}",
                        expected.layout_hash, header.layout_hash
                    ));
                }
                if header.byte_map_hash != expected.byte_map_hash {
                    issues.push(format!(
                        "image_header_byte_map_hash mismatch: expected {}, found {}",
                        expected.byte_map_hash, header.byte_map_hash
                    ));
                }
                (
                    Some(header.magic),
                    Some(header.version),
                    Some(header.header_size),
                    Some(header.payload_offset),
                    Some(header.payload_span),
                    Some(header.layout_hash),
                    Some(header.byte_map_hash),
                )
            }
            None => {
                issues.push("final-executable-image-header:invalid-or-too-short".to_owned());
                (None, None, None, None, None, None, None)
            }
        },
        Err(_) => (None, None, None, None, None, None, None),
    };
    let expected_image_bytes = patched_final_executable_image_bytes(manifest, plan);
    let payload_region = verify_final_executable_image_payload_region(
        &layout,
        &image_path,
        expected_image_bytes.as_deref(),
        &mut issues,
    );

    NsldFinalExecutableImageDryRunVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        image_path: image_path.display().to_string(),
        valid: issues.is_empty(),
        expected_layout_hash: expected.layout_hash,
        actual_layout_hash,
        expected_byte_map_hash: expected.byte_map_hash,
        actual_byte_map_hash,
        expected_image_magic: expected.image_magic,
        actual_image_magic,
        expected_image_version: FINAL_EXECUTABLE_IMAGE_VERSION,
        actual_image_version,
        expected_image_header_size: expected.image_header_size,
        actual_image_header_size: actual_header_image_size.or(actual_image_header_size),
        expected_payload_byte_offset: expected.payload_byte_offset,
        actual_payload_byte_offset: actual_header_payload_offset.or(actual_payload_byte_offset),
        expected_payload_byte_span: expected.payload_byte_span,
        actual_payload_byte_span: actual_header_payload_span,
        actual_header_layout_hash,
        actual_header_byte_map_hash,
        expected_payload_region_count: layout.byte_map_entries.len(),
        actual_payload_region_count: payload_region.actual_count,
        expected_payload_region_hash: payload_region.expected_hash,
        actual_payload_region_hash: payload_region.actual_hash,
        expected_scheduler_metadata_payload_id: expected.scheduler_metadata_payload_id,
        actual_scheduler_metadata_payload_id,
        expected_scheduler_metadata_present: expected.scheduler_metadata_present,
        actual_scheduler_metadata_present,
        expected_scheduler_metadata_offset: expected.scheduler_metadata_offset,
        actual_scheduler_metadata_offset,
        expected_scheduler_metadata_hash: expected.scheduler_metadata_hash,
        actual_scheduler_metadata_hash,
        expected_relocation_application_strategy: expected.relocation_application_strategy,
        actual_relocation_application_strategy,
        expected_relocation_application_count: expected.relocation_application_count,
        actual_relocation_application_count,
        expected_relocation_application_table_hash: expected.relocation_application_table_hash,
        actual_relocation_application_table_hash,
        expected_relocation_application_audit_status: expected.relocation_application_audit_status,
        actual_relocation_application_audit_status,
        expected_relocation_application_audit_count: expected.relocation_application_audit_count,
        actual_relocation_application_audit_count,
        expected_relocation_application_audit_table_hash: expected
            .relocation_application_audit_table_hash,
        actual_relocation_application_audit_table_hash,
        expected_relocation_application_audit_blockers: expected
            .relocation_application_audit_blockers,
        actual_relocation_application_audit_blockers,
        expected_relocation_patch_preview_status: expected.relocation_patch_preview_status,
        actual_relocation_patch_preview_status,
        expected_relocation_patch_preview_count: expected.relocation_patch_preview_count,
        actual_relocation_patch_preview_count,
        expected_relocation_patch_preview_table_hash: expected.relocation_patch_preview_table_hash,
        actual_relocation_patch_preview_table_hash,
        expected_relocation_patch_preview_entry_count: expected.relocation_patch_previews.len(),
        actual_relocation_patch_preview_entry_count,
        actual_relocation_patch_preview_record_table_hash,
        expected_relocation_patch_application_status: expected.relocation_patch_application_status,
        actual_relocation_patch_application_status,
        expected_relocation_patch_application_count: expected.relocation_patch_application_count,
        actual_relocation_patch_application_count,
        expected_relocation_patch_application_table_hash: expected
            .relocation_patch_application_table_hash,
        actual_relocation_patch_application_table_hash,
        expected_relocation_patch_application_blockers: expected
            .relocation_patch_application_blockers,
        actual_relocation_patch_application_blockers,
        expected_relocation_patch_byte_audit_status: "verified".to_owned(),
        actual_relocation_patch_byte_audit_status: actual_relocation_patch_byte_audit
            .as_ref()
            .map(|audit| audit.status.clone()),
        expected_relocation_patch_byte_audit_count: expected.relocation_patch_application_count,
        actual_relocation_patch_byte_audit_count: actual_relocation_patch_byte_audit
            .as_ref()
            .map(|audit| audit.count),
        actual_relocation_patch_byte_audit_hash: actual_relocation_patch_byte_audit
            .as_ref()
            .map(|audit| audit.table_hash.clone()),
        actual_relocation_patch_byte_audit_blockers: actual_relocation_patch_byte_audit
            .map(|audit| audit.blockers)
            .unwrap_or_default(),
        expected_image_constructed: expected.image_constructed,
        actual_image_constructed,
        expected_image_ready: expected.image_ready,
        actual_image_ready,
        expected_image_size_bytes: expected.image_size_bytes,
        actual_image_size_bytes,
        expected_image_hash: expected.image_hash,
        actual_image_hash,
        expected_blockers: expected.blockers,
        actual_blockers,
        issues,
    }
}
