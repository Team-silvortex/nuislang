use crate::{
    final_executable_elf_dynamic_provider::{
        dependency_matches_registered_provider, elf_version_name_hash,
        matching_dynamic_resolver_providers, provider_target_key,
        registered_dynamic_symbol_version, validate_dynamic_resolver_provider_registry,
        DynamicResolverProvider,
    },
    final_executable_elf_materialization::application::platform::{
        application::{
            bind_audit_hash, ElfAmd64PlatformDynamicBindRecord,
            ElfAmd64PlatformPatchApplicationReport,
        },
        ElfAmd64PlatformStructurePlanReport,
    },
};
use nuisc::linker::{LinkPlan, LinkPlanHostFfiEntry, LinkPlanHostFfiFootprint};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

pub(crate) const ELF_AMD64_DYNAMIC_DEPENDENCY_PLAN_CONTRACT: &str =
    "nuis-nsld-elf-amd64-dynamic-dependency-plan-v1";
pub(crate) use crate::final_executable_elf_dynamic_provider::ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT;

const HOST_FFI_POLICY: &str = "signature-whitelist-required";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64DynamicDependencyPlan {
    pub(crate) dependency_id: String,
    pub(crate) provider_id: String,
    pub(crate) provider_target_key: String,
    pub(crate) host_ffi_abi: String,
    pub(crate) interpreter_identity: String,
    pub(crate) interpreter_path: String,
    pub(crate) dependency_identity: String,
    pub(crate) needed_name: String,
    pub(crate) symbol_version_policy: String,
    pub(crate) resolver_identity: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64DynamicSymbolPlan {
    pub(crate) binding_id: String,
    pub(crate) target_key: String,
    pub(crate) target_symbol: String,
    pub(crate) dynamic_symbol_index: usize,
    pub(crate) platform_bind_audit_hash: String,
    pub(crate) host_ffi_abi: String,
    pub(crate) signature_pattern: String,
    pub(crate) signature_hash: String,
    pub(crate) whitelist_policy: String,
    pub(crate) memory_capabilities: Vec<String>,
    pub(crate) dependency_audit_hash: String,
    pub(crate) symbol_version_identity: String,
    pub(crate) symbol_version_name: String,
    pub(crate) symbol_version_index: u16,
    pub(crate) symbol_version_hash: u32,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64DynamicDependencyPlanReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) plan_ready: bool,
    pub(crate) plan_hash: String,
    pub(crate) registry_contract: &'static str,
    pub(crate) registry_hash: String,
    pub(crate) target_key: String,
    pub(crate) host_ffi_footprint_hash: String,
    pub(crate) platform_structure_plan_hash: String,
    pub(crate) platform_application_ledger_hash: String,
    pub(crate) unresolved_symbol_count: usize,
    pub(crate) dynamic_bind_count: usize,
    pub(crate) resolved_binding_count: usize,
    pub(crate) issues: Vec<String>,
    pub(crate) dependencies: Vec<ElfAmd64DynamicDependencyPlan>,
    pub(crate) bindings: Vec<ElfAmd64DynamicSymbolPlan>,
}

impl ElfAmd64DynamicDependencyPlanReport {
    pub(crate) fn canonical_plan(&self) -> String {
        canonical_plan_components(
            self.contract,
            &self.status,
            self.plan_ready,
            self.registry_contract,
            &self.registry_hash,
            &self.target_key,
            &self.host_ffi_footprint_hash,
            &self.platform_structure_plan_hash,
            &self.platform_application_ledger_hash,
            self.unresolved_symbol_count,
            self.dynamic_bind_count,
            self.resolved_binding_count,
            &self.issues,
            &self.dependencies,
            &self.bindings,
        )
    }
}

