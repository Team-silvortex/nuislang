use super::{
    final_executable_image::{
        parse_final_executable_image_header, FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT, FINAL_EXECUTABLE_IMAGE_VERSION,
    },
    final_executable_image_stage::nsld_verify_final_executable_image_dry_run_report,
    final_stage::{nsld_verify_final_executable_emit_report, nsld_verify_final_stage_plan_report},
    fnv1a64_hex,
    reports::NsldFinalExecutableOutputReport,
};
use std::{fs, path::Path};

pub(crate) fn nsld_final_executable_output_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableOutputReport {
    let final_stage = nsld_verify_final_stage_plan_report(manifest, plan);
    let final_emit = nsld_verify_final_executable_emit_report(manifest, plan);
    let image_dry_run = nsld_verify_final_executable_image_dry_run_report(manifest, plan);
    let output_path = plan.final_stage.output_path.clone();
    let host_native_output = plan.final_stage.link_mode == "host-toolchain-finalize";
    let output_kind = if host_native_output {
        "host-native-executable"
    } else {
        "nuis-image"
    }
    .to_owned();
    let output_validation_mode = if host_native_output {
        "host-native-presence-and-invoke-plan"
    } else {
        "nuis-image-header-size-and-hash"
    }
    .to_owned();
    let output_image_header_required = !host_native_output;
    let emitted = final_emit.actual_emitted == Some(true);
    let path_present = Path::new(&output_path).exists();
    let nsld_owned_output = emitted && path_present;
    let output_bytes = if emitted {
        fs::read(&output_path).ok()
    } else {
        None
    };
    let present = nsld_owned_output && output_bytes.is_some();
    let size_bytes = output_bytes.as_ref().map(Vec::len);
    let output_hash = output_bytes.as_ref().map(|bytes| fnv1a64_hex(bytes));
    let output_header = output_bytes
        .as_ref()
        .and_then(|bytes| parse_final_executable_image_header(bytes));
    let output_image_magic = output_header.as_ref().map(|header| header.magic.clone());
    let output_image_version = output_header.as_ref().map(|header| header.version as usize);
    let output_image_header_size = output_header.as_ref().map(|header| header.header_size);
    let output_payload_byte_offset = output_header.as_ref().map(|header| header.payload_offset);
    let output_payload_byte_span = output_header.as_ref().map(|header| header.payload_span);
    let output_layout_hash = output_header
        .as_ref()
        .map(|header| header.layout_hash.clone());
    let output_byte_map_hash = output_header
        .as_ref()
        .map(|header| header.byte_map_hash.clone());
    let scheduler_metadata_payload_id = image_dry_run.actual_scheduler_metadata_payload_id.clone();
    let scheduler_metadata_present = image_dry_run.actual_scheduler_metadata_present;
    let scheduler_metadata_offset = image_dry_run.actual_scheduler_metadata_offset;
    let scheduler_metadata_hash = image_dry_run.actual_scheduler_metadata_hash.clone();
    let output_image_header_valid = output_header.as_ref().is_some_and(|header| {
        let payload_end = header.payload_offset.saturating_add(header.payload_span);
        header.magic == FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT
            && header.version == FINAL_EXECUTABLE_IMAGE_VERSION
            && header.header_size == FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
            && header.payload_offset == FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
            && size_bytes.is_some_and(|size| payload_end <= size)
    });
    let expected_image_size_bytes = final_emit.actual_image_dry_run_size_bytes;
    let expected_image_hash = final_emit.actual_image_dry_run_hash.clone();
    let matches_expected_image = present
        && size_bytes == expected_image_size_bytes
        && output_hash == expected_image_hash
        && expected_image_hash.is_some();
    let mut blockers = Vec::new();
    let mut issues = Vec::new();

    if !final_stage.valid {
        blockers.push("final-stage-plan:invalid".to_owned());
        issues.extend(
            final_stage
                .issues
                .iter()
                .map(|issue| format!("final-stage-plan:{issue}")),
        );
    }
    if !final_emit.valid {
        blockers.push("final-executable-emit:invalid".to_owned());
        issues.extend(
            final_emit
                .issues
                .iter()
                .map(|issue| format!("final-executable-emit:{issue}")),
        );
    }
    if final_emit.actual_emitted != Some(true) {
        blockers.push("final-executable-emit:not-emitted".to_owned());
    }
    if !present {
        blockers.push("final-executable-output:missing".to_owned());
        if emitted {
            issues.push(format!(
                "missing_or_unreadable_final_executable_output `{output_path}`"
            ));
        }
    }
    if present && !host_native_output && !output_image_header_valid {
        blockers.push("final-executable-output:image-header-invalid".to_owned());
        issues.push(format!(
            "final executable output image header invalid: magic {} version {} header_size {} payload_offset {} payload_span {}",
            output_image_magic.as_deref().unwrap_or("missing"),
            output_image_version
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            output_image_header_size
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            output_payload_byte_offset
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            output_payload_byte_span
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if !host_native_output && final_emit.valid && expected_image_hash.is_none() {
        blockers.push("final-executable-output:expected-image-hash-missing".to_owned());
        issues.push("final executable output cannot be compared because verified image dry-run hash is missing".to_owned());
    }
    if !host_native_output
        && present
        && expected_image_size_bytes.is_some()
        && size_bytes != expected_image_size_bytes
    {
        blockers.push("final-executable-output:size-mismatch".to_owned());
        issues.push(format!(
            "final executable output size mismatch: expected {}, found {}",
            expected_image_size_bytes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            size_bytes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if !host_native_output
        && present
        && expected_image_hash.is_some()
        && output_hash != expected_image_hash
    {
        blockers.push("final-executable-output:hash-mismatch".to_owned());
        issues.push(format!(
            "final executable output hash mismatch: expected {}, found {}",
            expected_image_hash
                .clone()
                .unwrap_or_else(|| "missing".to_owned()),
            output_hash.clone().unwrap_or_else(|| "missing".to_owned())
        ));
    }

    let runnable_candidate = present
        && final_stage.valid
        && final_emit.valid
        && final_emit.actual_emitted == Some(true)
        && if host_native_output {
            final_emit.actual_host_invoke_plan_would_invoke == Some(true)
        } else {
            matches_expected_image && output_image_header_valid
        };

    NsldFinalExecutableOutputReport {
        manifest: manifest.display().to_string(),
        output_path,
        output_kind,
        output_validation_mode,
        path_present,
        nsld_owned_output,
        present,
        size_bytes,
        output_hash,
        output_image_header_required,
        output_image_header_valid,
        output_image_magic,
        output_image_version,
        output_image_header_size,
        output_payload_byte_offset,
        output_payload_byte_span,
        output_layout_hash,
        output_byte_map_hash,
        scheduler_metadata_payload_id,
        scheduler_metadata_present,
        scheduler_metadata_offset,
        scheduler_metadata_hash,
        expected_image_size_bytes,
        expected_image_hash,
        matches_expected_image,
        final_stage_plan_valid: final_stage.valid,
        final_stage_plan_hash: final_stage.actual_plan_hash,
        final_executable_emit_valid: final_emit.valid,
        final_executable_emitted: final_emit.actual_emitted,
        final_executable_blocker_count: final_emit.actual_blocker_count,
        runnable_candidate,
        blockers,
        issues,
    }
}
