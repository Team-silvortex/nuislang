use crate::{container::ContainerLoaderSummary, fnv1a64_hex};
use nuis_runtime::{
    CompiledEntryTransferResult, ExecutableEntryRequest, ExecutableMemoryAdapter,
    NativeHostExecutableMemoryAdapter, NUIS_NATIVE_ENTRY_SECTION_KIND,
};

pub(super) const NATIVE_ENTRY_HANDOFF_PROTOCOL: &str = "nuis-host-native-entry-handoff-v1";
const FINAL_IMAGE_PAYLOAD_ALIGNMENT: usize = 16;
const RELOCATION_SLOT_BYTES: usize = 8;
const CONTAINER_CAPSULE_END_MARKER: &[u8] = b"\n# nuis-nsld-container-end-v1\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NativeEntryHandoffEvidence {
    pub(super) protocol: &'static str,
    pub(super) status: String,
    pub(super) ready: bool,
    pub(super) container_payload_offset: Option<usize>,
    pub(super) container_payload_size_bytes: Option<usize>,
    pub(super) container_payload_hash: Option<String>,
    pub(super) section_id: Option<String>,
    pub(super) section_hash_status: String,
    pub(super) code_offset: Option<usize>,
    pub(super) code_size_bytes: Option<usize>,
    pub(super) code_hash_status: String,
    pub(super) target_machine_arch: Option<String>,
    pub(super) host_machine_arch: Option<String>,
    pub(super) machine_arch_status: String,
    pub(super) preparation_protocol: Option<String>,
    pub(super) preparation_status: String,
    pub(super) preparation_ready: bool,
    pub(super) mapping_size_bytes: usize,
    pub(super) protection_status: String,
    pub(super) invocation_status: &'static str,
    pub(super) blockers: Vec<String>,
}

pub(super) struct RecoveredNativeEntry {
    pub(super) evidence: NativeEntryHandoffEvidence,
    pub(super) mapped_payload_hash: String,
    pub(super) mapped_payload_size_bytes: usize,
    code_bytes: Vec<u8>,
}