pub(crate) fn build_elf_amd64_dynamic_dependency_plan(
    link_plan: &LinkPlan,
    unresolved_symbols: &[String],
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_application: &ElfAmd64PlatformPatchApplicationReport,
) -> Result<ElfAmd64DynamicDependencyPlanReport, String> {
    validate_upstream(unresolved_symbols, platform_plan, platform_application)?;
    let registry_hash = validate_dynamic_resolver_provider_registry()?;
    let target_key = target_key(link_plan);
    let host_ffi_footprint_hash = host_ffi_footprint_hash(&link_plan.host_ffi);
    let mut issues = Vec::new();
    let mut dependencies = Vec::new();
    let mut dependency_indexes = BTreeMap::new();
    let mut version_indexes = BTreeMap::new();
    let mut bindings = Vec::new();

    if !unresolved_symbols.is_empty() {
        issues.extend(validate_host_ffi_footprint(&link_plan.host_ffi));
        let footprint_valid = issues.is_empty();
        for bind in &platform_application.dynamic_bind_records {
            resolve_dynamic_bind(
                link_plan,
                bind,
                &link_plan.host_ffi.entries,
                footprint_valid,
                &mut dependencies,
                &mut dependency_indexes,
                &mut version_indexes,
                &mut bindings,
                &mut issues,
            );
        }
    }
    issues.sort();
    issues.dedup();

    let plan_ready =
        issues.is_empty() && bindings.len() == platform_application.dynamic_bind_records.len();
    let status = dependency_plan_status(unresolved_symbols.len(), plan_ready);
    let mut report = ElfAmd64DynamicDependencyPlanReport {
        contract: ELF_AMD64_DYNAMIC_DEPENDENCY_PLAN_CONTRACT,
        status: status.to_owned(),
        plan_ready,
        plan_hash: String::new(),
        registry_contract: ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT,
        registry_hash,
        target_key,
        host_ffi_footprint_hash,
        platform_structure_plan_hash: platform_plan.plan_hash.clone(),
        platform_application_ledger_hash: platform_application.application_ledger_hash.clone(),
        unresolved_symbol_count: unresolved_symbols.len(),
        dynamic_bind_count: platform_application.dynamic_bind_records.len(),
        resolved_binding_count: bindings.len(),
        issues,
        dependencies,
        bindings,
    };
    report.plan_hash = crate::fnv1a64_hex(report.canonical_plan().as_bytes());
    validate_elf_amd64_dynamic_dependency_plan(&report)?;
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn resolve_dynamic_bind(
    plan: &LinkPlan,
    bind: &ElfAmd64PlatformDynamicBindRecord,
    entries: &[LinkPlanHostFfiEntry],
    footprint_valid: bool,
    dependencies: &mut Vec<ElfAmd64DynamicDependencyPlan>,
    dependency_indexes: &mut BTreeMap<&'static str, usize>,
    version_indexes: &mut BTreeMap<(&'static str, &'static str), u16>,
    bindings: &mut Vec<ElfAmd64DynamicSymbolPlan>,
    issues: &mut Vec<String>,
) {
    let matches = entries
        .iter()
        .filter(|entry| entry.symbol == bind.target_symbol)
        .collect::<Vec<_>>();
    let entry = match matches.as_slice() {
        [entry] => *entry,
        [] => {
            issues.push(format!("missing-host-ffi-whitelist:{}", bind.target_symbol));
            return;
        }
        _ => {
            issues.push(format!(
                "ambiguous-host-ffi-signature:{}:{}",
                bind.target_symbol,
                matches.len()
            ));
            return;
        }
    };
    if !footprint_valid {
        return;
    }
    let providers = matching_dynamic_resolver_providers(plan, &entry.abi);
    let provider = match providers.as_slice() {
        [provider] => *provider,
        [] => {
            issues.push(format!(
                "registered-dynamic-provider-missing:{}:{}",
                entry.abi, bind.target_symbol
            ));
            return;
        }
        _ => {
            issues.push(format!(
                "registered-dynamic-provider-ambiguous:{}:{}",
                entry.abi, bind.target_symbol
            ));
            return;
        }
    };
    let Some(symbol_version) =
        registered_dynamic_symbol_version(provider.provider_id, &bind.target_symbol)
    else {
        issues.push(format!(
            "registered-symbol-version-missing:{}:{}",
            entry.abi, bind.target_symbol
        ));
        return;
    };
    let version_key = (provider.provider_id, symbol_version.version_identity);
    let version_index = match version_indexes.get(&version_key).copied() {
        Some(index) => index,
        None => {
            let Ok(index) = u16::try_from(version_indexes.len() + 2) else {
                issues.push("registered-symbol-version-index-overflow".to_owned());
                return;
            };
            version_indexes.insert(version_key, index);
            index
        }
    };
    let dependency_index = match dependency_indexes.get(provider.provider_id).copied() {
        Some(index) => index,
        None => {
            let index = dependencies.len();
            dependencies.push(build_dependency(index, provider));
            dependency_indexes.insert(provider.provider_id, index);
            index
        }
    };
    let dependency = &dependencies[dependency_index];
    let mut binding = ElfAmd64DynamicSymbolPlan {
        binding_id: format!("elf-amd64-dynamic-plan-binding-{:06}", bindings.len()),
        target_key: bind.target_key.clone(),
        target_symbol: bind.target_symbol.clone(),
        dynamic_symbol_index: bind.dynamic_symbol_index,
        platform_bind_audit_hash: bind.audit_hash.clone(),
        host_ffi_abi: entry.abi.clone(),
        signature_pattern: entry.signature_pattern.clone(),
        signature_hash: entry.signature_hash.clone(),
        whitelist_policy: entry.policy.clone(),
        memory_capabilities: entry.memory_capabilities.clone(),
        dependency_audit_hash: dependency.audit_hash.clone(),
        symbol_version_identity: symbol_version.version_identity.to_owned(),
        symbol_version_name: symbol_version.version_name.to_owned(),
        symbol_version_index: version_index,
        symbol_version_hash: elf_version_name_hash(symbol_version.version_name),
        status: "whitelist-provider-and-version-bound".to_owned(),
        audit_hash: String::new(),
    };
    binding.audit_hash = binding_audit_hash(&binding);
    bindings.push(binding);
}

fn build_dependency(
    index: usize,
    provider: DynamicResolverProvider,
) -> ElfAmd64DynamicDependencyPlan {
    let mut dependency = ElfAmd64DynamicDependencyPlan {
        dependency_id: format!("elf-amd64-dynamic-dependency-{index:04}"),
        provider_id: provider.provider_id.to_owned(),
        provider_target_key: provider_target_key(provider),
        host_ffi_abi: provider.host_ffi_abi.to_owned(),
        interpreter_identity: provider.interpreter_identity.to_owned(),
        interpreter_path: provider.interpreter_path.to_owned(),
        dependency_identity: provider.dependency_identity.to_owned(),
        needed_name: provider.needed_name.to_owned(),
        symbol_version_policy: provider.symbol_version_policy.to_owned(),
        resolver_identity: provider.resolver_identity.to_owned(),
        audit_hash: String::new(),
    };
    dependency.audit_hash = dependency_audit_hash(&dependency);
    dependency
}

fn validate_upstream(
    unresolved_symbols: &[String],
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    application: &ElfAmd64PlatformPatchApplicationReport,
) -> Result<(), String> {
    if platform_plan.plan_hash != crate::fnv1a64_hex(platform_plan.canonical_plan().as_bytes())
        || application.platform_structure_plan_hash != platform_plan.plan_hash
        || application.application_ledger_hash
            != crate::fnv1a64_hex(application.canonical_ledger().as_bytes())
    {
        return Err("ELF dynamic dependency plan rejects upstream lineage drift".to_owned());
    }
    let unresolved = unresolved_symbols
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let bound = application
        .dynamic_bind_records
        .iter()
        .map(|bind| bind.target_symbol.as_str())
        .collect::<BTreeSet<_>>();
    if unresolved.len() != unresolved_symbols.len()
        || bound.len() != application.dynamic_bind_records.len()
        || unresolved != bound
        || application.unresolved_dynamic_bind_count != application.dynamic_bind_records.len()
        || platform_plan.target_count != application.dynamic_bind_records.len()
    {
        return Err("ELF dynamic dependency plan rejects dynamic-symbol coverage drift".to_owned());
    }
    for bind in &application.dynamic_bind_records {
        if bind.status != "unresolved-external-dynamic-bind"
            || bind.audit_hash != bind_audit_hash(bind)
        {
            return Err(format!(
                "ELF dynamic dependency plan rejects bind audit `{}`",
                bind.bind_id
            ));
        }
    }
    Ok(())
}

fn validate_host_ffi_footprint(footprint: &LinkPlanHostFfiFootprint) -> Vec<String> {
    let mut issues = Vec::new();
    let policy_count = footprint
        .entries
        .iter()
        .filter(|entry| !entry.policy.is_empty())
        .count();
    let memory_count = footprint
        .entries
        .iter()
        .map(|entry| entry.memory_capabilities.len())
        .sum::<usize>();
    if footprint.index_path.as_deref().is_none_or(str::is_empty) {
        issues.push("host-ffi-index-source-missing".to_owned());
    }
    if footprint.symbol_count != footprint.entries.len()
        || footprint.policy_count != policy_count
        || footprint.memory_capability_count != memory_count
        || footprint.validation.checked != footprint.entries.len()
    {
        issues.push("host-ffi-footprint-count-drift".to_owned());
    }
    if footprint.policy != HOST_FFI_POLICY
        || !footprint.validation.valid
        || !footprint.validation.link_allowed
        || !footprint.validation.issues.is_empty()
    {
        issues.push("host-ffi-footprint-validation-rejected".to_owned());
    }
    let mut seen = BTreeSet::new();
    for entry in &footprint.entries {
        if entry.policy != HOST_FFI_POLICY {
            issues.push(format!("host-ffi-policy-drift:{}", entry.symbol));
        }
        if entry.signature_hash
            != yir_core::ffi::ffi_symbol_signature_hash(
                &entry.abi,
                &entry.symbol,
                &entry.signature_pattern,
            )
        {
            issues.push(format!("host-ffi-signature-hash-drift:{}", entry.symbol));
        }
        if !seen.insert((
            entry.abi.as_str(),
            entry.symbol.as_str(),
            entry.signature_pattern.as_str(),
        )) {
            issues.push(format!("host-ffi-duplicate-signature:{}", entry.symbol));
        }
    }
    issues
}

pub(crate) fn validate_elf_amd64_dynamic_dependency_plan(
    report: &ElfAmd64DynamicDependencyPlanReport,
) -> Result<(), String> {
    let expected_status = dependency_plan_status(report.unresolved_symbol_count, report.plan_ready);
    if report.contract != ELF_AMD64_DYNAMIC_DEPENDENCY_PLAN_CONTRACT
        || report.registry_contract != ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT
        || report.registry_hash != validate_dynamic_resolver_provider_registry()?
        || report.status != expected_status
        || report.target_key.is_empty()
        || report.host_ffi_footprint_hash.is_empty()
        || report.platform_structure_plan_hash.is_empty()
        || report.platform_application_ledger_hash.is_empty()
        || report.unresolved_symbol_count != report.dynamic_bind_count
        || report.dynamic_bind_count < report.resolved_binding_count
        || report.resolved_binding_count != report.bindings.len()
        || report.issues.windows(2).any(|pair| pair[0] >= pair[1])
        || (report.plan_ready
            && (!report.issues.is_empty()
                || report.dynamic_bind_count != report.resolved_binding_count))
        || (!report.plan_ready && report.unresolved_symbol_count == 0)
        || (report.unresolved_symbol_count == 0
            && (!report.dependencies.is_empty() || !report.bindings.is_empty()))
    {
        return Err("ELF dynamic dependency plan envelope drift".to_owned());
    }
    validate_records(&report.target_key, &report.dependencies, &report.bindings)?;
    if report.plan_hash != crate::fnv1a64_hex(report.canonical_plan().as_bytes()) {
        return Err("ELF dynamic dependency plan hash drift".to_owned());
    }
    Ok(())
}

fn validate_records(
    target_key: &str,
    dependencies: &[ElfAmd64DynamicDependencyPlan],
    bindings: &[ElfAmd64DynamicSymbolPlan],
) -> Result<(), String> {
    let mut dependency_hashes = BTreeSet::new();
    for (index, dependency) in dependencies.iter().enumerate() {
        if dependency.dependency_id != format!("elf-amd64-dynamic-dependency-{index:04}")
            || dependency.audit_hash != dependency_audit_hash(dependency)
            || !dependency_hashes.insert(dependency.audit_hash.as_str())
            || !dependency_matches_registered_provider(dependency)
            || dependency.provider_target_key != target_key
        {
            return Err(format!("ELF dynamic dependency plan record {index} drift"));
        }
    }
    let mut used_dependencies = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    let mut dynamic_symbol_indexes = BTreeSet::new();
    let mut image_version_indexes = BTreeMap::new();
    for (index, binding) in bindings.iter().enumerate() {
        let dependency = dependencies
            .iter()
            .find(|dependency| dependency.audit_hash == binding.dependency_audit_hash);
        let registered_version = dependency.and_then(|dependency| {
            registered_dynamic_symbol_version(&dependency.provider_id, &binding.target_symbol)
        });
        let expected_version_index = match (dependency, registered_version) {
            (Some(dependency), Some(version)) => {
                let version_key = (dependency.provider_id.as_str(), version.version_identity);
                match image_version_indexes.get(&version_key).copied() {
                    Some(index) => Some(index),
                    None => {
                        let assigned = u16::try_from(image_version_indexes.len() + 2).ok();
                        if let Some(assigned) = assigned {
                            image_version_indexes.insert(version_key, assigned);
                        }
                        assigned
                    }
                }
            }
            _ => None,
        };
        if binding.binding_id != format!("elf-amd64-dynamic-plan-binding-{index:06}")
            || binding.status != "whitelist-provider-and-version-bound"
            || binding.audit_hash != binding_audit_hash(binding)
            || !symbols.insert(binding.target_symbol.as_str())
            || binding.dynamic_symbol_index != index + 1
            || !dynamic_symbol_indexes.insert(binding.dynamic_symbol_index)
            || !dependency_hashes.contains(binding.dependency_audit_hash.as_str())
            || binding.whitelist_policy != HOST_FFI_POLICY
            || binding.signature_hash
                != yir_core::ffi::ffi_symbol_signature_hash(
                    &binding.host_ffi_abi,
                    &binding.target_symbol,
                    &binding.signature_pattern,
                )
            || dependency.is_none_or(|dependency| dependency.host_ffi_abi != binding.host_ffi_abi)
            || registered_version.is_none_or(|version| {
                binding.symbol_version_identity != version.version_identity
                    || binding.symbol_version_name != version.version_name
                    || binding.symbol_version_hash != elf_version_name_hash(version.version_name)
            })
            || expected_version_index != Some(binding.symbol_version_index)
        {
            return Err(format!("ELF dynamic symbol plan record {index} drift"));
        }
        used_dependencies.insert(binding.dependency_audit_hash.as_str());
    }
    if used_dependencies != dependency_hashes {
        return Err("ELF dynamic dependency plan coverage drift".to_owned());
    }
    Ok(())
}

pub(crate) fn dependency_plan_status(unresolved_count: usize, ready: bool) -> &'static str {
    if unresolved_count == 0 {
        "not-required-static-closure"
    } else if ready {
        "registered-dynamic-dependency-plan-ready"
    } else {
        "blocked-dynamic-dependency-plan"
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn canonical_plan_components(
    contract: &str,
    status: &str,
    ready: bool,
    registry_contract: &str,
    registry_hash: &str,
    target_key: &str,
    host_ffi_footprint_hash: &str,
    platform_structure_plan_hash: &str,
    platform_application_ledger_hash: &str,
    unresolved_symbol_count: usize,
    dynamic_bind_count: usize,
    resolved_binding_count: usize,
    issues: &[String],
    dependencies: &[ElfAmd64DynamicDependencyPlan],
    bindings: &[ElfAmd64DynamicSymbolPlan],
) -> String {
    let mut out = String::new();
    for value in [
        contract,
        status,
        registry_contract,
        registry_hash,
        target_key,
        host_ffi_footprint_hash,
        platform_structure_plan_hash,
        platform_application_ledger_hash,
    ] {
        append_text(&mut out, value);
    }
    writeln!(
        out,
        "shape={ready}|{unresolved_symbol_count}|{dynamic_bind_count}|{resolved_binding_count}|{}|{}|{}",
        issues.len(),
        dependencies.len(),
        bindings.len()
    )
    .unwrap();
    for issue in issues {
        append_text(&mut out, issue);
    }
    for dependency in dependencies {
        append_dependency(&mut out, dependency, true);
    }
    for binding in bindings {
        append_binding(&mut out, binding, true);
    }
    out
}

fn host_ffi_footprint_hash(footprint: &LinkPlanHostFfiFootprint) -> String {
    let mut out = String::new();
    append_text(
        &mut out,
        if footprint.index_path.is_some() {
            "registered-index-source-present"
        } else {
            "registered-index-source-absent"
        },
    );
    append_text(&mut out, &footprint.policy);
    writeln!(
        out,
        "counts={}|{}|{}|{}|{}|{}",
        footprint.symbol_count,
        footprint.policy_count,
        footprint.memory_capability_count,
        footprint.validation.checked,
        footprint.validation.valid,
        footprint.validation.link_allowed
    )
    .unwrap();
    for entry in &footprint.entries {
        for value in [
            entry.abi.as_str(),
            entry.symbol.as_str(),
            entry.signature_pattern.as_str(),
            entry.signature_hash.as_str(),
            entry.policy.as_str(),
        ] {
            append_text(&mut out, value);
        }
        for capability in &entry.memory_capabilities {
            append_text(&mut out, capability);
        }
    }
    for issue in &footprint.validation.issues {
        append_text(&mut out, issue);
    }
    for note in &footprint.validation.notes {
        append_text(&mut out, note);
    }
    crate::fnv1a64_hex(out.as_bytes())
}

fn target_key(plan: &LinkPlan) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        plan.cpu_target.machine_arch,
        plan.cpu_target.machine_os,
        plan.cpu_target.object_format,
        plan.cpu_target.calling_abi,
        plan.cpu_target.clang_target
    )
}

