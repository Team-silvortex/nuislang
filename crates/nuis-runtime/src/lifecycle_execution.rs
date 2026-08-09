use std::collections::{BTreeMap, BTreeSet};

use crate::{
    plan_lifecycle_bootstrap, AppliedRelocationFacts, LifecycleBootstrapFacts, MappedSectionFacts,
    ResolvedRuntimeDispatchImport, RuntimeDispatchImportFacts, RuntimeDispatchImportResolution,
    RuntimeServiceBindingFacts,
};

pub const LIFECYCLE_BOOTSTRAP_EXECUTION_PROTOCOL: &str =
    "nuis-runtime-lifecycle-bootstrap-execution-v1";
pub const LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT: &str =
    "nuis-runtime-lifecycle-bootstrap-execution-identity-v1";
pub const LIFECYCLE_BOOTSTRAP_DISPATCH_EXECUTION_IDENTITY_CONTRACT: &str =
    "nuis-runtime-lifecycle-bootstrap-dispatch-execution-identity-v2";
pub const COMPILED_ENTRY_TRANSFER_PROTOCOL: &str = "nuis-runtime-compiled-entry-transfer-v1";

#[derive(Debug, PartialEq, Eq)]
pub struct OwnedImageMapping {
    image_hash: String,
    size_bytes: usize,
}

