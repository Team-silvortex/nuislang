use crate::{
    final_executable_macho_application::MACHO_ARM64_PATCH_APPLICATION_CONTRACT,
    final_executable_macho_layout::MACHO_PLACEMENT_BINDING_CONTRACT,
    final_executable_macho_relocation::MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT,
    reports::{
        NsldMachOArm64PatchApplicationReport, NsldMachOArm64PlatformRelocationBinding,
        NsldMachOArm64PlatformStructurePlanReport, NsldMachOArm64PlatformTargetPlan,
        NsldMachOArm64RelocationApplication, NsldMachOArm64RelocationApplicationReport,
        NsldMachOPlacementBindingReport,
    },
};
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT: &str =
    "nuis-nsld-macho-arm64-platform-structure-plan-v1";

const STUB_ENTRY_SIZE: usize = 12;
const STUB_ALIGNMENT: usize = 4;
const GOT_ENTRY_SIZE: usize = 8;
const GOT_ALIGNMENT: usize = 8;

#[derive(Clone, Copy)]
struct PlatformRule {
    id: &'static str,
    relocation_kind: &'static str,
    action_kind: &'static str,
    resolver_status: Option<&'static str>,
    requires_got: bool,
    requires_stub: bool,
    patch_target_kind: &'static str,
}

const PLATFORM_RULES: &[PlatformRule] = &[
    PlatformRule {
        id: "arm64.got-load-page21.v1",
        relocation_kind: "arm64-got-load-page21",
        action_kind: "rewrite-got-load-page21",
        resolver_status: None,
        requires_got: true,
        requires_stub: false,
        patch_target_kind: "got-entry",
    },
    PlatformRule {
        id: "arm64.got-load-pageoff12.v1",
        relocation_kind: "arm64-got-load-pageoff12",
        action_kind: "rewrite-got-load-pageoff12",
        resolver_status: None,
        requires_got: true,
        requires_stub: false,
        patch_target_kind: "got-entry",
    },
    PlatformRule {
        id: "arm64.external-branch26.v1",
        relocation_kind: "arm64-branch26",
        action_kind: "rewrite-branch26",
        resolver_status: Some("external-compatibility"),
        requires_got: true,
        requires_stub: true,
        patch_target_kind: "branch-stub",
    },
];

struct TargetSeed {
    target_key: String,
    target_symbol: String,
    resolver_status: String,
    target_object_id: Option<String>,
    target_section_id: Option<String>,
    target_output_offset: Option<usize>,
    target_absolute_value: Option<u64>,
    requires_got: bool,
    requires_stub: bool,
    relocation_ids: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TargetIdentity {
    symbol: String,
    resolver_status: String,
    target_object_id: Option<String>,
    target_section_id: Option<String>,
    target_output_offset: Option<usize>,
    target_absolute_value: Option<u64>,
}

impl TargetIdentity {
    fn canonical(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, &self.symbol);
        append_text(&mut out, &self.resolver_status);
        append_text(&mut out, self.target_object_id.as_deref().unwrap_or("none"));
        append_text(
            &mut out,
            self.target_section_id.as_deref().unwrap_or("none"),
        );
        writeln!(
            out,
            "target={}|{}",
            optional_usize(self.target_output_offset),
            self.target_absolute_value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned())
        )
        .unwrap();
        out
    }
}

struct PlannedTarget {
    canonical_identity: String,
    plan: NsldMachOArm64PlatformTargetPlan,
}

