use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use nuis_semantics::model::{NirExpr, NirModule, NirStmt};
use yir_core::YirModule;

const INDEX_FILE: &str = "index.toml";
pub const NUSTAR_DOMAIN_CONTRACT_SCHEMA: &str = "nustar-domain-contract-v1";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY: &str = "package_identity";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER: &str = "loader_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_ABI: &str = "abi_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE: &str = "host_bridge_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME: &str = "runtime_capability_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER: &str = "scheduler_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION: &str = "execution_skeleton_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET: &str = "std_net_extension";

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn json_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape(value))
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\":\"{}\"", name, json_escape(value)),
        None => format!("\"{}\":null", name),
    }
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", name, entries)
}

fn json_object_field(name: &str, fields: &[String]) -> String {
    format!("\"{}\":{{{}}}", name, fields.join(","))
}

fn resolve_registry_root(root: &Path) -> PathBuf {
    if root.exists() {
        return root.to_path_buf();
    }
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let candidate = workspace_root.join(root);
    if candidate.exists() {
        candidate
    } else {
        root.to_path_buf()
    }
}

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
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
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
pub struct RegisteredAbiTarget {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub backend_family: Option<String>,
    pub host_adaptive: bool,
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
    pub default_lanes: Vec<String>,
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

pub fn capability_summary(manifest: &NustarPackageManifest) -> NustarCapabilitySummary {
    NustarCapabilitySummary {
        support_surface: manifest.support_surface.clone(),
        support_profile_slots: manifest.support_profile_slots.clone(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarSchedulerSummary {
    pub contract_stack: String,
    pub clock: NustarClockSummary,
    pub result_roles: String,
    pub sample_navigation: Option<String>,
    pub result_samples: Option<String>,
    pub transport_samples: Option<String>,
    pub summary_api: String,
    pub summary_samples: Option<String>,
    pub observer_classes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarStdNetSummary {
    pub sample_navigation: Option<String>,
    pub recipe_samples: Option<String>,
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
pub enum ProjectDomainRegistryIssueKind {
    DomainNotRegistered,
    ContractSchemaMismatch,
    AbiNotRegistered,
}

impl ProjectDomainRegistryIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "domain_not_registered",
            Self::ContractSchemaMismatch => "contract_schema_mismatch",
            Self::AbiNotRegistered => "abi_not_registered",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "NRG001",
            Self::ContractSchemaMismatch => "NRG002",
            Self::AbiNotRegistered => "NRG003",
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

pub fn project_domain_registry_issue_json(issue: &ProjectDomainRegistryIssue) -> String {
    format!(
        "{{{},{},{}}}",
        json_field("code", issue.kind.code()),
        json_field("kind", issue.kind.as_str()),
        json_field("message", &issue.message)
    )
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

pub fn project_domain_registry_check_json(check: &ProjectDomainRegistryCheck) -> String {
    let issue_json = check
        .issues
        .iter()
        .map(project_domain_registry_issue_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{},{},{},{},{},{},{}}}",
        json_field("domain", &check.domain),
        json_optional_string_field("package", check.package.as_deref()),
        json_optional_string_field("contract_schema", check.contract_schema.as_deref()),
        json_optional_string_field("abi", check.abi.as_deref()),
        json_bool_field("abi_registered", check.abi_registered),
        json_bool_field("ok", check.ok),
        format!("\"issues\":[{}]", issue_json)
    )
}

pub fn render_project_domain_registry_check_lines(
    check: &ProjectDomainRegistryCheck,
) -> Vec<String> {
    let mut lines = vec![format!(
        "registry: {} package={} schema={} abi={} ok={} abi_registered={} issues={}",
        check.domain,
        check.package.as_deref().unwrap_or("<missing>"),
        check.contract_schema.as_deref().unwrap_or("<missing>"),
        check.abi.as_deref().unwrap_or("<none>"),
        if check.ok { "yes" } else { "no" },
        if check.abi_registered { "yes" } else { "no" },
        check.issue_count()
    )];
    for issue in &check.issues {
        lines.push(format!(
            "registry_issue: {} {} {}",
            issue.kind.code(),
            issue.kind.as_str(),
            issue.message
        ));
    }
    lines
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

pub fn domain_registration(
    root: &Path,
    entry: &NustarPackageIndexEntry,
) -> Result<NustarDomainRegistration, String> {
    let root = resolve_registry_root(root);
    let path = manifest_path(&root, entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let manifest = parse_manifest(&source, &path)?;
    Ok(NustarDomainRegistration {
        manifest_path: path.display().to_string(),
        package_id: manifest.package_id.clone(),
        domain_family: manifest.domain_family.clone(),
        frontend: manifest.frontend.clone(),
        entry_crate: manifest.entry_crate.clone(),
        ast_entry: manifest.ast_entry.clone(),
        nir_entry: manifest.nir_entry.clone(),
        yir_lowering_entry: manifest.yir_lowering_entry.clone(),
        part_verify_entry: manifest.part_verify_entry.clone(),
        ast_surface: manifest.ast_surface.clone(),
        nir_surface: manifest.nir_surface.clone(),
        yir_lowering: manifest.yir_lowering.clone(),
        part_verify: manifest.part_verify.clone(),
        resource_families: manifest.resource_families.clone(),
        unit_types: manifest.unit_types.clone(),
        lowering_targets: manifest.lowering_targets.clone(),
        ops: manifest.ops.clone(),
        contract: domain_contract(&manifest),
    })
}

pub fn load_registered_domains(root: &Path) -> Result<Vec<NustarDomainRegistration>, String> {
    let root = resolve_registry_root(root);
    let mut registrations = load_index(&root)?
        .into_iter()
        .map(|entry| domain_registration(&root, &entry))
        .collect::<Result<Vec<_>, _>>()?;
    registrations.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(registrations)
}

pub fn load_domain_registration_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarDomainRegistration, String> {
    let root = resolve_registry_root(root);
    let entry = load_index(&root)?
        .into_iter()
        .find(|entry| entry.domain_family == domain_family)
        .ok_or_else(|| {
            format!(
                "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    domain_registration(&root, &entry)
}

pub fn validate_project_domain_registry(
    plan: &crate::project::ProjectCompilationPlan,
) -> Vec<ProjectDomainRegistryCheck> {
    plan.abi_resolution
        .requirements
        .iter()
        .map(|item| {
            let mut issues = Vec::new();
            let mut package = None;
            let mut contract_schema = None;
            let mut abi_registered = false;
            match load_domain_registration_for_domain(Path::new("nustar-packages"), &item.domain) {
                Ok(registration) => {
                    package = Some(registration.package_id.clone());
                    contract_schema = Some(registration.contract.contract_schema.clone());
                    if registration.contract.contract_schema != NUSTAR_DOMAIN_CONTRACT_SCHEMA {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ContractSchemaMismatch,
                            message: format!(
                                "unexpected contract schema `{}`",
                                registration.contract.contract_schema
                            ),
                        });
                    }
                    abi_registered = registration
                        .contract
                        .abi_profiles
                        .iter()
                        .any(|candidate| candidate == &item.abi);
                    if !abi_registered {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::AbiNotRegistered,
                            message: format!(
                                "abi `{}` is not declared by registered profiles",
                                item.abi
                            ),
                        });
                    }
                }
                Err(error) => issues.push(ProjectDomainRegistryIssue {
                    kind: ProjectDomainRegistryIssueKind::DomainNotRegistered,
                    message: error,
                }),
            }
            ProjectDomainRegistryCheck {
                domain: item.domain.clone(),
                package,
                contract_schema,
                abi: Some(item.abi.clone()),
                abi_registered,
                ok: issues.is_empty(),
                issues,
            }
        })
        .collect()
}

pub fn ensure_project_domain_registry_valid(
    plan: &crate::project::ProjectCompilationPlan,
) -> Result<(), String> {
    let checks = validate_project_domain_registry(plan);
    let failures = checks
        .iter()
        .filter(|check| !check.ok)
        .map(ProjectDomainRegistryCheck::summary_line)
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "project domain registry validation failed:\n{}",
            failures.join("\n")
        ))
    }
}

pub fn domain_contract_object_json(contract: &NustarDomainContract) -> String {
    let package_identity_fields = vec![
        json_field("package", &contract.package_id),
        json_field("domain", &contract.domain_family),
        json_field("frontend", &contract.frontend),
    ];
    let loader_contract_fields = vec![
        json_field("loader_abi", &contract.loader_abi),
        json_field("loader_entry", &contract.loader_entry),
    ];
    let abi_contract_fields = vec![
        json_field("machine_abi_policy", &contract.machine_abi_policy),
        json_string_array_field("abi_profiles", &contract.abi_profiles),
    ];
    let host_bridge_contract_fields = vec![
        json_string_array_field("host_ffi_surface", &contract.host_ffi_surface),
        json_string_array_field("host_ffi_abis", &contract.host_ffi_abis),
        json_optional_string_field("host_ffi_bridge", contract.host_ffi_bridge.as_deref()),
    ];
    let runtime_capability_contract_fields = vec![
        json_string_array_field("support_surface", &contract.capability.support_surface),
        json_string_array_field(
            "support_profile_slots",
            &contract.capability.support_profile_slots,
        ),
        json_string_array_field("default_lanes", &contract.capability.default_lanes),
        json_field("clock_domain_id", &contract.capability.clock.domain_id),
        json_field("clock_kind", &contract.capability.clock.kind),
        json_field("clock_epoch_kind", &contract.capability.clock.epoch_kind),
        json_field("clock_resolution", &contract.capability.clock.resolution),
        json_field(
            "clock_bridge_default",
            &contract.capability.clock.bridge_default,
        ),
    ];
    let execution_contract_fields = vec![
        json_field("skeleton_version", &contract.execution.skeleton_version),
        json_field("function_kind", &contract.execution.function_kind),
        json_field("graph_kind", &contract.execution.graph_kind),
        json_field("execution_domain", &contract.execution.execution_domain),
        json_field("default_time_mode", &contract.execution.default_time_mode),
        json_field("contract_family", &contract.execution.contract_family),
        json_string_array_field("lowering_targets", &contract.execution.lowering_targets),
    ];
    let scheduler_contract_fields = vec![
        json_field(
            "scheduler_contract_stack",
            &contract.scheduler.contract_stack,
        ),
        json_field("scheduler_clock", &contract.scheduler.clock.brief()),
        json_field("scheduler_result_roles", &contract.scheduler.result_roles),
        json_optional_string_field(
            "scheduler_sample_navigation",
            contract.scheduler.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_result_samples",
            contract.scheduler.result_samples.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_transport_samples",
            contract.scheduler.transport_samples.as_deref(),
        ),
        json_field("scheduler_summary_api", &contract.scheduler.summary_api),
        json_optional_string_field(
            "scheduler_summary_samples",
            contract.scheduler.summary_samples.as_deref(),
        ),
        json_field(
            "scheduler_observer_classes",
            &contract.scheduler.observer_classes,
        ),
    ];
    let std_net_extension_fields = vec![
        json_optional_string_field(
            "std_net_navigation",
            contract.std_net.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "std_net_samples",
            contract.std_net.recipe_samples.as_deref(),
        ),
    ];
    let contract_fields = vec![
        json_field("schema", &contract.contract_schema),
        json_string_array_field("groups", &contract.contract_groups),
        json_string_array_field("extensions", &contract.extension_groups),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY,
            &package_identity_fields,
        ),
        json_object_field(NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER, &loader_contract_fields),
        json_object_field(NUSTAR_DOMAIN_CONTRACT_GROUP_ABI, &abi_contract_fields),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE,
            &host_bridge_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME,
            &runtime_capability_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION,
            &execution_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER,
            &scheduler_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET,
            &std_net_extension_fields,
        ),
    ];
    format!("{{{}}}", contract_fields.join(","))
}

pub fn domain_contract_json(contract: &NustarDomainContract) -> String {
    let fields = vec![
        json_field("package", &contract.package_id),
        json_field("domain", &contract.domain_family),
        json_field("contract_schema", &contract.contract_schema),
        json_string_array_field("contract_groups", &contract.contract_groups),
        json_string_array_field("extension_groups", &contract.extension_groups),
        json_field("frontend", &contract.frontend),
        json_field("loader_abi", &contract.loader_abi),
        json_field("loader_entry", &contract.loader_entry),
        json_field("machine_abi_policy", &contract.machine_abi_policy),
        json_string_array_field("abi_profiles", &contract.abi_profiles),
        json_string_array_field("host_ffi_surface", &contract.host_ffi_surface),
        json_string_array_field("host_ffi_abis", &contract.host_ffi_abis),
        json_optional_string_field("host_ffi_bridge", contract.host_ffi_bridge.as_deref()),
        json_string_array_field("support_surface", &contract.capability.support_surface),
        json_string_array_field(
            "support_profile_slots",
            &contract.capability.support_profile_slots,
        ),
        json_string_array_field("default_lanes", &contract.capability.default_lanes),
        json_field(
            "execution_skeleton_version",
            &contract.execution.skeleton_version,
        ),
        json_field("execution_function_kind", &contract.execution.function_kind),
        json_field("execution_graph_kind", &contract.execution.graph_kind),
        json_field("execution_domain", &contract.execution.execution_domain),
        json_field(
            "execution_default_time_mode",
            &contract.execution.default_time_mode,
        ),
        json_field(
            "execution_contract_family",
            &contract.execution.contract_family,
        ),
        json_string_array_field(
            "execution_lowering_targets",
            &contract.execution.lowering_targets,
        ),
        json_field(
            "scheduler_contract_stack",
            &contract.scheduler.contract_stack,
        ),
        json_field("scheduler_clock", &contract.scheduler.clock.brief()),
        json_field("scheduler_result_roles", &contract.scheduler.result_roles),
        json_optional_string_field(
            "scheduler_sample_navigation",
            contract.scheduler.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_result_samples",
            contract.scheduler.result_samples.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_transport_samples",
            contract.scheduler.transport_samples.as_deref(),
        ),
        json_field("scheduler_summary_api", &contract.scheduler.summary_api),
        json_optional_string_field(
            "scheduler_summary_samples",
            contract.scheduler.summary_samples.as_deref(),
        ),
        json_field(
            "scheduler_observer_classes",
            &contract.scheduler.observer_classes,
        ),
        json_optional_string_field(
            "std_net_navigation",
            contract.std_net.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "std_net_samples",
            contract.std_net.recipe_samples.as_deref(),
        ),
        format!("\"contract\":{}", domain_contract_object_json(contract)),
    ];
    format!("{{{}}}", fields.join(","))
}

pub fn domain_registration_object_json(registration: &NustarDomainRegistration) -> String {
    let registration_fields = vec![
        json_field("manifest_path", &registration.manifest_path),
        json_field("entry_crate", &registration.entry_crate),
        json_field("ast_entry", &registration.ast_entry),
        json_field("nir_entry", &registration.nir_entry),
        json_field("yir_lowering_entry", &registration.yir_lowering_entry),
        json_field("part_verify_entry", &registration.part_verify_entry),
        json_string_array_field("ast_surface", &registration.ast_surface),
        json_string_array_field("nir_surface", &registration.nir_surface),
        json_string_array_field("yir_lowering", &registration.yir_lowering),
        json_string_array_field("part_verify", &registration.part_verify),
        json_string_array_field("resource_families", &registration.resource_families),
        json_string_array_field("unit_types", &registration.unit_types),
        json_string_array_field("lowering_targets", &registration.lowering_targets),
        json_string_array_field("ops", &registration.ops),
    ];
    format!("{{{}}}", registration_fields.join(","))
}

pub fn domain_registration_json(registration: &NustarDomainRegistration) -> String {
    let mut fields = domain_contract_json(&registration.contract);
    fields.pop();
    fields.push_str(&format!(
        ",\"registration\":{}",
        domain_registration_object_json(registration)
    ));
    fields.push('}');
    fields
}

pub fn scheduler_summary(manifest: &NustarPackageManifest) -> NustarSchedulerSummary {
    let domain = manifest.domain_family.as_str();
    NustarSchedulerSummary {
        contract_stack:
            "placement -> timing -> result observation -> async summary observation -> observer classification"
                .to_owned(),
        clock: capability_summary(manifest).clock,
        result_roles: "entry=result-entry, probe=result-ready-probe, value=result-payload-value; variants=config_ready|send_ready|recv_ready|connect_ready|accept_ready|closed".to_owned(),
        sample_navigation: scheduler_sample_navigation(domain).map(str::to_owned),
        result_samples: scheduler_result_samples(domain).map(str::to_owned),
        transport_samples: scheduler_transport_samples(domain).map(str::to_owned),
        summary_api: "policy=async-policy-summary, batch=async-batch-summary, windowed=async-windowed-summary; classes=transport_split|transport_windowed_split|transport_session_bridge_split|control_split|control_windowed|control_session_bridge".to_owned(),
        summary_samples: scheduler_summary_samples(domain).map(str::to_owned),
        observer_classes: "source=profile-backed|result-backed|summary-backed; stage=entry|ready|payload|policy|batch|windowed; scope=local|cross-lane|cross-domain|bridge-visible; branch=primary|secondary|fallback|send|recv".to_owned(),
    }
}

pub fn std_net_summary(domain: &str) -> NustarStdNetSummary {
    NustarStdNetSummary {
        sample_navigation: std_net_sample_navigation(domain).map(str::to_owned),
        recipe_samples: std_net_recipe_samples(domain).map(str::to_owned),
    }
}

fn scheduler_sample_navigation(domain: &str) -> Option<&'static str> {
    match domain {
        "shader" => Some("policy -> windowed"),
        "kernel" => Some("policy -> windowed"),
        "network" => Some(
            "result_ladder -> transport_split_ladder -> transport_summary_ladder -> summary_classes",
        ),
        _ => None,
    }
}

