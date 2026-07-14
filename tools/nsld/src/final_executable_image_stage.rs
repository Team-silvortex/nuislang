use super::{
    final_executable_image::{
        encode_final_executable_image, parse_final_executable_image_header,
        verify_final_executable_image_payload_region, FINAL_EXECUTABLE_IMAGE_FORMAT,
        FINAL_EXECUTABLE_IMAGE_HEADER_SIZE, FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT,
        FINAL_EXECUTABLE_IMAGE_VERSION,
    },
    final_executable_layout_stage::nsld_final_executable_layout_plan_report,
    final_executable_paths::{
        nsld_final_executable_image_dry_run_bytes_path, nsld_final_executable_image_dry_run_path,
    },
    final_executable_render::{optional_usize_toml, render_final_executable_image_dry_run},
    final_executable_verify_helpers::{
        non_empty_toml_string, optional_usize_value, push_optional_string_mismatch,
    },
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableImageDryRunEmitReport, NsldFinalExecutableImageDryRunReport,
        NsldFinalExecutableImageDryRunVerifyReport, NsldFinalExecutableRelocationApplicationRecord,
        NsldFinalExecutableRelocationPatchPreviewRecord,
    },
    toml,
};
use std::{fs, path::Path};

pub(crate) fn nsld_final_executable_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableImageDryRunReport {
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let image = encode_final_executable_image(&layout);
    let mut blockers = Vec::new();
    for payload in &layout.payloads {
        if payload.required && !payload.present {
            blockers.push(format!(
                "missing-final-executable-payload:{}",
                payload.payload_id
            ));
        }
    }
    if layout.byte_map_entries.len() != layout.payloads.len() {
        blockers.push("final-executable-byte-map:payload-count-mismatch".to_owned());
    }
    let relocation_audit = relocation_application_audit(
        &layout.relocation_applications,
        FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        layout.byte_span,
    );
    blockers.extend(
        relocation_audit
            .blockers
            .iter()
            .map(|blocker| format!("relocation-application-audit:{blocker}")),
    );
    let patch_preview = relocation_patch_preview(&layout.relocation_applications);
    let image_constructed = image.is_some();
    let image_ready = image_constructed && blockers.is_empty();
    let image_size_bytes = image.as_ref().map(Vec::len);
    let image_hash = image.as_ref().map(|bytes| fnv1a64_hex(bytes));
    let scheduler_metadata_payload_id = layout.scheduler_metadata_payload.clone();
    let scheduler_metadata_payload = layout
        .payloads
        .iter()
        .find(|payload| payload.payload_id == scheduler_metadata_payload_id);
    let scheduler_metadata_offset = layout
        .byte_map_entries
        .iter()
        .find(|entry| entry.payload_id == scheduler_metadata_payload_id)
        .map(|entry| entry.offset);
    let scheduler_metadata_present = scheduler_metadata_payload
        .map(|payload| payload.present)
        .unwrap_or(false);
    let scheduler_metadata_hash =
        scheduler_metadata_payload.map(|payload| payload.content_hash.clone());

    NsldFinalExecutableImageDryRunReport {
        manifest: manifest.display().to_string(),
        output_path: nsld_final_executable_image_dry_run_path(plan)
            .display()
            .to_string(),
        image_path: nsld_final_executable_image_dry_run_bytes_path(plan)
            .display()
            .to_string(),
        image_format: FINAL_EXECUTABLE_IMAGE_FORMAT.to_owned(),
        image_magic: FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT.to_owned(),
        image_header_size: FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        payload_byte_offset: FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        payload_byte_span: layout.byte_span,
        layout_hash: layout.layout_hash,
        byte_map_hash: layout.byte_map_hash,
        payload_count: layout.payload_count,
        byte_span: layout.byte_span,
        scheduler_metadata_payload_id,
        scheduler_metadata_present,
        scheduler_metadata_offset,
        scheduler_metadata_hash,
        relocation_application_strategy: layout.relocation_application_strategy,
        relocation_application_count: layout.relocation_application_count,
        relocation_application_table_hash: layout.relocation_application_table_hash,
        relocation_application_audit_status: relocation_audit.status,
        relocation_application_audit_count: relocation_audit.count,
        relocation_application_audit_table_hash: relocation_audit.table_hash,
        relocation_application_audit_blockers: relocation_audit.blockers,
        relocation_patch_preview_status: patch_preview.status,
        relocation_patch_preview_count: patch_preview.count,
        relocation_patch_preview_table_hash: patch_preview.table_hash,
        relocation_patch_previews: patch_preview.records,
        image_constructed,
        image_ready,
        image_size_bytes,
        image_hash,
        blockers,
    }
}

