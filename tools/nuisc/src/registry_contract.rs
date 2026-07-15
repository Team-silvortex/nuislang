use std::path::Path;

use crate::registry_build_contract_summary::{
    domain_build_contract_summary, NustarDomainBuildContractSummary,
};
use crate::registry_load::load_manifest_for_domain;
use crate::registry_scheduler_summary::{scheduler_summary, std_net_summary};
use crate::registry_types::{
    NustarCapabilitySummary, NustarClockSummary, NustarDispatchReadinessSummary,
    NustarDomainContract, NustarExecutionSummary, NustarPackageManifest,
};

pub const NUSTAR_DOMAIN_CONTRACT_SCHEMA: &str = "nustar-domain-contract-v1";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY: &str = "package_identity";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER: &str = "loader_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_ABI: &str = "abi_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE: &str = "host_bridge_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME: &str = "runtime_capability_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER: &str = "scheduler_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION: &str = "execution_skeleton_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_DISPATCH_READINESS: &str = "dispatch_readiness_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET: &str = "std_net_extension";

pub fn required_domain_contract_groups(manifest: &NustarPackageManifest) -> Vec<String> {
    let mut groups = vec![
        NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_ABI.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_DISPATCH_READINESS.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER.to_owned(),
    ];
    if !manifest.host_ffi_surface.is_empty() {
        groups.push(NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE.to_owned());
    }
    if manifest.domain_family == "network" {
        groups.push(NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned());
    }
    groups
}

fn group_is_complete(manifest: &NustarPackageManifest, group: &str) -> bool {
    match group {
        NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY => {
            !manifest.package_id.is_empty()
                && !manifest.domain_family.is_empty()
                && !manifest.frontend.is_empty()
        }
        NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER => {
            !manifest.loader_abi.is_empty() && !manifest.loader_entry.is_empty()
        }
        NUSTAR_DOMAIN_CONTRACT_GROUP_ABI => {
            !manifest.machine_abi_policy.is_empty() && !manifest.abi_profiles.is_empty()
        }
        NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE => {
            !manifest.host_ffi_surface.is_empty()
                && !manifest.host_ffi_abis.is_empty()
                && !manifest.host_ffi_bridge.is_empty()
        }
        NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME => {
            let support_surface_complete =
                manifest.domain_family == "cpu" || !manifest.support_surface.is_empty();
            let support_profile_slots_complete =
                manifest.domain_family == "cpu" || !manifest.support_profile_slots.is_empty();
            support_surface_complete
                && support_profile_slots_complete
                && !manifest.capability_tags.is_empty()
                && !manifest.default_lanes.is_empty()
                && !manifest.clock_domain_id.is_empty()
                && !manifest.clock_kind.is_empty()
                && !manifest.clock_epoch_kind.is_empty()
                && !manifest.clock_resolution.is_empty()
                && !manifest.clock_bridge_default.is_empty()
        }
        NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION => !manifest.lowering_targets.is_empty(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_DISPATCH_READINESS => dispatch_readiness_summary(manifest)
            .missing_signals
            .is_empty(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER => {
            let scheduler = scheduler_summary(manifest);
            !scheduler.contract_stack.is_empty()
                && !scheduler.result_roles.is_empty()
                && !scheduler.summary_api.is_empty()
                && !scheduler.observer_classes.is_empty()
        }
        NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET => {
            let std_net = std_net_summary(&manifest.domain_family);
            std_net.sample_navigation.is_some() && std_net.recipe_samples.is_some()
        }
        _ => false,
    }
}

fn required_dispatch_readiness_signals() -> Vec<String> {
    [
        "lowering_targets",
        "bridge_surface",
        "bridge_entry",
        "backend_stub_kind",
        "submission_mode",
        "wake_policy",
        "scheduler_binding",
        "phase_order",
        "phase_bind",
        "phase_submit",
        "phase_wait",
        "phase_finalize",
        "host_bridge_plan_begin",
        "host_bridge_plan_end",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn missing_dispatch_readiness_signals(
    manifest: &NustarPackageManifest,
    build_contract: &NustarDomainBuildContractSummary,
) -> Vec<String> {
    let mut missing = Vec::new();
    if manifest.lowering_targets.is_empty() {
        missing.push("lowering_targets".to_owned());
    }
    if build_contract.lowering.bridge_surface.is_empty() {
        missing.push("bridge_surface".to_owned());
    }
    if build_contract.backend.bridge_entry.is_empty() {
        missing.push("bridge_entry".to_owned());
    }
    if build_contract.backend.stub_kind.is_empty() {
        missing.push("backend_stub_kind".to_owned());
    }
    if build_contract.backend.submission_mode.is_empty() {
        missing.push("submission_mode".to_owned());
    }
    if build_contract.backend.wake_policy.is_empty() {
        missing.push("wake_policy".to_owned());
    }
    if build_contract.backend.scheduler_binding.is_empty() {
        missing.push("scheduler_binding".to_owned());
    }
    if build_contract.host_bridge.phase_order.is_empty() {
        missing.push("phase_order".to_owned());
    }
    if build_contract
        .backend
        .phase_bind
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        missing.push("phase_bind".to_owned());
    }
    if build_contract
        .backend
        .phase_submit
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        missing.push("phase_submit".to_owned());
    }
    if build_contract
        .backend
        .phase_wait
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        missing.push("phase_wait".to_owned());
    }
    if build_contract
        .backend
        .phase_finalize
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        missing.push("phase_finalize".to_owned());
    }
    if !build_contract.host_bridge.bridge_plan_begin {
        missing.push("host_bridge_plan_begin".to_owned());
    }
    if !build_contract.host_bridge.bridge_plan_end {
        missing.push("host_bridge_plan_end".to_owned());
    }
    missing
}