fn scheduler_result_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "result_ladder=network_result_profile_demo -> network_connect_result_demo -> network_accept_result_demo -> network_result_task_policy_demo -> network_result_task_batch_demo -> network_result_task_windowed_batch_demo -> network_result_session_bridge_demo; control_ladder=network_connect_result_demo -> network_accept_result_demo -> network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo",
        ),
        _ => None,
    }
}

fn scheduler_transport_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "transport_runtime=network_host_handle_runtime_demo -> network_host_handle_transport_runtime_demo -> network_owned_transport_result_demo -> network_host_transport_runtime_demo -> network_transport_result_demo; transport_split_ladder=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo; transport_summary_ladder=network_owned_transport_result_task_policy_demo -> network_owned_transport_result_task_batch_demo -> network_owned_transport_result_task_windowed_batch_demo -> network_owned_transport_result_session_bridge_demo -> network_transport_result_task_policy_demo -> network_transport_result_task_batch_demo -> network_transport_result_task_windowed_batch_demo -> network_transport_result_session_bridge_demo",
        ),
        _ => None,
    }
}

fn scheduler_summary_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "shader" => Some(
            "policy=shader_async_policy_profile_demo -> shader_async_fallback_profile_demo; windowed=shader_async_batch_profile_demo -> shader_async_windowed_batch_profile_demo",
        ),
        "kernel" => Some(
            "policy=kernel_async_tensor_policy_profile_demo -> kernel_async_tensor_fallback_profile_demo; windowed=kernel_async_tensor_batch_profile_demo -> kernel_async_tensor_windowed_batch_profile_demo",
        ),
        "network" => Some(
            "transport_split=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo; control_split=network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo",
        ),
        _ => None,
    }
}

fn std_net_sample_navigation(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => {
            Some("profile_core -> transport_edge -> syscall_edge -> socket_edge -> control_edge -> protocol_edge -> http_edge -> result_spine -> task_spine -> session")
        }
        _ => None,
    }
}

