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
        metadata_table_hash: container_file_emit.metadata_table_hash,
        container_layout_hash: container_emit.container_layout_hash,
        container_hash: container_file_emit.container_hash,
        payload_size_bytes: container_file_emit.payload_size_bytes,
        payload_hash: container_file_emit.payload_hash,
        issues,
    })
}
