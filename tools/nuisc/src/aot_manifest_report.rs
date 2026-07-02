use crate::aot_domain_index_verify::DomainIndexVerifyReport;
use crate::aot_domain_payload_verify::DomainPayloadVerifyReport;
use crate::aot_domain_unit_verify::DomainBuildUnitVerification;
use crate::aot_manifest_core_verify::ManifestCoreVerification;
use crate::aot_manifest_fields::ManifestFieldVerification;
use crate::aot_project_metadata_verify::ProjectMetadataVerifyReport;
use crate::aot_verify_report::BuildManifestVerifyReport;

pub(crate) fn build_manifest_verify_report(
    core: ManifestCoreVerification,
    fields: ManifestFieldVerification,
    domain_unit_report: DomainBuildUnitVerification,
    domain_payload_report: DomainPayloadVerifyReport,
    domain_index_report: DomainIndexVerifyReport,
    project_metadata_report: ProjectMetadataVerifyReport,
    artifacts_checked: usize,
) -> BuildManifestVerifyReport {
    let domain_build_unit_count = domain_unit_report.domain_build_units.len();
    BuildManifestVerifyReport {
        schema: core.schema,
        input: core.input,
        output_dir: core.output_dir,
        packaging_mode: core.packaging_mode,
        envelope_path: core.envelope_path,
        envelope_schema: core.envelope_schema,
        envelope_package_count: core.envelope_package_count,
        artifact_path: core.artifact_path,
        artifact_schema: core.artifact_schema,
        artifact_binary_name: core.artifact_binary_name,
        artifact_binary_bytes: core.artifact_binary_bytes,
        lifecycle_schema: core.lifecycle_schema,
        lifecycle_bootstrap_entry: core.lifecycle_bootstrap_entry,
        lifecycle_tick_policy: core.lifecycle_tick_policy,
        lifecycle_shutdown_policy: core.lifecycle_shutdown_policy,
        lifecycle_yalivia_rpc: core.lifecycle_yalivia_rpc,
        lifecycle_hook_count: core.lifecycle_hook_surface.len(),
        lifecycle_hook_surface: core.lifecycle_hook_surface,
        lifecycle_export_count: core.lifecycle_export_surface.len(),
        lifecycle_export_surface: core.lifecycle_export_surface,
        lifecycle_runtime_capability_flags: core.lifecycle_runtime_capability_flags,
        execution_contracts_checked: domain_unit_report.execution_contracts_checked,
        domain_build_unit_count,
        heterogeneous_domain_count: domain_unit_report.heterogeneous_domain_count,
        domain_payload_blobs_checked: domain_payload_report.domain_payload_blobs_checked,
        domain_payload_blob_sections_checked: domain_payload_report
            .domain_payload_blob_sections_checked,
        domain_payload_contract_sections_checked: domain_payload_report
            .domain_payload_contract_sections_checked,
        domain_payload_lowering_plans_checked: domain_payload_report
            .domain_payload_lowering_plans_checked,
        domain_payload_backend_stubs_checked: domain_payload_report
            .domain_payload_backend_stubs_checked,
        domain_payload_bridge_plans_checked: domain_payload_report
            .domain_payload_bridge_plans_checked,
        domain_bridge_stubs_checked: domain_payload_report.domain_bridge_stubs_checked,
        domain_build_units: domain_unit_report.domain_build_units,
        cpu_target_abi: fields.cpu_target_abi,
        cpu_target_machine_arch: fields.cpu_target_machine_arch,
        cpu_target_machine_os: fields.cpu_target_machine_os,
        cpu_target_object_format: fields.cpu_target_object_format,
        cpu_target_calling_abi: fields.cpu_target_calling_abi,
        cpu_target_clang: fields.cpu_target_clang,
        cpu_target_cross: fields.cpu_target_cross,
        loaded_nustar: fields.loaded_nustar,
        compile_cache_status: fields.compile_cache_status,
        compile_cache_key: fields.compile_cache_key,
        compile_cache_root: fields.compile_cache_root,
        doc_index_path: fields.doc_index_path,
        doc_index_module_count: fields.doc_index_module_count,
        doc_index_documented_item_count: fields.doc_index_documented_item_count,
        doc_index_checked: project_metadata_report.doc_index_checked,
        project_text_handle_rewrite_helper_hits: fields.project_text_handle_rewrite_helper_hits,
        project_text_handle_rewrite_local_hits: fields.project_text_handle_rewrite_local_hits,
        project_plan_index: fields.project_plan_index,
        project_docs_index: fields.project_docs_index,
        project_docs_module_count: fields.project_docs_module_count,
        project_docs_documented_module_count: fields.project_docs_documented_module_count,
        project_docs_documented_item_count: fields.project_docs_documented_item_count,
        project_imports_index: fields.project_imports_index,
        project_imports_library_count: fields.project_imports_library_count,
        project_imports_visible_library_count: fields.project_imports_visible_library_count,
        project_imports_visible_module_count: fields.project_imports_visible_module_count,
        project_imports_documented_visible_module_count: fields
            .project_imports_documented_visible_module_count,
        project_imports_documented_visible_item_count: fields
            .project_imports_documented_visible_item_count,
        project_galaxy_index: fields.project_galaxy_index,
        project_galaxy_count: fields.project_galaxy_count,
        project_documented_galaxy_count: fields.project_documented_galaxy_count,
        project_documented_galaxy_library_module_count: fields
            .project_documented_galaxy_library_module_count,
        project_documented_galaxy_item_count: fields.project_documented_galaxy_item_count,
        project_packet_index: fields.project_packet_index,
        project_host_ffi_index: fields.project_host_ffi_index,
        bridge_registry_path: fields.bridge_registry_path,
        bridge_registry_units: fields.bridge_registry_units,
        bridge_registry_checked: domain_index_report.bridge_registry_checked,
        bridge_registry_entries_checked: domain_index_report.bridge_registry_entries_checked,
        host_bridge_plan_index_path: fields.host_bridge_plan_index_path,
        host_bridge_plan_units: fields.host_bridge_plan_units,
        host_bridge_plan_checked: domain_index_report.host_bridge_plan_checked,
        host_bridge_plan_entries_checked: domain_index_report.host_bridge_plan_entries_checked,
        lowering_plan_index_path: fields.lowering_plan_index_path,
        lowering_plan_units: fields.lowering_plan_units,
        lowering_plan_index_checked: domain_index_report.lowering_plan_index_checked,
        lowering_plan_entries_checked: domain_index_report.lowering_plan_entries_checked,
        clock_protocol_path: fields.clock_protocol_path,
        clock_protocol_domains: fields.clock_protocol_domains,
        clock_protocol_checked: domain_index_report.clock_protocol_checked,
        clock_protocol_entries_checked: domain_index_report.clock_protocol_entries_checked,
        hetero_calculate_plan_path: fields.hetero_calculate_plan_path,
        hetero_calculate_plan_units: fields.hetero_calculate_plan_units,
        hetero_calculate_plan_checked: domain_index_report.hetero_calculate_plan_checked,
        hetero_calculate_plan_entries_checked: domain_index_report
            .hetero_calculate_plan_entries_checked,
        artifacts_checked,
        project_metadata_checked: project_metadata_report.project_metadata_checked,
    }
}
