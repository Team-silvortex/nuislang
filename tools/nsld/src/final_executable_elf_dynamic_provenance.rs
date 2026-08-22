use crate::{
    final_executable_elf_materialization::application::platform::{
        application::{
            bind_audit_hash, ElfAmd64PlatformDynamicBindRecord,
            ElfAmd64PlatformPatchApplicationReport,
        },
        ElfAmd64PlatformStructurePlanReport,
    },
    final_executable_elf_shell::ElfAmd64ShellImageValidationReport,
};
use nuisc::linker::{LinkPlan, LinkPlanHostFfiEntry, LinkPlanHostFfiFootprint};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

pub(crate) const ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT: &str =
    "nuis-nsld-elf-amd64-dynamic-resolution-provenance-v1";
pub(crate) const ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT: &str =
    "nuis-nsld-elf-dynamic-resolver-provider-registry-v1";

const HOST_FFI_POLICY: &str = "signature-whitelist-required";

#[derive(Clone, Copy)]
struct DynamicResolverProvider {
    provider_id: &'static str,
    machine_arch: &'static str,
    machine_os: &'static str,
    object_format: &'static str,
    calling_abi: &'static str,
    clang_target: &'static str,
    host_ffi_abi: &'static str,
    interpreter_identity: &'static str,
    interpreter_path: &'static str,
    dependency_identity: &'static str,
    needed_name: &'static str,
    symbol_version_policy: &'static str,
    resolver_identity: &'static str,
}

const DYNAMIC_RESOLVER_PROVIDERS: &[DynamicResolverProvider] = &[DynamicResolverProvider {
    provider_id: "nsld.elf.amd64.linux-gnu.libc-v1",
    machine_arch: "x86_64",
    machine_os: "linux",
    object_format: "elf",
    calling_abi: "sysv64",
    clang_target: "x86_64-unknown-linux-gnu",
    host_ffi_abi: "libc",
    interpreter_identity: "linux.gnu.ld-so.x86-64-v1",
    interpreter_path: "/lib64/ld-linux-x86-64.so.2",
    dependency_identity: "linux.gnu.libc.so.6-v1",
    needed_name: "libc.so.6",
    symbol_version_policy: "elf-global-default-symbol-version-v1",
    resolver_identity: "elf.sysv.amd64.lazy-plt-v1",
}];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64DynamicDependencyProvenance {
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
pub(crate) struct ElfAmd64DynamicSymbolProvenance {
    pub(crate) binding_id: String,
    pub(crate) target_key: String,
    pub(crate) target_symbol: String,
    pub(crate) platform_bind_audit_hash: String,
    pub(crate) host_ffi_abi: String,
    pub(crate) signature_pattern: String,
    pub(crate) signature_hash: String,
    pub(crate) whitelist_policy: String,
    pub(crate) memory_capabilities: Vec<String>,
    pub(crate) dependency_audit_hash: String,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64DynamicResolutionProvenanceReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) provenance_ready: bool,
    pub(crate) provenance_ledger_hash: String,
    pub(crate) registry_contract: &'static str,
    pub(crate) registry_hash: String,
    pub(crate) target_key: String,
    pub(crate) host_ffi_footprint_hash: String,
    pub(crate) platform_structure_plan_hash: String,
    pub(crate) platform_application_ledger_hash: String,
    pub(crate) shell_validation_ledger_hash: String,
    pub(crate) shell_image_hash: String,
    pub(crate) unresolved_symbol_count: usize,
    pub(crate) dynamic_bind_count: usize,
    pub(crate) resolved_binding_count: usize,
    pub(crate) issues: Vec<String>,
    pub(crate) dependencies: Vec<ElfAmd64DynamicDependencyProvenance>,
    pub(crate) bindings: Vec<ElfAmd64DynamicSymbolProvenance>,
}

impl ElfAmd64DynamicResolutionProvenanceReport {
    pub(crate) fn canonical_ledger(&self) -> String {
        let mut out = String::new();
        for value in [
            self.contract,
            &self.status,
            self.registry_contract,
            &self.registry_hash,
            &self.target_key,
            &self.host_ffi_footprint_hash,
            &self.platform_structure_plan_hash,
            &self.platform_application_ledger_hash,
            &self.shell_validation_ledger_hash,
            &self.shell_image_hash,
        ] {
            append_text(&mut out, value);
        }
        writeln!(
            out,
            "shape={}|{}|{}|{}|{}|{}|{}",
            self.provenance_ready,
            self.unresolved_symbol_count,
            self.dynamic_bind_count,
            self.resolved_binding_count,
            self.issues.len(),
            self.dependencies.len(),
            self.bindings.len()
        )
        .unwrap();
        for issue in &self.issues {
            append_text(&mut out, issue);
        }
        for dependency in &self.dependencies {
            append_dependency(&mut out, dependency, true);
        }
        for binding in &self.bindings {
            append_binding(&mut out, binding, true);
        }
        out
    }
}

