use crate::{
    final_executable_macho_layout::{
        build_macho_placement_binding_report, MachOLayoutObject, MACHO_PLACEMENT_BINDING_CONTRACT,
    },
    final_executable_macho_platform::MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT,
    final_executable_macho_platform_application::{
        MachOArm64PlatformAppliedImage, MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT,
    },
    final_executable_macho_relocation::MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT,
    final_executable_macho_shell_layout::{
        build_shell_layout_draft, finalize_shell_layout, MACHO_ARM64_IMAGE_BASE,
        MACHO_ARM64_PAGE_SIZE, MACHO_HEADER_SIZE,
    },
    final_executable_macho_shell_linkedit::build_shell_linkedit_plan,
    reports::{
        NsldMachOArm64PlatformStructurePlanReport, NsldMachOArm64RelocationApplicationReport,
        NsldMachOArm64ShellLayoutPlanReport, NsldMachOPlacementBindingReport,
    },
};
use std::collections::BTreeSet;
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_SHELL_LAYOUT_PLAN_CONTRACT: &str =
    "nuis-nsld-macho-arm64-shell-layout-plan-v1";

pub(crate) fn build_macho_arm64_shell_layout_plan(
    objects: &[MachOLayoutObject<'_>],
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    platform_plan: &NsldMachOArm64PlatformStructurePlanReport,
    platform_applied: &MachOArm64PlatformAppliedImage,
) -> Result<NsldMachOArm64ShellLayoutPlanReport, String> {
    validate_input_envelope(
        objects,
        placement,
        relocations,
        platform_plan,
        platform_applied,
    )?;
    let object_linkage_hash = macho_object_linkage_hash(objects);
    let layout = build_shell_layout_draft(
        placement,
        platform_plan,
        has_unresolved_external_symbols(objects),
    )?;
    let linkedit = build_shell_linkedit_plan(
        objects,
        placement,
        platform_plan,
        &platform_applied.report,
        &layout,
    )?;
    let finalized = finalize_shell_layout(&layout, linkedit.linkedit_bytes)?;
    validate_macho_field_widths(&layout, &linkedit, &finalized)?;
    let required_address_rewrite_count = relocations
        .applications
        .iter()
        .filter(|application| application.application_status != "paired-metadata")
        .count()
        .checked_add(platform_plan.stub_entry_count)
        .and_then(|count| count.checked_add(linkedit.rebases.len()))
        .ok_or_else(|| "Mach-O shell address rewrite count overflows".to_owned())?;
    let status = "layout-planned-with-code-signature-boundary";
    let plan_hash = shell_plan_hash(
        status,
        placement,
        relocations,
        platform_plan,
        platform_applied,
        &object_linkage_hash,
        &layout,
        &linkedit,
        &finalized,
        required_address_rewrite_count,
    );
    Ok(NsldMachOArm64ShellLayoutPlanReport {
        contract: MACHO_ARM64_SHELL_LAYOUT_PLAN_CONTRACT.to_owned(),
        status: status.to_owned(),
        object_linkage_hash,
        placement_plan_hash: placement.plan_hash.clone(),
        platform_structure_plan_hash: platform_plan.plan_hash.clone(),
        platform_application_ledger_hash: platform_applied.report.application_ledger_hash.clone(),
        platform_image_hash: platform_applied.report.platform_image_hash.clone(),
        page_size: MACHO_ARM64_PAGE_SIZE,
        image_base_vm_address: MACHO_ARM64_IMAGE_BASE,
        header_size_bytes: MACHO_HEADER_SIZE,
        load_command_count: layout.load_command_count,
        load_command_size_bytes: layout.load_command_size_bytes,
        first_content_file_offset: layout.first_content_file_offset,
        entry_rule_id: linkedit.entry.rule_id,
        entry_symbol: linkedit.entry.symbol,
        entry_source_image_offset: linkedit.entry.source_image_offset,
        entry_file_offset: linkedit.entry.file_offset,
        entry_vm_address: linkedit.entry.vm_address,
        segment_count: finalized.segments.len(),
        section_count: layout.sections.len(),
        defined_symbol_count: linkedit.defined_symbol_count,
        undefined_symbol_count: linkedit.undefined_symbol_count,
        symbol_table_offset: linkedit.symbol_table_offset,
        symbol_table_bytes: linkedit.symbol_table_bytes,
        indirect_symbol_table_offset: linkedit.indirect_symbol_table_offset,
        indirect_symbol_count: linkedit.indirect_symbols.len(),
        indirect_symbol_table_bytes: linkedit.indirect_symbol_table_bytes,
        string_table_offset: linkedit.string_table_offset,
        string_table_bytes: linkedit.string_table_bytes,
        rebase_stream_offset: linkedit.rebase_stream_offset,
        rebase_stream_bytes: linkedit.rebase_stream_bytes,
        bind_stream_offset: linkedit.bind_stream_offset,
        bind_stream_bytes: linkedit.bind_stream_bytes,
        linkedit_file_offset: layout.linkedit_file_offset,
        linkedit_bytes: linkedit.linkedit_bytes,
        code_signature_file_offset: finalized.code_signature_file_offset,
        code_signature_status: "required-payload-pending".to_owned(),
        required_address_rewrite_count,
        planned_file_span_bytes: finalized.planned_file_span_bytes,
        plan_hash,
        segments: finalized.segments,
        sections: layout.sections,
        symbols: linkedit.symbols,
        indirect_symbols: linkedit.indirect_symbols,
        binds: linkedit.binds,
        rebases: linkedit.rebases,
        load_commands: finalized.load_commands,
    })
}

fn validate_input_envelope(
    objects: &[MachOLayoutObject<'_>],
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    platform_plan: &NsldMachOArm64PlatformStructurePlanReport,
    platform_applied: &MachOArm64PlatformAppliedImage,
) -> Result<(), String> {
    if placement.contract != MACHO_PLACEMENT_BINDING_CONTRACT
        || relocations.contract != MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT
        || platform_plan.contract != MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT
        || platform_applied.report.contract != MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT
    {
        return Err("Mach-O shell layout rejects an upstream contract".to_owned());
    }
    let reconstructed = build_macho_placement_binding_report(objects)?;
    if &reconstructed != placement {
        return Err("Mach-O shell layout rejects object/placement drift".to_owned());
    }
    if relocations.placement_plan_hash != placement.plan_hash
        || platform_plan.placement_plan_hash != placement.plan_hash
        || platform_plan.relocation_plan_hash != relocations.plan_hash
        || platform_applied.report.placement_plan_hash != placement.plan_hash
        || platform_applied.report.relocation_plan_hash != relocations.plan_hash
        || platform_applied.report.platform_structure_plan_hash != platform_plan.plan_hash
    {
        return Err("Mach-O shell layout input hash drift".to_owned());
    }
    if platform_applied.bytes.len() != platform_plan.planned_image_span_bytes
        || platform_applied.report.platform_image_span_bytes != platform_applied.bytes.len()
        || crate::fnv1a64_hex(&platform_applied.bytes)
            != platform_applied.report.platform_image_hash
        || platform_applied.report.applied_deferred_patch_count
            != platform_plan.deferred_relocation_count
        || platform_applied.report.structure_writes.len()
            != platform_plan.stub_entry_count + platform_plan.got_entry_count
        || platform_applied.report.bind_records.len()
            != platform_applied.report.unresolved_bind_count
        || platform_applied.report.application_ledger_hash.is_empty()
    {
        return Err("Mach-O shell layout platform application drift".to_owned());
    }
    let expected_status = if platform_applied.report.bind_records.is_empty() {
        if platform_plan.deferred_relocation_count == 0 {
            "not-required"
        } else {
            "platform-patches-applied"
        }
    } else {
        "platform-patches-applied-with-unresolved-binds"
    };
    if platform_applied.report.status != expected_status {
        return Err("Mach-O shell layout platform status drift".to_owned());
    }
    let mut object_ids = BTreeSet::new();
    for object in objects {
        if !object_ids.insert(object.object_id) {
            return Err(format!(
                "Mach-O shell layout repeats object `{}`",
                object.object_id
            ));
        }
        let defined_count = object
            .linkage
            .symbols
            .iter()
            .filter(|symbol| symbol.defined)
            .count();
        let undefined_count = object
            .linkage
            .symbols
            .iter()
            .filter(|symbol| matches!(symbol.kind.as_str(), "undefined" | "prebound-undefined"))
            .count();
        let external_definitions = object
            .linkage
            .symbols
            .iter()
            .filter(|symbol| symbol.external && symbol.defined && !symbol.name.is_empty())
            .map(|symbol| symbol.name.clone())
            .collect::<BTreeSet<_>>();
        let external_undefined = object
            .linkage
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.external
                    && matches!(symbol.kind.as_str(), "undefined" | "prebound-undefined")
                    && !symbol.name.is_empty()
            })
            .map(|symbol| symbol.name.clone())
            .collect::<BTreeSet<_>>();
        if object.linkage.section_count != object.linkage.sections.len()
            || object.linkage.symbol_count != object.linkage.symbols.len()
            || object.linkage.relocation_count != object.linkage.relocations.len()
            || object.linkage.defined_symbol_count != defined_count
            || object.linkage.undefined_symbol_count != undefined_count
            || object.linkage.external_definitions != external_definitions
            || object.linkage.external_undefined != external_undefined
        {
            return Err(format!(
                "Mach-O shell object `{}` linkage summary drift",
                object.object_id
            ));
        }
    }
    if objects.is_empty() {
        return Err("Mach-O shell layout has no input objects".to_owned());
    }
    Ok(())
}