pub(crate) fn build_macho_arm64_platform_structure_plan(
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    applied: &NsldMachOArm64PatchApplicationReport,
) -> Result<NsldMachOArm64PlatformStructurePlanReport, String> {
    validate_input_envelope(placement, relocations, applied)?;
    let deferred = relocations
        .applications
        .iter()
        .filter(|item| item.application_status == "planned-platform-structure")
        .collect::<Vec<_>>();
    if deferred.len() != relocations.platform_structure_count {
        return Err("Mach-O platform plan deferred relocation coverage drift".to_owned());
    }

    let mut target_seeds = BTreeMap::<TargetIdentity, TargetSeed>::new();
    let mut relocation_rules = BTreeMap::<&str, PlatformRule>::new();
    for application in &deferred {
        let rule = registered_rule(application)?;
        if relocation_rules
            .insert(application.relocation_id.as_str(), rule)
            .is_some()
        {
            return Err(format!(
                "Mach-O platform plan repeats relocation `{}`",
                application.relocation_id
            ));
        }
        let (identity, seed) = target_seed(application, rule)?;
        match target_seeds.get_mut(&identity) {
            Some(existing) => {
                existing.requires_got |= rule.requires_got;
                existing.requires_stub |= rule.requires_stub;
                existing
                    .relocation_ids
                    .push(application.relocation_id.clone());
            }
            None => {
                target_seeds.insert(identity, seed);
            }
        }
    }

    let stub_entry_count = target_seeds
        .values()
        .filter(|target| target.requires_stub)
        .count();
    let got_entry_count = target_seeds
        .values()
        .filter(|target| target.requires_got)
        .count();
    let layout = platform_layout(applied.image_span_bytes, stub_entry_count, got_entry_count)?;
    let planned_targets = assign_target_slots(target_seeds, &layout)?;
    let target_lookup = planned_targets
        .iter()
        .map(|target| (target.canonical_identity.as_str(), &target.plan))
        .collect::<BTreeMap<_, _>>();
    let relocation_bindings = deferred
        .iter()
        .map(|application| {
            let rule = relocation_rules
                .get(application.relocation_id.as_str())
                .copied()
                .expect("registered deferred relocation rule must remain available");
            build_relocation_binding(application, rule, &target_lookup)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let targets = planned_targets
        .into_iter()
        .map(|target| target.plan)
        .collect::<Vec<_>>();
    let status = if deferred.is_empty() {
        "not-required"
    } else {
        "allocated-ready-for-platform-patching"
    };
    let plan_hash = platform_plan_hash(
        status,
        placement,
        relocations,
        applied,
        &layout,
        &targets,
        &relocation_bindings,
    );
    Ok(NsldMachOArm64PlatformStructurePlanReport {
        contract: MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT.to_owned(),
        status: status.to_owned(),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        patch_application_ledger_hash: applied.application_ledger_hash.clone(),
        applied_image_hash: applied.applied_image_hash.clone(),
        base_image_span_bytes: applied.image_span_bytes,
        planned_image_span_bytes: layout.planned_image_span_bytes,
        registered_rule_count: PLATFORM_RULES.len(),
        deferred_relocation_count: deferred.len(),
        target_count: targets.len(),
        stub_region_offset: layout.stub_region_offset,
        stub_region_bytes: layout.stub_region_bytes,
        stub_entry_size: STUB_ENTRY_SIZE,
        stub_alignment: STUB_ALIGNMENT,
        stub_entry_count,
        got_region_offset: layout.got_region_offset,
        got_region_bytes: layout.got_region_bytes,
        got_entry_size: GOT_ENTRY_SIZE,
        got_alignment: GOT_ALIGNMENT,
        got_entry_count,
        plan_hash,
        targets,
        relocation_bindings,
    })
}

fn validate_input_envelope(
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    applied: &NsldMachOArm64PatchApplicationReport,
) -> Result<(), String> {
    if placement.contract != MACHO_PLACEMENT_BINDING_CONTRACT
        || relocations.contract != MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT
    {
        return Err("Mach-O platform plan rejects an upstream contract".to_owned());
    }
    if applied.contract != MACHO_ARM64_PATCH_APPLICATION_CONTRACT {
        return Err(format!(
            "Mach-O platform plan rejects patch application contract `{}`",
            applied.contract
        ));
    }
    if relocations.placement_plan_hash != placement.plan_hash
        || applied.placement_plan_hash != placement.plan_hash
        || applied.relocation_plan_hash != relocations.plan_hash
    {
        return Err("Mach-O platform plan input hash drift".to_owned());
    }
    let direct_count = relocations
        .applications
        .iter()
        .filter(|item| item.application_status == "planned-direct")
        .count();
    let platform_count = relocations
        .applications
        .iter()
        .filter(|item| item.application_status == "planned-platform-structure")
        .count();
    let metadata_count = relocations
        .applications
        .iter()
        .filter(|item| item.application_status == "paired-metadata")
        .count();
    let external_count = relocations
        .applications
        .iter()
        .filter(|item| item.resolver_status == "external-compatibility")
        .count();
    let expected_relocation_status = if platform_count == 0 {
        "ready-for-byte-encoding"
    } else {
        "planned-with-platform-structure-boundary"
    };
    if relocations.status != expected_relocation_status
        || relocations.relocation_count != relocations.applications.len()
        || direct_count + platform_count + metadata_count != relocations.applications.len()
        || relocations.ready_application_count != direct_count
        || relocations.platform_structure_count != platform_count
        || relocations.metadata_record_count != metadata_count
        || relocations.external_compatibility_count != external_count
    {
        return Err("Mach-O platform plan relocation envelope drift".to_owned());
    }
    let expected_status = if relocations.platform_structure_count == 0 {
        "direct-patches-applied"
    } else {
        "direct-patches-applied-with-platform-structure-boundary"
    };
    if applied.status != expected_status
        || applied.image_span_bytes != placement.image_span_bytes
        || applied.expected_patch_count != relocations.ready_application_count
        || applied.applied_patch_count != applied.expected_patch_count
        || applied.write_once_span_count != applied.applied_patch_count
        || applied.patches.len() != applied.applied_patch_count
        || applied.deferred_patch_count != relocations.platform_structure_count
        || applied.applied_image_hash.is_empty()
        || applied.application_ledger_hash.is_empty()
    {
        return Err("Mach-O platform plan patch application envelope drift".to_owned());
    }
    Ok(())
}

fn registered_rule(
    application: &NsldMachOArm64RelocationApplication,
) -> Result<PlatformRule, String> {
    let matches = PLATFORM_RULES
        .iter()
        .filter(|rule| {
            rule.relocation_kind == application.relocation_kind
                && rule.action_kind == application.action_kind
                && rule
                    .resolver_status
                    .is_none_or(|status| status == application.resolver_status)
        })
        .copied()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [rule] => Ok(*rule),
        [] => Err(format!(
            "Mach-O deferred relocation `{}` has no registered platform structure rule for kind `{}`, action `{}`, resolver `{}`",
            application.relocation_id,
            application.relocation_kind,
            application.action_kind,
            application.resolver_status
        )),
        _ => Err(format!(
            "Mach-O deferred relocation `{}` matches multiple platform structure rules",
            application.relocation_id
        )),
    }
}

fn target_seed(
    application: &NsldMachOArm64RelocationApplication,
    rule: PlatformRule,
) -> Result<(TargetIdentity, TargetSeed), String> {
    let target_symbol = application
        .target_symbol
        .as_deref()
        .filter(|symbol| !symbol.is_empty())
        .ok_or_else(|| {
            format!(
                "Mach-O deferred relocation `{}` has no target symbol",
                application.relocation_id
            )
        })?;
    match application.resolver_status.as_str() {
        "external-compatibility" => {
            if application.target_object_id.is_some()
                || application.target_section_id.is_some()
                || application.target_output_offset.is_some()
                || application.target_absolute_value.is_some()
            {
                return Err(format!(
                    "Mach-O external target `{target_symbol}` unexpectedly owns an internal placement"
                ));
            }
        }
        "internal" | "internal-symbol" => {
            let image_target = application.target_section_id.is_some()
                && application.target_output_offset.is_some()
                && application.target_absolute_value.is_none();
            let absolute_target = application.target_section_id.is_none()
                && application.target_output_offset.is_none()
                && application.target_absolute_value.is_some();
            if application.target_object_id.is_none() || !(image_target || absolute_target) {
                return Err(format!(
                    "Mach-O internal platform target `{target_symbol}` has an incomplete placement"
                ));
            }
        }
        other => {
            return Err(format!(
                "Mach-O platform target `{target_symbol}` has unsupported resolver status `{other}`"
            ))
        }
    }
    let identity = target_identity(application, target_symbol);
    let target_key = crate::fnv1a64_hex(identity.canonical().as_bytes());
    Ok((
        identity,
        TargetSeed {
            target_key,
            target_symbol: target_symbol.to_owned(),
            resolver_status: application.resolver_status.clone(),
            target_object_id: application.target_object_id.clone(),
            target_section_id: application.target_section_id.clone(),
            target_output_offset: application.target_output_offset,
            target_absolute_value: application.target_absolute_value,
            requires_got: rule.requires_got,
            requires_stub: rule.requires_stub,
            relocation_ids: vec![application.relocation_id.clone()],
        },
    ))
}

fn target_identity(
    application: &NsldMachOArm64RelocationApplication,
    symbol: &str,
) -> TargetIdentity {
    TargetIdentity {
        symbol: symbol.to_owned(),
        resolver_status: application.resolver_status.clone(),
        target_object_id: application.target_object_id.clone(),
        target_section_id: application.target_section_id.clone(),
        target_output_offset: application.target_output_offset,
        target_absolute_value: application.target_absolute_value,
    }
}

struct PlatformLayout {
    stub_region_offset: usize,
    stub_region_bytes: usize,
    got_region_offset: usize,
    got_region_bytes: usize,
    planned_image_span_bytes: usize,
}

fn platform_layout(
    base_image_span: usize,
    stub_count: usize,
    got_count: usize,
) -> Result<PlatformLayout, String> {
    let stub_region_offset = if stub_count == 0 {
        base_image_span
    } else {
        align_up(base_image_span, STUB_ALIGNMENT)?
    };
    let stub_region_bytes = stub_count
        .checked_mul(STUB_ENTRY_SIZE)
        .ok_or_else(|| "Mach-O stub region size overflows".to_owned())?;
    let stub_end = stub_region_offset
        .checked_add(stub_region_bytes)
        .ok_or_else(|| "Mach-O stub region end overflows".to_owned())?;
    let got_region_offset = if got_count == 0 {
        stub_end
    } else {
        align_up(stub_end, GOT_ALIGNMENT)?
    };
    let got_region_bytes = got_count
        .checked_mul(GOT_ENTRY_SIZE)
        .ok_or_else(|| "Mach-O GOT region size overflows".to_owned())?;
    let planned_image_span_bytes = got_region_offset
        .checked_add(got_region_bytes)
        .ok_or_else(|| "Mach-O platform image span overflows".to_owned())?;
    Ok(PlatformLayout {
        stub_region_offset,
        stub_region_bytes,
        got_region_offset,
        got_region_bytes,
        planned_image_span_bytes,
    })
}

fn assign_target_slots(
    seeds: BTreeMap<TargetIdentity, TargetSeed>,
    layout: &PlatformLayout,
) -> Result<Vec<PlannedTarget>, String> {
    let mut next_stub = 0usize;
    let mut next_got = 0usize;
    let mut targets = Vec::with_capacity(seeds.len());
    for (index, (identity, seed)) in seeds.into_iter().enumerate() {
        let structure_id = format!("macho-arm64-platform-target-{index:06}");
        let (stub_slot_index, stub_output_offset) = if seed.requires_stub {
            let slot = next_stub;
            next_stub += 1;
            (
                Some(slot),
                Some(slot_offset(
                    layout.stub_region_offset,
                    slot,
                    STUB_ENTRY_SIZE,
                )?),
            )
        } else {
            (None, None)
        };
        let (got_slot_index, got_output_offset) = if seed.requires_got {
            let slot = next_got;
            next_got += 1;
            (
                Some(slot),
                Some(slot_offset(layout.got_region_offset, slot, GOT_ENTRY_SIZE)?),
            )
        } else {
            (None, None)
        };
        if let (Some(stub), Some(got)) = (stub_output_offset, got_output_offset) {
            validate_page21_reachability(stub, got, &structure_id)?;
            if got.checked_rem(GOT_ALIGNMENT) != Some(0) {
                return Err(format!(
                    "Mach-O platform target `{structure_id}` GOT slot is not {GOT_ALIGNMENT}-byte aligned"
                ));
            }
        }
        let audit_hash = target_audit_hash(
            &structure_id,
            &seed,
            got_slot_index,
            got_output_offset,
            stub_slot_index,
            stub_output_offset,
        );
        targets.push(PlannedTarget {
            canonical_identity: identity.canonical(),
            plan: NsldMachOArm64PlatformTargetPlan {
                structure_id,
                target_key: seed.target_key,
                target_symbol: seed.target_symbol,
                resolver_status: seed.resolver_status,
                target_object_id: seed.target_object_id,
                target_section_id: seed.target_section_id,
                target_output_offset: seed.target_output_offset,
                target_absolute_value: seed.target_absolute_value,
                got_slot_index,
                got_output_offset,
                stub_slot_index,
                stub_output_offset,
                relocation_ids: seed.relocation_ids,
                audit_hash,
            },
        });
    }
    Ok(targets)
}

fn build_relocation_binding(
    application: &NsldMachOArm64RelocationApplication,
    rule: PlatformRule,
    targets: &BTreeMap<&str, &NsldMachOArm64PlatformTargetPlan>,
) -> Result<NsldMachOArm64PlatformRelocationBinding, String> {
    let symbol = application.target_symbol.as_deref().ok_or_else(|| {
        format!(
            "Mach-O deferred relocation `{}` lost its target symbol",
            application.relocation_id
        )
    })?;
    let identity = target_identity(application, symbol).canonical();
    let target = targets.get(identity.as_str()).copied().ok_or_else(|| {
        format!(
            "Mach-O deferred relocation `{}` has no allocated platform target",
            application.relocation_id
        )
    })?;
    let patch_target_output_offset = match rule.patch_target_kind {
        "got-entry" => target.got_output_offset,
        "branch-stub" => target.stub_output_offset,
        other => {
            return Err(format!(
                "Mach-O platform rule `{}` has unknown patch target `{other}`",
                rule.id
            ))
        }
    }
    .ok_or_else(|| {
        format!(
            "Mach-O platform target `{}` has no `{}` slot",
            target.structure_id, rule.patch_target_kind
        )
    })?;
    validate_relocation_reachability(application, rule, patch_target_output_offset)?;
    let audit_hash = binding_audit_hash(application, rule, target, patch_target_output_offset);
    Ok(NsldMachOArm64PlatformRelocationBinding {
        relocation_id: application.relocation_id.clone(),
        relocation_kind: application.relocation_kind.clone(),
        action_kind: application.action_kind.clone(),
        source_output_offset: application.source_output_offset,
        width_bytes: application.width_bytes,
        structure_id: target.structure_id.clone(),
        patch_target_kind: rule.patch_target_kind.to_owned(),
        patch_target_output_offset,
        audit_hash,
    })
}

fn validate_relocation_reachability(
    application: &NsldMachOArm64RelocationApplication,
    rule: PlatformRule,
    target: usize,
) -> Result<(), String> {
    match rule.patch_target_kind {
        "branch-stub" => {
            let displacement = target as i128 - application.source_output_offset as i128;
            if displacement.checked_rem(4) != Some(0)
                || !(-0x0800_0000..=0x07ff_fffc).contains(&displacement)
            {
                return Err(format!(
                    "Mach-O platform branch `{}` displacement {displacement} is unaligned or out of range",
                    application.relocation_id
                ));
            }
        }
        "got-entry" if application.relocation_kind == "arm64-got-load-page21" => {
            validate_page21_reachability(
                application.source_output_offset,
                target,
                &application.relocation_id,
            )?;
        }
        "got-entry" if application.relocation_kind == "arm64-got-load-pageoff12" => {
            if target.checked_rem(GOT_ALIGNMENT) != Some(0) {
                return Err(format!(
                    "Mach-O platform GOT target for `{}` is not {GOT_ALIGNMENT}-byte aligned",
                    application.relocation_id
                ));
            }
        }
        "got-entry" => {}
        other => {
            return Err(format!(
                "Mach-O platform rule `{}` has unsupported reachability target `{other}`",
                rule.id
            ))
        }
    }
    Ok(())
}

fn validate_page21_reachability(source: usize, target: usize, label: &str) -> Result<(), String> {
    let source_page = source as i128 & !0xfff;
    let target_page = target as i128 & !0xfff;
    let page_delta = target_page - source_page;
    if !(-0x1_0000_0000..=0x0_ffff_f000).contains(&page_delta) {
        return Err(format!(
            "Mach-O platform page21 target `{label}` delta {page_delta} is out of range"
        ));
    }
    Ok(())
}

fn target_audit_hash(
    structure_id: &str,
    seed: &TargetSeed,
    got_slot: Option<usize>,
    got_offset: Option<usize>,
    stub_slot: Option<usize>,
    stub_offset: Option<usize>,
) -> String {
    let mut out = String::new();
    append_text(&mut out, structure_id);
    append_text(&mut out, &seed.target_key);
    append_text(&mut out, &seed.target_symbol);
    append_text(&mut out, &seed.resolver_status);
    for relocation_id in &seed.relocation_ids {
        append_text(&mut out, relocation_id);
    }
    writeln!(
        out,
        "slots={}|{}|{}|{}",
        optional_usize(got_slot),
        optional_usize(got_offset),
        optional_usize(stub_slot),
        optional_usize(stub_offset)
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn binding_audit_hash(
    application: &NsldMachOArm64RelocationApplication,
    rule: PlatformRule,
    target: &NsldMachOArm64PlatformTargetPlan,
    patch_target_offset: usize,
) -> String {
    let mut out = String::new();
    append_text(&mut out, &application.relocation_id);
    append_text(&mut out, rule.id);
    append_text(&mut out, &target.structure_id);
    append_text(&mut out, &target.audit_hash);
    append_text(&mut out, rule.patch_target_kind);
    writeln!(
        out,
        "facts={}|{}|{}",
        application.source_output_offset, application.width_bytes, patch_target_offset
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn platform_plan_hash(
    status: &str,
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    applied: &NsldMachOArm64PatchApplicationReport,
    layout: &PlatformLayout,
    targets: &[NsldMachOArm64PlatformTargetPlan],
    bindings: &[NsldMachOArm64PlatformRelocationBinding],
) -> String {
    let mut out = String::new();
    append_text(&mut out, MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT);
    append_text(&mut out, status);
    append_text(&mut out, &placement.plan_hash);
    append_text(&mut out, &relocations.plan_hash);
    append_text(&mut out, &applied.application_ledger_hash);
    append_text(&mut out, &applied.applied_image_hash);
    for rule in PLATFORM_RULES {
        append_text(&mut out, rule.id);
        append_text(&mut out, rule.relocation_kind);
        append_text(&mut out, rule.action_kind);
        append_text(&mut out, rule.resolver_status.unwrap_or("any"));
        append_text(&mut out, rule.patch_target_kind);
        writeln!(out, "rule={}|{}", rule.requires_got, rule.requires_stub).unwrap();
    }
    writeln!(
        out,
        "layout={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        applied.image_span_bytes,
        layout.planned_image_span_bytes,
        layout.stub_region_offset,
        layout.stub_region_bytes,
        STUB_ENTRY_SIZE,
        STUB_ALIGNMENT,
        layout.got_region_offset,
        layout.got_region_bytes,
        GOT_ENTRY_SIZE,
        GOT_ALIGNMENT,
        targets.len(),
        bindings.len()
    )
    .unwrap();
    for target in targets {
        append_text(&mut out, &target.structure_id);
        append_text(&mut out, &target.audit_hash);
    }
    for binding in bindings {
        append_text(&mut out, &binding.relocation_id);
        append_text(&mut out, &binding.audit_hash);
    }
    crate::fnv1a64_hex(out.as_bytes())
}

fn slot_offset(region_offset: usize, slot: usize, size: usize) -> Result<usize, String> {
    let relative = slot
        .checked_mul(size)
        .ok_or_else(|| "Mach-O platform slot offset overflows".to_owned())?;
    region_offset
        .checked_add(relative)
        .ok_or_else(|| "Mach-O platform output offset overflows".to_owned())
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    let remainder = value
        .checked_rem(alignment)
        .ok_or_else(|| "Mach-O platform alignment must be nonzero".to_owned())?;
    if remainder == 0 {
        return Ok(value);
    }
    value
        .checked_add(alignment - remainder)
        .ok_or_else(|| "Mach-O platform alignment overflows".to_owned())
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
#[path = "final_executable_macho_platform_tests.rs"]
mod tests;