pub(super) fn recover_native_entry(
    image_payload_region: Option<&[u8]>,
    container: &ContainerLoaderSummary,
) -> RecoveredNativeEntry {
    let mut evidence = empty_evidence();
    let Some(region) = image_payload_region else {
        evidence
            .blockers
            .push("native-entry:image-payload-unmapped".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(capsule_size) = container_capsule_size(region) else {
        evidence
            .blockers
            .push("native-entry:container-capsule-end-missing".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(payload_offset) = align_to(capsule_size, FINAL_IMAGE_PAYLOAD_ALIGNMENT) else {
        evidence
            .blockers
            .push("native-entry:container-payload-offset-overflow".to_owned());
        return blocked_recovery(evidence);
    };
    evidence.container_payload_offset = Some(payload_offset);
    if region
        .get(capsule_size..payload_offset)
        .is_none_or(|padding| padding.iter().any(|byte| *byte != 0))
    {
        evidence
            .blockers
            .push("native-entry:container-payload-padding-invalid".to_owned());
        return blocked_recovery(evidence);
    }
    let Some(payload_size) = container.container_payload_size_bytes else {
        evidence
            .blockers
            .push("native-entry:container-payload-size-missing".to_owned());
        return blocked_recovery(evidence);
    };
    evidence.container_payload_size_bytes = Some(payload_size);
    let Some(payload_end) = payload_offset.checked_add(payload_size) else {
        evidence
            .blockers
            .push("native-entry:container-payload-range-overflow".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(payload) = region.get(payload_offset..payload_end) else {
        evidence
            .blockers
            .push("native-entry:container-payload-out-of-bounds".to_owned());
        return blocked_recovery(evidence);
    };
    let mapped_payload_hash = fnv1a64_hex(payload);
    evidence.container_payload_hash = Some(mapped_payload_hash.clone());

    let Some(section_id) = container.loader_entry_section_id.as_deref() else {
        evidence
            .blockers
            .push("native-entry:section-id-missing".to_owned());
        return blocked_recovery(evidence);
    };
    evidence.section_id = Some(section_id.to_owned());
    let Some(section) = container
        .container_section
        .entries
        .iter()
        .find(|section| section.section_id == section_id)
    else {
        evidence
            .blockers
            .push("native-entry:section-not-found".to_owned());
        return blocked_recovery(evidence);
    };
    if section.section_kind != NUIS_NATIVE_ENTRY_SECTION_KIND {
        evidence
            .blockers
            .push("native-entry:section-kind-unsupported".to_owned());
    }
    let Some(section_end) = section.offset.checked_add(section.size_bytes) else {
        evidence
            .blockers
            .push("native-entry:section-range-overflow".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(section_bytes) = payload.get(section.offset..section_end) else {
        evidence
            .blockers
            .push("native-entry:section-out-of-bounds".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(relocation) = container.relocation.entries.iter().find(|relocation| {
        relocation.target_symbol_id == container.loader_symbol.symbol_id.as_deref().unwrap_or("")
            && relocation.source_section_id == section_id
    }) else {
        evidence
            .blockers
            .push("native-entry:relocation-missing".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(relocation_offset) = relocation.source_offset.checked_sub(section.offset) else {
        evidence
            .blockers
            .push("native-entry:relocation-before-section".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(relocation_end) = relocation_offset.checked_add(RELOCATION_SLOT_BYTES) else {
        evidence
            .blockers
            .push("native-entry:relocation-range-overflow".to_owned());
        return blocked_recovery(evidence);
    };
    if relocation_end > section_bytes.len() {
        evidence
            .blockers
            .push("native-entry:relocation-out-of-bounds".to_owned());
        return blocked_recovery(evidence);
    }
    let mut normalized_section = section_bytes.to_vec();
    normalized_section[relocation_offset..relocation_end].fill(0);
    if fnv1a64_hex(&normalized_section) == section.payload_hash {
        evidence.section_hash_status = "verified-after-relocation-normalization".to_owned();
    } else {
        evidence
            .blockers
            .push("native-entry:section-hash-mismatch".to_owned());
    }

    let Some(symbol_offset) = container.loader_symbol.offset else {
        evidence
            .blockers
            .push("native-entry:symbol-offset-missing".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(symbol_size) = container.loader_symbol.size_bytes.filter(|size| *size > 0) else {
        evidence
            .blockers
            .push("native-entry:symbol-size-invalid".to_owned());
        return blocked_recovery(evidence);
    };
    evidence.code_offset = Some(symbol_offset);
    evidence.code_size_bytes = Some(symbol_size);
    if symbol_offset
        != relocation
            .source_offset
            .saturating_add(RELOCATION_SLOT_BYTES)
    {
        evidence
            .blockers
            .push("native-entry:symbol-relocation-layout-mismatch".to_owned());
    }
    let Some(symbol_end) = symbol_offset.checked_add(symbol_size) else {
        evidence
            .blockers
            .push("native-entry:symbol-range-overflow".to_owned());
        return blocked_recovery(evidence);
    };
    let Some(code_bytes) = payload.get(symbol_offset..symbol_end) else {
        evidence
            .blockers
            .push("native-entry:symbol-out-of-bounds".to_owned());
        return blocked_recovery(evidence);
    };
    if container
        .loader_symbol
        .payload_hash
        .as_deref()
        .is_some_and(|expected| fnv1a64_hex(code_bytes) == expected)
    {
        evidence.code_hash_status = "verified".to_owned();
    } else {
        evidence
            .blockers
            .push("native-entry:code-hash-mismatch".to_owned());
    }
    evidence.ready = evidence.blockers.is_empty();
    evidence.status = if evidence.ready {
        "bytes-verified"
    } else {
        "blocked"
    }
    .to_owned();
    RecoveredNativeEntry {
        evidence,
        mapped_payload_hash,
        mapped_payload_size_bytes: payload.len(),
        code_bytes: code_bytes.to_vec(),
    }
}

pub(super) fn prepare_native_entry(
    recovered: &mut RecoveredNativeEntry,
    transfer: &CompiledEntryTransferResult,
) {
    if !recovered.evidence.ready {
        recovered.evidence.preparation_status = "not-attempted".to_owned();
        return;
    }
    let request = match ExecutableEntryRequest::from_transfer(transfer, &recovered.code_bytes) {
        Ok(request) => request,
        Err(blocker) => {
            recovered.evidence.blockers.push(blocker);
            recovered.evidence.status = "blocked".to_owned();
            recovered.evidence.preparation_status = "blocked".to_owned();
            return;
        }
    };
    recovered.evidence.target_machine_arch = Some(request.target_machine_arch.to_owned());
    let preparation = NativeHostExecutableMemoryAdapter.prepare(&request);
    recovered.evidence.preparation_protocol = Some(preparation.protocol.to_owned());
    recovered.evidence.preparation_status = preparation.status.to_owned();
    recovered.evidence.preparation_ready = preparation.ready;
    recovered.evidence.mapping_size_bytes = preparation.mapping_size_bytes;
    recovered.evidence.protection_status = preparation.protection_status.to_owned();
    recovered.evidence.host_machine_arch = preparation.host_machine_arch.clone();
    recovered.evidence.machine_arch_status = preparation.machine_arch_status.to_owned();
    recovered
        .evidence
        .blockers
        .extend(preparation.blockers.iter().cloned());
    recovered.evidence.ready = recovered.evidence.blockers.is_empty() && preparation.ready;
    recovered.evidence.status = if recovered.evidence.ready {
        "prepared"
    } else {
        "blocked"
    }
    .to_owned();
    drop(preparation);
}

fn blocked_recovery(mut evidence: NativeEntryHandoffEvidence) -> RecoveredNativeEntry {
    evidence.status = "blocked".to_owned();
    RecoveredNativeEntry {
        evidence,
        mapped_payload_hash: "none".to_owned(),
        mapped_payload_size_bytes: 0,
        code_bytes: Vec::new(),
    }
}

fn empty_evidence() -> NativeEntryHandoffEvidence {
    NativeEntryHandoffEvidence {
        protocol: NATIVE_ENTRY_HANDOFF_PROTOCOL,
        status: "not-attempted".to_owned(),
        ready: false,
        container_payload_offset: None,
        container_payload_size_bytes: None,
        container_payload_hash: None,
        section_id: None,
        section_hash_status: "not-verified".to_owned(),
        code_offset: None,
        code_size_bytes: None,
        code_hash_status: "not-verified".to_owned(),
        target_machine_arch: None,
        host_machine_arch: nuis_runtime::native_host_machine_arch().map(str::to_owned),
        machine_arch_status: "not-verified".to_owned(),
        preparation_protocol: None,
        preparation_status: "not-attempted".to_owned(),
        preparation_ready: false,
        mapping_size_bytes: 0,
        protection_status: "not-mapped".to_owned(),
        invocation_status: "not-invoked",
        blockers: Vec::new(),
    }
}

fn container_capsule_size(region: &[u8]) -> Option<usize> {
    region
        .windows(CONTAINER_CAPSULE_END_MARKER.len())
        .position(|window| window == CONTAINER_CAPSULE_END_MARKER)
        .and_then(|offset| offset.checked_add(CONTAINER_CAPSULE_END_MARKER.len()))
}

fn align_to(value: usize, alignment: usize) -> Option<usize> {
    let remainder = value % alignment;
    if remainder == 0 {
        Some(value)
    } else {
        value.checked_add(alignment - remainder)
    }
}
