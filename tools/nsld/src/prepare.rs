use super::{
    assembly::{
        nsld_emit_assemble_plan_report, nsld_emit_link_bundle_report,
        nsld_emit_section_manifest_report, nsld_verify_assemble_plan_report,
        nsld_verify_link_bundle_report, nsld_verify_section_manifest_report,
    },
    container_pipeline::{
        nsld_emit_container_plan_report, nsld_emit_container_report,
        nsld_verify_container_plan_report, nsld_verify_container_report,
    },
    link_units::{
        nsld_emit_link_inputs_report, nsld_emit_link_units_report, nsld_verify_link_inputs_report,
        nsld_verify_link_units_report,
    },
    object_byte_layout::{
        nsld_emit_object_byte_layout_report, nsld_verify_object_byte_layout_report,
    },
    object_emit::{nsld_emit_object_report, nsld_verify_object_emit_report},
    object_file_layout::{
        nsld_emit_object_file_layout_report, nsld_verify_object_file_layout_report,
    },
    object_image_dry_run::{
        nsld_emit_object_image_dry_run_report, nsld_verify_object_image_dry_run_report,
    },
    object_output::nsld_verify_object_output_report,
    object_plan::{nsld_emit_object_plan_report, nsld_verify_object_plan_report},
    object_writer_input::{
        nsld_emit_object_writer_dry_run_report, nsld_verify_object_writer_dry_run_report,
        nsld_verify_object_writer_input_report,
    },
    reports::NsldPrepareReport,
};
use std::path::Path;