fn std_net_recipe_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "profile_core=net_endpoint_recipe -> net_endpoint_recipe_demo; transport_edge=net_ip_packet_recipe -> net_tcp_stream_recipe -> net_udp_datagram_recipe -> net_ip_packet_recipe_demo -> net_tcp_stream_recipe_demo -> net_udp_datagram_recipe_demo; syscall_edge=net_tcp_open_recipe -> net_udp_open_recipe -> net_udp_bind_recipe -> net_udp_bound_socket_recipe -> net_udp_datagram_flow_recipe -> net_tcp_listener_recipe -> net_tcp_client_flow_recipe -> net_tcp_server_flow_recipe -> net_tcp_accepted_socket_recipe -> net_owned_send_recipe -> net_owned_recv_recipe -> net_owned_accept_recipe -> net_owned_close_recipe -> net_tcp_open_recipe_demo -> net_udp_open_recipe_demo -> net_udp_bind_recipe_demo -> net_udp_bound_socket_recipe_demo -> net_udp_datagram_flow_recipe_demo -> net_tcp_listener_recipe_demo -> net_tcp_client_flow_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_tcp_accepted_socket_recipe_demo -> net_owned_send_recipe_demo -> net_owned_recv_recipe_demo -> net_owned_accept_recipe_demo -> net_owned_close_recipe_demo; flow_group=tcp client flow -> tcp server flow -> udp datagram flow; flow=net_tcp_client_flow_recipe -> net_tcp_server_flow_recipe -> net_udp_datagram_flow_recipe -> net_tcp_client_flow_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_udp_datagram_flow_recipe_demo; socket_edge=net_tcp_connect_socket_recipe -> net_tcp_client_flow_recipe -> net_tcp_socket_recipe -> net_tcp_server_socket_recipe -> net_tcp_server_flow_recipe -> net_tcp_accepted_socket_recipe -> net_udp_bound_socket_recipe -> net_udp_datagram_flow_recipe -> net_udp_socket_recipe -> net_ip_socket_recipe -> net_tcp_connect_socket_recipe_demo -> net_tcp_client_flow_recipe_demo -> net_tcp_socket_recipe_demo -> net_tcp_server_socket_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_tcp_accepted_socket_recipe_demo -> net_udp_bound_socket_recipe_demo -> net_udp_datagram_flow_recipe_demo -> net_udp_socket_recipe_demo -> net_ip_socket_recipe_demo; control_edge=net_connect_recipe -> net_listen_recipe -> net_close_recipe -> net_connect_recipe_demo -> net_listen_recipe_demo -> net_close_recipe_demo; protocol_edge=net_protocol_experiment_recipe -> net_line_protocol_recipe -> net_datagram_protocol_recipe -> net_dnsish_protocol_recipe -> net_dnsish_query_recipe -> net_httpish_protocol_recipe -> net_httpish_request_recipe -> net_httpish_response_recipe -> net_httpish_roundtrip_recipe -> net_protocol_experiment_recipe_demo -> net_line_protocol_recipe_demo -> net_datagram_protocol_recipe_demo -> net_dnsish_protocol_recipe_demo -> net_dnsish_query_recipe_demo -> net_httpish_protocol_recipe_demo -> net_httpish_request_recipe_demo -> net_httpish_response_recipe_demo -> net_httpish_roundtrip_recipe_demo; http_edge=net_http_client_recipe -> net_http_request_builder_recipe -> net_http_client_headers_recipe -> net_http_client_url_recipe -> net_http_client_body_recipe -> net_http_client_status_recipe -> net_http_request_recipe -> net_http_response_recipe -> net_http_client_exchange_recipe -> net_http_client_session_recipe -> net_http_client_get_recipe -> net_http_client_post_recipe -> net_http_client_recipe_demo -> net_http_request_builder_recipe_demo -> net_http_client_headers_recipe_demo -> net_http_client_url_recipe_demo -> net_http_client_body_recipe_demo -> net_http_client_status_recipe_demo -> net_http_request_recipe_demo -> net_http_response_recipe_demo -> net_http_client_exchange_recipe_demo -> net_http_client_session_recipe_demo -> net_http_client_get_recipe_demo -> net_http_client_post_recipe_demo; result_spine=net_result_recipe -> net_result_bridge_recipe -> net_result_recipe_demo -> net_result_bridge_recipe_demo; task_spine=net_task_policy_recipe -> net_task_batch_recipe -> net_task_windowed_recipe -> net_task_windowed_bridge_recipe -> net_task_policy_recipe_demo -> net_task_batch_recipe_demo -> net_task_windowed_recipe_demo -> net_task_windowed_bridge_recipe_demo; compare_group=transport compare -> dnsish compare -> httpish compare; compare=net_transport_path_compare_recipe -> net_dnsish_path_compare_recipe -> net_httpish_path_compare_recipe -> net_transport_path_compare_recipe_demo -> net_dnsish_path_compare_recipe_demo -> net_httpish_path_compare_recipe_demo; owned_session=net_owned_transport_session_recipe -> net_owned_datagram_session_recipe -> net_owned_dnsish_exchange_session_recipe -> net_owned_dnsish_pipeline_recipe -> net_owned_transport_session_recipe_demo -> net_owned_datagram_session_recipe_demo -> net_owned_dnsish_exchange_session_recipe_demo -> net_owned_dnsish_pipeline_recipe_demo; session=net_control_session_recipe -> net_transport_session_recipe -> net_owned_transport_session_recipe -> net_tcp_listener_session_recipe -> net_transport_path_compare_recipe -> net_protocol_session_recipe -> net_datagram_session_recipe -> net_owned_datagram_session_recipe -> net_udp_bound_session_recipe -> net_datagram_exchange_session_recipe -> net_datagram_pipeline_recipe -> net_dnsish_exchange_session_recipe -> net_owned_dnsish_exchange_session_recipe -> net_dnsish_path_compare_recipe -> net_dnsish_pipeline_recipe -> net_owned_dnsish_pipeline_recipe -> net_http_client_session_recipe -> net_httpish_session_recipe -> net_httpish_exchange_session_recipe -> net_httpish_path_compare_recipe -> net_session_recipe -> net_control_session_recipe_demo -> net_transport_session_recipe_demo -> net_owned_transport_session_recipe_demo -> net_tcp_listener_session_recipe_demo -> net_transport_path_compare_recipe_demo -> net_protocol_session_recipe_demo -> net_datagram_session_recipe_demo -> net_owned_datagram_session_recipe_demo -> net_udp_bound_session_recipe_demo -> net_datagram_exchange_session_recipe_demo -> net_datagram_pipeline_recipe_demo -> net_dnsish_exchange_session_recipe_demo -> net_owned_dnsish_exchange_session_recipe_demo -> net_dnsish_path_compare_recipe_demo -> net_dnsish_pipeline_recipe_demo -> net_owned_dnsish_pipeline_recipe_demo -> net_http_client_session_recipe_demo -> net_httpish_session_recipe_demo -> net_httpish_exchange_session_recipe_demo -> net_httpish_path_compare_recipe_demo -> net_session_recipe_demo",
        ),
        _ => None,
    }
}

