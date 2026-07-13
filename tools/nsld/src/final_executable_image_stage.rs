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
        NsldFinalExecutableImageDryRunVerifyReport,
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
                Vec::new(),
            )
        }
    };
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
