use crate::{
    final_executable_elf_dynamic_plan::{
        dependency_plan_status, validate_elf_amd64_dynamic_dependency_plan,
        ElfAmd64DynamicDependencyPlan, ElfAmd64DynamicDependencyPlanReport,
        ElfAmd64DynamicSymbolPlan, ELF_AMD64_DYNAMIC_DEPENDENCY_PLAN_CONTRACT,
        ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT,
    },
    final_executable_elf_shell::ElfAmd64ShellImageValidationReport,
};
use std::fmt::Write as _;

pub(crate) const ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT: &str =
    "nuis-nsld-elf-amd64-dynamic-resolution-provenance-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64DynamicResolutionProvenanceReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) provenance_ready: bool,
    pub(crate) provenance_ledger_hash: String,
    pub(crate) dependency_plan_hash: String,
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
    pub(crate) dependencies: Vec<ElfAmd64DynamicDependencyPlan>,
    pub(crate) bindings: Vec<ElfAmd64DynamicSymbolPlan>,
}

impl ElfAmd64DynamicResolutionProvenanceReport {
    pub(crate) fn canonical_ledger(&self) -> String {
        let mut out = String::new();
        for value in [
            self.contract,
            &self.status,
            &self.dependency_plan_hash,
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
            append_text(&mut out, &dependency.dependency_id);
            append_text(&mut out, &dependency.audit_hash);
        }
        for binding in &self.bindings {
            append_text(&mut out, &binding.binding_id);
            append_text(&mut out, &binding.audit_hash);
        }
        out
    }
}

pub(crate) fn build_elf_amd64_dynamic_resolution_provenance(
    dependency_plan: &ElfAmd64DynamicDependencyPlanReport,
    shell_validation: &ElfAmd64ShellImageValidationReport,
) -> Result<ElfAmd64DynamicResolutionProvenanceReport, String> {
    validate_elf_amd64_dynamic_dependency_plan(dependency_plan)?;
    validate_shell_lineage(dependency_plan, shell_validation)?;
    let status = provenance_status(
        dependency_plan.unresolved_symbol_count,
        dependency_plan.plan_ready,
    );
    let mut report = ElfAmd64DynamicResolutionProvenanceReport {
        contract: ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT,
        status: status.to_owned(),
        provenance_ready: dependency_plan.plan_ready,
        provenance_ledger_hash: String::new(),
        dependency_plan_hash: dependency_plan.plan_hash.clone(),
        registry_contract: dependency_plan.registry_contract,
        registry_hash: dependency_plan.registry_hash.clone(),
        target_key: dependency_plan.target_key.clone(),
        host_ffi_footprint_hash: dependency_plan.host_ffi_footprint_hash.clone(),
        platform_structure_plan_hash: dependency_plan.platform_structure_plan_hash.clone(),
        platform_application_ledger_hash: dependency_plan.platform_application_ledger_hash.clone(),
        shell_validation_ledger_hash: shell_validation.validation_ledger_hash.clone(),
        shell_image_hash: shell_validation.shell_image_hash.clone(),
        unresolved_symbol_count: dependency_plan.unresolved_symbol_count,
        dynamic_bind_count: dependency_plan.dynamic_bind_count,
        resolved_binding_count: dependency_plan.resolved_binding_count,
        issues: dependency_plan.issues.clone(),
        dependencies: dependency_plan.dependencies.clone(),
        bindings: dependency_plan.bindings.clone(),
    };
    report.provenance_ledger_hash = crate::fnv1a64_hex(report.canonical_ledger().as_bytes());
    validate_elf_amd64_dynamic_resolution_provenance_report(&report)?;
    Ok(report)
}

