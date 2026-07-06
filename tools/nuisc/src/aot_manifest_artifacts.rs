use std::path::{Path, PathBuf};

use nuis_artifact::{BuildManifestDomainBuildUnit, NuisLifecycleContract};

use crate::aot_domain_artifact_writer::write_domain_build_unit_stubs;
use crate::aot_domain_index_render::{
    render_domain_bridge_registry, render_domain_lowering_plan_index,
    render_host_bridge_plan_index, write_domain_bridge_registry, write_domain_lowering_plan_index,
    write_host_bridge_plan_index,
};
use crate::aot_manifest_types::CompileArtifacts;
use crate::linker::{self, LinkPlanDomainUnit, LinkPlanLifecycle};

pub(crate) struct BuildManifestArtifactSet {
    pub(crate) artifacts: Vec<(String, PathBuf)>,
    pub(crate) bridge_registry_path: Option<PathBuf>,
    pub(crate) bridge_registry_inline: Option<String>,
    pub(crate) host_bridge_plan_index_path: Option<PathBuf>,
    pub(crate) host_bridge_plan_index_inline: Option<String>,
    pub(crate) lowering_plan_index_path: Option<PathBuf>,
    pub(crate) lowering_plan_index_inline: Option<String>,
    pub(crate) clock_protocol_path: Option<PathBuf>,
    pub(crate) clock_protocol_inline: Option<String>,
    pub(crate) hetero_calculate_plan_path: Option<PathBuf>,
    pub(crate) hetero_calculate_plan_inline: Option<String>,
}

pub(crate) fn prepare_build_manifest_artifacts(
    output_dir: &Path,
    written: &CompileArtifacts,
    lifecycle: &NuisLifecycleContract,
    domain_build_units: &mut [BuildManifestDomainBuildUnit],
) -> Result<BuildManifestArtifactSet, String> {
    let mut artifacts = vec![
        ("ast".to_owned(), PathBuf::from(&written.ast_path)),
        ("nir".to_owned(), PathBuf::from(&written.nir_path)),
        ("yir".to_owned(), PathBuf::from(&written.yir_path)),
        ("llvm_ir".to_owned(), PathBuf::from(&written.llvm_ir_path)),
        ("binary".to_owned(), PathBuf::from(&written.binary_path)),
    ];
    artifacts.extend(write_domain_build_unit_stubs(
        output_dir,
        domain_build_units,
    )?);

    let hetero_units = domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let bridge_registry_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_domain_bridge_registry(&hetero_units))
    };
    let host_bridge_plan_index_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_host_bridge_plan_index(&hetero_units))
    };
    let lowering_plan_index_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_domain_lowering_plan_index(&hetero_units))
    };

    let bridge_registry_path = write_domain_bridge_registry(output_dir, domain_build_units)?;
    if let Some(bridge_registry_path) = &bridge_registry_path {
        artifacts.push((
            "domain_bridge_registry".to_owned(),
            bridge_registry_path.clone(),
        ));
    }
    let host_bridge_plan_index_path = write_host_bridge_plan_index(output_dir, domain_build_units)?;
    if let Some(host_bridge_plan_index_path) = &host_bridge_plan_index_path {
        artifacts.push((
            "host_bridge_plan_index".to_owned(),
            host_bridge_plan_index_path.clone(),
        ));
    }
    let lowering_plan_index_path =
        write_domain_lowering_plan_index(output_dir, domain_build_units)?;
    if let Some(lowering_plan_index_path) = &lowering_plan_index_path {
        artifacts.push((
            "domain_lowering_plan_index".to_owned(),
            lowering_plan_index_path.clone(),
        ));
    }
    let hetero_calculate_plan_inline = render_hetero_calculate_plan(lifecycle, domain_build_units)?;
    let hetero_calculate_plan_path =
        write_hetero_calculate_plan(output_dir, hetero_calculate_plan_inline.as_deref())?;
    if let Some(hetero_calculate_plan_path) = &hetero_calculate_plan_path {
        artifacts.push((
            "hetero_calculate_plan".to_owned(),
            hetero_calculate_plan_path.clone(),
        ));
    }
    let clock_protocol_inline = render_clock_protocol(lifecycle, domain_build_units)?;
    let clock_protocol_path = write_clock_protocol(output_dir, clock_protocol_inline.as_deref())?;
    if let Some(clock_protocol_path) = &clock_protocol_path {
        artifacts.push(("clock_protocol".to_owned(), clock_protocol_path.clone()));
    }

    Ok(BuildManifestArtifactSet {
        artifacts,
        bridge_registry_path,
        bridge_registry_inline,
        host_bridge_plan_index_path,
        host_bridge_plan_index_inline,
        lowering_plan_index_path,
        lowering_plan_index_inline,
        clock_protocol_path,
        clock_protocol_inline,
        hetero_calculate_plan_path,
        hetero_calculate_plan_inline,
    })
}

