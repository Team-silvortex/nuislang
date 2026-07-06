use std::{collections::BTreeMap, path::Path};

use nuis_artifact::{BuildManifestDomainBuildUnit, NuisExecutableEnvelope};

use crate::aot_cpu_target::resolve_cpu_build_target_from_abi;
use crate::aot_lifecycle::{
    build_nuis_envelope as build_nuis_envelope_from_domain_summaries, NuisEnvelopeDomainSummary,
};
use crate::aot_manifest_types::{BuildManifestContext, BuildManifestProjectInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildManifestExecutionContract {
    pub(crate) package_id: String,
    pub(crate) domain_family: String,
    pub(crate) execution: crate::registry::NustarExecutionSummary,
}

pub(crate) fn resolve_execution_contracts(
    loaded_nustar: &[String],
) -> Result<Vec<BuildManifestExecutionContract>, String> {
    let mut contracts = Vec::new();
    for package_id in loaded_nustar {
        let manifest =
            match crate::registry::load_manifest(Path::new("nustar-packages"), package_id) {
                Ok(manifest) => manifest,
                Err(package_error) => {
                    let Some(domain) = package_id.strip_prefix("official.") else {
                        return Err(package_error);
                    };
                    crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), domain)
                        .map_err(|_| package_error)?
                }
            };
        contracts.push(BuildManifestExecutionContract {
            package_id: manifest.package_id.clone(),
            domain_family: manifest.domain_family.clone(),
            execution: crate::registry::execution_summary(&manifest),
        });
    }
    Ok(contracts)
}