pub(crate) fn nsld_prepare_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldPrepareReport, String> {
    let input_emit = nsld_emit_link_inputs_report(manifest, plan)?;
    let input_verify = nsld_verify_link_inputs_report(manifest, plan);
    let unit_emit = nsld_emit_link_units_report(manifest, plan)?;
    let unit_verify = nsld_verify_link_units_report(manifest, plan);
    let bundle_emit = nsld_emit_link_bundle_report(manifest, plan)?;
    let bundle_verify = nsld_verify_link_bundle_report(manifest, plan);
    let assemble_emit = nsld_emit_assemble_plan_report(manifest, plan)?;
    let assemble_verify = nsld_verify_assemble_plan_report(manifest, plan);
    let section_emit = nsld_emit_section_manifest_report(manifest, plan)?;
    let section_verify = nsld_verify_section_manifest_report(manifest, plan);
    let object_plan_emit = nsld_emit_object_plan_report(manifest, plan)?;
    let object_plan_verify = nsld_verify_object_plan_report(manifest, plan);
    let object_emit = nsld_emit_object_report(manifest, plan)?;
    let object_writer_input_verify = nsld_verify_object_writer_input_report(manifest, plan);
    let object_byte_layout_emit = nsld_emit_object_byte_layout_report(manifest, plan)?;
    let object_byte_layout_verify = nsld_verify_object_byte_layout_report(manifest, plan);
    let object_file_layout_emit = nsld_emit_object_file_layout_report(manifest, plan)?;
    let object_file_layout_verify = nsld_verify_object_file_layout_report(manifest, plan);
    let object_image_dry_run_emit = nsld_emit_object_image_dry_run_report(manifest, plan)?;
    let object_image_dry_run_verify = nsld_verify_object_image_dry_run_report(manifest, plan);
    let object_emit_verify = nsld_verify_object_emit_report(manifest, plan);
    let object_output_verify = object_emit
        .emitted
        .then(|| nsld_verify_object_output_report(manifest, plan));
    let object_writer_dry_run_emit = nsld_emit_object_writer_dry_run_report(manifest, plan)?;
    let object_writer_dry_run_verify = nsld_verify_object_writer_dry_run_report(manifest, plan);
    let container_emit = nsld_emit_container_plan_report(manifest, plan)?;
    let container_verify = nsld_verify_container_plan_report(manifest, plan);
    let container_file_emit = nsld_emit_container_report(manifest, plan)?;
    let container_file_verify = nsld_verify_container_report(manifest, plan);

    let mut issues = Vec::new();
    if !input_verify.valid {
        issues.extend(
            input_verify
                .issues
                .iter()
                .map(|issue| format!("link-inputs:{issue}")),
        );
    }
    if !unit_verify.valid {
        issues.extend(
            unit_verify
                .issues
                .iter()
                .map(|issue| format!("link-units:{issue}")),
        );
    }
    if !bundle_verify.valid {
        issues.extend(
            bundle_verify
                .issues
                .iter()
                .map(|issue| format!("link-bundle:{issue}")),
        );
    }
    if !assemble_verify.valid {
        issues.extend(
            assemble_verify
                .issues
                .iter()
                .map(|issue| format!("assemble-plan:{issue}")),
        );
    }
    if !section_verify.valid {
        issues.extend(
            section_verify
                .issues
                .iter()
                .map(|issue| format!("section-manifest:{issue}")),
        );
    }
    if !object_plan_verify.valid {
        issues.extend(
            object_plan_verify
                .issues
                .iter()
                .map(|issue| format!("object-plan:{issue}")),
        );
    }
    if !object_writer_input_verify.valid {
        issues.extend(
            object_writer_input_verify
                .issues
                .iter()
                .map(|issue| format!("object-writer-input:{issue}")),
        );
    }
    if !object_byte_layout_verify.valid {
        issues.extend(
            object_byte_layout_verify
                .issues
                .iter()
                .map(|issue| format!("object-byte-layout:{issue}")),
        );
    }
    if !object_file_layout_verify.valid {
        issues.extend(
            object_file_layout_verify
                .issues
                .iter()
                .map(|issue| format!("object-file-layout:{issue}")),
        );
    }
    if !object_image_dry_run_verify.valid {
        issues.extend(
            object_image_dry_run_verify
                .issues
                .iter()
                .map(|issue| format!("object-image-dry-run:{issue}")),
        );
    }
    if !object_emit_verify.valid {
        issues.extend(
            object_emit_verify
                .issues
                .iter()
                .map(|issue| format!("object-emit:{issue}")),
        );
    }
    if let Some(object_output_verify) = object_output_verify.as_ref() {
        if !object_output_verify.valid {
            issues.extend(
                object_output_verify
                    .issues
                    .iter()
                    .map(|issue| format!("object-output:{issue}")),
            );
        }
    }
    if !object_writer_dry_run_verify.valid {
        issues.extend(
            object_writer_dry_run_verify
                .issues
                .iter()
                .map(|issue| format!("object-writer-dry-run:{issue}")),
        );
    }
    if !container_verify.valid {
        issues.extend(
            container_verify
                .issues
                .iter()
                .map(|issue| format!("container-plan:{issue}")),
        );
    }
    if !container_file_verify.valid {
        issues.extend(
            container_file_verify
                .issues
                .iter()
                .map(|issue| format!("container:{issue}")),
        );
    }

    Ok(NsldPrepareReport {
        manifest: manifest.display().to_string(),
        valid: issues.is_empty(),
        output_dir: plan.output_dir.clone(),
        link_input_table_path: input_emit.output_path,
        link_unit_table_path: unit_emit.output_path,
        link_bundle_path: bundle_emit.output_path,
        assemble_plan_path: assemble_emit.output_path,
        section_manifest_path: section_emit.output_path,
        object_plan_path: object_plan_emit.output_path,
        object_writer_input_path: object_emit.writer_input_path,
        object_byte_layout_path: object_byte_layout_emit.output_path,
        object_file_layout_path: object_file_layout_emit.output_path,
        object_image_dry_run_path: object_image_dry_run_emit.output_path,
        object_image_dry_run_bytes_path: object_image_dry_run_emit.image_path,
        object_emit_blocked_path: object_emit.blocked_report_path,
        object_output_path: object_emit.output_path,
        object_writer_dry_run_path: object_writer_dry_run_emit.output_path,
        container_plan_path: container_emit.output_path,
        container_path: container_file_emit.output_path,
        container_payload_path: container_file_emit.payload_path,
        link_input_count: input_emit.link_input_count,
        link_input_table_hash: input_emit.link_input_table_hash,
        unit_count: unit_emit.unit_count,
        unit_table_hash: unit_emit.unit_table_hash,
        bundle_id: bundle_emit.bundle_id,
        bundle_hash: bundle_emit.bundle_hash,
        bundle_ready: bundle_emit.bundle_ready,
        assemble_plan_hash: assemble_emit.assemble_plan_hash,
        section_table_hash: section_emit.section_table_hash,
        object_plan_hash: object_plan_emit.object_plan_hash,
        object_emitted: object_emit.emitted,
        byte_layout_hash: object_byte_layout_emit.byte_layout_hash,
        file_layout_hash: object_file_layout_emit.file_layout_hash,
        object_image_hash: object_image_dry_run_emit.image_hash,
        metadata_table_hash: container_file_emit.metadata_table_hash,
        container_layout_hash: container_emit.container_layout_hash,
        container_hash: container_file_emit.container_hash,
        payload_size_bytes: container_file_emit.payload_size_bytes,
        payload_hash: container_file_emit.payload_hash,
        issues,
    })
}