pub fn load_index(root: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let path = root.join(INDEX_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_index(&source, &path)
}

pub fn load_manifest(root: &Path, package_id: &str) -> Result<NustarPackageManifest, String> {
    let index = load_index(root)?;
    let entry = index
        .into_iter()
        .find(|entry| entry.package_id == package_id)
        .ok_or_else(|| {
            format!(
                "nustar package `{package_id}` is not present in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    let path = manifest_path(root, &entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_manifest_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarPackageManifest, String> {
    let root = resolve_registry_root(root);
    let path = match load_index(&root) {
        Ok(index) => {
            match index
                .into_iter()
                .find(|entry| entry.domain_family == domain_family)
            {
                Some(entry) => manifest_path(&root, &entry),
                None => {
                    let direct = root.join(format!("{domain_family}.toml"));
                    if direct.exists() {
                        direct
                    } else {
                        return Err(format!(
                            "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
                            root.join(INDEX_FILE).display()
                        ));
                    }
                }
            }
        }
        Err(index_error) => {
            let direct = root.join(format!("{domain_family}.toml"));
            if direct.exists() {
                direct
            } else {
                return Err(index_error);
            }
        }
    };
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_all_manifests(root: &Path) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for entry in load_index(root)? {
        manifests.push(load_manifest(root, &entry.package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}

pub fn required_package_ids(module: &YirModule) -> Vec<String> {
    let mut package_ids = BTreeSet::new();
    for node in &module.nodes {
        package_ids.insert(format!("official.{}", node.op.module));
        if node.op.module == "cpu" && node.op.instruction == "instantiate_unit" {
            if let Some(domain) = node.op.args.first() {
                package_ids.insert(format!("official.{domain}"));
            }
        }
    }
    package_ids.into_iter().collect()
}

pub fn load_required_manifests(
    root: &Path,
    module: &YirModule,
) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for package_id in required_package_ids(module) {
        manifests.push(load_manifest(root, &package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}

pub fn plan_bindings(
    root: &Path,
    nir: &NirModule,
    module: &YirModule,
    domain: &str,
    unit: &str,
    declared_used_units: &[(String, String)],
    declared_externs: &[(String, String)],
) -> Result<NustarBindingPlan, String> {
    let mut manifests = load_required_manifests(root, module)?;
    let mut loaded_domains = manifests
        .iter()
        .map(|manifest| manifest.domain_family.clone())
        .collect::<BTreeSet<_>>();
    if loaded_domains.insert(domain.to_owned()) {
        manifests.push(load_manifest_for_domain(root, domain)?);
    }
    for (used_domain, _) in declared_used_units {
        if loaded_domains.insert(used_domain.clone()) {
            manifests.push(load_manifest_for_domain(root, used_domain)?);
        }
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    validate_unit_binding(&manifests, domain, unit)?;
    let mut bindings = Vec::new();

    for manifest in manifests {
        let registered_units = manifest
            .unit_types
            .iter()
            .filter(|unit| !unit.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        let bound_unit = if manifest.domain_family == domain {
            Some(unit.to_owned())
        } else {
            None
        };
        let used_units = declared_used_units
            .iter()
            .filter(|(used_domain, _)| used_domain == &manifest.domain_family)
            .map(|(_, used_unit)| used_unit.clone())
            .collect::<Vec<_>>();
        let instantiated_units = module
            .nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "instantiate_unit"
                    && node.op.args.first().map(String::as_str)
                        == Some(manifest.domain_family.as_str())
            })
            .filter_map(|node| node.op.args.get(1).cloned())
            .collect::<Vec<_>>();
        let used_host_ffi_abis = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(abi, _)| abi.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let used_host_ffi_symbols = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(_, symbol)| symbol.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let (matched_support_surface, matched_support_profile_slots) =
            detect_matched_support_usage(nir, &manifest.domain_family);
        let covered_support_profile_slots = covered_profile_slots(
            &manifest.domain_family,
            &matched_support_surface,
            &matched_support_profile_slots,
        );
        let uncovered_support_profile_slots = manifest
            .support_profile_slots
            .iter()
            .filter(|slot| {
                !covered_support_profile_slots
                    .iter()
                    .any(|covered| covered == *slot)
            })
            .cloned()
            .collect::<Vec<_>>();

        let matched_resources = module
            .resources
            .iter()
            .filter(|resource| {
                manifest
                    .resource_families
                    .iter()
                    .any(|family| family == resource.kind.family())
            })
            .map(|resource| resource.name.clone())
            .collect::<Vec<_>>();

        let matched_ops = module
            .nodes
            .iter()
            .filter(|node| node.op.module == manifest.domain_family)
            .map(|node| node.op.full_name())
            .collect::<Vec<_>>();

        if matched_ops.is_empty() && instantiated_units.is_empty() && used_units.is_empty() {
            return Err(format!(
                "nustar package `{}` was selected but no matching ops were bound",
                manifest.package_id
            ));
        }

        let undeclared_ops = matched_ops
            .iter()
            .filter(|op| !manifest.ops.iter().any(|candidate| candidate == *op))
            .cloned()
            .collect::<Vec<_>>();

        bindings.push(NustarBinding {
            package_id: manifest.package_id,
            domain_family: manifest.domain_family,
            ast_entry: manifest.ast_entry,
            nir_entry: manifest.nir_entry,
            yir_lowering_entry: manifest.yir_lowering_entry,
            part_verify_entry: manifest.part_verify_entry,
            machine_abi_policy: manifest.machine_abi_policy,
            abi_profiles: manifest.abi_profiles,
            abi_capabilities: manifest.abi_capabilities,
            ast_surface: manifest.ast_surface,
            nir_surface: manifest.nir_surface,
            yir_lowering: manifest.yir_lowering,
            part_verify: manifest.part_verify,
            support_surface: manifest.support_surface,
            support_profile_slots: manifest.support_profile_slots,
            default_lanes: manifest.default_lanes,
            matched_support_surface,
            matched_support_profile_slots,
            covered_support_profile_slots,
            uncovered_support_profile_slots,
            registered_units,
            bound_unit,
            used_units,
            instantiated_units,
            used_host_ffi_abis,
            used_host_ffi_symbols,
            matched_resources,
            matched_ops,
            undeclared_ops,
            frontend: manifest.frontend,
            entry_crate: manifest.entry_crate,
        });
    }

    bindings.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(NustarBindingPlan { bindings })
}

fn covered_profile_slots(
    domain_family: &str,
    matched_support_surface: &[String],
    matched_support_profile_slots: &[String],
) -> Vec<String> {
    let mut covered = matched_support_profile_slots
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for surface in matched_support_surface {
        for slot in implied_slots_for_surface(domain_family, surface) {
            covered.insert(slot.to_string());
        }
    }
    covered.into_iter().collect::<Vec<_>>()
}

fn implied_slots_for_surface(domain_family: &str, surface: &str) -> &'static [&'static str] {
    match (domain_family, surface) {
        ("shader", "shader.profile.render.v1") => &[
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
            "pass_kind",
            "packet_field_count",
        ],
        ("shader", "shader.profile.draw.v1") => &[
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
            "pass_kind",
            "packet_field_count",
        ],
        ("shader", "shader.profile.seed.color.v1") => {
            &["packet_color_slot", "slider_color_slot", "material_mode"]
        }
        ("shader", "shader.profile.seed.speed.v1") => {
            &["packet_speed_slot", "slider_speed_slot", "packet_tag"]
        }
        ("shader", "shader.profile.seed.radius.v1") => &[
            "packet_radius_slot",
            "slider_radius_slot",
            "packet_field_count",
        ],
        ("shader", "shader.profile.packet.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.target.v1") => &["target"],
        ("shader", "shader.profile.viewport.v1") => &["viewport"],
        ("shader", "shader.profile.pipeline.v1") => &["pipeline"],
        ("shader", "shader.profile.draw-budget.v1") => &["vertex_count", "instance_count"],
        ("shader", "shader.profile.packet-slots.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.packet-tag.v1") => &["packet_tag"],
        ("shader", "shader.profile.material-mode.v1") => &["material_mode"],
        ("shader", "shader.profile.pass-kind.v1") => &["pass_kind"],
        ("shader", "shader.profile.packet-field-count.v1") => &["packet_field_count"],
        ("data", "data.profile.bind-core.v1") => &["bind_core"],
        ("data", "data.profile.send.uplink.v1") => &[
            "window_offset",
            "uplink_len",
            "marker:cpu_to_shader",
            "marker:uplink_pipe",
            "marker:uplink_pipe_class",
            "marker:uplink_payload_class",
            "marker:uplink_payload_shape",
            "marker:uplink_window_policy",
        ],
        ("data", "data.profile.send.downlink.v1") => &[
            "window_offset",
            "downlink_len",
            "marker:shader_to_cpu",
            "marker:downlink_pipe",
            "marker:downlink_pipe_class",
            "marker:downlink_payload_class",
            "marker:downlink_payload_shape",
            "marker:downlink_window_policy",
        ],
        ("data", "data.profile.handle-table.v1") => &["handle_table"],
        ("data", "data.profile.window-layout.v1") => {
            &["window_offset", "uplink_len", "downlink_len"]
        }
        ("data", "data.profile.sync-markers.v1") => {
            &["marker:cpu_to_shader", "marker:shader_to_cpu"]
        }
        ("data", "data.profile.pipe-markers.v1") => &["marker:uplink_pipe", "marker:downlink_pipe"],
        ("data", "data.profile.pipe-class.v1") => {
            &["marker:uplink_pipe_class", "marker:downlink_pipe_class"]
        }
        ("data", "data.profile.payload-class.v1") => &[
            "marker:uplink_payload_class",
            "marker:downlink_payload_class",
        ],
        ("data", "data.profile.payload-shape.v1") => &[
            "marker:uplink_payload_shape",
            "marker:downlink_payload_shape",
        ],
        ("data", "data.profile.window-policy.v1") => &[
            "marker:uplink_window_policy",
            "marker:downlink_window_policy",
        ],
        ("network", "network.profile.bind-core.v1") => &["bind_core"],
        ("network", "network.profile.connect.v1") => {
            &["remote_port", "connect_timeout_ms", "endpoint_kind"]
        }
        ("network", "network.profile.accept.v1") => &[
            "local_port",
            "read_timeout_ms",
            "write_timeout_ms",
            "endpoint_kind",
        ],
        ("network", "network.profile.send.v1") => &["send_window", "stream_window"],
        ("network", "network.profile.recv.v1") => &["recv_window", "stream_window"],
        ("network", "network.profile.close.v1") => &[],
        ("network", "network.profile.timeout.v1") => &[
            "connect_timeout_ms",
            "read_timeout_ms",
            "write_timeout_ms",
            "timeout_budget",
        ],
        ("network", "network.profile.retry.v1") => &["retry_budget"],
        ("network", "network.profile.endpoint-kind.v1") => &["endpoint_kind"],
        ("network", "network.profile.stream-window.v1") => {
            &["stream_window", "recv_window", "send_window"]
        }
        ("network", "network.profile.transport.v1") => &["transport_family"],
        ("network", "network.profile.protocol.v1") => {
            &["protocol_kind", "protocol_version", "protocol_header_bytes"]
        }
        _ => &[],
    }
}

fn detect_matched_support_usage(
    module: &NirModule,
    domain_family: &str,
) -> (Vec<String>, Vec<String>) {
    let mut surfaces = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for function in &module.functions {
        for stmt in &function.body {
            collect_support_usage_stmt(stmt, domain_family, &mut surfaces, &mut slots);
        }
    }
    (
        surfaces.into_iter().collect::<Vec<_>>(),
        slots.into_iter().collect::<Vec<_>>(),
    )
}

fn collect_support_usage_stmt(
    stmt: &NirStmt,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => collect_support_usage_expr(value, domain_family, surfaces, slots),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_support_usage_expr(condition, domain_family, surfaces, slots);
            for stmt in then_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
            for stmt in else_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
        }
        NirStmt::While { condition, body } => {
            collect_support_usage_expr(condition, domain_family, surfaces, slots);
            for stmt in body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
        }
        NirStmt::Break | NirStmt::Continue => {}
        NirStmt::Return(value) => {
            if let Some(value) = value {
                collect_support_usage_expr(value, domain_family, surfaces, slots);
            }
        }
    }
}

fn collect_support_usage_expr(
    expr: &NirExpr,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match expr {
        NirExpr::ShaderProfileTargetRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.target.v1".to_owned());
            slots.insert("target".to_owned());
        }
        NirExpr::ShaderProfileViewportRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.viewport.v1".to_owned());
            slots.insert("viewport".to_owned());
        }
        NirExpr::ShaderProfilePipelineRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pipeline.v1".to_owned());
            slots.insert("pipeline".to_owned());
        }
        NirExpr::ShaderProfileVertexCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("vertex_count".to_owned());
        }
        NirExpr::ShaderProfileInstanceCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("instance_count".to_owned());
        }
        NirExpr::ShaderProfilePacketColorSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_color_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketSpeedSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_speed_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketRadiusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_radius_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketTagRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-tag.v1".to_owned());
            slots.insert("packet_tag".to_owned());
        }
        NirExpr::ShaderProfileMaterialModeRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.material-mode.v1".to_owned());
            slots.insert("material_mode".to_owned());
        }
        NirExpr::ShaderProfilePassKindRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pass-kind.v1".to_owned());
            slots.insert("pass_kind".to_owned());
        }
        NirExpr::ShaderProfilePacketFieldCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-field-count.v1".to_owned());
            slots.insert("packet_field_count".to_owned());
        }
        NirExpr::ShaderProfileColorSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.color.v1".to_owned());
        }
        NirExpr::ShaderProfileSpeedSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.speed.v1".to_owned());
        }
        NirExpr::ShaderProfileRadiusSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.radius.v1".to_owned());
        }
        NirExpr::ShaderProfilePacket { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet.v1".to_owned());
        }
        NirExpr::ShaderProfileRender { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.render.v1".to_owned());
        }
        NirExpr::ShaderDrawInstanced { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw.v1".to_owned());
        }
        NirExpr::ShaderInlineWgsl { .. } if domain_family == "shader" => {
            surfaces.insert("shader.inline.wgsl.v1".to_owned());
        }
        NirExpr::DataProfileBindCoreRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::DataProfileWindowOffsetRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("window_offset".to_owned());
        }
        NirExpr::DataProfileUplinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("uplink_len".to_owned());
        }
        NirExpr::DataProfileDownlinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("downlink_len".to_owned());
        }
        NirExpr::DataProfileHandleTableRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.handle-table.v1".to_owned());
            slots.insert("handle_table".to_owned());
        }
        NirExpr::DataProfileMarkerRef { tag, .. } if domain_family == "data" => {
            let (surface, slot) = match tag.as_str() {
                "cpu_to_shader" | "shader_to_cpu" => {
                    ("data.profile.sync-markers.v1", format!("marker:{tag}"))
                }
                "uplink_pipe" | "downlink_pipe" => {
                    ("data.profile.pipe-markers.v1", format!("marker:{tag}"))
                }
                "uplink_pipe_class" | "downlink_pipe_class" => {
                    ("data.profile.pipe-class.v1", format!("marker:{tag}"))
                }
                "uplink_payload_class" | "downlink_payload_class" => {
                    ("data.profile.payload-class.v1", format!("marker:{tag}"))
                }
                "uplink_payload_shape" | "downlink_payload_shape" => {
                    ("data.profile.payload-shape.v1", format!("marker:{tag}"))
                }
                "uplink_window_policy" | "downlink_window_policy" => {
                    ("data.profile.window-policy.v1", format!("marker:{tag}"))
                }
                _ => ("data.profile.sync-markers.v1", format!("marker:{tag}")),
            };
            surfaces.insert(surface.to_owned());
            slots.insert(slot);
        }
        NirExpr::NetworkProfileBindCoreRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::NetworkProfileEndpointKindRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.endpoint-kind.v1".to_owned());
            slots.insert("endpoint_kind".to_owned());
        }
        NirExpr::NetworkProfileTransportFamilyRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.transport.v1".to_owned());
            slots.insert("transport_family".to_owned());
        }
        NirExpr::NetworkProfileLocalPortRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.accept.v1".to_owned());
            slots.insert("local_port".to_owned());
        }
        NirExpr::NetworkProfileRemotePortRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.connect.v1".to_owned());
            slots.insert("remote_port".to_owned());
        }
        NirExpr::NetworkProfileConnectTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("connect_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileReadTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("read_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileWriteTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("write_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileTimeoutBudgetRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("timeout_budget".to_owned());
        }
        NirExpr::NetworkProfileRetryBudgetRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.retry.v1".to_owned());
            slots.insert("retry_budget".to_owned());
        }
        NirExpr::NetworkProfileStreamWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.stream-window.v1".to_owned());
            slots.insert("stream_window".to_owned());
        }
        NirExpr::NetworkProfileRecvWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.recv.v1".to_owned());
            slots.insert("recv_window".to_owned());
        }
        NirExpr::NetworkProfileSendWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.send.v1".to_owned());
            slots.insert("send_window".to_owned());
        }
        NirExpr::NetworkProfileProtocolKindRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_kind".to_owned());
        }
        NirExpr::NetworkProfileProtocolVersionRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_version".to_owned());
        }
        NirExpr::NetworkProfileProtocolHeaderBytesRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_header_bytes".to_owned());
        }
        NirExpr::KernelProfileBindCoreRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::KernelProfileQueueDepthRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.queue-depth.v1".to_owned());
            slots.insert("queue_depth".to_owned());
        }
        NirExpr::KernelProfileBatchLanesRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.batch-lanes.v1".to_owned());
            slots.insert("batch_lanes".to_owned());
        }
        NirExpr::DataProfileSendUplink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.uplink.v1".to_owned());
        }
        NirExpr::DataProfileSendDownlink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.downlink.v1".to_owned());
        }
        _ => {}
    }

    walk_child_exprs(expr, &mut |child| {
        collect_support_usage_expr(child, domain_family, surfaces, slots);
    });
}

