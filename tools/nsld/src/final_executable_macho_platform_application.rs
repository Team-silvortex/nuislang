use crate::{
    final_executable_macho_application::{
        MachOArm64AppliedImage, MACHO_ARM64_PATCH_APPLICATION_CONTRACT,
    },
    final_executable_macho_layout::MACHO_PLACEMENT_BINDING_CONTRACT,
    final_executable_macho_materialization::encode_macho_arm64_platform_patch,
    final_executable_macho_platform::{
        build_macho_arm64_platform_structure_plan, MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT,
    },
    final_executable_macho_relocation::MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT,
    reports::{
        NsldMachOArm64PlatformBindRecord, NsldMachOArm64PlatformPatchApplicationReport,
        NsldMachOArm64PlatformPatchAudit, NsldMachOArm64PlatformRelocationBinding,
        NsldMachOArm64PlatformStructurePlanReport, NsldMachOArm64PlatformTargetPlan,
        NsldMachOArm64PlatformWriteAudit, NsldMachOArm64RelocationApplication,
        NsldMachOArm64RelocationApplicationReport, NsldMachOPlacementBindingReport,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT: &str =
    "nuis-nsld-macho-arm64-platform-patch-application-v1";

const STUB_WIDTH: usize = 12;
const GOT_WIDTH: usize = 8;

#[derive(Debug)]
pub(crate) struct MachOArm64PlatformAppliedImage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) report: NsldMachOArm64PlatformPatchApplicationReport,
}

