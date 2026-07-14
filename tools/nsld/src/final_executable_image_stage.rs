use super::{
    final_executable_image::{
        encode_final_executable_image, FINAL_EXECUTABLE_IMAGE_FORMAT,
        FINAL_EXECUTABLE_IMAGE_HEADER_SIZE, FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT,
    },
    final_executable_image_relocation::{
        apply_relocation_patches, relocation_application_audit, relocation_patch_preview,
        RelocationPatchApplication,
    },
    final_executable_layout_stage::nsld_final_executable_layout_plan_report,
    final_executable_paths::{
        nsld_final_executable_image_dry_run_bytes_path, nsld_final_executable_image_dry_run_path,
    },
    final_executable_render::render_final_executable_image_dry_run,
    fnv1a64_hex,
    reports::{NsldFinalExecutableImageDryRunEmitReport, NsldFinalExecutableImageDryRunReport},
};
use std::{fs, path::Path};

pub(crate) fn nsld_final_executable_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableImageDryRunReport {
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let mut image = encode_final_executable_image(&layout);
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
    let patch_preview = relocation_patch_preview(
        &layout.relocation_applications,
        &layout.payloads,
        &layout.byte_map_entries,
    );
    blockers.extend(
        patch_preview
            .blockers
            .iter()
            .map(|blocker| format!("relocation-resolver:{blocker}")),
    );
    let patch_application = image
        .as_mut()
        .map(|bytes| apply_relocation_patches(bytes, &patch_preview.records))
        .unwrap_or_else(|| RelocationPatchApplication {
            status: "blocked".to_owned(),
            count: 0,
            table_hash: fnv1a64_hex(b""),
            blockers: vec!["image-not-constructed".to_owned()],
        });
    blockers.extend(
        patch_application
            .blockers
            .iter()
            .map(|blocker| format!("relocation-patch-application:{blocker}")),
    );
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
        relocation_patch_application_status: patch_application.status,
        relocation_patch_application_count: patch_application.count,
        relocation_patch_application_table_hash: patch_application.table_hash,
        relocation_patch_application_blockers: patch_application.blockers,
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
    let image = patched_final_executable_image_bytes(manifest, plan);
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

pub(crate) use super::final_executable_image_verify::nsld_verify_final_executable_image_dry_run_report;

pub(crate) fn patched_final_executable_image_bytes(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Option<Vec<u8>> {
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let patch_preview = relocation_patch_preview(
        &layout.relocation_applications,
        &layout.payloads,
        &layout.byte_map_entries,
    );
    let mut image = encode_final_executable_image(&layout)?;
    let patch_application = apply_relocation_patches(&mut image, &patch_preview.records);
    patch_application.blockers.is_empty().then_some(image)
}