fn walk_child_exprs(expr: &NirExpr, f: &mut dyn FnMut(&NirExpr)) {
    match expr {
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => f(inner),
        NirExpr::AllocNode { value, next } => {
            f(value);
            f(next);
        }
        NirExpr::AllocBuffer { len, fill } => {
            f(len);
            f(fill);
        }
        NirExpr::LoadAt { buffer, index } => {
            f(buffer);
            f(index);
        }
        NirExpr::StoreValue { target, value } => {
            f(target);
            f(value);
        }
        NirExpr::StoreNext { target, next } => {
            f(target);
            f(next);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            f(buffer);
            f(index);
            f(value);
        }
        NirExpr::DataResult { value: input, .. }
        | NirExpr::NetworkResult { value: input, .. }
        | NirExpr::ShaderResult { value: input, .. }
        | NirExpr::KernelResult { value: input, .. } => f(input),
        NirExpr::DataReadWindow { window, index } => {
            f(window);
            f(index);
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            f(window);
            f(index);
            f(value);
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            f(input);
            f(offset);
            f(len);
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            f(base);
            f(delta);
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
            ..
        } => {
            f(color);
            f(speed);
            f(radius);
            if let Some(accent) = accent {
                f(accent);
            }
            if let Some(toggle_state) = toggle_state {
                f(toggle_state);
            }
            if let Some(focus_index) = focus_index {
                f(focus_index);
            }
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            f(delta);
            f(scale);
            f(base);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. }
        | NirExpr::FieldAccess { base: input, .. } => f(input),
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::Call { args, .. } => {
            for arg in args {
                f(arg);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            f(task);
            f(limit);
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            f(receiver);
            for arg in args {
                f(arg);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                f(value);
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            f(lhs);
            f(rhs);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            f(target);
            f(pipeline);
            f(viewport);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            f(pass);
            f(packet);
            f(vertex_count);
            f(instance_count);
        }
        _ => {}
    }
}

pub fn validate_unit_binding(
    manifests: &[NustarPackageManifest],
    domain: &str,
    unit: &str,
) -> Result<(), String> {
    let manifest = manifests
        .iter()
        .find(|manifest| manifest.domain_family == domain)
        .ok_or_else(|| format!("no nustar manifest loaded for mod domain `{domain}`"))?;

    if manifest.unit_types.is_empty() {
        return Ok(());
    }

    if manifest
        .unit_types
        .iter()
        .any(|candidate| candidate == unit)
    {
        return Ok(());
    }

    Err(format!(
        "unit `{unit}` is not registered by nustar package `{}` for mod domain `{domain}`",
        manifest.package_id
    ))
}

pub fn validate_manifest_abi(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<(), String> {
    if manifest
        .abi_profiles
        .iter()
        .any(|profile| profile == required_abi)
    {
        return Ok(());
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare required ABI `{}`; declared ABI profiles: {}",
        manifest.package_id,
        manifest.domain_family,
        required_abi,
        if manifest.abi_profiles.is_empty() {
            "<none>".to_owned()
        } else {
            manifest.abi_profiles.join(", ")
        }
    ))
}

pub fn registered_abi_target(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        if abi.trim() != required_abi {
            continue;
        }
        return parse_registered_abi_target(required_abi, fields, manifest, raw);
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare abi target metadata for `{}`",
        manifest.package_id, manifest.domain_family, required_abi
    ))
}

pub fn registered_abi_target_for_clang(
    manifest: &NustarPackageManifest,
    clang_target: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    let mut matches = Vec::new();
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        let target = parse_registered_abi_target(abi.trim(), fields, manifest, raw)?;
        if target.clang_target == clang_target {
            matches.push(target);
        }
    }
    matches.into_iter().next().ok_or_else(|| {
        format!(
            "nustar package `{}` for domain `{}` does not register clang target `{}` in `abi_targets`",
            manifest.package_id, manifest.domain_family, clang_target
        )
    })
}

pub fn used_ops_for_domain(module: &YirModule, domain_family: &str) -> Vec<String> {
    let mut ops = module
        .nodes
        .iter()
        .filter(|node| node.op.module == domain_family)
        .map(|node| node.op.full_name())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    ops.sort();
    ops
}

pub fn validate_abi_capabilities(
    manifest: &NustarPackageManifest,
    required_abi: &str,
    used_surfaces: &[String],
    used_ops: &[String],
) -> Result<(), String> {
    if manifest.abi_capabilities.is_empty() {
        return Ok(());
    }

    let mut surface_allowed = BTreeSet::new();
    let mut op_allowed = BTreeSet::new();
    let mut saw_required_abi = false;
    for raw in &manifest.abi_capabilities {
        let Some((abi, caps)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; expected `abi:kind:value[|kind:value...]`",
                manifest.package_id, raw
            ));
        };
        if abi.trim().is_empty() {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; ABI id must not be empty",
                manifest.package_id, raw
            ));
        }
        let abi_matches = abi.trim() == required_abi;
        if !abi_matches {
            continue;
        }
        saw_required_abi = true;
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(value) = cap.strip_prefix("surface:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `surface:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                surface_allowed.insert(value.to_owned());
            } else if let Some(value) = cap.strip_prefix("op:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `op:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                op_allowed.insert(value.to_owned());
            } else {
                return Err(format!(
                    "nustar package `{}` has invalid abi_capabilities capability `{}` in `{}`; expected `surface:<pattern>` or `op:<pattern>`",
                    manifest.package_id, cap, raw
                ));
            }
        }
    }

    if !saw_required_abi {
        return Err(format!(
            "ABI `{}` of nustar package `{}` has no abi_capabilities mapping; add `{}:...` in manifest",
            required_abi, manifest.package_id, required_abi
        ));
    }

    if !surface_allowed.is_empty() && !surface_allowed.contains("*") {
        for surface in used_surfaces {
            if !surface_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, surface))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow support surface `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    surface,
                    surface_allowed
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    if !op_allowed.is_empty() && !op_allowed.contains("*") {
        for op in used_ops {
            if !op_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, op))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow op `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    op,
                    op_allowed.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }

    Ok(())
}