impl OwnedImageMapping {
    pub fn new(image_hash: impl Into<String>, size_bytes: usize) -> Self {
        Self {
            image_hash: image_hash.into(),
            size_bytes,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OwnedMappedSectionHandle {
    plan_identity_hash: String,
    section_id: String,
    section_kind: String,
    offset: usize,
    size_bytes: usize,
    payload_hash: String,
}

impl OwnedMappedSectionHandle {
    pub fn from_facts(plan_identity_hash: impl Into<String>, facts: &MappedSectionFacts) -> Self {
        Self {
            plan_identity_hash: plan_identity_hash.into(),
            section_id: facts.section_id.clone(),
            section_kind: facts.section_kind.clone(),
            offset: facts.offset,
            size_bytes: facts.size_bytes,
            payload_hash: facts.payload_hash.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OwnedAppliedRelocationHandle {
    plan_identity_hash: String,
    relocation_id: String,
    relocation_kind: String,
    source_section_id: String,
    source_offset: usize,
    target_symbol_id: String,
    addend: isize,
}

impl OwnedAppliedRelocationHandle {
    pub fn from_facts(
        plan_identity_hash: impl Into<String>,
        facts: &AppliedRelocationFacts,
    ) -> Self {
        Self {
            plan_identity_hash: plan_identity_hash.into(),
            relocation_id: facts.relocation_id.clone(),
            relocation_kind: facts.relocation_kind.clone(),
            source_section_id: facts.source_section_id.clone(),
            source_offset: facts.source_offset,
            target_symbol_id: facts.target_symbol_id.clone(),
            addend: facts.addend,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OwnedRuntimeServiceHandle {
    plan_identity_hash: String,
    binding_id: String,
    contract: String,
    value_count: usize,
    value_hash: String,
}

impl OwnedRuntimeServiceHandle {
    pub fn from_facts(
        plan_identity_hash: impl Into<String>,
        facts: &RuntimeServiceBindingFacts,
    ) -> Self {
        Self {
            plan_identity_hash: plan_identity_hash.into(),
            binding_id: facts.binding_id.clone(),
            contract: facts.contract.clone(),
            value_count: facts.value_count,
            value_hash: facts.value_hash.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LifecycleBootstrapExecutionPreparation {
    pub protocol: &'static str,
    pub status: &'static str,
    pub ready: bool,
    pub plan_identity_hash: String,
    pub execution_identity_hash: String,
    pub blockers: Vec<String>,
    context: Option<LifecycleBootstrapExecutionContext>,
}

impl LifecycleBootstrapExecutionPreparation {
    pub fn transfer(self) -> CompiledEntryTransferResult {
        match self.context {
            Some(context) => context.transfer(),
            None => CompiledEntryTransferResult::blocked(
                self.plan_identity_hash,
                self.execution_identity_hash,
                self.blockers,
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LifecycleBootstrapExecutionContext {
    plan_identity_hash: String,
    execution_identity_hash: String,
    image_mapping: OwnedImageMapping,
    mapped_sections: Vec<OwnedMappedSectionHandle>,
    applied_relocations: Vec<OwnedAppliedRelocationHandle>,
    runtime_services: Vec<OwnedRuntimeServiceHandle>,
    entry_symbol: String,
    entry_section_id: String,
    entry_abi_contract: String,
    entry_machine_arch: String,
    entry_symbol_offset: usize,
    entry_symbol_size_bytes: usize,
    entry_symbol_payload_hash: String,
    lifecycle_hook: String,
    scheduler_entry: String,
    runtime_dispatch_import: Option<ResolvedRuntimeDispatchImport>,
}

impl LifecycleBootstrapExecutionContext {
    fn transfer(self) -> CompiledEntryTransferResult {
        let entry_section = self
            .mapped_sections
            .iter()
            .find(|section| section.section_id == self.entry_section_id)
            .expect("validated lifecycle entry section");
        let entry_section_kind = entry_section.section_kind.clone();
        let entry_section_offset = entry_section.offset;
        let entry_section_size_bytes = entry_section.size_bytes;
        let entry_section_payload_hash = entry_section.payload_hash.clone();
        let mut trace = Vec::new();
        for section in &self.mapped_sections {
            trace.push(format!("consume-mapped-section:{}", section.section_id));
        }
        for relocation in &self.applied_relocations {
            trace.push(format!(
                "consume-applied-relocation:{}",
                relocation.relocation_id
            ));
        }
        let activated_service_ids = self
            .runtime_services
            .iter()
            .map(|service| {
                trace.push(format!("activate-runtime-service:{}", service.binding_id));
                service.binding_id.clone()
            })
            .collect::<Vec<_>>();
        trace.push(format!(
            "transfer-compiled-entry:{}@{}",
            self.entry_symbol, self.entry_section_id
        ));
        if let Some(binding) = &self.runtime_dispatch_import {
            trace.push(format!(
                "bind-runtime-dispatch-import:{}",
                binding.import_identity_hash
            ));
        }

        let mut transfer = CompiledEntryTransferResult {
            protocol: COMPILED_ENTRY_TRANSFER_PROTOCOL,
            status: "transfer-ready",
            ready: true,
            plan_identity_hash: self.plan_identity_hash,
            execution_identity_hash: self.execution_identity_hash,
            execution_identity_contract: execution_identity_contract(
                self.runtime_dispatch_import.as_ref(),
            ),
            image_hash: Some(self.image_mapping.image_hash),
            image_size_bytes: Some(self.image_mapping.size_bytes),
            entry_symbol: Some(self.entry_symbol),
            entry_section_id: Some(self.entry_section_id),
            entry_section_kind: Some(entry_section_kind),
            entry_section_offset: Some(entry_section_offset),
            entry_section_size_bytes: Some(entry_section_size_bytes),
            entry_section_payload_hash: Some(entry_section_payload_hash),
            entry_abi_contract: Some(self.entry_abi_contract),
            entry_machine_arch: Some(self.entry_machine_arch),
            entry_symbol_offset: Some(self.entry_symbol_offset),
            entry_symbol_size_bytes: Some(self.entry_symbol_size_bytes),
            entry_symbol_payload_hash: Some(self.entry_symbol_payload_hash),
            entry_context_protocol: None,
            entry_context_identity_hash: None,
            lifecycle_hook: Some(self.lifecycle_hook),
            scheduler_entry: Some(self.scheduler_entry),
            runtime_dispatch_import: self.runtime_dispatch_import,
            consumed_mapped_section_count: self.mapped_sections.len(),
            consumed_applied_relocation_count: self.applied_relocations.len(),
            activated_service_ids,
            trace,
            blockers: Vec::new(),
        };
        crate::native_entry_context::bind_transfer_context(&mut transfer)
            .expect("validated lifecycle execution forms native entry context");
        transfer
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledEntryTransferResult {
    pub protocol: &'static str,
    pub status: &'static str,
    pub ready: bool,
    pub plan_identity_hash: String,
    pub execution_identity_hash: String,
    pub execution_identity_contract: &'static str,
    pub image_hash: Option<String>,
    pub image_size_bytes: Option<usize>,
    pub entry_symbol: Option<String>,
    pub entry_section_id: Option<String>,
    pub entry_section_kind: Option<String>,
    pub entry_section_offset: Option<usize>,
    pub entry_section_size_bytes: Option<usize>,
    pub entry_section_payload_hash: Option<String>,
    pub entry_abi_contract: Option<String>,
    pub entry_machine_arch: Option<String>,
    pub entry_symbol_offset: Option<usize>,
    pub entry_symbol_size_bytes: Option<usize>,
    pub entry_symbol_payload_hash: Option<String>,
    pub entry_context_protocol: Option<String>,
    pub entry_context_identity_hash: Option<String>,
    pub lifecycle_hook: Option<String>,
    pub scheduler_entry: Option<String>,
    pub runtime_dispatch_import: Option<ResolvedRuntimeDispatchImport>,
    pub consumed_mapped_section_count: usize,
    pub consumed_applied_relocation_count: usize,
    pub activated_service_ids: Vec<String>,
    pub trace: Vec<String>,
    pub blockers: Vec<String>,
}

impl CompiledEntryTransferResult {
    fn blocked(
        plan_identity_hash: String,
        execution_identity_hash: String,
        blockers: Vec<String>,
    ) -> Self {
        Self {
            protocol: COMPILED_ENTRY_TRANSFER_PROTOCOL,
            status: "blocked",
            ready: false,
            plan_identity_hash,
            execution_identity_hash,
            execution_identity_contract: LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT,
            image_hash: None,
            image_size_bytes: None,
            entry_symbol: None,
            entry_section_id: None,
            entry_section_kind: None,
            entry_section_offset: None,
            entry_section_size_bytes: None,
            entry_section_payload_hash: None,
            entry_abi_contract: None,
            entry_machine_arch: None,
            entry_symbol_offset: None,
            entry_symbol_size_bytes: None,
            entry_symbol_payload_hash: None,
            entry_context_protocol: None,
            entry_context_identity_hash: None,
            lifecycle_hook: None,
            scheduler_entry: None,
            runtime_dispatch_import: None,
            consumed_mapped_section_count: 0,
            consumed_applied_relocation_count: 0,
            activated_service_ids: Vec::new(),
            trace: Vec::new(),
            blockers,
        }
    }
}

pub fn prepare_lifecycle_bootstrap_execution(
    facts: &LifecycleBootstrapFacts,
    image_mapping: OwnedImageMapping,
    mapped_sections: Vec<OwnedMappedSectionHandle>,
    applied_relocations: Vec<OwnedAppliedRelocationHandle>,
    runtime_services: Vec<OwnedRuntimeServiceHandle>,
) -> LifecycleBootstrapExecutionPreparation {
    let dispatch_resolution = crate::resolve_runtime_dispatch_import(
        facts.loader_entry_abi_contract.as_deref().unwrap_or(""),
        &RuntimeDispatchImportFacts::default(),
    );
    prepare_lifecycle_bootstrap_execution_with_dispatch(
        facts,
        image_mapping,
        mapped_sections,
        applied_relocations,
        runtime_services,
        &dispatch_resolution,
    )
}

pub fn prepare_lifecycle_bootstrap_execution_with_dispatch(
    facts: &LifecycleBootstrapFacts,
    image_mapping: OwnedImageMapping,
    mapped_sections: Vec<OwnedMappedSectionHandle>,
    applied_relocations: Vec<OwnedAppliedRelocationHandle>,
    runtime_services: Vec<OwnedRuntimeServiceHandle>,
    dispatch_resolution: &RuntimeDispatchImportResolution,
) -> LifecycleBootstrapExecutionPreparation {
    let plan = plan_lifecycle_bootstrap(facts);
    if !plan.ready {
        return blocked_preparation(plan.identity_hash, plan.blockers);
    }

    let mut blockers = Vec::new();
    validate_image_mapping(&image_mapping, &mut blockers);
    validate_sections(
        &plan.identity_hash,
        facts,
        &image_mapping,
        &mapped_sections,
        &mut blockers,
    );
    validate_relocations(
        &plan.identity_hash,
        facts,
        &applied_relocations,
        &mut blockers,
    );
    validate_services(&plan.identity_hash, facts, &runtime_services, &mut blockers);
    if !dispatch_resolution.ready {
        blockers.extend(dispatch_resolution.blockers.iter().cloned());
    }
    blockers.sort();
    blockers.dedup();
    if !blockers.is_empty() {
        return blocked_preparation(plan.identity_hash, blockers);
    }

    let runtime_dispatch_import = dispatch_resolution.binding.clone();
    let execution_identity_hash = execution_identity_hash(
        &plan.identity_hash,
        &image_mapping,
        runtime_dispatch_import.as_ref(),
    );
    let context = LifecycleBootstrapExecutionContext {
        plan_identity_hash: plan.identity_hash.clone(),
        execution_identity_hash: execution_identity_hash.clone(),
        image_mapping,
        mapped_sections: sorted_sections(mapped_sections),
        applied_relocations: sorted_relocations(applied_relocations),
        runtime_services: sorted_services(runtime_services),
        entry_symbol: facts.loader_entry_symbol.clone().unwrap_or_default(),
        entry_section_id: facts.loader_entry_section_id.clone().unwrap_or_default(),
        entry_abi_contract: facts.loader_entry_abi_contract.clone().unwrap_or_default(),
        entry_machine_arch: facts.loader_entry_machine_arch.clone().unwrap_or_default(),
        entry_symbol_offset: facts.loader_symbol_offset.unwrap_or_default(),
        entry_symbol_size_bytes: facts.loader_symbol_size_bytes.unwrap_or_default(),
        entry_symbol_payload_hash: facts.loader_symbol_payload_hash.clone().unwrap_or_default(),
        lifecycle_hook: facts
            .loader_symbol_lifecycle_hook
            .clone()
            .unwrap_or_default(),
        scheduler_entry: facts.scheduler_entry.clone(),
        runtime_dispatch_import,
    };
    LifecycleBootstrapExecutionPreparation {
        protocol: LIFECYCLE_BOOTSTRAP_EXECUTION_PROTOCOL,
        status: "ready",
        ready: true,
        plan_identity_hash: plan.identity_hash,
        execution_identity_hash,
        blockers: Vec::new(),
        context: Some(context),
    }
}

fn blocked_preparation(
    plan_identity_hash: String,
    blockers: Vec<String>,
) -> LifecycleBootstrapExecutionPreparation {
    LifecycleBootstrapExecutionPreparation {
        protocol: LIFECYCLE_BOOTSTRAP_EXECUTION_PROTOCOL,
        status: "blocked",
        ready: false,
        plan_identity_hash,
        execution_identity_hash: "none".to_owned(),
        blockers,
        context: None,
    }
}

fn validate_image_mapping(mapping: &OwnedImageMapping, blockers: &mut Vec<String>) {
    if !valid_hash(&mapping.image_hash) {
        blockers.push("runtime-bootstrap-execution:image-hash-invalid".to_owned());
    }
    if mapping.size_bytes == 0 {
        blockers.push("runtime-bootstrap-execution:image-mapping-empty".to_owned());
    }
}

fn validate_sections(
    plan_hash: &str,
    facts: &LifecycleBootstrapFacts,
    image: &OwnedImageMapping,
    handles: &[OwnedMappedSectionHandle],
    blockers: &mut Vec<String>,
) {
    if handles.len() != facts.mapped_sections.len() {
        blockers.push("runtime-bootstrap-execution:section-handle-count-mismatch".to_owned());
    }
    let expected = facts
        .mapped_sections
        .iter()
        .map(|facts| (facts.section_id.as_str(), facts))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for handle in handles {
        if !seen.insert(handle.section_id.as_str()) {
            blockers.push(format!(
                "runtime-bootstrap-execution:section-handle-duplicate:{}",
                handle.section_id
            ));
        }
        let valid = expected
            .get(handle.section_id.as_str())
            .is_some_and(|facts| {
                handle.plan_identity_hash == plan_hash
                    && handle.section_kind == facts.section_kind
                    && handle.offset == facts.offset
                    && handle.size_bytes == facts.size_bytes
                    && handle.payload_hash == facts.payload_hash
                    && handle
                        .offset
                        .checked_add(handle.size_bytes)
                        .is_some_and(|end| end <= image.size_bytes)
            });
        if !valid {
            blockers.push(format!(
                "runtime-bootstrap-execution:section-handle-invalid:{}",
                handle.section_id
            ));
        }
    }
}

fn validate_relocations(
    plan_hash: &str,
    facts: &LifecycleBootstrapFacts,
    handles: &[OwnedAppliedRelocationHandle],
    blockers: &mut Vec<String>,
) {
    if handles.len() != facts.applied_relocations.len() {
        blockers.push("runtime-bootstrap-execution:relocation-handle-count-mismatch".to_owned());
    }
    let expected = facts
        .applied_relocations
        .iter()
        .map(|facts| (facts.relocation_id.as_str(), facts))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for handle in handles {
        if !seen.insert(handle.relocation_id.as_str()) {
            blockers.push(format!(
                "runtime-bootstrap-execution:relocation-handle-duplicate:{}",
                handle.relocation_id
            ));
        }
        let valid = expected
            .get(handle.relocation_id.as_str())
            .is_some_and(|facts| {
                handle.plan_identity_hash == plan_hash
                    && handle.relocation_kind == facts.relocation_kind
                    && handle.source_section_id == facts.source_section_id
                    && handle.source_offset == facts.source_offset
                    && handle.target_symbol_id == facts.target_symbol_id
                    && handle.addend == facts.addend
            });
        if !valid {
            blockers.push(format!(
                "runtime-bootstrap-execution:relocation-handle-invalid:{}",
                handle.relocation_id
            ));
        }
    }
}

fn validate_services(
    plan_hash: &str,
    facts: &LifecycleBootstrapFacts,
    handles: &[OwnedRuntimeServiceHandle],
    blockers: &mut Vec<String>,
) {
    if handles.len() != facts.runtime_service_bindings.len() {
        blockers.push("runtime-bootstrap-execution:service-handle-count-mismatch".to_owned());
    }
    let expected = facts
        .runtime_service_bindings
        .iter()
        .map(|facts| (facts.binding_id.as_str(), facts))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for handle in handles {
        if !seen.insert(handle.binding_id.as_str()) {
            blockers.push(format!(
                "runtime-bootstrap-execution:service-handle-duplicate:{}",
                handle.binding_id
            ));
        }
        let valid = expected
            .get(handle.binding_id.as_str())
            .is_some_and(|facts| {
                handle.plan_identity_hash == plan_hash
                    && handle.contract == facts.contract
                    && handle.value_count == facts.value_count
                    && handle.value_hash == facts.value_hash
            });
        if !valid {
            blockers.push(format!(
                "runtime-bootstrap-execution:service-handle-invalid:{}",
                handle.binding_id
            ));
        }
    }
}

fn sorted_sections(mut handles: Vec<OwnedMappedSectionHandle>) -> Vec<OwnedMappedSectionHandle> {
    handles.sort_by(|left, right| left.section_id.cmp(&right.section_id));
    handles
}

fn sorted_relocations(
    mut handles: Vec<OwnedAppliedRelocationHandle>,
) -> Vec<OwnedAppliedRelocationHandle> {
    handles.sort_by(|left, right| left.relocation_id.cmp(&right.relocation_id));
    handles
}

fn sorted_services(mut handles: Vec<OwnedRuntimeServiceHandle>) -> Vec<OwnedRuntimeServiceHandle> {
    handles.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    handles
}

fn execution_identity_contract(binding: Option<&ResolvedRuntimeDispatchImport>) -> &'static str {
    if binding.is_some() {
        LIFECYCLE_BOOTSTRAP_DISPATCH_EXECUTION_IDENTITY_CONTRACT
    } else {
        LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT
    }
}

fn execution_identity_hash(
    plan_hash: &str,
    mapping: &OwnedImageMapping,
    binding: Option<&ResolvedRuntimeDispatchImport>,
) -> String {
    match binding {
        Some(binding) => fnv1a64_hex(
            format!(
                "{}\t{}\t{}\t{}\t{}",
                LIFECYCLE_BOOTSTRAP_DISPATCH_EXECUTION_IDENTITY_CONTRACT,
                plan_hash,
                mapping.image_hash,
                mapping.size_bytes,
                binding.import_identity_hash
            )
            .as_bytes(),
        ),
        None => fnv1a64_hex(
            format!(
                "{}\t{}\t{}\t{}",
                LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT,
                plan_hash,
                mapping.image_hash,
                mapping.size_bytes
            )
            .as_bytes(),
        ),
    }
}

pub(crate) fn validate_transfer_execution_identity(
    transfer: &CompiledEntryTransferResult,
) -> Result<(), String> {
    match transfer.entry_abi_contract.as_deref() {
        Some(crate::NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1)
            if transfer.runtime_dispatch_import.is_none() => {}
        Some(crate::NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2)
            if transfer.runtime_dispatch_import.is_some() => {}
        Some(crate::NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1) => {
            return Err("runtime-bootstrap-execution:legacy-dispatch-binding-forbidden".to_owned());
        }
        Some(crate::NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2) => {
            return Err("runtime-bootstrap-execution:dispatch-binding-missing".to_owned());
        }
        _ => return Err("runtime-bootstrap-execution:entry-abi-unsupported".to_owned()),
    }
    let image_hash = transfer
        .image_hash
        .as_deref()
        .filter(|value| valid_hash(value))
        .ok_or_else(|| "runtime-bootstrap-execution:transfer-image-hash-invalid".to_owned())?;
    let image_size_bytes = transfer
        .image_size_bytes
        .filter(|size| *size > 0)
        .ok_or_else(|| "runtime-bootstrap-execution:transfer-image-size-invalid".to_owned())?;
    if let Some(binding) = &transfer.runtime_dispatch_import {
        binding.validate_static()?;
    }
    let mapping = OwnedImageMapping::new(image_hash, image_size_bytes);
    let expected_contract = execution_identity_contract(transfer.runtime_dispatch_import.as_ref());
    if transfer.execution_identity_contract != expected_contract {
        return Err("runtime-bootstrap-execution:identity-contract-mismatch".to_owned());
    }
    let expected_hash = execution_identity_hash(
        &transfer.plan_identity_hash,
        &mapping,
        transfer.runtime_dispatch_import.as_ref(),
    );
    if transfer.execution_identity_hash != expected_hash {
        return Err("runtime-bootstrap-execution:identity-mismatch".to_owned());
    }
    Ok(())
}

fn valid_hash(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
#[path = "lifecycle_execution_tests.rs"]
mod tests;
