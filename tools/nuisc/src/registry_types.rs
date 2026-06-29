use crate::registry_scheduler_summary::{NustarSchedulerSummary, NustarStdNetSummary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageIndexEntry {
    pub package_id: String,
    pub manifest: String,
    pub domain_family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageManifest {
    pub manifest_schema: String,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub entry_crate: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub binary_extension: String,
    pub package_layout: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub abi_capabilities: Vec<String>,
    pub abi_targets: Vec<String>,
    pub implementation_kinds: Vec<String>,
    pub loader_entry: String,
    pub loader_abi: String,
    pub host_ffi_surface: Vec<String>,
    pub host_ffi_abis: Vec<String>,
    pub host_ffi_bridge: String,
    pub bridge_lane_policy: Option<String>,
    pub bridge_surface: Option<String>,
    pub bridge_emission_kind: Option<String>,
    pub bridge_entry: Option<String>,
    pub bridge_kind: Option<String>,
    pub bridge_scheduler_binding: Option<String>,
    pub backend_stub_kind: Option<String>,
    pub backend_submission_mode: Option<String>,
    pub backend_wake_policy: Option<String>,
    pub backend_transport_model: Option<String>,
    pub backend_request_shape: Option<String>,
    pub backend_response_shape: Option<String>,
    pub backend_dispatch_shape: Option<String>,
    pub backend_memory_binding: Option<String>,
    pub backend_resource_binding: Option<String>,
    pub backend_completion_model: Option<String>,
    pub phase_bind: Option<String>,
    pub phase_submit: Option<String>,
    pub phase_wait: Option<String>,
    pub phase_finalize: Option<String>,
    pub host_bridge_host_ffi_surface: Option<Vec<String>>,
    pub host_bridge_handle_family: Option<Vec<String>>,
    pub host_bridge_phase_order: Option<Vec<String>>,
    pub host_bridge_phase_bind_inputs: Option<Vec<String>>,
    pub host_bridge_phase_bind_outputs: Option<Vec<String>>,
    pub host_bridge_phase_submit_inputs: Option<Vec<String>>,
    pub host_bridge_phase_submit_outputs: Option<Vec<String>>,
    pub host_bridge_phase_wait_inputs: Option<Vec<String>>,
    pub host_bridge_phase_wait_outputs: Option<Vec<String>>,
    pub host_bridge_phase_finalize_inputs: Option<Vec<String>>,
    pub host_bridge_phase_finalize_outputs: Option<Vec<String>>,
    pub host_bridge_phase_bind_wake: Option<String>,
    pub host_bridge_phase_submit_wake: Option<String>,
    pub host_bridge_phase_wait_wake: Option<String>,
    pub host_bridge_phase_finalize_wake: Option<String>,
    pub host_bridge_plan_begin: Option<bool>,
    pub host_bridge_plan_end: Option<bool>,
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub capability_tags: Vec<String>,
    pub default_lanes: Vec<String>,
    pub clock_domain_id: String,
    pub clock_kind: String,
    pub clock_epoch_kind: String,
    pub clock_resolution: String,
    pub clock_bridge_default: String,
    pub profiles: Vec<String>,
    pub resource_families: Vec<String>,
    pub unit_types: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub ops: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBinding {
    pub package_id: String,
    pub domain_family: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub abi_capabilities: Vec<String>,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub capability_tags: Vec<String>,
    pub default_lanes: Vec<String>,
    pub execution: NustarExecutionSummary,
    pub matched_support_surface: Vec<String>,
    pub matched_support_profile_slots: Vec<String>,
    pub covered_support_profile_slots: Vec<String>,
    pub uncovered_support_profile_slots: Vec<String>,
    pub registered_units: Vec<String>,
    pub bound_unit: Option<String>,
    pub used_units: Vec<String>,
    pub instantiated_units: Vec<String>,
    pub used_host_ffi_abis: Vec<String>,
    pub used_host_ffi_symbols: Vec<String>,
    pub matched_resources: Vec<String>,
    pub matched_ops: Vec<String>,
    pub undeclared_ops: Vec<String>,
    pub frontend: String,
    pub entry_crate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBindingPlan {
    pub bindings: Vec<NustarBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarClockSummary {
    pub domain_id: String,
    pub kind: String,
    pub epoch_kind: String,
    pub resolution: String,
    pub bridge_default: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarCapabilitySummary {
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub capability_tags: Vec<String>,
    pub default_lanes: Vec<String>,
    pub clock: NustarClockSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarExecutionSummary {
    pub skeleton_version: String,
    pub function_kind: String,
    pub graph_kind: String,
    pub execution_domain: String,
    pub default_time_mode: String,
    pub contract_family: String,
    pub lowering_targets: Vec<String>,
}

impl NustarClockSummary {
    pub fn brief(&self) -> String {
        format!(
            "{} [{}] bridge={}",
            self.domain_id, self.kind, self.bridge_default
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainContract {
    pub contract_schema: String,
    pub contract_groups: Vec<String>,
    pub extension_groups: Vec<String>,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub loader_abi: String,
    pub loader_entry: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub host_ffi_surface: Vec<String>,
    pub host_ffi_abis: Vec<String>,
    pub host_ffi_bridge: Option<String>,
    pub capability: NustarCapabilitySummary,
    pub execution: NustarExecutionSummary,
    pub scheduler: NustarSchedulerSummary,
    pub std_net: NustarStdNetSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainRegistration {
    pub manifest_path: String,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub entry_crate: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub resource_families: Vec<String>,
    pub unit_types: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub ops: Vec<String>,
    pub contract: NustarDomainContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NustarRegistryIssueKind {
    IndexEmpty,
    DuplicatePackageId,
    DuplicateDomainFamily,
    PackageIdentityMismatch,
    DomainFamilyMismatch,
    ManifestSchemaMismatch,
    LoaderContractMismatch,
    ResourceFamilyContractMismatch,
    OpContractMismatch,
    LaneContractMismatch,
    PackagingContractMismatch,
    DomainContractMismatch,
}

impl NustarRegistryIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IndexEmpty => "index_empty",
            Self::DuplicatePackageId => "duplicate_package_id",
            Self::DuplicateDomainFamily => "duplicate_domain_family",
            Self::PackageIdentityMismatch => "package_identity_mismatch",
            Self::DomainFamilyMismatch => "domain_family_mismatch",
            Self::ManifestSchemaMismatch => "manifest_schema_mismatch",
            Self::LoaderContractMismatch => "loader_contract_mismatch",
            Self::ResourceFamilyContractMismatch => "resource_family_contract_mismatch",
            Self::OpContractMismatch => "op_contract_mismatch",
            Self::LaneContractMismatch => "lane_contract_mismatch",
            Self::PackagingContractMismatch => "packaging_contract_mismatch",
            Self::DomainContractMismatch => "domain_contract_mismatch",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::IndexEmpty => "NRV001",
            Self::DuplicatePackageId => "NRV002",
            Self::DuplicateDomainFamily => "NRV003",
            Self::PackageIdentityMismatch => "NRV004",
            Self::DomainFamilyMismatch => "NRV005",
            Self::ManifestSchemaMismatch => "NRV006",
            Self::LoaderContractMismatch => "NRV007",
            Self::ResourceFamilyContractMismatch => "NRV008",
            Self::OpContractMismatch => "NRV009",
            Self::LaneContractMismatch => "NRV010",
            Self::PackagingContractMismatch => "NRV011",
            Self::DomainContractMismatch => "NRV012",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarRegistryIssue {
    pub kind: NustarRegistryIssueKind,
    pub package: Option<String>,
    pub domain: Option<String>,
    pub manifest_path: Option<String>,
    pub message: String,
}

impl NustarRegistryIssue {
    pub fn summary(&self) -> String {
        let package = self.package.as_deref().unwrap_or("<none>");
        let domain = self.domain.as_deref().unwrap_or("<none>");
        format!(
            "{} {} package={} domain={}: {}",
            self.kind.code(),
            self.kind.as_str(),
            package,
            domain,
            self.message
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDomainRegistryIssueKind {
    DomainNotRegistered,
    ContractSchemaMismatch,
    AbiNotRegistered,
    ExecutionContractMismatch,
}

impl ProjectDomainRegistryIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "domain_not_registered",
            Self::ContractSchemaMismatch => "contract_schema_mismatch",
            Self::AbiNotRegistered => "abi_not_registered",
            Self::ExecutionContractMismatch => "execution_contract_mismatch",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "NRG001",
            Self::ContractSchemaMismatch => "NRG002",
            Self::AbiNotRegistered => "NRG003",
            Self::ExecutionContractMismatch => "NRG004",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDomainRegistryIssue {
    pub kind: ProjectDomainRegistryIssueKind,
    pub message: String,
}

impl ProjectDomainRegistryIssue {
    pub fn summary(&self) -> String {
        format!(
            "{} {}: {}",
            self.kind.code(),
            self.kind.as_str(),
            self.message
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDomainRegistryCheck {
    pub domain: String,
    pub package: Option<String>,
    pub contract_schema: Option<String>,
    pub abi: Option<String>,
    pub abi_registered: bool,
    pub ok: bool,
    pub issues: Vec<ProjectDomainRegistryIssue>,
}

impl ProjectDomainRegistryCheck {
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "{} (package={}, abi={}): {}",
            self.domain,
            self.package.as_deref().unwrap_or("<missing>"),
            self.abi.as_deref().unwrap_or("<none>"),
            if self.issues.is_empty() {
                "ok".to_owned()
            } else {
                self.issues
                    .iter()
                    .map(ProjectDomainRegistryIssue::summary)
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        )
    }
}
