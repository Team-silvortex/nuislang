use std::path::Path;

use crate::registry_load::load_manifest_for_domain;
use crate::registry_scheduler_summary::{scheduler_summary, std_net_summary};
use crate::registry_types::{
    NustarCapabilitySummary, NustarClockSummary, NustarDomainContract, NustarExecutionSummary,
    NustarPackageManifest,
};

pub const NUSTAR_DOMAIN_CONTRACT_SCHEMA: &str = "nustar-domain-contract-v1";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY: &str = "package_identity";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER: &str = "loader_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_ABI: &str = "abi_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE: &str = "host_bridge_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME: &str = "runtime_capability_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER: &str = "scheduler_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION: &str = "execution_skeleton_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET: &str = "std_net_extension";

pub fn capability_summary(manifest: &NustarPackageManifest) -> NustarCapabilitySummary {
    NustarCapabilitySummary {
        support_surface: manifest.support_surface.clone(),
        support_profile_slots: manifest.support_profile_slots.clone(),
        capability_tags: manifest.capability_tags.clone(),
        default_lanes: manifest.default_lanes.clone(),
        clock: NustarClockSummary {
            domain_id: manifest.clock_domain_id.clone(),
            kind: manifest.clock_kind.clone(),
            epoch_kind: manifest.clock_epoch_kind.clone(),
            resolution: manifest.clock_resolution.clone(),
            bridge_default: manifest.clock_bridge_default.clone(),
        },
    }
}

pub fn execution_summary(manifest: &NustarPackageManifest) -> NustarExecutionSummary {
    NustarExecutionSummary {
        skeleton_version: "nustar-execution-skeleton-v1".to_owned(),
        function_kind: "function-node".to_owned(),
        graph_kind: "function-graph".to_owned(),
        execution_domain: manifest.domain_family.clone(),
        default_time_mode: "logical".to_owned(),
        contract_family: format!("nustar.{}", manifest.domain_family),
        lowering_targets: manifest.lowering_targets.clone(),
    }
}

pub fn domain_contract(manifest: &NustarPackageManifest) -> NustarDomainContract {
    let mut contract_groups = vec![
        NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_ABI.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER.to_owned(),
    ];
    if !manifest.host_ffi_surface.is_empty() {
        contract_groups.push(NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE.to_owned());
    }
    let mut extension_groups = Vec::new();
    if manifest.domain_family == "network" {
        extension_groups.push(NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned());
    }
    NustarDomainContract {
        contract_schema: NUSTAR_DOMAIN_CONTRACT_SCHEMA.to_owned(),
        contract_groups,
        extension_groups,
        package_id: manifest.package_id.clone(),
        domain_family: manifest.domain_family.clone(),
        frontend: manifest.frontend.clone(),
        loader_abi: manifest.loader_abi.clone(),
        loader_entry: manifest.loader_entry.clone(),
        machine_abi_policy: manifest.machine_abi_policy.clone(),
        abi_profiles: manifest.abi_profiles.clone(),
        host_ffi_surface: manifest.host_ffi_surface.clone(),
        host_ffi_abis: manifest.host_ffi_abis.clone(),
        host_ffi_bridge: if manifest.host_ffi_surface.is_empty() {
            None
        } else {
            Some(manifest.host_ffi_bridge.clone())
        },
        capability: capability_summary(manifest),
        execution: execution_summary(manifest),
        scheduler: scheduler_summary(manifest),
        std_net: std_net_summary(&manifest.domain_family),
    }
}

pub fn load_domain_contract_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarDomainContract, String> {
    let manifest = load_manifest_for_domain(root, domain_family)?;
    Ok(domain_contract(&manifest))
}