fn validate_macho_field_widths(
    layout: &crate::final_executable_macho_shell_layout::ShellLayoutDraft,
    linkedit: &crate::final_executable_macho_shell_linkedit::ShellLinkeditPlan,
    finalized: &crate::final_executable_macho_shell_layout::FinalizedShellLayout,
) -> Result<(), String> {
    for (name, value) in [
        ("load command count", layout.load_command_count),
        ("load command bytes", layout.load_command_size_bytes),
        ("first content offset", layout.first_content_file_offset),
        ("linkedit offset", layout.linkedit_file_offset),
        ("linkedit bytes", linkedit.linkedit_bytes),
        ("defined symbol count", linkedit.defined_symbol_count),
        ("undefined symbol count", linkedit.undefined_symbol_count),
        ("rebase stream offset", linkedit.rebase_stream_offset),
        ("rebase stream bytes", linkedit.rebase_stream_bytes),
        ("bind stream offset", linkedit.bind_stream_offset),
        ("bind stream bytes", linkedit.bind_stream_bytes),
        ("symbol table offset", linkedit.symbol_table_offset),
        ("symbol table bytes", linkedit.symbol_table_bytes),
        (
            "indirect symbol table offset",
            linkedit.indirect_symbol_table_offset,
        ),
        (
            "indirect symbol table bytes",
            linkedit.indirect_symbol_table_bytes,
        ),
        ("string table offset", linkedit.string_table_offset),
        ("string table bytes", linkedit.string_table_bytes),
        (
            "code signature offset",
            finalized.code_signature_file_offset,
        ),
        ("planned file span", finalized.planned_file_span_bytes),
    ] {
        u32::try_from(value).map_err(|_| format!("Mach-O shell {name} exceeds u32"))?;
    }
    for section in &layout.sections {
        u8::try_from(section.section_ordinal)
            .map_err(|_| "Mach-O shell section ordinal exceeds u8".to_owned())?;
        if let Some(offset) = section.file_offset {
            u32::try_from(offset)
                .map_err(|_| "Mach-O shell section file offset exceeds u32".to_owned())?;
        }
    }
    for symbol in &linkedit.symbols {
        u32::try_from(symbol.symbol_table_index)
            .map_err(|_| "Mach-O shell symbol index exceeds u32".to_owned())?;
        u32::try_from(symbol.string_table_offset)
            .map_err(|_| "Mach-O shell string offset exceeds u32".to_owned())?;
        if let Some(ordinal) = symbol.dylib_ordinal {
            u8::try_from(ordinal)
                .map_err(|_| "Mach-O shell dylib ordinal exceeds u8".to_owned())?;
        }
    }
    for symbol in &linkedit.indirect_symbols {
        if let Some(index) = symbol.symbol_table_index {
            u32::try_from(index)
                .map_err(|_| "Mach-O indirect symbol index exceeds u32".to_owned())?;
        }
    }
    for command in &finalized.load_commands {
        u32::try_from(command.command_offset)
            .map_err(|_| "Mach-O load-command offset exceeds u32".to_owned())?;
        u32::try_from(command.command_size_bytes)
            .map_err(|_| "Mach-O load-command size exceeds u32".to_owned())?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn shell_plan_hash(
    status: &str,
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    platform_plan: &NsldMachOArm64PlatformStructurePlanReport,
    platform_applied: &MachOArm64PlatformAppliedImage,
    object_linkage_hash: &str,
    layout: &crate::final_executable_macho_shell_layout::ShellLayoutDraft,
    linkedit: &crate::final_executable_macho_shell_linkedit::ShellLinkeditPlan,
    finalized: &crate::final_executable_macho_shell_layout::FinalizedShellLayout,
    required_address_rewrite_count: usize,
) -> String {
    let mut out = String::new();
    for value in [
        MACHO_ARM64_SHELL_LAYOUT_PLAN_CONTRACT,
        status,
        &placement.plan_hash,
        &relocations.plan_hash,
        &platform_plan.plan_hash,
        &platform_applied.report.application_ledger_hash,
        &platform_applied.report.platform_image_hash,
        object_linkage_hash,
        &linkedit.entry.rule_id,
        &linkedit.entry.symbol,
    ] {
        append_text(&mut out, value);
    }
    writeln!(
        out,
        "layout={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        MACHO_ARM64_PAGE_SIZE,
        MACHO_ARM64_IMAGE_BASE,
        layout.load_command_count,
        layout.load_command_size_bytes,
        layout.first_content_file_offset,
        layout.linkedit_file_offset,
        linkedit.linkedit_bytes,
        finalized.code_signature_file_offset,
        finalized.planned_file_span_bytes,
        linkedit.entry.file_offset,
        required_address_rewrite_count
    )
    .unwrap();
    for segment in &finalized.segments {
        append_text(&mut out, &segment.segment_id);
        append_text(&mut out, &segment.audit_hash);
    }
    for section in &layout.sections {
        append_text(&mut out, &section.section_id);
        append_text(&mut out, &section.audit_hash);
    }
    for symbol in &linkedit.symbols {
        append_text(&mut out, &symbol.symbol_id);
        append_text(&mut out, &symbol.audit_hash);
    }
    for indirect in &linkedit.indirect_symbols {
        append_text(&mut out, &indirect.indirect_id);
        append_text(&mut out, &indirect.audit_hash);
    }
    for bind in &linkedit.binds {
        append_text(&mut out, &bind.bind_id);
        append_text(&mut out, &bind.audit_hash);
    }
    for rebase in &linkedit.rebases {
        append_text(&mut out, &rebase.rebase_id);
        append_text(&mut out, &rebase.audit_hash);
    }
    for command in &finalized.load_commands {
        append_text(&mut out, &command.command_id);
        append_text(&mut out, &command.audit_hash);
    }
    crate::fnv1a64_hex(out.as_bytes())
}

fn has_unresolved_external_symbols(objects: &[MachOLayoutObject<'_>]) -> bool {
    let definitions = objects
        .iter()
        .flat_map(|object| object.linkage.symbols.iter())
        .filter(|symbol| symbol.external && symbol.defined && !symbol.name.is_empty())
        .map(|symbol| symbol.name.as_str())
        .collect::<BTreeSet<_>>();
    objects
        .iter()
        .flat_map(|object| object.linkage.symbols.iter())
        .any(|symbol| {
            symbol.external
                && matches!(symbol.kind.as_str(), "undefined" | "prebound-undefined")
                && !symbol.name.is_empty()
                && !definitions.contains(symbol.name.as_str())
        })
}

fn macho_object_linkage_hash(objects: &[MachOLayoutObject<'_>]) -> String {
    let mut objects = objects.iter().collect::<Vec<_>>();
    objects.sort_by(|lhs, rhs| {
        lhs.role
            .cmp(rhs.role)
            .then(lhs.object_id.cmp(rhs.object_id))
    });
    let mut out = String::new();
    for object in objects {
        append_text(&mut out, object.object_id);
        append_text(&mut out, object.role);
        writeln!(
            out,
            "counts={}|{}|{}|{}|{}",
            object.linkage.section_count,
            object.linkage.symbol_count,
            object.linkage.relocation_count,
            object.linkage.defined_symbol_count,
            object.linkage.undefined_symbol_count
        )
        .unwrap();
        for section in &object.linkage.sections {
            append_text(&mut out, &section.segment_name);
            append_text(&mut out, &section.name);
            writeln!(
                out,
                "section={}|{}|{}|{}|{:08x}|{}|{}|{}|{}",
                section.ordinal,
                section.address,
                section.size,
                section.alignment,
                section.flags,
                section.zero_fill,
                section.payload_offset,
                section.relocation_offset,
                section.relocation_count
            )
            .unwrap();
        }
        for symbol in &object.linkage.symbols {
            append_text(&mut out, &symbol.name);
            append_text(&mut out, &symbol.kind);
            append_text(
                &mut out,
                symbol.indirect_target.as_deref().unwrap_or("none"),
            );
            writeln!(
                out,
                "symbol={}|{}|{}|{}|{}|{}|{}",
                symbol.index,
                symbol.external,
                symbol.defined,
                optional_usize(symbol.section_ordinal),
                symbol.value,
                optional_u64(symbol.common_alignment),
                symbol.indirect_target.is_some()
            )
            .unwrap();
        }
        for relocation in &object.linkage.relocations {
            writeln!(
                out,
                "relocation={}|{}|{}|{}|{}|{}|{}",
                relocation.section_ordinal,
                relocation.offset,
                relocation.symbol_number,
                relocation.width_bytes,
                relocation.pc_relative,
                relocation.external,
                relocation.relocation_type
            )
            .unwrap();
        }
        for symbol in &object.linkage.external_definitions {
            append_text(&mut out, symbol);
        }
        for symbol in &object.linkage.external_undefined {
            append_text(&mut out, symbol);
        }
    }
    crate::fnv1a64_hex(out.as_bytes())
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
#[path = "final_executable_macho_shell_tests.rs"]
mod tests;