pub(crate) fn nsld_emit_final_executable_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableImageDryRunEmitReport, String> {
    let report = nsld_final_executable_image_dry_run_report(manifest, plan);
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let image = encode_final_executable_image(&layout);
    let image_emitted = match image {
        Some(bytes) => {
            fs::write(&report.image_path, bytes).map_err(|error| {
                format!(
                    "failed to write nsld final executable image dry-run bytes `{}`: {error}",
                    report.image_path
                )
            })?;
            true
        }
        None => false,
    };
    fs::write(
        &report.output_path,
        render_final_executable_image_dry_run(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld final executable image dry-run `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldFinalExecutableImageDryRunEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        image_path: report.image_path,
        image_emitted,
        image_constructed: report.image_constructed,
        image_ready: report.image_ready,
        image_format: report.image_format,
        image_header_size: report.image_header_size,
        payload_byte_offset: report.payload_byte_offset,
        image_size_bytes: report.image_size_bytes,
        image_hash: report.image_hash,
    })
}

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
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_image_dry_run `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_layout_hash,
        actual_byte_map_hash,
        actual_image_header_size,
        actual_payload_byte_offset,
        actual_image_constructed,
        actual_image_ready,
        actual_image_size_bytes,
        actual_image_hash,
        actual_scheduler_metadata_payload_id,
        actual_scheduler_metadata_present,
        actual_scheduler_metadata_offset,
        actual_scheduler_metadata_hash,
        actual_relocation_application_strategy,
        actual_relocation_application_count,
        actual_relocation_application_table_hash,
        actual_relocation_application_audit_status,
        actual_relocation_application_audit_count,
        actual_relocation_application_audit_table_hash,
        actual_relocation_application_audit_blockers,
        actual_relocation_patch_preview_status,
        actual_relocation_patch_preview_count,
        actual_relocation_patch_preview_table_hash,
        actual_relocation_patch_preview_records,
        actual_blockers,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "layout_hash"),
            toml::string_value(source, "byte_map_hash"),
            toml::usize_value(source, "image_header_size"),
            toml::usize_value(source, "payload_byte_offset"),
            toml::bool_value(source, "image_constructed"),
            toml::bool_value(source, "image_ready"),
            optional_usize_value(source, "image_size_bytes"),
            non_empty_toml_string(source, "image_hash"),
            non_empty_toml_string(source, "scheduler_metadata_payload_id"),
            toml::bool_value(source, "scheduler_metadata_present"),
            optional_usize_value(source, "scheduler_metadata_offset"),
            non_empty_toml_string(source, "scheduler_metadata_hash"),
            non_empty_toml_string(source, "relocation_application_strategy"),
            toml::usize_value(source, "relocation_application_count"),
            non_empty_toml_string(source, "relocation_application_table_hash"),
            non_empty_toml_string(source, "relocation_application_audit_status"),
            toml::usize_value(source, "relocation_application_audit_count"),
            non_empty_toml_string(source, "relocation_application_audit_table_hash"),
            toml::string_array_value(source, "relocation_application_audit_blockers"),
            non_empty_toml_string(source, "relocation_patch_preview_status"),
            toml::usize_value(source, "relocation_patch_preview_count"),
            non_empty_toml_string(source, "relocation_patch_preview_table_hash"),
            relocation_patch_preview_records_from_source(source),
            toml::string_array_value(source, "blockers"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
            )
        }
    };
    let actual_relocation_patch_preview_entry_count = actual
        .as_ref()
        .ok()
        .map(|_| actual_relocation_patch_preview_records.len());
    let actual_relocation_patch_preview_record_table_hash = actual
        .as_ref()
        .ok()
        .map(|_| relocation_patch_preview_table_hash(&actual_relocation_patch_preview_records));
    if let Ok(actual) = actual {
        if actual != expected_source {
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
    let payload_region =
        verify_final_executable_image_payload_region(&layout, &image_path, &mut issues);

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

struct RelocationPatchPreview {
    status: String,
    count: usize,
    table_hash: String,
    records: Vec<NsldFinalExecutableRelocationPatchPreviewRecord>,
}

fn relocation_patch_preview_records_from_source(
    source: &str,
) -> Vec<NsldFinalExecutableRelocationPatchPreviewRecord> {
    toml_table_blocks(source, "relocation_patch_preview")
        .into_iter()
        .filter_map(|block| {
            Some(NsldFinalExecutableRelocationPatchPreviewRecord {
                order_index: toml_block_usize_value(&block, "order_index")?,
                relocation_id: toml_block_string_value(&block, "relocation_id")?,
                patch_kind: toml_block_string_value(&block, "patch_kind")?,
                patch_offset: toml_block_usize_value(&block, "patch_offset")?,
                patch_width_bytes: toml_block_usize_value(&block, "patch_width_bytes")?,
                patch_value_hash: toml_block_string_value(&block, "patch_value_hash")?,
                target_symbol_id: toml_block_string_value(&block, "target_symbol_id")?,
                preview_status: toml_block_string_value(&block, "preview_status")?,
            })
        })
        .collect()
}

fn toml_table_blocks<'a>(source: &'a str, table: &str) -> Vec<Vec<&'a str>> {
    let header = format!("[[{table}]]");
    let mut blocks = Vec::new();
    let mut current: Option<Vec<&'a str>> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            if let Some(block) = current.take() {
                blocks.push(block);
            }
            current = (trimmed == header).then(Vec::new);
        } else if let Some(block) = current.as_mut() {
            block.push(line);
        }
    }
    if let Some(block) = current {
        blocks.push(block);
    }
    blocks
}

