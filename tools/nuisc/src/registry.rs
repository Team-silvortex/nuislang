pub use crate::registry_abi_helpers::{
    registered_abi_target, registered_abi_target_for_clang, used_ops_for_domain,
    validate_manifest_abi, validate_unit_binding,
};
pub use crate::registry_abi_target::RegisteredAbiTarget;
pub use crate::registry_binding_plan::plan_bindings;
pub use crate::registry_build_contract_summary::{
    domain_build_contract_summary, domain_build_contract_summary_for_domain,
    NustarDomainBackendStubSummary, NustarDomainBridgePlanSummary,
    NustarDomainBuildContractSummary, NustarDomainLoweringPlanSummary, NustarHostBridgeSpecSummary,
};
pub use crate::registry_contract::{
    capability_summary, domain_contract, execution_summary, load_domain_contract_for_domain,
    missing_domain_contract_groups, required_domain_contract_groups,
    NUSTAR_DOMAIN_CONTRACT_GROUP_ABI, NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION,
    NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE, NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER,
    NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY, NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME,
    NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER, NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET,
    NUSTAR_DOMAIN_CONTRACT_SCHEMA,
};
pub use crate::registry_domain_json::{
    domain_contract_json, domain_contract_object_json, domain_registration_json,
    domain_registration_object_json,
};
pub use crate::registry_host_ffi::{
    validate_abi_capabilities, HostFfiRegistryView, HostFfiSymbolRegistration,
};
pub use crate::registry_load::{
    load_all_manifests, load_index, load_manifest, load_manifest_for_domain,
    load_required_manifests, required_package_ids,
};
pub use crate::registry_manifest_parse::manifest_path;
pub use crate::registry_project_check_render::{
    project_domain_registry_check_json, project_domain_registry_issue_json,
    render_project_domain_registry_check_lines, write_project_domain_registry_check_lines,
};
pub use crate::registry_scheduler_summary::{
    scheduler_summary, std_net_summary, NustarSchedulerSummary, NustarStdNetSummary,
};
pub use crate::registry_types::{
    NustarBinding, NustarBindingPlan, NustarCapabilitySummary, NustarClockSummary,
    NustarDomainContract, NustarDomainRegistration, NustarExecutionSummary,
    NustarPackageIndexEntry, NustarPackageManifest, NustarRegistryIssue, NustarRegistryIssueKind,
    ProjectDomainRegistryCheck, ProjectDomainRegistryIssue, ProjectDomainRegistryIssueKind,
};
pub use crate::registry_validation::{
    domain_registration, ensure_project_domain_registry_valid, ensure_registered_domains_valid,
    load_domain_registration_for_domain, load_registered_domains, validate_project_domain_registry,
    validate_registered_domains,
};

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
