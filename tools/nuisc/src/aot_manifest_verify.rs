use std::{fs, path::Path};

use crate::aot_domain_index_verify::{
    verify_domain_index_artifacts, DomainIndexArtifactRef, DomainIndexVerifyInput,
};
use crate::aot_domain_payload_verify::verify_domain_payload_blobs;
use crate::aot_domain_unit_verify::verify_domain_build_units;
use crate::aot_manifest_core_verify::{verify_manifest_artifacts, verify_manifest_core};
use crate::aot_manifest_fields::verify_manifest_fields;
use crate::aot_manifest_report::build_manifest_verify_report;
use crate::aot_project_metadata_verify::{
    verify_project_metadata_artifacts, ProjectMetadataArtifactsVerifyInput,
};
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

    let domain_index_report = verify_domain_index_artifacts(DomainIndexVerifyInput {
        manifest_path: path,
        bridge_registry: DomainIndexArtifactRef {
            path: fields.bridge_registry_path.as_deref(),
            schema: fields.bridge_registry_schema.as_deref(),
            count: fields.bridge_registry_units,
            inline: fields.bridge_registry_inline.as_deref(),
        },
        host_bridge_plan_index: DomainIndexArtifactRef {
            path: fields.host_bridge_plan_index_path.as_deref(),
            schema: fields.host_bridge_plan_index_schema.as_deref(),
            count: fields.host_bridge_plan_units,
            inline: fields.host_bridge_plan_index_inline.as_deref(),
        },
        lowering_plan_index: DomainIndexArtifactRef {
            path: fields.lowering_plan_index_path.as_deref(),
            schema: fields.lowering_plan_index_schema.as_deref(),
            count: fields.lowering_plan_units,
            inline: fields.lowering_plan_index_inline.as_deref(),
        },
        clock_protocol: DomainIndexArtifactRef {
            path: fields.clock_protocol_path.as_deref(),
            schema: fields.clock_protocol_schema.as_deref(),
            count: fields.clock_protocol_domains,
            inline: fields.clock_protocol_inline.as_deref(),
        },
        hetero_calculate_plan: DomainIndexArtifactRef {
            path: fields.hetero_calculate_plan_path.as_deref(),
            schema: fields.hetero_calculate_plan_schema.as_deref(),
            count: fields.hetero_calculate_plan_units,
            inline: fields.hetero_calculate_plan_inline.as_deref(),
        },
        heterogeneous_domain_count,
        domain_build_units,
    })?;

    let project_metadata_report =
        verify_project_metadata_artifacts(ProjectMetadataArtifactsVerifyInput {
            manifest_path: path,
            input: &core.input,
            output_dir: &core.output_dir,
            doc_index_path: fields.doc_index_path.as_deref(),
            doc_index_module_count: fields.doc_index_module_count,
            doc_index_documented_item_count: fields.doc_index_documented_item_count,
            project_plan_index: fields.project_plan_index.as_deref(),
            project_plan_summary: fields.project_plan_summary.as_deref(),
            project_docs_index: fields.project_docs_index.as_deref(),
            project_docs_module_count: fields.project_docs_module_count,
            project_docs_documented_module_count: fields.project_docs_documented_module_count,
            project_docs_documented_item_count: fields.project_docs_documented_item_count,
            project_imports_index: fields.project_imports_index.as_deref(),
            project_imports_library_count: fields.project_imports_library_count,
            project_imports_visible_library_count: fields.project_imports_visible_library_count,
            project_imports_visible_module_count: fields.project_imports_visible_module_count,
            project_imports_documented_visible_module_count: fields
                .project_imports_documented_visible_module_count,
            project_imports_documented_visible_item_count: fields
                .project_imports_documented_visible_item_count,
            project_galaxy_index: fields.project_galaxy_index.as_deref(),
            project_galaxy_count: fields.project_galaxy_count,
            project_documented_galaxy_count: fields.project_documented_galaxy_count,
            project_documented_galaxy_library_module_count: fields
                .project_documented_galaxy_library_module_count,
            project_documented_galaxy_item_count: fields.project_documented_galaxy_item_count,
            project_packet_index: fields.project_packet_index.as_deref(),
            project_host_ffi_index: fields.project_host_ffi_index.as_deref(),
        })?;
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
