use std::collections::{BTreeMap, BTreeSet};

use crate::{
    plan_lifecycle_bootstrap, AppliedRelocationFacts, LifecycleBootstrapFacts, MappedSectionFacts,
    RuntimeServiceBindingFacts,
};

pub const LIFECYCLE_BOOTSTRAP_EXECUTION_PROTOCOL: &str =
    "nuis-runtime-lifecycle-bootstrap-execution-v1";
pub const LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT: &str =
    "nuis-runtime-lifecycle-bootstrap-execution-identity-v1";
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

        CompiledEntryTransferResult {
            protocol: COMPILED_ENTRY_TRANSFER_PROTOCOL,
            status: "transfer-ready",
            ready: true,
            plan_identity_hash: self.plan_identity_hash,
            execution_identity_hash: self.execution_identity_hash,
            image_hash: Some(self.image_mapping.image_hash),
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
            lifecycle_hook: Some(self.lifecycle_hook),
            scheduler_entry: Some(self.scheduler_entry),
            consumed_mapped_section_count: self.mapped_sections.len(),
            consumed_applied_relocation_count: self.applied_relocations.len(),
            activated_service_ids,
            trace,
            blockers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledEntryTransferResult {
    pub protocol: &'static str,
    pub status: &'static str,
    pub ready: bool,
    pub plan_identity_hash: String,
    pub execution_identity_hash: String,
    pub image_hash: Option<String>,
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
    pub lifecycle_hook: Option<String>,
    pub scheduler_entry: Option<String>,
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
            image_hash: None,
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
            lifecycle_hook: None,
            scheduler_entry: None,
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
    blockers.sort();
    blockers.dedup();
    if !blockers.is_empty() {
        return blocked_preparation(plan.identity_hash, blockers);
    }

    let execution_identity_hash = execution_identity_hash(&plan.identity_hash, &image_mapping);
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

fn execution_identity_hash(plan_hash: &str, mapping: &OwnedImageMapping) -> String {
    fnv1a64_hex(
        format!(
            "{}\t{}\t{}\t{}",
            LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT,
            plan_hash,
            mapping.image_hash,
            mapping.size_bytes
        )
        .as_bytes(),
    )
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
mod tests {
    use super::*;
    use crate::{
        ExecutableEntryRequest, ExecutableMemoryAdapter, NativeEntryInvocationPermit,
        NativeHostExecutableMemoryAdapter, CLOCK_ROOT_BINDING_ID, CLOCK_ROOT_CONTRACT,
        GLM_ROOT_BINDING_ID, GLM_ROOT_CONTRACT,
    };

    fn ready_facts() -> LifecycleBootstrapFacts {
        LifecycleBootstrapFacts {
            image_verified: true,
            container_handoff_ready: true,
            scheduler_entry: "nuis.scheduler.loop.v1".to_owned(),
            process_lifecycle_hook: "on_process_start".to_owned(),
            loader_entry_kind: Some("lifecycle-bootstrap".to_owned()),
            loader_entry_abi_contract: Some(crate::NUIS_LIFECYCLE_ENTRY_ABI_V1.to_owned()),
            loader_entry_machine_arch: Some(
                crate::native_host_machine_arch()
                    .unwrap_or(crate::NUIS_MACHINE_ARCH_AARCH64)
                    .to_owned(),
            ),
            loader_entry_symbol: Some("main".to_owned()),
            loader_entry_section_id: Some("sec0001.nuis-native-entry-code".to_owned()),
            loader_symbol_status: "parsed".to_owned(),
            loader_symbol_kind: Some("lifecycle-bootstrap".to_owned()),
            loader_symbol_name: Some("main".to_owned()),
            loader_symbol_lifecycle_hook: Some("on_lifecycle_bootstrap".to_owned()),
            loader_symbol_section_id: Some("sec0001.nuis-native-entry-code".to_owned()),
            loader_symbol_offset: Some(136),
            loader_symbol_size_bytes: Some(8),
            loader_symbol_payload_hash: Some(fnv1a64_hex(&[0; 8])),
            relocation_targets_loader_symbol: true,
            relocation_source_matches_loader_symbol: true,
            source_section_count: 2,
            source_section_table_hash: "0x3333333333333333".to_owned(),
            mapped_sections: vec![
                MappedSectionFacts {
                    section_id: "sec0000.compiled-artifact".to_owned(),
                    section_kind: "compiled-artifact".to_owned(),
                    offset: 0,
                    size_bytes: 128,
                    payload_hash: "0x4444444444444444".to_owned(),
                    required: true,
                    mapping_status: "mapped".to_owned(),
                },
                MappedSectionFacts {
                    section_id: "sec0001.nuis-native-entry-code".to_owned(),
                    section_kind: crate::NUIS_NATIVE_ENTRY_SECTION_KIND.to_owned(),
                    offset: 128,
                    size_bytes: 16,
                    payload_hash: "0x7777777777777777".to_owned(),
                    required: true,
                    mapping_status: "mapped".to_owned(),
                },
            ],
            source_relocation_count: 1,
            source_relocation_table_hash: "0x5555555555555555".to_owned(),
            applied_relocations: vec![AppliedRelocationFacts {
                relocation_id: "rel0000.lifecycle-entry".to_owned(),
                relocation_kind: "lifecycle-entry-binding".to_owned(),
                source_section_id: "sec0001.nuis-native-entry-code".to_owned(),
                source_offset: 128,
                target_symbol_id: "sym0000.loader-entry".to_owned(),
                addend: 0,
                application_status: "applied".to_owned(),
            }],
            runtime_service_bindings: vec![
                RuntimeServiceBindingFacts {
                    binding_id: CLOCK_ROOT_BINDING_ID.to_owned(),
                    contract: CLOCK_ROOT_CONTRACT.to_owned(),
                    value_count: 3,
                    value_hash: "0x1111111111111111".to_owned(),
                    validation_status: "verified".to_owned(),
                    required: true,
                },
                RuntimeServiceBindingFacts {
                    binding_id: GLM_ROOT_BINDING_ID.to_owned(),
                    contract: GLM_ROOT_CONTRACT.to_owned(),
                    value_count: 2,
                    value_hash: "0x2222222222222222".to_owned(),
                    validation_status: "verified".to_owned(),
                    required: true,
                },
            ],
            provider_dispatch_status: "verified-empty".to_owned(),
        }
    }

    fn owned_inputs(
        facts: &LifecycleBootstrapFacts,
    ) -> (
        OwnedImageMapping,
        Vec<OwnedMappedSectionHandle>,
        Vec<OwnedAppliedRelocationHandle>,
        Vec<OwnedRuntimeServiceHandle>,
    ) {
        let plan_hash = plan_lifecycle_bootstrap(facts).identity_hash;
        (
            OwnedImageMapping::new("0x9999999999999999", 256),
            facts
                .mapped_sections
                .iter()
                .map(|facts| OwnedMappedSectionHandle::from_facts(&plan_hash, facts))
                .collect(),
            facts
                .applied_relocations
                .iter()
                .map(|facts| OwnedAppliedRelocationHandle::from_facts(&plan_hash, facts))
                .collect(),
            facts
                .runtime_service_bindings
                .iter()
                .map(|facts| OwnedRuntimeServiceHandle::from_facts(&plan_hash, facts))
                .collect(),
        )
    }

    #[test]
    fn ready_context_consumes_every_capability_before_entry_transfer() {
        let facts = ready_facts();
        let (image, sections, relocations, services) = owned_inputs(&facts);
        let preparation =
            prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services);
        assert!(preparation.ready);
        assert_eq!(preparation.status, "ready");
        assert!(valid_hash(&preparation.execution_identity_hash));

        let transfer = preparation.transfer();
        assert!(transfer.ready);
        assert_eq!(transfer.status, "transfer-ready");
        assert_eq!(transfer.consumed_mapped_section_count, 2);
        assert_eq!(transfer.consumed_applied_relocation_count, 1);
        assert_eq!(
            transfer.entry_section_kind.as_deref(),
            Some(crate::NUIS_NATIVE_ENTRY_SECTION_KIND)
        );
        assert_eq!(transfer.entry_section_offset, Some(128));
        assert_eq!(transfer.entry_section_size_bytes, Some(16));
        assert_eq!(
            transfer.entry_section_payload_hash.as_deref(),
            Some("0x7777777777777777")
        );
        assert_eq!(
            transfer.entry_abi_contract.as_deref(),
            Some(crate::NUIS_LIFECYCLE_ENTRY_ABI_V1)
        );
        assert_eq!(
            transfer.entry_machine_arch.as_deref(),
            crate::native_host_machine_arch()
        );
        assert_eq!(transfer.entry_symbol_offset, Some(136));
        assert_eq!(transfer.entry_symbol_size_bytes, Some(8));
        assert_eq!(
            transfer.entry_symbol_payload_hash.as_deref(),
            Some(fnv1a64_hex(&[0; 8]).as_str())
        );
        assert_eq!(transfer.activated_service_ids.len(), 2);
        assert_eq!(
            transfer.trace,
            vec![
                "consume-mapped-section:sec0000.compiled-artifact",
                "consume-mapped-section:sec0001.nuis-native-entry-code",
                "consume-applied-relocation:rel0000.lifecycle-entry",
                "activate-runtime-service:runtime.clock-root",
                "activate-runtime-service:runtime.glm-root",
                "transfer-compiled-entry:main@sec0001.nuis-native-entry-code",
            ]
        );
        let request = ExecutableEntryRequest::from_transfer(&transfer, &[0; 8]).unwrap();
        let permit = NativeEntryInvocationPermit::from_transfer(&transfer).unwrap();
        assert_eq!(
            permit.protocol(),
            crate::NATIVE_ENTRY_INVOCATION_PERMIT_PROTOCOL
        );
        let native = NativeHostExecutableMemoryAdapter.prepare(&request);
        assert!(native.ready, "{:?}", native.blockers);
        assert_eq!(native.protection_status, "sealed-read-execute");
        assert!(native.authorize(permit).is_ok());
    }

    #[test]
    fn section_capability_drift_fails_closed_without_consuming_anything() {
        let facts = ready_facts();
        let (image, mut sections, relocations, services) = owned_inputs(&facts);
        sections[0].payload_hash = "0xaaaaaaaaaaaaaaaa".to_owned();

        let transfer =
            prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services)
                .transfer();
        assert!(!transfer.ready);
        assert_eq!(transfer.status, "blocked");
        assert_eq!(transfer.consumed_mapped_section_count, 0);
        assert_eq!(transfer.consumed_applied_relocation_count, 0);
        assert!(transfer.activated_service_ids.is_empty());
        assert!(transfer.trace.is_empty());
        assert!(transfer.blockers.iter().any(|blocker| {
            blocker
                == "runtime-bootstrap-execution:section-handle-invalid:sec0000.compiled-artifact"
        }));
    }

    #[test]
    fn capabilities_cannot_cross_plan_identity_boundaries() {
        let facts = ready_facts();
        let (image, sections, mut relocations, services) = owned_inputs(&facts);
        relocations[0].plan_identity_hash = "0xaaaaaaaaaaaaaaaa".to_owned();

        let transfer =
            prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services)
                .transfer();
        assert!(!transfer.ready);
        assert!(transfer.blockers.iter().any(|blocker| {
            blocker
                == "runtime-bootstrap-execution:relocation-handle-invalid:rel0000.lifecycle-entry"
        }));
    }

    #[test]
    fn undersized_mapping_fails_closed_before_section_consumption() {
        let facts = ready_facts();
        let (_, sections, relocations, services) = owned_inputs(&facts);
        let transfer = prepare_lifecycle_bootstrap_execution(
            &facts,
            OwnedImageMapping::new("0x9999999999999999", 64),
            sections,
            relocations,
            services,
        )
        .transfer();

        assert!(!transfer.ready);
        assert_eq!(transfer.image_hash, None);
        assert!(transfer.blockers.iter().any(|blocker| {
            blocker
                == "runtime-bootstrap-execution:section-handle-invalid:sec0000.compiled-artifact"
        }));
    }

    #[test]
    fn blocked_plan_never_materializes_an_execution_context() {
        let mut facts = ready_facts();
        facts.image_verified = false;
        let (image, sections, relocations, services) = owned_inputs(&facts);
        let transfer =
            prepare_lifecycle_bootstrap_execution(&facts, image, sections, relocations, services)
                .transfer();

        assert!(!transfer.ready);
        assert_eq!(transfer.plan_identity_hash, "none");
        assert!(transfer
            .blockers
            .contains(&"runtime-bootstrap:image-unverified".to_owned()));
    }
}
