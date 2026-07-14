use std::path::Path;

use crate::aot;
use nuis_artifact::protocol::COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML;

use super::{
    build_artifact_lowering_alignment_summary, linker_clock_protocol, linker_final_stage,
    linker_hetero_calculate, linker_host_ffi, LinkPlan, LinkPlanArtifact, LinkPlanCpuTarget,
    LinkPlanDomainUnit, LinkPlanEnvelope, LinkPlanLifecycle, LINK_PLAN_SCHEMA,
};

pub fn build_link_plan(
    report: &aot::BuildManifestVerifyReport,
    artifact: &aot::NuisCompiledArtifact,
) -> LinkPlan {
    let binary_path = final_output_path(report, artifact);
    let domain_units = report
        .domain_build_units
        .iter()
        .map(|unit| LinkPlanDomainUnit {
            kind: if unit.is_heterogeneous() {
                "heterogeneous".to_owned()
            } else {
                "host".to_owned()
            },
            package_id: unit.package_id.clone(),
            domain_family: unit.domain_family.clone(),
            abi: unit.abi.clone(),
            machine_arch: unit.machine_arch.clone(),
            machine_os: unit.machine_os.clone(),
            backend_family: unit.backend_family.clone(),
            vendor: unit.vendor.clone(),
            device_class: unit.device_class.clone(),
            target_device: unit.target_device.clone(),
            ir_format: unit.ir_format.clone(),
            dispatch_abi: unit.dispatch_abi.clone(),
            backend_priority: unit.backend_priority,
            verification: unit.verification.clone(),
            selected_lowering_target: unit.selected_lowering_target.clone(),
            contract_family: unit.contract_family.clone(),
            packaging_role: unit.packaging_role.clone(),
            artifact_stub_path: unit.artifact_stub_path.clone(),
            artifact_stub_inline: unit.artifact_stub_inline.clone(),
            artifact_payload_path: unit.artifact_payload_path.clone(),
            artifact_bridge_stub_path: unit.artifact_bridge_stub_path.clone(),
            artifact_ir_sidecar_path: unit.artifact_ir_sidecar_path.clone(),
            artifact_bridge_stub_inline: unit.artifact_bridge_stub_inline.clone(),
            artifact_payload_blob_path: unit.artifact_payload_blob_path.clone(),
            artifact_payload_blob_bytes: unit.artifact_payload_blob_bytes,
            artifact_payload_format: unit.artifact_payload_format.clone(),
            artifact_payload_blob_inline: unit.artifact_payload_blob_inline.clone(),
        })
        .collect::<Vec<_>>();

    let artifact_container =
        aot::inspect_nuis_compiled_artifact_container(Path::new(&report.artifact_path)).ok();
    let compiled_artifact = LinkPlanArtifact {
        path: report.artifact_path.clone(),
        binary_name: artifact.binary_name.clone(),
        binary_path: binary_path.clone(),
        binary_bytes: artifact.binary_bytes,
        build_manifest_bytes: artifact.build_manifest_bytes,
        container_kind: artifact_container
            .as_ref()
            .map(|container| container.container_kind.clone()),
        container_version: artifact_container
            .as_ref()
            .map(|container| container.binary_version),
        section_count: artifact_container
            .as_ref()
            .map(|container| container.section_count),
        section_names: artifact_container
            .as_ref()
            .map(|container| container.section_names.clone())
            .unwrap_or_default(),
        section_table_valid: artifact_container
            .as_ref()
            .map(|container| container.section_table_valid),
        lowering_unit_count: artifact_container
            .as_ref()
            .map(|container| container.lowering_unit_count),
        lowering_domain_families: artifact_container
            .as_ref()
            .map(|container| container.lowering_domain_families.clone())
            .unwrap_or_default(),
        lowering_targets: artifact_container
            .as_ref()
            .map(|container| container.lowering_targets.clone())
            .unwrap_or_default(),
        lowering_units: artifact_container
            .as_ref()
            .map(|container| container.lowering_units.clone())
            .unwrap_or_default(),
    };
    let artifact_lowering_alignment =
        build_artifact_lowering_alignment_summary(&compiled_artifact, &domain_units);
    let lowering_plan_index_source = lowering_plan_index_source(
        report.lowering_plan_index_path.as_deref(),
        &compiled_artifact,
    );
    let lifecycle = LinkPlanLifecycle {
        bootstrap_entry: report.lifecycle_bootstrap_entry.clone(),
        tick_policy: report.lifecycle_tick_policy.clone(),
        shutdown_policy: report.lifecycle_shutdown_policy.clone(),
        yalivia_rpc: report.lifecycle_yalivia_rpc.clone(),
        hook_surface: report.lifecycle_hook_surface.clone(),
        export_surface: report.lifecycle_export_surface.clone(),
        runtime_capability_flags: report.lifecycle_runtime_capability_flags.clone(),
    };
    let hetero_calculate =
        linker_hetero_calculate::derive_hetero_calculate_plan(&lifecycle, &domain_units);
    let clock_protocol = linker_clock_protocol::derive_clock_protocol(
        &lifecycle,
        &artifact.envelope.default_time_mode,
        &domain_units,
        &hetero_calculate,
    );
    let host_ffi =
        linker_host_ffi::build_host_ffi_footprint(report.project_host_ffi_index.as_deref());

    LinkPlan {
        schema: LINK_PLAN_SCHEMA.to_owned(),
        input: report.input.clone(),
        output_dir: report.output_dir.clone(),
        packaging_mode: report.packaging_mode.clone(),
        cpu_target: LinkPlanCpuTarget {
            abi: report.cpu_target_abi.clone(),
            machine_arch: report.cpu_target_machine_arch.clone(),
            machine_os: report.cpu_target_machine_os.clone(),
            object_format: report.cpu_target_object_format.clone(),
            calling_abi: report.cpu_target_calling_abi.clone(),
            clang_target: report.cpu_target_clang.clone(),
            cross_compile: report.cpu_target_cross,
        },
        lifecycle,
        envelope: LinkPlanEnvelope {
            schema: artifact.envelope.schema.clone(),
            package_count: artifact.envelope.package_count,
            contract_families: artifact.envelope.contract_families.clone(),
            domain_families: artifact.envelope.domain_families.clone(),
            function_kind: artifact.envelope.function_kind.clone(),
            graph_kind: artifact.envelope.graph_kind.clone(),
            default_time_mode: artifact.envelope.default_time_mode.clone(),
        },
        compiled_artifact,
        bridge_registry_path: report.bridge_registry_path.clone(),
        host_bridge_plan_index_path: report.host_bridge_plan_index_path.clone(),
        lowering_plan_index_path: report.lowering_plan_index_path.clone(),
        lowering_plan_index_source,
        host_ffi,
        domain_units,
        artifact_lowering_alignment,
        clock_protocol,
        hetero_calculate,
        final_stage: linker_final_stage::derive_final_stage(report, &binary_path),
    }
}

pub fn build_link_plan_from_manifest(path: &Path) -> Result<LinkPlan, String> {
    let report = aot::verify_build_manifest(path)?;
    let artifact = aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path))?;
    Ok(build_link_plan(&report, &artifact))
}

fn lowering_plan_index_source(path: Option<&str>, artifact: &LinkPlanArtifact) -> String {
    if path.is_some() {
        "manifest_path".to_owned()
    } else if artifact
        .section_names
        .iter()
        .any(|name| name == COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
    {
        "compiled_artifact_section".to_owned()
    } else {
        "unavailable".to_owned()
    }
}

fn final_output_path(
    report: &aot::BuildManifestVerifyReport,
    artifact: &aot::NuisCompiledArtifact,
) -> String {
    let file_name = if report.packaging_mode == "nuis-self-contained-image" {
        format!("{}.nsb", artifact.binary_name)
    } else {
        artifact.binary_name.clone()
    };
    Path::new(&report.output_dir)
        .join(file_name)
        .display()
        .to_string()
}