pub(crate) fn apply_macho_arm64_platform_structure(
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    applied: &MachOArm64AppliedImage,
    plan: &NsldMachOArm64PlatformStructurePlanReport,
) -> Result<MachOArm64PlatformAppliedImage, String> {
    validate_input_envelope(placement, relocations, applied, plan)?;

    let mut bytes = applied.bytes.clone();
    bytes.resize(plan.planned_image_span_bytes, 0);
    let mut occupied = vec![false; bytes.len()];
    reserve_direct_patch_spans(applied, &bytes, &mut occupied)?;

    let mut structure_writes = Vec::with_capacity(plan.stub_entry_count + plan.got_entry_count);
    let mut bind_records = Vec::with_capacity(plan.got_entry_count);
    for target in &plan.targets {
        if let Some(stub_offset) = target.stub_output_offset {
            let got_offset = target.got_output_offset.ok_or_else(|| {
                format!(
                    "Mach-O platform target `{}` has a stub without a GOT slot",
                    target.structure_id
                )
            })?;
            let encoded = encode_arm64_stub(stub_offset, got_offset, &target.structure_id)?;
            write_structure(
                &mut bytes,
                &mut occupied,
                target,
                "arm64-branch-stub",
                stub_offset,
                &encoded,
                &mut structure_writes,
            )?;
        }
        if let Some(got_offset) = target.got_output_offset {
            let (write_kind, encoded) = got_entry_bytes(target)?;
            write_structure(
                &mut bytes,
                &mut occupied,
                target,
                write_kind,
                got_offset,
                &encoded,
                &mut structure_writes,
            )?;
            if target.resolver_status == "external-compatibility" {
                bind_records.push(build_bind_record(
                    bind_records.len(),
                    target,
                    got_offset,
                    &encoded,
                ));
            }
        }
    }
    validate_structure_write_coverage(plan, &structure_writes, &bind_records)?;

    let applications = relocation_application_map(relocations)?;
    let targets = target_map(plan)?;
    let mut seen_relocations = BTreeSet::new();
    let mut patches = Vec::with_capacity(plan.deferred_relocation_count);
    for binding in &plan.relocation_bindings {
        if !seen_relocations.insert(binding.relocation_id.as_str()) {
            return Err(format!(
                "Mach-O platform patch application repeats relocation `{}`",
                binding.relocation_id
            ));
        }
        let application = applications
            .get(binding.relocation_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "Mach-O platform binding references missing relocation `{}`",
                    binding.relocation_id
                )
            })?;
        let target = targets
            .get(binding.structure_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "Mach-O platform binding references missing target `{}`",
                    binding.structure_id
                )
            })?;
        validate_binding_identity(application, binding, target)?;
        let range = checked_range(
            binding.source_output_offset,
            binding.width_bytes,
            plan.base_image_span_bytes,
            &binding.relocation_id,
        )?;
        let source = bytes[range].to_vec();
        let (encoded, effective_addend) = encode_macho_arm64_platform_patch(
            application,
            &source,
            binding.patch_target_output_offset,
            &applications,
        )?;
        if encoded.len() != binding.width_bytes {
            return Err(format!(
                "Mach-O platform patch `{}` encoded width drift",
                binding.relocation_id
            ));
        }
        apply_write_once(
            &mut bytes,
            &mut occupied,
            binding.source_output_offset,
            &source,
            &encoded,
            &binding.relocation_id,
        )?;
        patches.push(build_patch_audit(
            application,
            binding,
            effective_addend,
            &source,
            &encoded,
        ));
    }
    if seen_relocations.len() != plan.deferred_relocation_count {
        return Err(format!(
            "Mach-O platform patch coverage drift: planned={}, applied={}",
            plan.deferred_relocation_count,
            seen_relocations.len()
        ));
    }

    let platform_image_hash = crate::fnv1a64_hex(&bytes);
    let status = if plan.deferred_relocation_count == 0 {
        "not-required"
    } else if bind_records.is_empty() {
        "platform-patches-applied"
    } else {
        "platform-patches-applied-with-unresolved-binds"
    };
    let write_once_span_count = structure_writes
        .len()
        .checked_add(patches.len())
        .ok_or_else(|| "Mach-O platform write count overflows".to_owned())?;
    let application_ledger_hash = platform_application_ledger_hash(
        status,
        placement,
        relocations,
        applied,
        plan,
        &platform_image_hash,
        &structure_writes,
        &patches,
        &bind_records,
    );
    Ok(MachOArm64PlatformAppliedImage {
        bytes,
        report: NsldMachOArm64PlatformPatchApplicationReport {
            contract: MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT.to_owned(),
            status: status.to_owned(),
            placement_plan_hash: placement.plan_hash.clone(),
            relocation_plan_hash: relocations.plan_hash.clone(),
            direct_patch_application_ledger_hash: applied.report.application_ledger_hash.clone(),
            platform_structure_plan_hash: plan.plan_hash.clone(),
            base_applied_image_hash: applied.report.applied_image_hash.clone(),
            platform_image_hash,
            base_image_span_bytes: plan.base_image_span_bytes,
            platform_image_span_bytes: plan.planned_image_span_bytes,
            expected_deferred_patch_count: plan.deferred_relocation_count,
            applied_deferred_patch_count: patches.len(),
            stub_write_count: plan.stub_entry_count,
            got_write_count: plan.got_entry_count,
            unresolved_bind_count: bind_records.len(),
            write_once_span_count,
            application_ledger_hash,
            structure_writes,
            patches,
            bind_records,
        },
    })
}

fn validate_input_envelope(
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    applied: &MachOArm64AppliedImage,
    plan: &NsldMachOArm64PlatformStructurePlanReport,
) -> Result<(), String> {
    if placement.contract != MACHO_PLACEMENT_BINDING_CONTRACT
        || relocations.contract != MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT
        || applied.report.contract != MACHO_ARM64_PATCH_APPLICATION_CONTRACT
        || plan.contract != MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT
    {
        return Err("Mach-O platform patch application rejects an upstream contract".to_owned());
    }
    if applied.bytes.len() != applied.report.image_span_bytes
        || crate::fnv1a64_hex(&applied.bytes) != applied.report.applied_image_hash
    {
        return Err("Mach-O platform patch application base image drift".to_owned());
    }
    let expected =
        build_macho_arm64_platform_structure_plan(placement, relocations, &applied.report)?;
    if &expected != plan {
        return Err("Mach-O platform patch application plan drift".to_owned());
    }
    if plan.planned_image_span_bytes < plan.base_image_span_bytes {
        return Err("Mach-O platform plan shrinks the applied image".to_owned());
    }
    Ok(())
}