fn capability_matches(pattern: &str, actual: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return actual.starts_with(prefix);
    }
    pattern == actual
}

pub fn manifest_path(root: &Path, entry: &NustarPackageIndexEntry) -> PathBuf {
    root.join(&entry.manifest)
}

fn parse_index(source: &str, path: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let mut entries = Vec::new();
    let mut current = Vec::<String>::new();

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            if !current.is_empty() {
                entries.push(parse_index_entry(&current.join("\n"), path)?);
                current.clear();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        current.push(line.to_owned());
    }

    if !current.is_empty() {
        entries.push(parse_index_entry(&current.join("\n"), path)?);
    }

    entries.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(entries)
}

fn parse_index_entry(source: &str, path: &Path) -> Result<NustarPackageIndexEntry, String> {
    Ok(NustarPackageIndexEntry {
        package_id: parse_required_string(source, "package_id", path)?,
        manifest: parse_required_string(source, "manifest", path)?,
        domain_family: parse_required_string(source, "domain_family", path)?,
    })
}

fn parse_manifest(source: &str, path: &Path) -> Result<NustarPackageManifest, String> {
    let manifest_schema = parse_optional_string(source, "manifest_schema")
        .unwrap_or_else(|| "nustar-manifest-v1".to_owned());
    let package_id = parse_required_string(source, "package_id", path)?;
    let domain_family = parse_required_string(source, "domain_family", path)?;
    let frontend = parse_required_string(source, "frontend", path)?;
    let entry_crate = parse_required_string(source, "entry_crate", path)?;
    let ast_entry = parse_optional_string(source, "ast_entry")
        .unwrap_or_else(|| format!("{}.ast.bootstrap.v1", domain_family));
    let nir_entry = parse_optional_string(source, "nir_entry")
        .unwrap_or_else(|| format!("{}.nir.bootstrap.v1", domain_family));
    let yir_lowering_entry = parse_optional_string(source, "yir_lowering_entry")
        .unwrap_or_else(|| format!("{}.yir.lowering.v1", domain_family));
    let part_verify_entry = parse_optional_string(source, "part_verify_entry")
        .unwrap_or_else(|| format!("{}.verify.partial.v1", domain_family));
    let ast_surface = parse_optional_string_array(source, "ast_surface")
        .unwrap_or_else(|| vec![format!("{domain_family}.mod-ast.v1")]);
    let nir_surface = parse_optional_string_array(source, "nir_surface")
        .unwrap_or_else(|| vec![format!("nir.{domain_family}.surface.v1")]);
    let yir_lowering = parse_optional_string_array(source, "yir_lowering")
        .unwrap_or_else(|| vec![format!("yir.{domain_family}.lowering.v1")]);
    let part_verify = parse_optional_string_array(source, "part_verify")
        .unwrap_or_else(|| vec![format!("verify.{domain_family}.contract.v1")]);
    let binary_extension =
        parse_optional_string(source, "binary_extension").unwrap_or_else(|| "nustar".to_owned());
    let package_layout = parse_optional_string(source, "package_layout")
        .unwrap_or_else(|| "single-envelope".to_owned());
    let machine_abi_policy = parse_optional_string(source, "machine_abi_policy")
        .unwrap_or_else(|| "exact-match".to_owned());
    let abi_profiles = parse_optional_string_array(source, "abi_profiles").unwrap_or_default();
    let abi_capabilities =
        parse_optional_string_array(source, "abi_capabilities").unwrap_or_default();
    let abi_targets = parse_optional_string_array(source, "abi_targets").unwrap_or_default();
    let implementation_kinds = parse_optional_string_array(source, "implementation_kinds")
        .unwrap_or_else(|| vec!["native-stub".to_owned()]);
    let loader_entry = parse_optional_string(source, "loader_entry")
        .unwrap_or_else(|| "nustar.bootstrap.v1".to_owned());
    let loader_abi = parse_optional_string(source, "loader_abi")
        .unwrap_or_else(|| "nustar-loader-v1".to_owned());
    let host_ffi_surface =
        parse_optional_string_array(source, "host_ffi_surface").unwrap_or_default();
    let host_ffi_abis = parse_optional_string_array(source, "host_ffi_abis").unwrap_or_default();
    let host_ffi_bridge =
        parse_optional_string(source, "host_ffi_bridge").unwrap_or_else(|| "none".to_owned());
    let support_surface =
        parse_optional_string_array(source, "support_surface").unwrap_or_default();
    let support_profile_slots =
        parse_optional_string_array(source, "support_profile_slots").unwrap_or_default();
    let default_lanes = parse_optional_string_array(source, "default_lanes").unwrap_or_default();
    let clock_domain_id = parse_optional_string(source, "clock_domain_id")
        .unwrap_or_else(|| format!("{domain_family}.clock.local.v1"));
    let clock_kind =
        parse_optional_string(source, "clock_kind").unwrap_or_else(|| "local-monotonic".to_owned());
    let clock_epoch_kind = parse_optional_string(source, "clock_epoch_kind")
        .unwrap_or_else(|| "domain-epoch".to_owned());
    let clock_resolution =
        parse_optional_string(source, "clock_resolution").unwrap_or_else(|| "tick:1ns".to_owned());
    let clock_bridge_default =
        parse_optional_string(source, "clock_bridge_default").unwrap_or_else(|| "self".to_owned());
    let profiles = parse_string_array(source, "profiles", path)?;
    let resource_families = parse_string_array(source, "resource_families", path)?;
    let unit_types = parse_optional_string_array(source, "unit_types").unwrap_or_default();
    let lowering_targets = parse_string_array(source, "lowering_targets", path)?;
    let ops = parse_string_array(source, "ops", path)?;

    Ok(NustarPackageManifest {
        manifest_schema,
        package_id,
        domain_family,
        frontend,
        entry_crate,
        ast_entry,
        nir_entry,
        yir_lowering_entry,
        part_verify_entry,
        ast_surface,
        nir_surface,
        yir_lowering,
        part_verify,
        binary_extension,
        package_layout,
        machine_abi_policy,
        abi_profiles,
        abi_capabilities,
        abi_targets,
        implementation_kinds,
        loader_entry,
        loader_abi,
        host_ffi_surface,
        host_ffi_abis,
        host_ffi_bridge,
        support_surface,
        support_profile_slots,
        default_lanes,
        clock_domain_id,
        clock_kind,
        clock_epoch_kind,
        clock_resolution,
        clock_bridge_default,
        profiles,
        resource_families,
        unit_types,
        lowering_targets,
        ops,
    })
}

fn parse_registered_abi_target(
    abi: &str,
    fields: &str,
    manifest: &NustarPackageManifest,
    raw: &str,
) -> Result<RegisteredAbiTarget, String> {
    let mut host_adaptive = false;
    let mut machine_arch = None::<String>;
    let mut machine_os = None::<String>;
    let mut object_format = None::<String>;
    let mut calling_abi = None::<String>;
    let mut clang_target = None::<String>;
    let mut backend_family = None::<String>;
    for field in fields
        .split('|')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        let Some((key, value)) = field.split_once('=') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets field `{}` in `{}`; expected `key=value`",
                manifest.package_id, field, raw
            ));
        };
        let value = value.trim();
        if value == "host" {
            host_adaptive = true;
        }
        match key.trim() {
            "arch" => machine_arch = Some(resolve_host_adaptive_arch(value).to_owned()),
            "os" => machine_os = Some(resolve_host_adaptive_os(value).to_owned()),
            "object" => object_format = Some(resolve_host_adaptive_object(value).to_owned()),
            "calling" => calling_abi = Some(resolve_host_adaptive_calling(value).to_owned()),
            "clang" => clang_target = Some(resolve_host_adaptive_clang(value).to_owned()),
            "backend" => backend_family = Some(value.to_owned()),
            other => {
                return Err(format!(
                    "nustar package `{}` has invalid abi_targets key `{}` in `{}`; expected `arch`, `os`, `object`, `calling`, `clang`, or `backend`",
                    manifest.package_id, other, raw
                ));
            }
        }
    }
    Ok(RegisteredAbiTarget {
        abi: abi.to_owned(),
        machine_arch: machine_arch.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `arch=`",
                manifest.package_id, raw
            )
        })?,
        machine_os: machine_os.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `os=`",
                manifest.package_id, raw
            )
        })?,
        object_format: object_format.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `object=`",
                manifest.package_id, raw
            )
        })?,
        calling_abi: calling_abi.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `calling=`",
                manifest.package_id, raw
            )
        })?,
        clang_target: clang_target.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `clang=`",
                manifest.package_id, raw
            )
        })?,
        backend_family,
        host_adaptive,
    })
}