pub(crate) fn build_manifest_domain_units(
    context: &BuildManifestContext,
    execution_contracts: &[BuildManifestExecutionContract],
) -> Result<Vec<BuildManifestDomainBuildUnit>, String> {
    let abi_by_domain = build_abi_map(&context.cpu_target.abi, context.project.as_ref());
    let mut units = execution_contracts
        .iter()
        .map(|contract| {
            let abi = abi_by_domain.get(&contract.domain_family).cloned();
            let (
                machine_arch,
                machine_os,
                backend_family,
                vendor,
                device_class,
                selected_lowering_target,
            ) = resolve_domain_build_unit_target(&contract.domain_family, abi.as_deref())?;
            let artifact_metadata = domain_artifact_metadata(
                &contract.domain_family,
                backend_family.as_deref(),
                selected_lowering_target.as_deref(),
                device_class.as_deref(),
            );
            Ok(BuildManifestDomainBuildUnit {
                package_id: contract.package_id.clone(),
                domain_family: contract.domain_family.clone(),
                abi,
                machine_arch,
                machine_os,
                backend_family,
                vendor,
                device_class,
                target_device: artifact_metadata.target_device,
                ir_format: artifact_metadata.ir_format,
                dispatch_abi: artifact_metadata.dispatch_abi,
                backend_priority: artifact_metadata.backend_priority,
                verification: artifact_metadata.verification,
                selected_lowering_target,
                artifact_stub_path: None,
                artifact_stub_inline: None,
                artifact_payload_path: None,
                artifact_bridge_stub_path: None,
                artifact_ir_sidecar_path: None,
                artifact_bridge_stub_inline: None,
                artifact_payload_blob_path: None,
                artifact_payload_blob_bytes: None,
                artifact_payload_format: None,
                artifact_payload_blob_inline: None,
                contract_family: contract.execution.contract_family.clone(),
                packaging_role: if contract.domain_family == "cpu" {
                    "host-binary".to_owned()
                } else {
                    "hetero-contract".to_owned()
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    units.sort_by(|lhs, rhs| {
        lhs.domain_family
            .cmp(&rhs.domain_family)
            .then_with(|| lhs.package_id.cmp(&rhs.package_id))
    });
    Ok(units)
}

fn build_abi_map(
    cpu_target_abi: &str,
    project: Option<&BuildManifestProjectInfo>,
) -> BTreeMap<String, String> {
    let mut abi_by_domain = BTreeMap::<String, String>::new();
    abi_by_domain.insert("cpu".to_owned(), cpu_target_abi.to_owned());
    if let Some(project) = project {
        for (domain, abi) in &project.abi_entries {
            abi_by_domain.insert(domain.clone(), abi.clone());
        }
    }
    abi_by_domain
}

fn resolve_domain_build_unit_target(
    domain_family: &str,
    abi: Option<&str>,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    String,
> {
    let Some(abi) = abi else {
        return Ok((None, None, None, None, None, None));
    };
    match domain_family {
        "cpu" => {
            let target = resolve_cpu_build_target_from_abi(Path::new("nustar-packages"), abi)?;
            Ok((
                Some(target.machine_arch),
                Some(target.machine_os),
                Some("llvm".to_owned()),
                None,
                None,
                Some("llvm".to_owned()),
            ))
        }
        "shader" | "kernel" | "network" => {
            let manifest = crate::registry::load_manifest_for_domain(
                Path::new("nustar-packages"),
                domain_family,
            )?;
            let target = crate::registry::registered_abi_target(&manifest, abi)?;
            let selected_lowering_target =
                crate::project::selected_lowering_target_for_registered_abi_target(
                    domain_family,
                    &target,
                    &manifest.lowering_targets,
                );
            let backend_family =
                crate::project::backend_family_for_registered_abi_target(domain_family, &target);
            Ok((
                Some(target.machine_arch),
                Some(target.machine_os),
                backend_family,
                target.vendor,
                target.device_class,
                selected_lowering_target,
            ))
        }
        _ => Ok((None, None, None, None, None, None)),
    }
}

struct DomainArtifactMetadata {
    target_device: Option<String>,
    ir_format: Option<String>,
    dispatch_abi: Option<String>,
    backend_priority: Option<usize>,
    verification: Option<String>,
}

fn domain_artifact_metadata(
    domain_family: &str,
    backend_family: Option<&str>,
    selected_lowering_target: Option<&str>,
    device_class: Option<&str>,
) -> DomainArtifactMetadata {
    let backend = backend_family.unwrap_or("none");
    let selected = selected_lowering_target.unwrap_or(backend);
    let target_device = match domain_family {
        "cpu" => Some("host-cpu".to_owned()),
        "shader" | "kernel" => device_class
            .map(str::to_owned)
            .or_else(|| backend_target_device(backend).map(str::to_owned)),
        "network" => Some(
            match backend {
                "urlsession" => "urlsession-stack",
                "winsock" => "winsock-stack",
                _ => "socket-io",
            }
            .to_owned(),
        ),
        _ => None,
    };
    let (ir_format, dispatch_abi, priority) = match domain_family {
        "cpu" => ("llvm-bitcode", "nuis-host-call", 100),
        "shader" => shader_artifact_metadata(backend),
        "kernel" => kernel_artifact_metadata(backend, selected),
        "network" => ("host-ffi-plan", "nuis-host-call", 700),
        _ => ("unknown", "unknown", 900),
    };
    DomainArtifactMetadata {
        target_device,
        ir_format: Some(ir_format.to_owned()).filter(|value| value != "unknown"),
        dispatch_abi: Some(dispatch_abi.to_owned()).filter(|value| value != "unknown"),
        backend_priority: Some(priority),
        verification: Some("contract-only".to_owned()),
    }
}

fn backend_target_device(backend: &str) -> Option<&'static str> {
    match backend {
        "metal" | "mps-graph" => Some("apple-gpu"),
        "coreml" => Some("apple-ane"),
        "vulkan" => Some("vulkan-device"),
        "directx" => Some("d3d12-device"),
        "webgpu" => Some("webgpu-device"),
        "opengl" => Some("opengl-device"),
        "cpu-fallback" | "llvm" => Some("host-cpu"),
        _ => None,
    }
}

fn shader_artifact_metadata(backend: &str) -> (&'static str, &'static str, usize) {
    match backend {
        "metal" => ("msl", "metal-render-pipeline", 10),
        "vulkan" => ("glsl450", "vulkan-graphics-pipeline", 20),
        "directx" => ("hlsl", "d3d12-graphics-pipeline", 30),
        "webgpu" => ("wgsl", "webgpu-render-pipeline", 40),
        "opengl" => ("glsl460", "opengl-graphics-pipeline", 80),
        _ => ("unknown", "unknown", 900),
    }
}

fn kernel_artifact_metadata(
    backend: &str,
    selected_lowering_target: &str,
) -> (&'static str, &'static str, usize) {
    match backend {
        "coreml" => (
            if selected_lowering_target.contains("graph") {
                "mlpackage"
            } else {
                "mlmodel"
            },
            "coreml-predict",
            10,
        ),
        "mps-graph" => ("mps-graph-json", "mps-graph-dispatch", 20),
        "vulkan" => ("spirv", "vulkan-compute-pipeline", 30),
        "cpu-fallback" => ("llvm-bitcode", "nuis-host-call", 900),
        _ => ("unknown", "unknown", 900),
    }
}

pub(crate) fn build_nuis_envelope(
    execution_contracts: &[BuildManifestExecutionContract],
    packaging_mode: &str,
) -> NuisExecutableEnvelope {
    let domains = execution_contracts
        .iter()
        .map(|item| NuisEnvelopeDomainSummary {
            domain_family: item.domain_family.clone(),
            contract_family: item.execution.contract_family.clone(),
            function_kind: item.execution.function_kind.clone(),
            graph_kind: item.execution.graph_kind.clone(),
            default_time_mode: item.execution.default_time_mode.clone(),
        })
        .collect::<Vec<_>>();
    build_nuis_envelope_from_domain_summaries(&domains, packaging_mode)
}