fn validate_shell_lineage(
    dependency_plan: &ElfAmd64DynamicDependencyPlanReport,
    shell: &ElfAmd64ShellImageValidationReport,
) -> Result<(), String> {
    if shell.platform_application_ledger_hash != dependency_plan.platform_application_ledger_hash
        || shell.validation_ledger_hash != crate::fnv1a64_hex(shell.canonical_ledger().as_bytes())
    {
        return Err("ELF dynamic provenance rejects shell lineage drift".to_owned());
    }
    let dynamic = dependency_plan.unresolved_symbol_count != 0;
    if (dynamic && (shell.dynamic_segment_count != 1 || shell.dynamic_entry_count == 0))
        || (!dynamic && (shell.dynamic_segment_count != 0 || shell.dynamic_entry_count != 0))
    {
        return Err("ELF dynamic provenance rejects shell dynamic-shape drift".to_owned());
    }
    let mut expected_paths = dependency_plan
        .dependencies
        .iter()
        .map(|dependency| dependency.interpreter_path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let expected_path = if dependency_plan.plan_ready && dynamic {
        if expected_paths.len() != 1 {
            return Err("ELF dynamic provenance rejects interpreter selection drift".to_owned());
        }
        expected_paths.pop_first()
    } else {
        None
    };
    let expected_needed = dependency_plan
        .dependencies
        .iter()
        .map(|dependency| dependency.needed_name.as_str())
        .collect::<Vec<_>>();
    if shell.interpreter_path.as_deref() != expected_path
        || shell
            .needed_libraries
            .iter()
            .map(String::as_str)
            .ne(expected_needed)
    {
        return Err("ELF dynamic provenance rejects parsed loader metadata drift".to_owned());
    }
    Ok(())
}

pub(crate) fn validate_elf_amd64_dynamic_resolution_provenance_report(
    report: &ElfAmd64DynamicResolutionProvenanceReport,
) -> Result<(), String> {
    let expected_status =
        provenance_status(report.unresolved_symbol_count, report.provenance_ready);
    if report.contract != ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT
        || report.registry_contract != ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT
        || report.status != expected_status
        || report.shell_validation_ledger_hash.is_empty()
        || report.shell_image_hash.is_empty()
    {
        return Err("ELF dynamic provenance report envelope drift".to_owned());
    }
    let projection = ElfAmd64DynamicDependencyPlanReport {
        contract: ELF_AMD64_DYNAMIC_DEPENDENCY_PLAN_CONTRACT,
        status: dependency_plan_status(report.unresolved_symbol_count, report.provenance_ready)
            .to_owned(),
        plan_ready: report.provenance_ready,
        plan_hash: report.dependency_plan_hash.clone(),
        registry_contract: report.registry_contract,
        registry_hash: report.registry_hash.clone(),
        target_key: report.target_key.clone(),
        host_ffi_footprint_hash: report.host_ffi_footprint_hash.clone(),
        platform_structure_plan_hash: report.platform_structure_plan_hash.clone(),
        platform_application_ledger_hash: report.platform_application_ledger_hash.clone(),
        unresolved_symbol_count: report.unresolved_symbol_count,
        dynamic_bind_count: report.dynamic_bind_count,
        resolved_binding_count: report.resolved_binding_count,
        issues: report.issues.clone(),
        dependencies: report.dependencies.clone(),
        bindings: report.bindings.clone(),
    };
    validate_elf_amd64_dynamic_dependency_plan(&projection)?;
    if report.provenance_ledger_hash != crate::fnv1a64_hex(report.canonical_ledger().as_bytes()) {
        return Err("ELF dynamic provenance report ledger drift".to_owned());
    }
    Ok(())
}

pub(crate) fn elf_amd64_loader_admission_evidence_hash(
    shell: &ElfAmd64ShellImageValidationReport,
    provenance: &ElfAmd64DynamicResolutionProvenanceReport,
) -> Result<String, String> {
    validate_elf_amd64_dynamic_resolution_provenance_report(provenance)?;
    if shell.validation_ledger_hash != crate::fnv1a64_hex(shell.canonical_ledger().as_bytes())
        || provenance.shell_validation_ledger_hash != shell.validation_ledger_hash
        || provenance.shell_image_hash != shell.shell_image_hash
    {
        return Err("ELF loader admission evidence rejects provenance lineage drift".to_owned());
    }
    let mut canonical = String::new();
    for value in [
        "nuis-nsld-elf-amd64-loader-admission-evidence-v1",
        shell.contract,
        &shell.validation_ledger_hash,
        &shell.shell_image_hash,
        provenance.contract,
        &provenance.provenance_ledger_hash,
    ] {
        append_text(&mut canonical, value);
    }
    Ok(crate::fnv1a64_hex(canonical.as_bytes()))
}

fn provenance_status(unresolved_count: usize, ready: bool) -> &'static str {
    if unresolved_count == 0 {
        "not-required-static-closure"
    } else if ready {
        "verified-registered-dynamic-resolution-provenance"
    } else {
        "blocked-dynamic-resolution-provenance"
    }
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}