fn resolve_host_adaptive_arch(value: &str) -> &'static str {
    if value == "host" {
        host_arch()
    } else {
        match value {
            "arm64" => "arm64",
            "x86_64" => "x86_64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_os(value: &str) -> &'static str {
    if value == "host" {
        host_os()
    } else {
        match value {
            "darwin" => "darwin",
            "linux" => "linux",
            "windows" => "windows",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_object(value: &str) -> &'static str {
    if value == "host" {
        host_object_format()
    } else {
        match value {
            "mach-o" => "mach-o",
            "elf" => "elf",
            "coff" => "coff",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_calling(value: &str) -> &'static str {
    if value == "host" {
        host_calling_abi()
    } else {
        match value {
            "aapcs64-darwin" => "aapcs64-darwin",
            "aapcs64" => "aapcs64",
            "sysv64" => "sysv64",
            "win64" => "win64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_clang(value: &str) -> String {
    if value == "host" {
        host_clang_target()
    } else {
        value.to_owned()
    }
}

fn host_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_object_format() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mach-o",
        "linux" => "elf",
        "windows" => "coff",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_calling_abi() -> &'static str {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aapcs64-darwin",
        ("arm64", _) => "aapcs64",
        ("x86_64", "windows") => "win64",
        ("x86_64", _) => "sysv64",
        _ => "unknown",
    }
}

fn host_clang_target() -> String {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aarch64-apple-darwin".to_owned(),
        ("arm64", "linux") => "aarch64-unknown-linux-gnu".to_owned(),
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_owned(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_owned(),
        (arch, os) => format!("{arch}-unknown-{os}"),
    }
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid string value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_string_array(source: &str, key: &str, path: &Path) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid array value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest);
        }
    }
    None
}

fn parse_quoted(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        Some(trimmed[1..trimmed.len() - 1].to_owned())
    } else {
        None
    }
}

fn parse_array(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    for part in inner.split(',') {
        items.push(parse_quoted(part.trim())?);
    }
    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{
        ProjectAbiRequirement, ProjectAbiResolution, ProjectCompilationPlan,
        ProjectExchangeOrganization, ProjectOrganization, ProjectOutputIntent,
        ProjectSyntheticInput,
    };

    fn test_project_plan(domain: &str, abi: &str) -> ProjectCompilationPlan {
        ProjectCompilationPlan {
            project_name: "registry-check-demo".to_owned(),
            entry: "main.ns".to_owned(),
            organization: ProjectOrganization {
                entry: "main.ns".to_owned(),
                domains: vec![domain.to_owned()],
                modules: Vec::new(),
                links: Vec::new(),
            },
            exchanges: ProjectExchangeOrganization { routes: Vec::new() },
            abi_resolution: ProjectAbiResolution {
                requirements: vec![ProjectAbiRequirement {
                    domain: domain.to_owned(),
                    abi: abi.to_owned(),
                }],
                explicit: true,
            },
            dependencies: Vec::new(),
            synthetic_input: ProjectSyntheticInput {
                kind: "test".to_owned(),
                path: PathBuf::from("main.ns"),
            },
            output_intents: Vec::<ProjectOutputIntent>::new(),
            effective_input_path: PathBuf::from("main.ns"),
        }
    }
    use crate::pipeline;

    const DATA_BINDING_SOURCE: &str = r#"
use data FabricPlane;

mod cpu Main {
  fn capture_data_profile_summary() -> i64 {
    let bind_core: Unit = data_profile_bind_core("FabricPlane");
    let window_offset: i64 = data_profile_window_offset("FabricPlane");
    let uplink_len: i64 = data_profile_uplink_len("FabricPlane");
    let downlink_len: i64 = data_profile_downlink_len("FabricPlane");
    let _ = bind_core;
    return window_offset + uplink_len + downlink_len;
  }

  fn main() {
    print(capture_data_profile_summary());
  }
}
"#;

    fn binding_plan_from_source(source: &str) -> NustarBindingPlan {
        let artifacts = pipeline::compile_source(source).expect("source should compile");
        let declared_used_units = artifacts
            .ast
            .uses
            .iter()
            .map(|item| (item.domain.clone(), item.unit.clone()))
            .collect::<Vec<_>>();
        let declared_externs = artifacts
            .ast
            .externs
            .iter()
            .map(|item| (item.abi.clone(), item.name.clone()))
            .chain(
                artifacts
                    .ast
                    .extern_interfaces
                    .iter()
                    .flat_map(|interface| {
                        interface.methods.iter().map(move |method| {
                            (
                                method.abi.clone(),
                                format!("{}__{}", interface.name, method.name),
                            )
                        })
                    }),
            )
            .collect::<Vec<_>>();

        let registry_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("nustar-packages");
        plan_bindings(
            &registry_root,
            &artifacts.nir,
            &artifacts.yir,
            &artifacts.ast.domain,
            &artifacts.ast.unit,
            &declared_used_units,
            &declared_externs,
        )
        .expect("binding plan should resolve")
    }

    fn cpu_manifest_with_host_target() -> NustarPackageManifest {
        NustarPackageManifest {
            manifest_schema: "nustar-manifest-v1".to_owned(),
            package_id: "official.cpu".to_owned(),
            domain_family: "cpu".to_owned(),
            frontend: "nustar-cpu".to_owned(),
            entry_crate: "crates/yir-domain-cpu".to_owned(),
            ast_entry: "cpu.ast.bootstrap.v1".to_owned(),
            nir_entry: "cpu.nir.bootstrap.v1".to_owned(),
            yir_lowering_entry: "cpu.yir.lowering.v1".to_owned(),
            part_verify_entry: "cpu.verify.partial.v1".to_owned(),
            ast_surface: vec!["cpu.mod-ast.v1".to_owned()],
            nir_surface: vec!["nir.cpu.surface.v1".to_owned()],
            yir_lowering: vec!["yir.cpu.lowering.v1".to_owned()],
            part_verify: vec!["verify.cpu.contract.v1".to_owned()],
            binary_extension: "nustar".to_owned(),
            package_layout: "single-envelope".to_owned(),
            machine_abi_policy: "exact-match".to_owned(),
            abi_profiles: vec!["cpu.host.v1".to_owned()],
            abi_capabilities: vec!["cpu.host.v1:op:cpu.*".to_owned()],
            abi_targets: vec![
                "cpu.host.v1:arch=host|os=host|object=host|calling=host|clang=host".to_owned(),
            ],
            implementation_kinds: vec!["native-stub".to_owned()],
            loader_entry: "nustar.bootstrap.v1".to_owned(),
            loader_abi: "nustar-loader-v1".to_owned(),
            host_ffi_surface: Vec::new(),
            host_ffi_abis: Vec::new(),
            host_ffi_bridge: "none".to_owned(),
            support_surface: Vec::new(),
            support_profile_slots: Vec::new(),
            default_lanes: Vec::new(),
            clock_domain_id: "cpu.clock.host.v1".to_owned(),
            clock_kind: "host-monotonic".to_owned(),
            clock_epoch_kind: "host-epoch".to_owned(),
            clock_resolution: "cpu.tick_i64".to_owned(),
            clock_bridge_default: "global->monotonic:bridge".to_owned(),
            profiles: vec!["aot".to_owned()],
            resource_families: vec!["cpu".to_owned()],
            unit_types: vec!["Main".to_owned()],
            lowering_targets: vec!["llvm".to_owned()],
            ops: vec!["cpu.const".to_owned()],
        }
    }

    #[test]
    fn registered_abi_target_expands_host_adaptive_contract() {
        let manifest = cpu_manifest_with_host_target();
        let target = registered_abi_target(&manifest, "cpu.host.v1").unwrap();
        assert_eq!(target.machine_arch, host_arch());
        assert_eq!(target.machine_os, host_os());
        assert_eq!(target.object_format, host_object_format());
        assert_eq!(target.calling_abi, host_calling_abi());
        assert_eq!(target.clang_target, host_clang_target());
        assert!(target.host_adaptive);
    }

    #[test]
    fn registered_abi_target_preserves_backend_family() {
        let mut manifest = cpu_manifest_with_host_target();
        manifest.abi_profiles = vec!["cpu.backend.v1".to_owned()];
        manifest.abi_capabilities = vec!["cpu.backend.v1:op:cpu.*".to_owned()];
        manifest.abi_targets = vec![
            "cpu.backend.v1:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin|backend=metal".to_owned(),
        ];
        let target = registered_abi_target(&manifest, "cpu.backend.v1").unwrap();
        assert_eq!(target.backend_family.as_deref(), Some("metal"));
        assert!(!target.host_adaptive);
    }

    #[test]
    fn network_manifest_skeleton_is_registered() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        assert_eq!(manifest.package_id, "official.network");
        assert_eq!(manifest.clock_domain_id, "network.clock.io.v1");
        assert_eq!(manifest.clock_kind, "io-monotonic");
        assert!(manifest
            .support_surface
            .contains(&"network.profile.bind-core.v1".to_owned()));
        assert!(manifest
            .support_surface
            .contains(&"network.profile.connect.v1".to_owned()));
        assert!(manifest
            .support_surface
            .contains(&"network.profile.stream-window.v1".to_owned()));
        assert!(manifest
            .support_surface
            .contains(&"network.profile.transport.v1".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"bind_core".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"endpoint_kind".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"transport_family".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"retry_budget".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"stream_window".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"protocol_kind".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"network.send=tx".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"network.recv=rx".to_owned()));
    }

    #[test]
    fn cpu_manifest_contract_is_registered() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
        assert_eq!(manifest.package_id, "official.cpu");
        assert_eq!(manifest.loader_abi, "nustar-loader-v1");
        assert_eq!(manifest.loader_entry, "nustar.bootstrap.v1");
        assert_eq!(manifest.machine_abi_policy, "exact-match");
        assert_eq!(manifest.clock_domain_id, "cpu.clock.host.v1");
        assert_eq!(manifest.clock_kind, "host-monotonic");
        assert_eq!(manifest.clock_bridge_default, "global->monotonic:bridge");
        assert!(manifest
            .host_ffi_surface
            .contains(&"cpu.host-ffi.nurs.v1".to_owned()));
        assert!(manifest
            .host_ffi_surface
            .contains(&"cpu.host-ffi.c-bridge.v1".to_owned()));
        assert!(manifest.host_ffi_abis.contains(&"nurs".to_owned()));
        assert!(manifest.host_ffi_abis.contains(&"c".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"cpu.window=main".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"cpu.alloc_node=mem".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"cpu.instantiate_unit=main".to_owned()));
        assert!(manifest
            .abi_profiles
            .contains(&"cpu.arm64.apple_aapcs64".to_owned()));
    }

    #[test]
    fn scheduler_summary_uses_manifest_clock_and_domain_samples() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        let summary = scheduler_summary(&manifest);
        assert_eq!(summary.clock.domain_id, "network.clock.io.v1");
        assert_eq!(summary.clock.kind, "io-monotonic");
        assert_eq!(
            summary.sample_navigation.as_deref(),
            Some(
                "result_ladder -> transport_split_ladder -> transport_summary_ladder -> summary_classes"
            )
        );
        assert!(summary
            .result_samples
            .as_deref()
            .unwrap_or_default()
            .contains("network_result_profile_demo"));
        assert!(summary
            .transport_samples
            .as_deref()
            .unwrap_or_default()
            .contains("network_transport_result_policy_split_demo"));
    }

    #[test]
    fn capability_summary_tracks_support_and_clock_contract() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        let summary = capability_summary(&manifest);
        assert!(summary
            .support_surface
            .contains(&"network.profile.transport.v1".to_owned()));
        assert!(summary
            .support_profile_slots
            .contains(&"protocol_kind".to_owned()));
        assert!(summary
            .default_lanes
            .contains(&"network.send=tx".to_owned()));
        assert_eq!(summary.clock.domain_id, "network.clock.io.v1");
        assert_eq!(summary.clock.bridge_default, "global->io:bridge");
    }

    #[test]
    fn execution_summary_derives_minimum_execution_skeleton() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
        let summary = execution_summary(&manifest);
        assert_eq!(summary.skeleton_version, "nustar-execution-skeleton-v1");
        assert_eq!(summary.function_kind, "function-node");
        assert_eq!(summary.graph_kind, "function-graph");
        assert_eq!(summary.execution_domain, "kernel");
        assert_eq!(summary.default_time_mode, "logical");
        assert_eq!(summary.contract_family, "nustar.kernel");
        assert!(summary.lowering_targets.contains(&"coreml".to_owned()));
    }

    #[test]
    fn domain_contract_collects_registered_runtime_and_loader_facts() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        let contract = domain_contract(&manifest);
        assert_eq!(contract.contract_schema, NUSTAR_DOMAIN_CONTRACT_SCHEMA);
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_ABI.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER.to_owned()));
        assert!(contract
            .extension_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned()));
        assert_eq!(contract.package_id, "official.network");
        assert_eq!(contract.domain_family, "network");
        assert_eq!(contract.frontend, "nustar-network");
        assert_eq!(contract.loader_abi, "nustar-loader-v1");
        assert_eq!(contract.loader_entry, "nustar.bootstrap.v1");
        assert_eq!(contract.machine_abi_policy, "exact-match");
        assert!(contract
            .abi_profiles
            .contains(&"network.socket.v1".to_owned()));
        assert!(contract
            .capability
            .support_surface
            .contains(&"network.profile.transport.v1".to_owned()));
        assert_eq!(contract.execution.execution_domain, "network");
        assert_eq!(contract.execution.contract_family, "nustar.network");
        assert_eq!(contract.scheduler.clock.domain_id, "network.clock.io.v1");
        assert!(contract
            .std_net
            .recipe_samples
            .as_deref()
            .unwrap_or_default()
            .contains("net_http_client_recipe"));
        let json = domain_contract_json(&contract);
        assert!(json.contains("\"execution_skeleton_version\":\"nustar-execution-skeleton-v1\""));
        assert!(json.contains("\"execution_contract_family\":\"nustar.network\""));
    }

    #[test]
    fn std_net_summary_is_owned_by_registry() {
        let summary = std_net_summary("network");
        assert_eq!(
            summary.sample_navigation.as_deref(),
            Some(
                "profile_core -> transport_edge -> syscall_edge -> socket_edge -> control_edge -> protocol_edge -> http_edge -> result_spine -> task_spine -> session"
            )
        );
        assert!(summary
            .recipe_samples
            .as_deref()
            .unwrap_or_default()
            .contains("net_http_client_recipe"));
    }

    #[test]
    fn load_registered_domains_covers_all_indexed_nustar_modules() {
        let registrations = load_registered_domains(Path::new("nustar-packages")).unwrap();
        let domains = registrations
            .iter()
            .map(|item| item.domain_family.as_str())
            .collect::<Vec<_>>();
        assert_eq!(domains, vec!["cpu", "data", "kernel", "network", "shader"]);
        let network = registrations
            .iter()
            .find(|item| item.domain_family == "network")
            .unwrap();
        assert!(network
            .manifest_path
            .ends_with("nustar-packages/network.toml"));
        assert_eq!(
            network.contract.contract_schema,
            NUSTAR_DOMAIN_CONTRACT_SCHEMA
        );
        assert!(network
            .contract
            .extension_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned()));
        assert!(!network.ops.is_empty());
    }

    #[test]
    fn ensure_project_domain_registry_valid_accepts_registered_abi() {
        let plan = test_project_plan("network", "network.socket.macos.arm64.v1");
        let checks = validate_project_domain_registry(&plan);
        assert!(checks.iter().all(|check| check.issues.is_empty()));
        let network = checks
            .iter()
            .find(|check| check.domain == "network")
            .unwrap();
        assert_eq!(network.issue_count(), 0);
        assert!(network.summary_line().contains(": ok"));
        ensure_project_domain_registry_valid(&plan).unwrap();
    }

    #[test]
    fn ensure_project_domain_registry_valid_rejects_unknown_abi() {
        let plan = test_project_plan("network", "network.socket.unknown.v1");
        let checks = validate_project_domain_registry(&plan);
        let network = checks
            .iter()
            .find(|check| check.domain == "network")
            .unwrap();
        assert!(network
            .issues
            .iter()
            .any(|issue| issue.kind == ProjectDomainRegistryIssueKind::AbiNotRegistered));
        assert!(network
            .issues
            .iter()
            .any(|issue| issue.kind.code() == "NRG003"));
        assert!(network.summary_line().contains("NRG003 abi_not_registered"));
        let error = ensure_project_domain_registry_valid(&plan).unwrap_err();
        assert!(error.contains("project domain registry validation failed"));
        assert!(error.contains("network"));
        assert!(error.contains("network.socket.unknown.v1"));
        assert!(error.contains("NRG003"));
        assert!(error.contains("abi_not_registered"));
    }

    #[test]
    fn project_domain_registry_check_renderers_expose_codes_and_issue_counts() {
        let plan = test_project_plan("network", "network.socket.unknown.v1");
        let check = validate_project_domain_registry(&plan)
            .into_iter()
            .find(|check| check.domain == "network")
            .expect("network check");
        let json = project_domain_registry_check_json(&check);
        assert!(json.contains("\"domain\":\"network\""));
        assert!(json.contains("\"code\":\"NRG003\""));
        assert!(json.contains("\"kind\":\"abi_not_registered\""));
        let lines = render_project_domain_registry_check_lines(&check);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("issues=1"));
        assert!(lines
            .iter()
            .any(|line| line.contains("NRG003 abi_not_registered")));
    }

    #[test]
    fn network_binding_plan_detects_profile_surfaces_and_slots() {
        let source = r#"
use network NetworkUnit;

mod cpu Main {
  fn capture_network_profile_summary() -> i64 {
    let bind_core: i64 = network_profile_bind_core("NetworkUnit");
    let endpoint_kind: i64 = network_profile_endpoint_kind("NetworkUnit");
    let timeout_budget: i64 = network_profile_timeout_budget("NetworkUnit");
    let retry_budget: i64 = network_profile_retry_budget("NetworkUnit");
    let stream_window: i64 = network_profile_stream_window("NetworkUnit");
    let recv_window: i64 = network_profile_recv_window("NetworkUnit");
    let send_window: i64 = network_profile_send_window("NetworkUnit");
    return bind_core + endpoint_kind + timeout_budget + retry_budget + stream_window + recv_window + send_window;
  }

  fn main() {
    print(capture_network_profile_summary());
  }
}
"#;
        let plan = binding_plan_from_source(source);

        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "network")
            .expect("network binding should be present");

        for surface in [
            "network.profile.bind-core.v1",
            "network.profile.endpoint-kind.v1",
            "network.profile.timeout.v1",
            "network.profile.retry.v1",
            "network.profile.stream-window.v1",
            "network.profile.recv.v1",
            "network.profile.send.v1",
        ] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched network surface `{surface}`"
            );
        }

        for slot in [
            "bind_core",
            "endpoint_kind",
            "timeout_budget",
            "retry_budget",
            "stream_window",
            "recv_window",
            "send_window",
        ] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched network slot `{slot}`"
            );
            assert!(
                binding
                    .covered_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected covered network slot `{slot}`"
            );
        }
    }

    #[test]
    fn data_binding_plan_detects_profile_surfaces_and_slots() {
        let plan = binding_plan_from_source(DATA_BINDING_SOURCE);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "data")
            .expect("data binding should be present");
        for surface in ["data.profile.bind-core.v1", "data.profile.window-layout.v1"] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched data surface `{surface}`"
            );
        }
        for slot in ["bind_core", "window_offset", "uplink_len", "downlink_len"] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched data slot `{slot}`"
            );
        }
    }

    #[test]
    fn kernel_binding_plan_detects_profile_surfaces_and_slots() {
        let source = r#"
use kernel KernelUnit;

mod cpu Main {
  fn capture_kernel_profile_summary() -> i64 {
    let bind_core: i64 = kernel_profile_bind_core("KernelUnit");
    let queue_depth: i64 = kernel_profile_queue_depth("KernelUnit");
    let batch_lanes: i64 = kernel_profile_batch_lanes("KernelUnit");
    return bind_core + queue_depth + batch_lanes;
  }

  fn main() {
    print(capture_kernel_profile_summary());
  }
}
"#;
        let plan = binding_plan_from_source(source);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "kernel")
            .expect("kernel binding should be present");
        for surface in [
            "kernel.profile.bind-core.v1",
            "kernel.profile.queue-depth.v1",
            "kernel.profile.batch-lanes.v1",
        ] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched kernel surface `{surface}`"
            );
        }
        for slot in ["bind_core", "queue_depth", "batch_lanes"] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched kernel slot `{slot}`"
            );
        }
    }

    #[test]
    fn shader_binding_plan_detects_profile_surfaces_and_slots() {
        let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn capture_shader_profile_summary() -> i64 {
    let target: Target = shader_profile_target("SurfaceShader");
    let viewport: Viewport = shader_profile_viewport("SurfaceShader");
    let pipeline: Pipeline = shader_profile_pipeline("SurfaceShader");
    let vertex_count: i64 = shader_profile_vertex_count("SurfaceShader");
    let instance_count: i64 = shader_profile_instance_count("SurfaceShader");
    let _ = target;
    let _ = viewport;
    let _ = pipeline;
    return vertex_count + instance_count;
  }

  fn main() {
    print(capture_shader_profile_summary());
  }
}
"#;
        let plan = binding_plan_from_source(source);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "shader")
            .expect("shader binding should be present");
        for surface in [
            "shader.profile.target.v1",
            "shader.profile.viewport.v1",
            "shader.profile.pipeline.v1",
            "shader.profile.draw-budget.v1",
        ] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched shader surface `{surface}`"
            );
        }
        for slot in [
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
        ] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched shader slot `{slot}`"
            );
        }
    }
}