pub fn missing_domain_contract_groups(manifest: &NustarPackageManifest) -> Vec<String> {
    required_domain_contract_groups(manifest)
        .into_iter()
        .filter(|group| !group_is_complete(manifest, group))
        .collect()
}

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

pub fn dispatch_readiness_summary(
    manifest: &NustarPackageManifest,
) -> NustarDispatchReadinessSummary {
    let build_contract = domain_build_contract_summary(manifest);
    let missing_signals = missing_dispatch_readiness_signals(manifest, &build_contract);
    let dispatch_bridge_materialized = missing_signals
        .iter()
        .all(|signal| signal != "bridge_surface" && signal != "bridge_entry")
        && !build_contract.backend.stub_kind.is_empty()
        && build_contract.host_bridge.bridge_plan_begin
        && build_contract.host_bridge.bridge_plan_end;
    let execution_readiness_materialized = missing_signals
        .iter()
        .all(|signal| signal != "lowering_targets" && signal != "scheduler_binding")
        && !build_contract.backend.submission_mode.is_empty()
        && !build_contract.backend.wake_policy.is_empty();
    NustarDispatchReadinessSummary {
        status: if missing_signals.is_empty() {
            "ready"
        } else {
            "blocked"
        }
        .to_owned(),
        required_signals: required_dispatch_readiness_signals(),
        missing_signals,
        execution_readiness_materialized,
        dispatch_bridge_materialized,
        lifecycle_phase_order: build_contract.host_bridge.phase_order,
        scheduler_binding: build_contract.backend.scheduler_binding,
        bridge_entry: build_contract.backend.bridge_entry,
        bridge_surface: build_contract.lowering.bridge_surface,
        backend_stub_kind: build_contract.backend.stub_kind,
        submission_mode: build_contract.backend.submission_mode,
        wake_policy: build_contract.backend.wake_policy,
    }
}

pub fn domain_contract(manifest: &NustarPackageManifest) -> NustarDomainContract {
    let required_contract_groups = required_domain_contract_groups(manifest);
    let contract_groups = required_contract_groups
        .iter()
        .filter(|group| group.as_str() != NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET)
        .cloned()
        .collect::<Vec<_>>();
    let mut extension_groups = Vec::new();
    if manifest.domain_family == "network" {
        extension_groups.push(NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned());
    }
    let missing_contract_groups = missing_domain_contract_groups(manifest);
    let contract_status = if missing_contract_groups.is_empty() {
        "complete"
    } else {
        "incomplete"
    }
    .to_owned();
    let build_contract = domain_build_contract_summary(manifest);
    let dispatch_readiness = dispatch_readiness_summary(manifest);
    NustarDomainContract {
        contract_schema: NUSTAR_DOMAIN_CONTRACT_SCHEMA.to_owned(),
        contract_status,
        contract_groups,
        required_contract_groups,
        missing_contract_groups,
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
        build_contract,
        dispatch_readiness,
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