fn reserve_direct_patch_spans(
    applied: &MachOArm64AppliedImage,
    bytes: &[u8],
    occupied: &mut [bool],
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for patch in &applied.report.patches {
        if !seen.insert(patch.relocation_id.as_str()) {
            return Err(format!(
                "Mach-O platform patch application repeats inherited patch `{}`",
                patch.relocation_id
            ));
        }
        let range = checked_range(
            patch.source_output_offset,
            patch.width_bytes,
            applied.report.image_span_bytes,
            &patch.relocation_id,
        )?;
        if occupied[range.clone()].iter().any(|value| *value) {
            return Err(format!(
                "Mach-O inherited patch `{}` overlaps an earlier direct patch",
                patch.relocation_id
            ));
        }
        let observed_hash = crate::fnv1a64_hex(&bytes[range.clone()]);
        if observed_hash != patch.post_write_bytes_hash
            || patch.post_write_bytes_hash != patch.encoded_bytes_hash
        {
            return Err(format!(
                "Mach-O inherited patch `{}` byte hash drift",
                patch.relocation_id
            ));
        }
        occupied[range].fill(true);
    }
    if seen.len() != applied.report.applied_patch_count
        || applied.report.write_once_span_count != applied.report.applied_patch_count
    {
        return Err("Mach-O inherited direct patch coverage drift".to_owned());
    }
    Ok(())
}

fn write_structure(
    bytes: &mut [u8],
    occupied: &mut [bool],
    target: &NsldMachOArm64PlatformTargetPlan,
    write_kind: &str,
    output_offset: usize,
    encoded: &[u8],
    audits: &mut Vec<NsldMachOArm64PlatformWriteAudit>,
) -> Result<(), String> {
    let write_id = format!("macho-arm64-platform-write-{:06}", audits.len());
    let source = vec![0; encoded.len()];
    apply_write_once(bytes, occupied, output_offset, &source, encoded, &write_id)?;
    let encoded_bytes_hash = crate::fnv1a64_hex(encoded);
    let write_audit_hash = structure_write_audit_hash(
        &write_id,
        target,
        write_kind,
        output_offset,
        encoded.len(),
        &encoded_bytes_hash,
    );
    audits.push(NsldMachOArm64PlatformWriteAudit {
        write_id,
        structure_id: target.structure_id.clone(),
        write_kind: write_kind.to_owned(),
        target_symbol: target.target_symbol.clone(),
        output_offset,
        width_bytes: encoded.len(),
        encoded_bytes_hex: hex_bytes(encoded),
        encoded_bytes_hash,
        write_audit_hash,
    });
    Ok(())
}

fn got_entry_bytes(
    target: &NsldMachOArm64PlatformTargetPlan,
) -> Result<(&'static str, Vec<u8>), String> {
    match target.resolver_status.as_str() {
        "external-compatibility" => Ok(("unresolved-external-got-placeholder", vec![0; GOT_WIDTH])),
        "internal" | "internal-symbol" => {
            match (target.target_output_offset, target.target_absolute_value) {
                (Some(output_offset), None) => {
                    let value = u64::try_from(output_offset).map_err(|_| {
                        format!(
                            "Mach-O internal GOT target `{}` exceeds 64-bit image-relative space",
                            target.structure_id
                        )
                    })?;
                    Ok(("internal-image-relative-got", value.to_le_bytes().to_vec()))
                }
                (None, Some(value)) => Ok(("internal-absolute-got", value.to_le_bytes().to_vec())),
                _ => Err(format!(
                    "Mach-O internal GOT target `{}` has an invalid target coordinate",
                    target.structure_id
                )),
            }
        }
        other => Err(format!(
            "Mach-O platform target `{}` has unsupported resolver status `{other}`",
            target.structure_id
        )),
    }
}