fn render_clock_protocol(
    lifecycle: &NuisLifecycleContract,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<String>, String> {
    let lifecycle = LinkPlanLifecycle {
        bootstrap_entry: lifecycle.bootstrap_entry.clone(),
        tick_policy: lifecycle.tick_policy.clone(),
        shutdown_policy: lifecycle.shutdown_policy.clone(),
        yalivia_rpc: lifecycle.yalivia_rpc.clone(),
        hook_surface: lifecycle.hook_surface.clone(),
        export_surface: lifecycle.export_surface.clone(),
        runtime_capability_flags: lifecycle.runtime_capability_flags.clone(),
    };
    let domain_units = domain_build_units
        .iter()
        .map(link_domain_unit_from_build_manifest_unit)
        .collect::<Vec<_>>();
    let hetero_calculate = linker::build_hetero_calculate_plan(&lifecycle, &domain_units);
    let clock_protocol =
        linker::build_clock_protocol(&lifecycle, "logical", &domain_units, &hetero_calculate);
    Ok(Some(linker::render_clock_protocol_toml(&clock_protocol)))
}

fn render_hetero_calculate_plan(
    lifecycle: &NuisLifecycleContract,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<String>, String> {
    if !domain_build_units
        .iter()
        .any(|unit| unit.domain_family != "cpu")
    {
        return Ok(None);
    }
    let lifecycle = LinkPlanLifecycle {
        bootstrap_entry: lifecycle.bootstrap_entry.clone(),
        tick_policy: lifecycle.tick_policy.clone(),
        shutdown_policy: lifecycle.shutdown_policy.clone(),
        yalivia_rpc: lifecycle.yalivia_rpc.clone(),
        hook_surface: lifecycle.hook_surface.clone(),
        export_surface: lifecycle.export_surface.clone(),
        runtime_capability_flags: lifecycle.runtime_capability_flags.clone(),
    };
    let domain_units = domain_build_units
        .iter()
        .map(link_domain_unit_from_build_manifest_unit)
        .collect::<Vec<_>>();
    let plan = linker::build_hetero_calculate_plan(&lifecycle, &domain_units);
    Ok(Some(linker::render_hetero_calculate_plan_toml(&plan)))
}

fn write_hetero_calculate_plan(
    output_dir: &Path,
    source: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    let Some(source) = source else {
        return Ok(None);
    };
    let path = output_dir.join("nuis.hetero-calculate.plan.toml");
    std::fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

fn write_clock_protocol(
    output_dir: &Path,
    source: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    let Some(source) = source else {
        return Ok(None);
    };
    let path = output_dir.join("nuis.clock-protocol.toml");
    std::fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

fn link_domain_unit_from_build_manifest_unit(
    unit: &BuildManifestDomainBuildUnit,
) -> LinkPlanDomainUnit {
    LinkPlanDomainUnit {
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
    }
}