fn dependency_audit_hash(dependency: &ElfAmd64DynamicDependencyPlan) -> String {
    let mut out = String::new();
    append_dependency(&mut out, dependency, false);
    crate::fnv1a64_hex(out.as_bytes())
}

fn binding_audit_hash(binding: &ElfAmd64DynamicSymbolPlan) -> String {
    let mut out = String::new();
    append_binding(&mut out, binding, false);
    crate::fnv1a64_hex(out.as_bytes())
}

fn append_dependency(
    out: &mut String,
    dependency: &ElfAmd64DynamicDependencyPlan,
    include_audit: bool,
) {
    for value in [
        dependency.dependency_id.as_str(),
        dependency.provider_id.as_str(),
        dependency.provider_target_key.as_str(),
        dependency.host_ffi_abi.as_str(),
        dependency.interpreter_identity.as_str(),
        dependency.interpreter_path.as_str(),
        dependency.dependency_identity.as_str(),
        dependency.needed_name.as_str(),
        dependency.symbol_version_policy.as_str(),
        dependency.resolver_identity.as_str(),
    ] {
        append_text(out, value);
    }
    if include_audit {
        append_text(out, &dependency.audit_hash);
    }
}

fn append_binding(out: &mut String, binding: &ElfAmd64DynamicSymbolPlan, include_audit: bool) {
    for value in [
        binding.binding_id.as_str(),
        binding.target_key.as_str(),
        binding.target_symbol.as_str(),
        binding.platform_bind_audit_hash.as_str(),
        binding.host_ffi_abi.as_str(),
        binding.signature_pattern.as_str(),
        binding.signature_hash.as_str(),
        binding.whitelist_policy.as_str(),
        binding.dependency_audit_hash.as_str(),
        binding.symbol_version_identity.as_str(),
        binding.symbol_version_name.as_str(),
        binding.status.as_str(),
    ] {
        append_text(out, value);
    }
    writeln!(
        out,
        "symbol={}|{}|{}",
        binding.dynamic_symbol_index, binding.symbol_version_index, binding.symbol_version_hash
    )
    .unwrap();
    for capability in &binding.memory_capabilities {
        append_text(out, capability);
    }
    if include_audit {
        append_text(out, &binding.audit_hash);
    }
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}
