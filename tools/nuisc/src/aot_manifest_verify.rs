use std::{fs, path::Path};

use crate::aot_domain_index_verify::verify_domain_index_artifacts;
use crate::aot_domain_payload_verify::verify_domain_payload_blobs;
use crate::aot_domain_unit_verify::verify_domain_build_units;
use crate::aot_manifest_core_verify::{verify_manifest_artifacts, verify_manifest_core};
use crate::aot_manifest_fields::verify_manifest_fields;
use crate::aot_manifest_report::build_manifest_verify_report;
use crate::aot_project_metadata_verify::verify_project_metadata_artifacts;
use crate::aot_verify_report::BuildManifestVerifyReport;

pub fn verify_build_manifest(path: &Path) -> Result<BuildManifestVerifyReport, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let core = verify_manifest_core(&source, path)?;
    let fields = verify_manifest_fields(&source, path)?;

    let domain_unit_report = verify_domain_build_units(&source, path, &core)?;
    let heterogeneous_domain_count = domain_unit_report.heterogeneous_domain_count;
    let domain_build_units = &domain_unit_report.domain_build_units;
    let artifacts_checked = verify_manifest_artifacts(
        &source,
        path,
        &core,
        domain_build_units,
        fields.bridge_registry_inline.as_deref(),
        fields.host_bridge_plan_index_inline.as_deref(),
        fields.lowering_plan_index_inline.as_deref(),
    )?;

    let domain_payload_report = verify_domain_payload_blobs(path, domain_build_units)?;

    let domain_index_report = verify_domain_index_artifacts(
        path,
        fields.bridge_registry_path.as_deref(),
        fields.bridge_registry_schema.as_deref(),
        fields.bridge_registry_units,
        fields.bridge_registry_inline.as_deref(),
        fields.host_bridge_plan_index_path.as_deref(),
        fields.host_bridge_plan_index_schema.as_deref(),
        fields.host_bridge_plan_units,
        fields.host_bridge_plan_index_inline.as_deref(),
        fields.lowering_plan_index_path.as_deref(),
        fields.lowering_plan_index_schema.as_deref(),
        fields.lowering_plan_units,
        fields.lowering_plan_index_inline.as_deref(),
        heterogeneous_domain_count,
        domain_build_units,
    )?;

    let project_metadata_report = verify_project_metadata_artifacts(
        path,
        &core.input,
        &core.output_dir,
        fields.doc_index_path.as_deref(),
        fields.doc_index_module_count,
        fields.doc_index_documented_item_count,
        fields.project_plan_index.as_deref(),
        fields.project_plan_summary.as_deref(),
        fields.project_docs_index.as_deref(),
        fields.project_docs_module_count,
        fields.project_docs_documented_module_count,
        fields.project_docs_documented_item_count,
        fields.project_imports_index.as_deref(),
        fields.project_imports_library_count,
        fields.project_imports_visible_library_count,
        fields.project_imports_visible_module_count,
        fields.project_imports_documented_visible_module_count,
        fields.project_imports_documented_visible_item_count,
        fields.project_galaxy_index.as_deref(),
        fields.project_galaxy_count,
        fields.project_documented_galaxy_count,
        fields.project_documented_galaxy_library_module_count,
        fields.project_documented_galaxy_item_count,
        fields.project_packet_index.as_deref(),
    )?;
    Ok(build_manifest_verify_report(
        core,
        fields,
        domain_unit_report,
        domain_payload_report,
        domain_index_report,
        project_metadata_report,
        artifacts_checked,
    ))
}
