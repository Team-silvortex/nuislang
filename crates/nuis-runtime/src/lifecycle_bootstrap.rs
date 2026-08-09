pub const LIFECYCLE_BOOTSTRAP_PLAN_PROTOCOL: &str = "nuis-runtime-lifecycle-bootstrap-plan-v1";
pub const LIFECYCLE_BOOTSTRAP_PLAN_IDENTITY_CONTRACT: &str =
    "nuis-runtime-lifecycle-bootstrap-plan-identity-v1";
pub const LIFECYCLE_BOOTSTRAP_ENTRY_KIND: &str = "lifecycle-bootstrap";
pub const CLOCK_ROOT_BINDING_ID: &str = "runtime.clock-root";
pub const CLOCK_ROOT_CONTRACT: &str = "nuis-clock-protocol-v1";
pub const GLM_ROOT_BINDING_ID: &str = "runtime.glm-root";
pub const GLM_ROOT_CONTRACT: &str = "nuis-yir-glm-binding-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeServiceBindingFacts {
    pub binding_id: String,
    pub contract: String,
    pub value_count: usize,
    pub value_hash: String,
    pub validation_status: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedSectionFacts {
    pub section_id: String,
    pub section_kind: String,
    pub offset: usize,
    pub size_bytes: usize,
    pub payload_hash: String,
    pub required: bool,
    pub mapping_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedRelocationFacts {
    pub relocation_id: String,
    pub relocation_kind: String,
    pub source_section_id: String,
    pub source_offset: usize,
    pub target_symbol_id: String,
    pub addend: isize,
    pub application_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleBootstrapFacts {
    pub image_verified: bool,
    pub container_handoff_ready: bool,
    pub scheduler_entry: String,
    pub process_lifecycle_hook: String,
    pub loader_entry_kind: Option<String>,
    pub loader_entry_abi_contract: Option<String>,
    pub loader_entry_machine_arch: Option<String>,
    pub loader_entry_symbol: Option<String>,
    pub loader_entry_section_id: Option<String>,
    pub loader_symbol_status: String,
    pub loader_symbol_kind: Option<String>,
    pub loader_symbol_name: Option<String>,
    pub loader_symbol_lifecycle_hook: Option<String>,
    pub loader_symbol_section_id: Option<String>,
    pub loader_symbol_offset: Option<usize>,
    pub loader_symbol_size_bytes: Option<usize>,
    pub loader_symbol_payload_hash: Option<String>,
    pub relocation_targets_loader_symbol: bool,
    pub relocation_source_matches_loader_symbol: bool,
    pub source_section_count: usize,
    pub source_section_table_hash: String,
    pub mapped_sections: Vec<MappedSectionFacts>,
    pub source_relocation_count: usize,
    pub source_relocation_table_hash: String,
    pub applied_relocations: Vec<AppliedRelocationFacts>,
    pub runtime_service_bindings: Vec<RuntimeServiceBindingFacts>,
    pub provider_dispatch_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleBootstrapStage {
    pub ordinal: usize,
    pub kind: &'static str,
    pub subject: String,
    pub waits_for_ordinal: Option<usize>,
}

impl LifecycleBootstrapStage {
    pub fn render(&self) -> String {
        format!("{}:{}", self.kind, self.subject)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleBootstrapPlan {
    pub protocol: &'static str,
    pub identity_contract: &'static str,
    pub identity_hash: String,
    pub status: &'static str,
    pub ready: bool,
    pub stages: Vec<LifecycleBootstrapStage>,
    pub blockers: Vec<String>,
}

impl LifecycleBootstrapPlan {
    pub fn rendered_stages(&self) -> Vec<String> {
        self.stages
            .iter()
            .map(LifecycleBootstrapStage::render)
            .collect()
    }
}

pub fn plan_lifecycle_bootstrap(facts: &LifecycleBootstrapFacts) -> LifecycleBootstrapPlan {
    let mut blockers = Vec::new();
    if !facts.image_verified {
        blockers.push("runtime-bootstrap:image-unverified".to_owned());
    }
    if !facts.container_handoff_ready {
        blockers.push("runtime-bootstrap:container-handoff-blocked".to_owned());
    }
    require_non_empty(
        &facts.scheduler_entry,
        "runtime-bootstrap:scheduler-entry-missing",
        &mut blockers,
    );
    require_non_empty(
        &facts.process_lifecycle_hook,
        "runtime-bootstrap:process-lifecycle-hook-missing",
        &mut blockers,
    );
    require_exact(
        facts.loader_entry_kind.as_deref(),
        LIFECYCLE_BOOTSTRAP_ENTRY_KIND,
        "runtime-bootstrap:entry-kind-unsupported",
        &mut blockers,
    );
    if !facts
        .loader_entry_abi_contract
        .as_deref()
        .is_some_and(crate::is_supported_lifecycle_entry_abi)
    {
        blockers.push("runtime-bootstrap:entry-abi-unsupported".to_owned());
    }
    validate_machine_arch(facts.loader_entry_machine_arch.as_deref(), &mut blockers);
    require_optional_non_empty(
        facts.loader_entry_symbol.as_deref(),
        "runtime-bootstrap:entry-symbol-missing",
        &mut blockers,
    );
    require_optional_non_empty(
        facts.loader_entry_section_id.as_deref(),
        "runtime-bootstrap:entry-section-missing",
        &mut blockers,
    );
    if facts.loader_symbol_status != "parsed" {
        blockers.push("runtime-bootstrap:loader-symbol-unverified".to_owned());
    }
    require_matching(
        facts.loader_entry_kind.as_deref(),
        facts.loader_symbol_kind.as_deref(),
        "runtime-bootstrap:loader-symbol-kind-mismatch",
        &mut blockers,
    );
    require_matching(
        facts.loader_entry_symbol.as_deref(),
        facts.loader_symbol_name.as_deref(),
        "runtime-bootstrap:loader-symbol-name-mismatch",
        &mut blockers,
    );
    require_matching(
        facts.loader_entry_section_id.as_deref(),
        facts.loader_symbol_section_id.as_deref(),
        "runtime-bootstrap:loader-symbol-section-mismatch",
        &mut blockers,
    );
    require_optional_non_empty(
        facts.loader_symbol_lifecycle_hook.as_deref(),
        "runtime-bootstrap:nuis-lifecycle-hook-missing",
        &mut blockers,
    );
    validate_loader_symbol_range(facts, &mut blockers);
    if !facts.relocation_targets_loader_symbol {
        blockers.push("runtime-bootstrap:entry-relocation-target-mismatch".to_owned());
    }
    if !facts.relocation_source_matches_loader_symbol {
        blockers.push("runtime-bootstrap:entry-relocation-source-mismatch".to_owned());
    }
    validate_mapped_sections(facts, &mut blockers);
    validate_applied_relocations(facts, &mut blockers);
    validate_runtime_service_bindings(&facts.runtime_service_bindings, &mut blockers);
    require_non_empty(
        &facts.provider_dispatch_status,
        "runtime-bootstrap:provider-dispatch-status-missing",
        &mut blockers,
    );

    let stages = if blockers.is_empty() {
        ready_stages(facts)
    } else {
        Vec::new()
    };
    LifecycleBootstrapPlan {
        protocol: LIFECYCLE_BOOTSTRAP_PLAN_PROTOCOL,
        identity_contract: LIFECYCLE_BOOTSTRAP_PLAN_IDENTITY_CONTRACT,
        identity_hash: if blockers.is_empty() {
            bootstrap_plan_identity_hash(facts)
        } else {
            "none".to_owned()
        },
        status: if blockers.is_empty() {
            "ready"
        } else {
            "blocked"
        },
        ready: blockers.is_empty(),
        stages,
        blockers,
    }
}

fn ready_stages(facts: &LifecycleBootstrapFacts) -> Vec<LifecycleBootstrapStage> {
    let entry_symbol = facts.loader_entry_symbol.as_deref().unwrap_or_default();
    let entry_section = facts.loader_entry_section_id.as_deref().unwrap_or_default();
    let entry_abi = facts
        .loader_entry_abi_contract
        .as_deref()
        .unwrap_or_default();
    let entry_machine_arch = facts
        .loader_entry_machine_arch
        .as_deref()
        .unwrap_or_default();
    let nuis_hook = facts
        .loader_symbol_lifecycle_hook
        .as_deref()
        .unwrap_or_default();
    let mut specs = vec![(
        "accept-verified-image",
        LIFECYCLE_BOOTSTRAP_PLAN_PROTOCOL.to_owned(),
    )];
    let mut sections = facts.mapped_sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| left.section_id.cmp(&right.section_id));
    specs.extend(sections.into_iter().map(|section| {
        (
            "map-section",
            format!(
                "{}@{}+{}#{}",
                section.section_id, section.offset, section.size_bytes, section.payload_hash
            ),
        )
    }));
    let mut relocations = facts.applied_relocations.iter().collect::<Vec<_>>();
    relocations.sort_by(|left, right| left.relocation_id.cmp(&right.relocation_id));
    specs.extend(relocations.into_iter().map(|relocation| {
        (
            "apply-relocation",
            format!(
                "{}:{}@{}+{}->{}+{}",
                relocation.relocation_id,
                relocation.relocation_kind,
                relocation.source_section_id,
                relocation.source_offset,
                relocation.target_symbol_id,
                relocation.addend
            ),
        )
    }));
    specs.push((
        "bind-loader-entry",
        format!("{entry_symbol}@{entry_section}#{entry_abi}@{entry_machine_arch}"),
    ));
    let mut services = facts.runtime_service_bindings.iter().collect::<Vec<_>>();
    services.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    specs.extend(services.into_iter().map(|service| {
        (
            "bind-runtime-service",
            format!(
                "{}@{}#{}",
                service.binding_id, service.contract, service.value_hash
            ),
        )
    }));
    specs.extend([
        ("enter-lifecycle-hook", facts.process_lifecycle_hook.clone()),
        (
            "enter-nuis-bootstrap",
            format!("{nuis_hook}:{entry_symbol}"),
        ),
        (
            "bind-provider-dispatch",
            facts.provider_dispatch_status.clone(),
        ),
        ("activate-scheduler", facts.scheduler_entry.clone()),
    ]);
    specs
        .into_iter()
        .enumerate()
        .map(|(ordinal, (kind, subject))| LifecycleBootstrapStage {
            ordinal,
            kind,
            subject,
            waits_for_ordinal: ordinal.checked_sub(1),
        })
        .collect()
}

fn validate_mapped_sections(facts: &LifecycleBootstrapFacts, blockers: &mut Vec<String>) {
    if facts.source_section_count == 0 || facts.source_section_count != facts.mapped_sections.len()
    {
        blockers.push("runtime-bootstrap:mapped-section-count-mismatch".to_owned());
    }
    if !valid_table_hash(&facts.source_section_table_hash) {
        blockers.push("runtime-bootstrap:section-table-hash-invalid".to_owned());
    }
    let mut ids = std::collections::BTreeSet::new();
    for section in &facts.mapped_sections {
        if !ids.insert(section.section_id.as_str()) {
            blockers.push(format!(
                "runtime-bootstrap:mapped-section-duplicate:{}",
                section.section_id
            ));
        }
        if section.section_id.is_empty()
            || section.section_kind.is_empty()
            || section.size_bytes == 0
            || !valid_table_hash(&section.payload_hash)
            || section.mapping_status != "mapped"
        {
            blockers.push(format!(
                "runtime-bootstrap:mapped-section-invalid:{}",
                section.section_id
            ));
        }
    }
    if facts
        .loader_entry_section_id
        .as_deref()
        .is_none_or(|entry| {
            !facts
                .mapped_sections
                .iter()
                .any(|section| section.section_id == entry && section.required)
        })
    {
        blockers.push("runtime-bootstrap:entry-section-not-mapped".to_owned());
    }
}

fn validate_loader_symbol_range(facts: &LifecycleBootstrapFacts, blockers: &mut Vec<String>) {
    let Some(entry_section_id) = facts.loader_entry_section_id.as_deref() else {
        return;
    };
    let Some(section) = facts
        .mapped_sections
        .iter()
        .find(|section| section.section_id == entry_section_id)
    else {
        return;
    };
    let Some(symbol_offset) = facts.loader_symbol_offset else {
        blockers.push("runtime-bootstrap:loader-symbol-offset-missing".to_owned());
        return;
    };
    let Some(symbol_size) = facts.loader_symbol_size_bytes.filter(|size| *size > 0) else {
        blockers.push("runtime-bootstrap:loader-symbol-size-invalid".to_owned());
        return;
    };
    let symbol_end = symbol_offset.checked_add(symbol_size);
    let section_end = section.offset.checked_add(section.size_bytes);
    if symbol_offset < section.offset
        || symbol_end.is_none()
        || section_end.is_none()
        || symbol_end > section_end
    {
        blockers.push("runtime-bootstrap:loader-symbol-range-invalid".to_owned());
    }
    if facts
        .loader_symbol_payload_hash
        .as_deref()
        .is_none_or(|hash| !valid_table_hash(hash))
    {
        blockers.push("runtime-bootstrap:loader-symbol-hash-invalid".to_owned());
    }
}

fn validate_applied_relocations(facts: &LifecycleBootstrapFacts, blockers: &mut Vec<String>) {
    if facts.source_relocation_count == 0
        || facts.source_relocation_count != facts.applied_relocations.len()
    {
        blockers.push("runtime-bootstrap:applied-relocation-count-mismatch".to_owned());
    }
    if !valid_table_hash(&facts.source_relocation_table_hash) {
        blockers.push("runtime-bootstrap:relocation-table-hash-invalid".to_owned());
    }
    let mut ids = std::collections::BTreeSet::new();
    for relocation in &facts.applied_relocations {
        if !ids.insert(relocation.relocation_id.as_str()) {
            blockers.push(format!(
                "runtime-bootstrap:applied-relocation-duplicate:{}",
                relocation.relocation_id
            ));
        }
        let source_valid = facts.mapped_sections.iter().any(|section| {
            section.section_id == relocation.source_section_id
                && relocation.source_offset >= section.offset
                && relocation.source_offset < section.offset.saturating_add(section.size_bytes)
        });
        if relocation.relocation_id.is_empty()
            || relocation.relocation_kind.is_empty()
            || relocation.target_symbol_id.is_empty()
            || relocation.application_status != "applied"
            || !source_valid
        {
            blockers.push(format!(
                "runtime-bootstrap:applied-relocation-invalid:{}",
                relocation.relocation_id
            ));
        }
    }
}

fn bootstrap_plan_identity_hash(facts: &LifecycleBootstrapFacts) -> String {
    let mut material = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        LIFECYCLE_BOOTSTRAP_PLAN_IDENTITY_CONTRACT,
        facts.scheduler_entry,
        facts.process_lifecycle_hook,
        facts.loader_entry_kind.as_deref().unwrap_or("none"),
        facts.loader_entry_abi_contract.as_deref().unwrap_or("none"),
        facts.loader_entry_machine_arch.as_deref().unwrap_or("none"),
        facts.loader_entry_symbol.as_deref().unwrap_or("none"),
        facts.loader_entry_section_id.as_deref().unwrap_or("none"),
        facts.loader_symbol_offset.unwrap_or(usize::MAX),
        facts.loader_symbol_size_bytes.unwrap_or(0),
        facts
            .loader_symbol_payload_hash
            .as_deref()
            .unwrap_or("none"),
        facts.source_section_table_hash,
        facts.source_relocation_table_hash
    );
    let mut sections = facts.mapped_sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| left.section_id.cmp(&right.section_id));
    for section in sections {
        material.push_str(&format!(
            "section\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            section.section_id,
            section.section_kind,
            section.offset,
            section.size_bytes,
            section.payload_hash,
            section.required,
            section.mapping_status
        ));
    }
    let mut relocations = facts.applied_relocations.iter().collect::<Vec<_>>();
    relocations.sort_by(|left, right| left.relocation_id.cmp(&right.relocation_id));
    for relocation in relocations {
        material.push_str(&format!(
            "relocation\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            relocation.relocation_id,
            relocation.relocation_kind,
            relocation.source_section_id,
            relocation.source_offset,
            relocation.target_symbol_id,
            relocation.addend,
            relocation.application_status
        ));
    }
    let mut services = facts.runtime_service_bindings.iter().collect::<Vec<_>>();
    services.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    for service in services {
        material.push_str(&format!(
            "service\t{}\t{}\t{}\t{}\t{}\t{}\n",
            service.binding_id,
            service.contract,
            service.value_count,
            service.value_hash,
            service.validation_status,
            service.required
        ));
    }
    material.push_str("dispatch\t");
    material.push_str(&facts.provider_dispatch_status);
    material.push('\n');
    fnv1a64_hex(material.as_bytes())
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn validate_runtime_service_bindings(
    bindings: &[RuntimeServiceBindingFacts],
    blockers: &mut Vec<String>,
) {
    let mut ids = std::collections::BTreeSet::new();
    for binding in bindings {
        if !ids.insert(binding.binding_id.as_str()) {
            blockers.push(format!(
                "runtime-bootstrap:service-binding-duplicate:{}",
                binding.binding_id
            ));
        }
        if binding.required && binding.validation_status != "verified" {
            blockers.push(format!(
                "runtime-bootstrap:service-binding-unverified:{}",
                binding.binding_id
            ));
        }
        if binding.value_count == 0 || !valid_table_hash(&binding.value_hash) {
            blockers.push(format!(
                "runtime-bootstrap:service-binding-value-invalid:{}",
                binding.binding_id
            ));
        }
    }
    require_runtime_service(
        bindings,
        CLOCK_ROOT_BINDING_ID,
        CLOCK_ROOT_CONTRACT,
        blockers,
    );
    require_runtime_service(bindings, GLM_ROOT_BINDING_ID, GLM_ROOT_CONTRACT, blockers);
}

fn require_runtime_service(
    bindings: &[RuntimeServiceBindingFacts],
    binding_id: &str,
    contract: &str,
    blockers: &mut Vec<String>,
) {
    match bindings
        .iter()
        .find(|binding| binding.binding_id == binding_id)
    {
        Some(binding)
            if binding.contract == contract
                && binding.required
                && binding.validation_status == "verified" => {}
        Some(_) => blockers.push(format!(
            "runtime-bootstrap:service-binding-contract-invalid:{binding_id}"
        )),
        None => blockers.push(format!(
            "runtime-bootstrap:service-binding-missing:{binding_id}"
        )),
    }
}

fn valid_table_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn validate_machine_arch(value: Option<&str>, blockers: &mut Vec<String>) {
    match value {
        Some(value) if crate::canonical_machine_arch(value) == Some(value) => {}
        Some(_) => blockers.push("runtime-bootstrap:entry-machine-arch-unsupported".to_owned()),
        None => blockers.push("runtime-bootstrap:entry-machine-arch-missing".to_owned()),
    }
}

fn require_non_empty(value: &str, blocker: &str, blockers: &mut Vec<String>) {
    if value.is_empty() {
        blockers.push(blocker.to_owned());
    }
}

fn require_optional_non_empty(value: Option<&str>, blocker: &str, blockers: &mut Vec<String>) {
    if value.is_none_or(str::is_empty) {
        blockers.push(blocker.to_owned());
    }
}

fn require_exact(actual: Option<&str>, expected: &str, blocker: &str, blockers: &mut Vec<String>) {
    if actual != Some(expected) {
        blockers.push(blocker.to_owned());
    }
}

fn require_matching(
    expected: Option<&str>,
    actual: Option<&str>,
    blocker: &str,
    blockers: &mut Vec<String>,
) {
    if expected.is_none() || expected != actual {
        blockers.push(blocker.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_facts() -> LifecycleBootstrapFacts {
        LifecycleBootstrapFacts {
            image_verified: true,
            container_handoff_ready: true,
            scheduler_entry: "nuis.scheduler.loop.v1".to_owned(),
            process_lifecycle_hook: "on_process_start".to_owned(),
            loader_entry_kind: Some("lifecycle-bootstrap".to_owned()),
            loader_entry_abi_contract: Some(crate::NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1.to_owned()),
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
            loader_symbol_payload_hash: Some("0x6666666666666666".to_owned()),
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

    #[test]
    fn ready_plan_is_deterministic_and_strictly_ordered() {
        let plan = plan_lifecycle_bootstrap(&ready_facts());
        let bind_loader_entry = format!(
            "bind-loader-entry:main@sec0001.nuis-native-entry-code#nuis-runtime-lifecycle-entry-context-i64-v1@{}",
            crate::native_host_machine_arch().unwrap_or(crate::NUIS_MACHINE_ARCH_AARCH64)
        );

        assert!(plan.ready);
        assert_eq!(plan.status, "ready");
        assert_eq!(
            plan.identity_contract,
            LIFECYCLE_BOOTSTRAP_PLAN_IDENTITY_CONTRACT
        );
        assert!(valid_table_hash(&plan.identity_hash));
        assert_eq!(plan.stages.len(), 11);
        assert_eq!(plan.stages[0].waits_for_ordinal, None);
        assert_eq!(plan.stages[10].waits_for_ordinal, Some(9));
        assert_eq!(
            plan.rendered_stages(),
            vec![
                "accept-verified-image:nuis-runtime-lifecycle-bootstrap-plan-v1",
                "map-section:sec0000.compiled-artifact@0+128#0x4444444444444444",
                "map-section:sec0001.nuis-native-entry-code@128+16#0x7777777777777777",
                "apply-relocation:rel0000.lifecycle-entry:lifecycle-entry-binding@sec0001.nuis-native-entry-code+128->sym0000.loader-entry+0",
                bind_loader_entry.as_str(),
                "bind-runtime-service:runtime.clock-root@nuis-clock-protocol-v1#0x1111111111111111",
                "bind-runtime-service:runtime.glm-root@nuis-yir-glm-binding-v1#0x2222222222222222",
                "enter-lifecycle-hook:on_process_start",
                "enter-nuis-bootstrap:on_lifecycle_bootstrap:main",
                "bind-provider-dispatch:verified-empty",
                "activate-scheduler:nuis.scheduler.loop.v1",
            ]
        );
    }

    #[test]
    fn entry_identity_or_relocation_drift_blocks_every_stage() {
        let mut facts = ready_facts();
        facts.loader_symbol_name = Some("other".to_owned());
        facts.relocation_targets_loader_symbol = false;

        let plan = plan_lifecycle_bootstrap(&facts);

        assert!(!plan.ready);
        assert_eq!(plan.status, "blocked");
        assert_eq!(plan.identity_hash, "none");
        assert!(plan.stages.is_empty());
        assert!(plan
            .blockers
            .contains(&"runtime-bootstrap:loader-symbol-name-mismatch".to_owned()));
        assert!(plan
            .blockers
            .contains(&"runtime-bootstrap:entry-relocation-target-mismatch".to_owned()));
    }

    #[test]
    fn missing_or_tampered_required_service_blocks_every_stage() {
        let mut facts = ready_facts();
        facts.runtime_service_bindings.pop();
        let missing = plan_lifecycle_bootstrap(&facts);
        assert!(!missing.ready);
        assert!(missing.stages.is_empty());
        assert!(missing.blockers.iter().any(|blocker| {
            blocker == "runtime-bootstrap:service-binding-missing:runtime.glm-root"
        }));

        let mut facts = ready_facts();
        facts.runtime_service_bindings[0].value_hash = "0xinvalid".to_owned();
        let tampered = plan_lifecycle_bootstrap(&facts);
        assert!(!tampered.ready);
        assert!(tampered.stages.is_empty());
        assert!(tampered.blockers.iter().any(|blocker| {
            blocker == "runtime-bootstrap:service-binding-value-invalid:runtime.clock-root"
        }));
    }

    #[test]
    fn incomplete_mapping_or_unapplied_relocation_blocks_every_stage() {
        let mut facts = ready_facts();
        facts.mapped_sections[0].mapping_status = "pending".to_owned();
        let unmapped = plan_lifecycle_bootstrap(&facts);
        assert!(!unmapped.ready);
        assert!(unmapped.stages.is_empty());
        assert!(unmapped.blockers.iter().any(|blocker| {
            blocker == "runtime-bootstrap:mapped-section-invalid:sec0000.compiled-artifact"
        }));

        let mut facts = ready_facts();
        facts.applied_relocations[0].application_status = "planned".to_owned();
        let unapplied = plan_lifecycle_bootstrap(&facts);
        assert!(!unapplied.ready);
        assert!(unapplied.stages.is_empty());
        assert!(unapplied.blockers.iter().any(|blocker| {
            blocker == "runtime-bootstrap:applied-relocation-invalid:rel0000.lifecycle-entry"
        }));
    }

    #[test]
    fn plan_identity_is_order_independent_but_fact_sensitive() {
        let base = plan_lifecycle_bootstrap(&ready_facts());
        let mut reordered = ready_facts();
        reordered.runtime_service_bindings.reverse();
        assert_eq!(
            plan_lifecycle_bootstrap(&reordered).identity_hash,
            base.identity_hash
        );

        let mut drifted = ready_facts();
        drifted.mapped_sections[0].size_bytes += 1;
        assert_ne!(
            plan_lifecycle_bootstrap(&drifted).identity_hash,
            base.identity_hash
        );
    }
}