fn toml_block_string_value(block: &[&str], key: &str) -> Option<String> {
    let raw = toml_block_value(block, key)?.trim();
    let quoted = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn toml_block_usize_value(block: &[&str], key: &str) -> Option<usize> {
    toml_block_value(block, key)?.trim().parse().ok()
}

fn toml_block_value<'a>(block: &'a [&str], key: &str) -> Option<&'a str> {
    let prefix = format!("{key} =");
    block
        .iter()
        .map(|line| line.trim())
        .find_map(|line| line.strip_prefix(&prefix))
}

fn relocation_patch_preview(
    applications: &[NsldFinalExecutableRelocationApplicationRecord],
) -> RelocationPatchPreview {
    let records = applications
        .iter()
        .map(
            |application| NsldFinalExecutableRelocationPatchPreviewRecord {
                order_index: application.order_index,
                relocation_id: application.relocation_id.clone(),
                patch_kind: "u64-le-zero-placeholder".to_owned(),
                patch_offset: FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
                    .saturating_add(application.image_offset),
                patch_width_bytes: 8,
                patch_value_hash: fnv1a64_hex(&[0; 8]),
                target_symbol_id: application.target_symbol_id.clone(),
                preview_status: "planned".to_owned(),
            },
        )
        .collect::<Vec<_>>();
    let table_hash = relocation_patch_preview_table_hash(&records);
    RelocationPatchPreview {
        status: if records.is_empty() {
            "empty".to_owned()
        } else {
            "planned".to_owned()
        },
        count: records.len(),
        table_hash,
        records,
    }
}

fn relocation_patch_preview_table_hash(
    records: &[NsldFinalExecutableRelocationPatchPreviewRecord],
) -> String {
    let mut material = String::new();
    for record in records {
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.patch_kind);
        material.push('\t');
        material.push_str(&record.patch_offset.to_string());
        material.push('\t');
        material.push_str(&record.patch_width_bytes.to_string());
        material.push('\t');
        material.push_str(&record.patch_value_hash);
        material.push('\t');
        material.push_str(&record.target_symbol_id);
        material.push('\t');
        material.push_str(&record.preview_status);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

struct RelocationApplicationAudit {
    status: String,
    count: usize,
    table_hash: String,
    blockers: Vec<String>,
}

fn relocation_application_audit(
    records: &[NsldFinalExecutableRelocationApplicationRecord],
    payload_byte_offset: usize,
    payload_byte_span: usize,
) -> RelocationApplicationAudit {
    let payload_end = payload_byte_offset.saturating_add(payload_byte_span);
    let mut blockers = Vec::new();
    let mut material = String::new();
    for record in records {
        let image_offset = payload_byte_offset.saturating_add(record.image_offset);
        let status = if record.application_status != "planned" {
            "status-blocked"
        } else if image_offset >= payload_end {
            "out-of-bounds"
        } else {
            "audited"
        };
        if status != "audited" {
            blockers.push(format!("{}:{status}", record.relocation_id));
        }
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.relocation_kind);
        material.push('\t');
        material.push_str(&record.source_payload_id);
        material.push('\t');
        material.push_str(&record.source_section_id);
        material.push('\t');
        material.push_str(&record.source_offset.to_string());
        material.push('\t');
        material.push_str(&image_offset.to_string());
        material.push('\t');
        material.push_str(&record.target_symbol_id);
        material.push('\t');
        material.push_str(&record.addend.to_string());
        material.push('\t');
        material.push_str(status);
        material.push('\n');
    }
    RelocationApplicationAudit {
        status: if blockers.is_empty() {
            "ready".to_owned()
        } else {
            "blocked".to_owned()
        },
        count: records.len(),
        table_hash: fnv1a64_hex(material.as_bytes()),
        blockers,
    }
}