fn build_bind_record(
    index: usize,
    target: &NsldMachOArm64PlatformTargetPlan,
    got_output_offset: usize,
    placeholder: &[u8],
) -> NsldMachOArm64PlatformBindRecord {
    let bind_id = format!("macho-arm64-platform-bind-{index:06}");
    let placeholder_bytes_hash = crate::fnv1a64_hex(placeholder);
    let audit_hash = bind_audit_hash(&bind_id, target, got_output_offset, &placeholder_bytes_hash);
    NsldMachOArm64PlatformBindRecord {
        bind_id,
        structure_id: target.structure_id.clone(),
        target_key: target.target_key.clone(),
        target_symbol: target.target_symbol.clone(),
        got_output_offset,
        width_bytes: placeholder.len(),
        placeholder_bytes_hash,
        status: "unresolved-external".to_owned(),
        audit_hash,
    }
}

fn validate_structure_write_coverage(
    plan: &NsldMachOArm64PlatformStructurePlanReport,
    writes: &[NsldMachOArm64PlatformWriteAudit],
    binds: &[NsldMachOArm64PlatformBindRecord],
) -> Result<(), String> {
    let stub_count = writes
        .iter()
        .filter(|write| write.write_kind == "arm64-branch-stub")
        .count();
    let got_count = writes
        .len()
        .checked_sub(stub_count)
        .ok_or_else(|| "Mach-O platform structure write accounting underflows".to_owned())?;
    let expected_binds = plan
        .targets
        .iter()
        .filter(|target| {
            target.resolver_status == "external-compatibility" && target.got_output_offset.is_some()
        })
        .count();
    if stub_count != plan.stub_entry_count
        || got_count != plan.got_entry_count
        || binds.len() != expected_binds
    {
        return Err("Mach-O platform structure write coverage drift".to_owned());
    }
    Ok(())
}

fn relocation_application_map(
    relocations: &NsldMachOArm64RelocationApplicationReport,
) -> Result<BTreeMap<&str, &NsldMachOArm64RelocationApplication>, String> {
    let mut applications = BTreeMap::new();
    for application in &relocations.applications {
        if applications
            .insert(application.relocation_id.as_str(), application)
            .is_some()
        {
            return Err(format!(
                "Mach-O relocation report repeats id `{}`",
                application.relocation_id
            ));
        }
    }
    Ok(applications)
}

fn target_map(
    plan: &NsldMachOArm64PlatformStructurePlanReport,
) -> Result<BTreeMap<&str, &NsldMachOArm64PlatformTargetPlan>, String> {
    let mut targets = BTreeMap::new();
    for target in &plan.targets {
        if targets
            .insert(target.structure_id.as_str(), target)
            .is_some()
        {
            return Err(format!(
                "Mach-O platform plan repeats target `{}`",
                target.structure_id
            ));
        }
    }
    Ok(targets)
}

fn validate_binding_identity(
    application: &NsldMachOArm64RelocationApplication,
    binding: &NsldMachOArm64PlatformRelocationBinding,
    target: &NsldMachOArm64PlatformTargetPlan,
) -> Result<(), String> {
    let expected_target = match binding.patch_target_kind.as_str() {
        "branch-stub" => target.stub_output_offset,
        "got-entry" => target.got_output_offset,
        other => {
            return Err(format!(
                "Mach-O platform binding `{}` has unsupported patch target `{other}`",
                binding.relocation_id
            ))
        }
    };
    if application.application_status != "planned-platform-structure"
        || application.relocation_kind != binding.relocation_kind
        || application.action_kind != binding.action_kind
        || application.source_output_offset != binding.source_output_offset
        || application.width_bytes != binding.width_bytes
        || expected_target != Some(binding.patch_target_output_offset)
        || !target.relocation_ids.contains(&binding.relocation_id)
    {
        return Err(format!(
            "Mach-O platform binding `{}` identity drift",
            binding.relocation_id
        ));
    }
    Ok(())
}