pub(crate) fn build_elf_amd64_dynamic_resolution_provenance(
    plan: &LinkPlan,
    unresolved_symbols: &[String],
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_application: &ElfAmd64PlatformPatchApplicationReport,
    shell_validation: &ElfAmd64ShellImageValidationReport,
) -> Result<ElfAmd64DynamicResolutionProvenanceReport, String> {
    validate_upstream(
        unresolved_symbols,
        platform_plan,
        platform_application,
        shell_validation,
    )?;
    let registry_hash = validate_provider_registry()?;
    let target_key = target_key(plan);
    let host_ffi_footprint_hash = host_ffi_footprint_hash(&plan.host_ffi);
    let mut issues = Vec::new();
    let mut dependencies = Vec::new();
    let mut dependency_indexes = BTreeMap::new();
    let mut bindings = Vec::new();

    if !unresolved_symbols.is_empty() {
        issues.extend(validate_host_ffi_footprint(&plan.host_ffi));
        let footprint_valid = issues.is_empty();
        for bind in &platform_application.dynamic_bind_records {
            resolve_dynamic_bind(
                plan,
                bind,
                &plan.host_ffi.entries,
                footprint_valid,
                &mut dependencies,
                &mut dependency_indexes,
                &mut bindings,
                &mut issues,
            );
        }
    }
    issues.sort();
    issues.dedup();

    let provenance_ready =
        issues.is_empty() && bindings.len() == platform_application.dynamic_bind_records.len();
    let status = if unresolved_symbols.is_empty() {
        "not-required-static-closure"
    } else if provenance_ready {
        "verified-registered-dynamic-resolution-provenance"
    } else {
        "blocked-dynamic-resolution-provenance"
    };
    let mut report = ElfAmd64DynamicResolutionProvenanceReport {
        contract: ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT,
        status: status.to_owned(),
        provenance_ready,
        provenance_ledger_hash: String::new(),
        registry_contract: ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT,
        registry_hash,
        target_key,
        host_ffi_footprint_hash,
        platform_structure_plan_hash: platform_plan.plan_hash.clone(),
        platform_application_ledger_hash: platform_application.application_ledger_hash.clone(),
        shell_validation_ledger_hash: shell_validation.validation_ledger_hash.clone(),
        shell_image_hash: shell_validation.shell_image_hash.clone(),
        unresolved_symbol_count: unresolved_symbols.len(),
        dynamic_bind_count: platform_application.dynamic_bind_records.len(),
        resolved_binding_count: bindings.len(),
        issues,
        dependencies,
        bindings,
    };
    report.provenance_ledger_hash = crate::fnv1a64_hex(report.canonical_ledger().as_bytes());
    validate_elf_amd64_dynamic_resolution_provenance_report(&report)?;
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn resolve_dynamic_bind(
    plan: &LinkPlan,
    bind: &ElfAmd64PlatformDynamicBindRecord,
    entries: &[LinkPlanHostFfiEntry],
    footprint_valid: bool,
    dependencies: &mut Vec<ElfAmd64DynamicDependencyProvenance>,
    dependency_indexes: &mut BTreeMap<&'static str, usize>,
    bindings: &mut Vec<ElfAmd64DynamicSymbolProvenance>,
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
    let providers = matching_providers(plan, &entry.abi);
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
    let mut binding = ElfAmd64DynamicSymbolProvenance {
        binding_id: format!("elf-amd64-dynamic-provenance-binding-{:06}", bindings.len()),
        target_key: bind.target_key.clone(),
        target_symbol: bind.target_symbol.clone(),
        platform_bind_audit_hash: bind.audit_hash.clone(),
        host_ffi_abi: entry.abi.clone(),
        signature_pattern: entry.signature_pattern.clone(),
        signature_hash: entry.signature_hash.clone(),
        whitelist_policy: entry.policy.clone(),
        memory_capabilities: entry.memory_capabilities.clone(),
        dependency_audit_hash: dependency.audit_hash.clone(),
        status: "whitelist-and-provider-bound".to_owned(),
        audit_hash: String::new(),
    };
    binding.audit_hash = binding_audit_hash(&binding);
    bindings.push(binding);
}

fn build_dependency(
    index: usize,
    provider: DynamicResolverProvider,
) -> ElfAmd64DynamicDependencyProvenance {
    let mut dependency = ElfAmd64DynamicDependencyProvenance {
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

fn matching_providers(plan: &LinkPlan, host_ffi_abi: &str) -> Vec<DynamicResolverProvider> {
    DYNAMIC_RESOLVER_PROVIDERS
        .iter()
        .filter(|provider| {
            provider.machine_arch == plan.cpu_target.machine_arch
                && provider.machine_os == plan.cpu_target.machine_os
                && provider.object_format == plan.cpu_target.object_format
                && provider.calling_abi == plan.cpu_target.calling_abi
                && provider.clang_target == plan.cpu_target.clang_target
                && provider.host_ffi_abi == host_ffi_abi
        })
        .copied()
        .collect()
}

fn validate_upstream(
    unresolved_symbols: &[String],
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    application: &ElfAmd64PlatformPatchApplicationReport,
    shell: &ElfAmd64ShellImageValidationReport,
) -> Result<(), String> {
    if platform_plan.plan_hash != crate::fnv1a64_hex(platform_plan.canonical_plan().as_bytes())
        || application.platform_structure_plan_hash != platform_plan.plan_hash
        || application.application_ledger_hash
            != crate::fnv1a64_hex(application.canonical_ledger().as_bytes())
        || shell.platform_application_ledger_hash != application.application_ledger_hash
        || shell.validation_ledger_hash != crate::fnv1a64_hex(shell.canonical_ledger().as_bytes())
    {
        return Err("ELF dynamic provenance rejects upstream lineage drift".to_owned());
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
        return Err("ELF dynamic provenance rejects dynamic-symbol coverage drift".to_owned());
    }
    for bind in &application.dynamic_bind_records {
        if bind.status != "unresolved-external-dynamic-bind"
            || bind.audit_hash != bind_audit_hash(bind)
        {
            return Err(format!(
                "ELF dynamic provenance rejects bind audit `{}`",
                bind.bind_id
            ));
        }
    }
    let dynamic = !unresolved_symbols.is_empty();
    if (dynamic && (shell.dynamic_segment_count != 1 || shell.dynamic_entry_count == 0))
        || (!dynamic && (shell.dynamic_segment_count != 0 || shell.dynamic_entry_count != 0))
    {
        return Err("ELF dynamic provenance rejects shell dynamic-shape drift".to_owned());
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

fn validate_provider_registry() -> Result<String, String> {
    let mut provider_ids = BTreeSet::new();
    let mut target_abis = BTreeSet::new();
    let mut canonical = String::new();
    append_text(
        &mut canonical,
        ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT,
    );
    for provider in DYNAMIC_RESOLVER_PROVIDERS {
        let values = [
            provider.provider_id,
            provider.machine_arch,
            provider.machine_os,
            provider.object_format,
            provider.calling_abi,
            provider.clang_target,
            provider.host_ffi_abi,
            provider.interpreter_identity,
            provider.interpreter_path,
            provider.dependency_identity,
            provider.needed_name,
            provider.symbol_version_policy,
            provider.resolver_identity,
        ];
        if values.iter().any(|value| value.is_empty())
            || !provider_ids.insert(provider.provider_id)
            || !target_abis.insert((
                provider.machine_arch,
                provider.machine_os,
                provider.object_format,
                provider.calling_abi,
                provider.clang_target,
                provider.host_ffi_abi,
            ))
            || !provider.interpreter_path.starts_with('/')
            || provider.needed_name.contains('/')
        {
            return Err("invalid ELF dynamic resolver provider registry".to_owned());
        }
        for value in values {
            append_text(&mut canonical, value);
        }
    }
    Ok(crate::fnv1a64_hex(canonical.as_bytes()))
}

pub(crate) fn validate_elf_amd64_dynamic_resolution_provenance_report(
    report: &ElfAmd64DynamicResolutionProvenanceReport,
) -> Result<(), String> {
    let expected_registry_hash = validate_provider_registry()?;
    let expected_status = if report.unresolved_symbol_count == 0 {
        "not-required-static-closure"
    } else if report.provenance_ready {
        "verified-registered-dynamic-resolution-provenance"
    } else {
        "blocked-dynamic-resolution-provenance"
    };
    if report.contract != ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT
        || report.registry_contract != ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT
        || report.registry_hash != expected_registry_hash
        || report.status != expected_status
        || report.target_key.is_empty()
        || report.host_ffi_footprint_hash.is_empty()
        || report.platform_structure_plan_hash.is_empty()
        || report.platform_application_ledger_hash.is_empty()
        || report.shell_validation_ledger_hash.is_empty()
        || report.shell_image_hash.is_empty()
        || report.unresolved_symbol_count != report.dynamic_bind_count
        || report.dynamic_bind_count < report.resolved_binding_count
        || report.resolved_binding_count != report.bindings.len()
        || report.issues.windows(2).any(|pair| pair[0] >= pair[1])
        || (report.provenance_ready
            && (!report.issues.is_empty()
                || report.dynamic_bind_count != report.resolved_binding_count))
        || (!report.provenance_ready && report.unresolved_symbol_count == 0)
        || (report.unresolved_symbol_count == 0
            && (!report.dependencies.is_empty() || !report.bindings.is_empty()))
    {
        return Err("ELF dynamic provenance report envelope drift".to_owned());
    }
    let mut dependency_hashes = BTreeSet::new();
    for (index, dependency) in report.dependencies.iter().enumerate() {
        if dependency.dependency_id != format!("elf-amd64-dynamic-dependency-{index:04}")
            || dependency.audit_hash != dependency_audit_hash(dependency)
            || !dependency_hashes.insert(dependency.audit_hash.as_str())
            || !dependency_matches_registered_provider(dependency)
            || dependency.provider_target_key != report.target_key
        {
            return Err(format!("ELF dynamic dependency provenance {index} drift"));
        }
    }
    let mut used_dependencies = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for (index, binding) in report.bindings.iter().enumerate() {
        let dependency = report
            .dependencies
            .iter()
            .find(|dependency| dependency.audit_hash == binding.dependency_audit_hash);
        if binding.binding_id != format!("elf-amd64-dynamic-provenance-binding-{index:06}")
            || binding.status != "whitelist-and-provider-bound"
            || binding.audit_hash != binding_audit_hash(binding)
            || !symbols.insert(binding.target_symbol.as_str())
            || !dependency_hashes.contains(binding.dependency_audit_hash.as_str())
            || binding.whitelist_policy != HOST_FFI_POLICY
            || binding.signature_hash
                != yir_core::ffi::ffi_symbol_signature_hash(
                    &binding.host_ffi_abi,
                    &binding.target_symbol,
                    &binding.signature_pattern,
                )
            || dependency.is_none_or(|dependency| dependency.host_ffi_abi != binding.host_ffi_abi)
        {
            return Err(format!("ELF dynamic symbol provenance {index} drift"));
        }
        used_dependencies.insert(binding.dependency_audit_hash.as_str());
    }
    if used_dependencies != dependency_hashes {
        return Err("ELF dynamic provenance dependency coverage drift".to_owned());
    }
    if report.provenance_ledger_hash != crate::fnv1a64_hex(report.canonical_ledger().as_bytes()) {
        return Err("ELF dynamic provenance report ledger drift".to_owned());
    }
    Ok(())
}

fn dependency_matches_registered_provider(
    dependency: &ElfAmd64DynamicDependencyProvenance,
) -> bool {
    DYNAMIC_RESOLVER_PROVIDERS.iter().any(|provider| {
        dependency.provider_id == provider.provider_id
            && dependency.provider_target_key == provider_target_key(*provider)
            && dependency.host_ffi_abi == provider.host_ffi_abi
            && dependency.interpreter_identity == provider.interpreter_identity
            && dependency.interpreter_path == provider.interpreter_path
            && dependency.dependency_identity == provider.dependency_identity
            && dependency.needed_name == provider.needed_name
            && dependency.symbol_version_policy == provider.symbol_version_policy
            && dependency.resolver_identity == provider.resolver_identity
    })
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

fn provider_target_key(provider: DynamicResolverProvider) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        provider.machine_arch,
        provider.machine_os,
        provider.object_format,
        provider.calling_abi,
        provider.clang_target
    )
}

fn dependency_audit_hash(dependency: &ElfAmd64DynamicDependencyProvenance) -> String {
    let mut out = String::new();
    append_dependency(&mut out, dependency, false);
    crate::fnv1a64_hex(out.as_bytes())
}

fn binding_audit_hash(binding: &ElfAmd64DynamicSymbolProvenance) -> String {
    let mut out = String::new();
    append_binding(&mut out, binding, false);
    crate::fnv1a64_hex(out.as_bytes())
}

fn append_dependency(
    out: &mut String,
    dependency: &ElfAmd64DynamicDependencyProvenance,
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

fn append_binding(
    out: &mut String,
    binding: &ElfAmd64DynamicSymbolProvenance,
    include_audit: bool,
) {
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
        binding.status.as_str(),
    ] {
        append_text(out, value);
    }
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
