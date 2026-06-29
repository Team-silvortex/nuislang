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
            Ok(BuildManifestDomainBuildUnit {
                package_id: contract.package_id.clone(),
                domain_family: contract.domain_family.clone(),
                abi,
                machine_arch,
                machine_os,
                backend_family,
                vendor,
                device_class,
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