fn build_patch_audit(
    application: &NsldMachOArm64RelocationApplication,
    binding: &NsldMachOArm64PlatformRelocationBinding,
    effective_addend: i64,
    source: &[u8],
    encoded: &[u8],
) -> NsldMachOArm64PlatformPatchAudit {
    let source_bytes_hash = crate::fnv1a64_hex(source);
    let encoded_bytes_hash = crate::fnv1a64_hex(encoded);
    let write_audit_hash = platform_patch_audit_hash(
        application,
        binding,
        effective_addend,
        &source_bytes_hash,
        &encoded_bytes_hash,
    );
    NsldMachOArm64PlatformPatchAudit {
        relocation_id: application.relocation_id.clone(),
        relocation_kind: application.relocation_kind.clone(),
        source_output_offset: application.source_output_offset,
        width_bytes: application.width_bytes,
        patch_target_output_offset: binding.patch_target_output_offset,
        effective_addend,
        source_bytes_hex: hex_bytes(source),
        encoded_bytes_hex: hex_bytes(encoded),
        source_bytes_hash,
        encoded_bytes_hash,
        binding_audit_hash: binding.audit_hash.clone(),
        write_audit_hash,
    }
}

fn encode_arm64_stub(
    stub_output_offset: usize,
    got_output_offset: usize,
    structure_id: &str,
) -> Result<Vec<u8>, String> {
    if !stub_output_offset.is_multiple_of(4) || !got_output_offset.is_multiple_of(GOT_WIDTH) {
        return Err(format!(
            "Mach-O platform stub `{structure_id}` has unaligned stub or GOT placement"
        ));
    }
    let source_page = stub_output_offset as i128 & !0xfff;
    let target_page = got_output_offset as i128 & !0xfff;
    let page_delta = target_page - source_page;
    if page_delta % 4096 != 0 || !(-0x1_0000_0000..=0x0_ffff_f000).contains(&page_delta) {
        return Err(format!(
            "Mach-O platform stub `{structure_id}` GOT page delta {page_delta} is out of range"
        ));
    }
    let page_immediate = ((page_delta >> 12) as i64 as u32) & 0x001f_ffff;
    let adrp =
        0x9000_0010u32 | (page_immediate & 0x3) << 29 | ((page_immediate >> 2) & 0x7ffff) << 5;
    let page_offset = got_output_offset & 0x0fff;
    if !page_offset.is_multiple_of(GOT_WIDTH) {
        return Err(format!(
            "Mach-O platform stub `{structure_id}` GOT page offset {page_offset} is unaligned"
        ));
    }
    let ldr_immediate = page_offset / GOT_WIDTH;
    if ldr_immediate > 0x0fff {
        return Err(format!(
            "Mach-O platform stub `{structure_id}` GOT page offset is out of range"
        ));
    }
    let ldr = 0xf940_0210u32 | (ldr_immediate as u32) << 10;
    let branch = 0xd61f_0200u32;
    let mut encoded = Vec::with_capacity(STUB_WIDTH);
    encoded.extend_from_slice(&adrp.to_le_bytes());
    encoded.extend_from_slice(&ldr.to_le_bytes());
    encoded.extend_from_slice(&branch.to_le_bytes());
    Ok(encoded)
}

fn apply_write_once(
    image: &mut [u8],
    occupied: &mut [bool],
    offset: usize,
    source: &[u8],
    encoded: &[u8],
    label: &str,
) -> Result<(), String> {
    if image.len() != occupied.len() || source.len() != encoded.len() {
        return Err(format!(
            "Mach-O platform write `{label}` buffer shape drift"
        ));
    }
    let range = checked_range(offset, encoded.len(), image.len(), label)?;
    if occupied[range.clone()].iter().any(|value| *value) {
        return Err(format!(
            "Mach-O platform write `{label}` overlaps a previously committed span"
        ));
    }
    if image[range.clone()] != *source {
        return Err(format!("Mach-O platform write `{label}` source drift"));
    }
    image[range.clone()].copy_from_slice(encoded);
    occupied[range].fill(true);
    Ok(())
}

fn checked_range(
    offset: usize,
    size: usize,
    limit: usize,
    label: &str,
) -> Result<std::ops::Range<usize>, String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("Mach-O platform span `{label}` overflows"))?;
    if end > limit {
        return Err(format!(
            "Mach-O platform span `{label}` {offset}..{end} exceeds image {limit}"
        ));
    }
    Ok(offset..end)
}

fn structure_write_audit_hash(
    write_id: &str,
    target: &NsldMachOArm64PlatformTargetPlan,
    write_kind: &str,
    output_offset: usize,
    width_bytes: usize,
    encoded_bytes_hash: &str,
) -> String {
    let mut canonical = String::new();
    append_text(
        &mut canonical,
        MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT,
    );
    append_text(&mut canonical, write_id);
    append_text(&mut canonical, &target.structure_id);
    append_text(&mut canonical, &target.audit_hash);
    append_text(&mut canonical, write_kind);
    append_text(&mut canonical, encoded_bytes_hash);
    writeln!(canonical, "span={output_offset}|{width_bytes}").unwrap();
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn bind_audit_hash(
    bind_id: &str,
    target: &NsldMachOArm64PlatformTargetPlan,
    got_output_offset: usize,
    placeholder_bytes_hash: &str,
) -> String {
    let mut canonical = String::new();
    append_text(
        &mut canonical,
        MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT,
    );
    append_text(&mut canonical, bind_id);
    append_text(&mut canonical, &target.structure_id);
    append_text(&mut canonical, &target.target_key);
    append_text(&mut canonical, &target.target_symbol);
    append_text(&mut canonical, placeholder_bytes_hash);
    writeln!(canonical, "got={got_output_offset}|{GOT_WIDTH}").unwrap();
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn platform_patch_audit_hash(
    application: &NsldMachOArm64RelocationApplication,
    binding: &NsldMachOArm64PlatformRelocationBinding,
    effective_addend: i64,
    source_bytes_hash: &str,
    encoded_bytes_hash: &str,
) -> String {
    let mut canonical = String::new();
    append_text(
        &mut canonical,
        MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT,
    );
    append_text(&mut canonical, &application.relocation_id);
    append_text(&mut canonical, &binding.audit_hash);
    append_text(&mut canonical, source_bytes_hash);
    append_text(&mut canonical, encoded_bytes_hash);
    writeln!(
        canonical,
        "patch={}|{}|{}|{}",
        application.source_output_offset,
        application.width_bytes,
        binding.patch_target_output_offset,
        effective_addend
    )
    .unwrap();
    crate::fnv1a64_hex(canonical.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn platform_application_ledger_hash(
    status: &str,
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    applied: &MachOArm64AppliedImage,
    plan: &NsldMachOArm64PlatformStructurePlanReport,
    platform_image_hash: &str,
    writes: &[NsldMachOArm64PlatformWriteAudit],
    patches: &[NsldMachOArm64PlatformPatchAudit],
    binds: &[NsldMachOArm64PlatformBindRecord],
) -> String {
    let mut canonical = String::new();
    append_text(
        &mut canonical,
        MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT,
    );
    append_text(&mut canonical, status);
    append_text(&mut canonical, &placement.plan_hash);
    append_text(&mut canonical, &relocations.plan_hash);
    append_text(&mut canonical, &applied.report.application_ledger_hash);
    append_text(&mut canonical, &applied.report.applied_image_hash);
    append_text(&mut canonical, &plan.plan_hash);
    append_text(&mut canonical, platform_image_hash);
    writeln!(
        canonical,
        "counts={}|{}|{}|{}|{}",
        plan.base_image_span_bytes,
        plan.planned_image_span_bytes,
        writes.len(),
        patches.len(),
        binds.len()
    )
    .unwrap();
    for write in writes {
        append_text(&mut canonical, &write.write_id);
        append_text(&mut canonical, &write.write_audit_hash);
    }
    for patch in patches {
        append_text(&mut canonical, &patch.relocation_id);
        append_text(&mut canonical, &patch.write_audit_hash);
    }
    for bind in binds {
        append_text(&mut canonical, &bind.bind_id);
        append_text(&mut canonical, &bind.audit_hash);
    }
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
#[path = "final_executable_macho_platform_application_tests.rs"]
mod tests;
