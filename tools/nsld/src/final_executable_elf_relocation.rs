use crate::{
    final_executable_elf_input::{ParsedElfRelocation, ParsedElfSymbol},
    final_executable_elf_layout::ELF_AMD64_PLACEMENT_BINDING_CONTRACT,
    final_executable_elf_layout_report::{
        ElfAmd64PlacementBindingReport, ElfAmd64SectionPlacement, ElfAmd64SymbolBinding,
    },
    final_executable_elf_object::ElfAmd64ObjectLinkage,
    final_executable_elf_relocation_report::{
        ElfAmd64RelocationApplication, ElfAmd64RelocationApplicationReport,
    },
};
use std::collections::BTreeSet;

pub(crate) const ELF_AMD64_RELOCATION_APPLICATION_CONTRACT: &str =
    "nuis-nsld-elf-amd64-relocation-application-v1";

const R_X86_64_NONE: u32 = 0;
const R_X86_64_64: u32 = 1;
const R_X86_64_PC32: u32 = 2;
const R_X86_64_PLT32: u32 = 4;
const R_X86_64_32: u32 = 10;
const R_X86_64_32S: u32 = 11;

#[derive(Clone, Copy)]
struct RelocationShape {
    kind: &'static str,
    action: &'static str,
    width_bytes: usize,
    pc_relative: bool,
    no_op: bool,
}

#[derive(Default)]
struct TargetResolution {
    symbol: Option<String>,
    symbol_index: Option<usize>,
    symbol_external: bool,
    object_id: Option<String>,
    kind: Option<String>,
    section_id: Option<String>,
    image_offset: Option<usize>,
    virtual_address: Option<u64>,
    absolute_value: Option<u64>,
    resolver_status: String,
    platform_structure: bool,
}

pub(crate) fn build_elf_amd64_relocation_application(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
) -> Result<ElfAmd64RelocationApplicationReport, String> {
    validate_placement_report(placement)?;
    let objects = sorted_objects(objects)?;
    let mut applications = Vec::new();
    let mut registered_kinds = BTreeSet::new();
    for object in objects {
        for relocation in &object.linkage.relocations {
            let shape = registered_shape(relocation)?;
            registered_kinds.insert(shape.kind);
            applications.push(build_application(
                object,
                relocation,
                shape,
                placement,
                applications.len(),
            )?);
        }
    }
    let direct_preview_count = count_status(&applications, "planned-direct");
    let platform_structure_count = count_status(&applications, "planned-platform-structure");
    let no_op_count = count_status(&applications, "no-op");
    let status = if platform_structure_count == 0 {
        "ready-for-byte-preview"
    } else {
        "preview-ready-with-platform-structure-boundary"
    };
    let mut report = ElfAmd64RelocationApplicationReport {
        contract: ELF_AMD64_RELOCATION_APPLICATION_CONTRACT,
        status: status.to_owned(),
        plan_hash: String::new(),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_count: applications.len(),
        registered_kind_count: registered_kinds.len(),
        direct_preview_count,
        platform_structure_count,
        no_op_count,
        applications,
    };
    report.plan_hash = crate::fnv1a64_hex(report.canonical_plan().as_bytes());
    Ok(report)
}

fn validate_placement_report(placement: &ElfAmd64PlacementBindingReport) -> Result<(), String> {
    if placement.contract != ELF_AMD64_PLACEMENT_BINDING_CONTRACT {
        return Err(format!(
            "ELF relocation application received placement contract `{}`",
            placement.contract
        ));
    }
    let actual = crate::fnv1a64_hex(placement.canonical_plan().as_bytes());
    if placement.plan_hash != actual {
        return Err(format!(
            "ELF relocation placement hash mismatch: declared={}, actual={actual}",
            placement.plan_hash
        ));
    }
    Ok(())
}

fn sorted_objects(
    objects: &[ElfAmd64ObjectLinkage],
) -> Result<Vec<&ElfAmd64ObjectLinkage>, String> {
    let mut seen = BTreeSet::new();
    for object in objects {
        if !seen.insert(object.object_id.as_str()) {
            return Err(format!(
                "ELF relocation application contains duplicate object id `{}`",
                object.object_id
            ));
        }
    }
    let mut sorted = objects.iter().collect::<Vec<_>>();
    sorted.sort_by(|lhs, rhs| {
        object_role_rank(&lhs.role)
            .cmp(&object_role_rank(&rhs.role))
            .then(lhs.role.cmp(&rhs.role))
            .then(lhs.object_id.cmp(&rhs.object_id))
    });
    Ok(sorted)
}

fn build_application(
    object: &ElfAmd64ObjectLinkage,
    relocation: &ParsedElfRelocation,
    shape: RelocationShape,
    placement: &ElfAmd64PlacementBindingReport,
    ordinal: usize,
) -> Result<ElfAmd64RelocationApplication, String> {
    let source = source_placement(object, relocation, &placement.section_placements)?;
    let source_offset = checked_usize(relocation.offset, "ELF relocation source offset")?;
    let source_end = source_offset
        .checked_add(shape.width_bytes)
        .ok_or_else(|| "ELF relocation source span overflows".to_owned())?;
    if source_end > source.size_bytes {
        return Err(format!(
            "ELF object `{}` relocation source span {source_offset}..{source_end} exceeds placed section size {}",
            object.object_id, source.size_bytes
        ));
    }
    let source_file_offset = source
        .file_offset
        .ok_or_else(|| {
            format!(
                "ELF object `{}` relocation source section {} has no file placement",
                object.object_id, relocation.target_section_index
            )
        })?
        .checked_add(source_offset)
        .ok_or_else(|| "ELF relocation source file offset overflows".to_owned())?;
    let source_image_offset = source
        .image_offset
        .checked_add(source_offset)
        .ok_or_else(|| "ELF relocation source image offset overflows".to_owned())?;
    let source_virtual_address = source
        .virtual_address
        .checked_add(relocation.offset)
        .ok_or_else(|| "ELF relocation source virtual address overflows".to_owned())?;
    let target = resolve_target(object, relocation, shape, placement)?;
    let application_status = if shape.no_op {
        "no-op"
    } else if target.platform_structure {
        "planned-platform-structure"
    } else {
        "planned-direct"
    };
    let (computed_value, encoded_value, encoded_bytes) = if application_status == "planned-direct" {
        let target_address = target.virtual_address.ok_or_else(|| {
            format!(
                "ELF object `{}` relocation target has no runtime address",
                object.object_id
            )
        })?;
        let computed = relocation_value(relocation, target_address, source_virtual_address);
        let encoded = encode_value(relocation.relocation_type, computed)?;
        (
            Some(computed),
            Some(encoded),
            little_endian_bytes(encoded, shape.width_bytes),
        )
    } else {
        (None, None, Vec::new())
    };

    Ok(ElfAmd64RelocationApplication {
        relocation_id: format!("elf-amd64-reloc-{ordinal:06}"),
        object_id: object.object_id.clone(),
        object_role: object.role.clone(),
        relocation_section_index: relocation.relocation_section_index,
        input_section_index: relocation.target_section_index,
        source_section_id: source.output_section_id.clone(),
        source_offset,
        source_file_offset,
        source_image_offset,
        source_virtual_address,
        width_bytes: shape.width_bytes,
        pc_relative: relocation.pc_relative,
        relocation_type: relocation.relocation_type,
        relocation_kind: shape.kind.to_owned(),
        action_kind: shape.action.to_owned(),
        target_symbol: target.symbol,
        target_symbol_index: target.symbol_index,
        target_symbol_external: target.symbol_external,
        target_object_id: target.object_id,
        target_kind: target.kind,
        target_section_id: target.section_id,
        target_image_offset: target.image_offset,
        target_virtual_address: target.virtual_address,
        target_absolute_value: target.absolute_value,
        addend: relocation.addend,
        computed_value,
        encoded_value,
        encoded_bytes,
        resolver_status: target.resolver_status,
        application_status: application_status.to_owned(),
    })
}

fn registered_shape(relocation: &ParsedElfRelocation) -> Result<RelocationShape, String> {
    let shape = match relocation.relocation_type {
        R_X86_64_NONE => RelocationShape {
            kind: "x86_64-none",
            action: "no-op",
            width_bytes: 0,
            pc_relative: false,
            no_op: true,
        },
        R_X86_64_64 => direct_shape("x86_64-64", "write-absolute-64", 8, false),
        R_X86_64_PC32 => direct_shape("x86_64-pc32", "write-pc-relative-32", 4, true),
        R_X86_64_PLT32 => direct_shape("x86_64-plt32", "write-plt-relative-32", 4, true),
        R_X86_64_32 => direct_shape("x86_64-32", "write-unsigned-32", 4, false),
        R_X86_64_32S => direct_shape("x86_64-32s", "write-signed-32", 4, false),
        other => {
            return Err(format!(
                "unregistered ELF AMD64 relocation type {other}; the provider fails closed outside its static application registry"
            ));
        }
    };
    if relocation.width_bytes != shape.width_bytes as u64 {
        return Err(format!(
            "{} relocation has width {}, expected {}",
            shape.kind, relocation.width_bytes, shape.width_bytes
        ));
    }
    if relocation.pc_relative != shape.pc_relative {
        return Err(format!(
            "{} relocation has pc_relative={}, expected {}",
            shape.kind, relocation.pc_relative, shape.pc_relative
        ));
    }
    Ok(shape)
}

fn direct_shape(
    kind: &'static str,
    action: &'static str,
    width_bytes: usize,
    pc_relative: bool,
) -> RelocationShape {
    RelocationShape {
        kind,
        action,
        width_bytes,
        pc_relative,
        no_op: false,
    }
}

fn source_placement<'a>(
    object: &ElfAmd64ObjectLinkage,
    relocation: &ParsedElfRelocation,
    placements: &'a [ElfAmd64SectionPlacement],
) -> Result<&'a ElfAmd64SectionPlacement, String> {
    placements
        .iter()
        .find(|placement| {
            placement.object_id == object.object_id
                && placement.input_section_index == relocation.target_section_index
        })
        .ok_or_else(|| {
            format!(
                "ELF object `{}` relocation has no placement for section {}",
                object.object_id, relocation.target_section_index
            )
        })
}

fn resolve_target(
    object: &ElfAmd64ObjectLinkage,
    relocation: &ParsedElfRelocation,
    shape: RelocationShape,
    placement: &ElfAmd64PlacementBindingReport,
) -> Result<TargetResolution, String> {
    if shape.no_op {
        return Ok(TargetResolution {
            resolver_status: "not-applicable".to_owned(),
            ..TargetResolution::default()
        });
    }
    let symbol = object
        .linkage
        .symbols
        .get(relocation.symbol_index)
        .ok_or_else(|| {
            format!(
                "ELF object `{}` relocation references missing symbol index {}",
                object.object_id, relocation.symbol_index
            )
        })?;
    if symbol.index == 0 {
        return Err(format!(
            "ELF object `{}` non-NONE relocation references the null symbol",
            object.object_id
        ));
    }
    let binding = symbol_binding(object, symbol, &placement.symbol_bindings)?;
    if binding.status == "external-compatibility" {
        return Ok(target_from_binding(symbol, binding, true));
    }
    if binding.target_virtual_address.is_none() {
        return Err(format!(
            "ELF object `{}` relocation symbol `{}` binding `{}` has no runtime address",
            object.object_id, symbol.name, binding.status
        ));
    }
    Ok(target_from_binding(symbol, binding, false))
}

fn symbol_binding<'a>(
    object: &ElfAmd64ObjectLinkage,
    symbol: &ParsedElfSymbol,
    bindings: &'a [ElfAmd64SymbolBinding],
) -> Result<&'a ElfAmd64SymbolBinding, String> {
    bindings
        .iter()
        .find(|binding| {
            binding.reference_object_id == object.object_id
                && binding.reference_symbol_index == symbol.index
        })
        .ok_or_else(|| {
            format!(
                "ELF object `{}` relocation symbol `{}` has no placement binding",
                object.object_id, symbol.name
            )
        })
}

fn target_from_binding(
    symbol: &ParsedElfSymbol,
    binding: &ElfAmd64SymbolBinding,
    platform_structure: bool,
) -> TargetResolution {
    TargetResolution {
        symbol: Some(symbol.name.clone()),
        symbol_index: Some(symbol.index),
        symbol_external: symbol.external,
        object_id: binding.target_object_id.clone(),
        kind: binding.target_kind.clone(),
        section_id: binding.target_section_id.clone(),
        image_offset: binding.target_image_offset,
        virtual_address: binding.target_virtual_address,
        absolute_value: binding.target_absolute_value,
        resolver_status: binding.status.clone(),
        platform_structure,
    }
}

fn relocation_value(
    relocation: &ParsedElfRelocation,
    target_address: u64,
    source_address: u64,
) -> i128 {
    let value = i128::from(target_address) + i128::from(relocation.addend);
    if relocation.pc_relative {
        value - i128::from(source_address)
    } else {
        value
    }
}

fn encode_value(relocation_type: u32, value: i128) -> Result<u64, String> {
    match relocation_type {
        R_X86_64_64 => encode_word64(value),
        R_X86_64_PC32 | R_X86_64_PLT32 | R_X86_64_32S => encode_signed32(value),
        R_X86_64_32 => encode_unsigned32(value),
        _ => Err(format!(
            "ELF relocation type {relocation_type} has no direct encoder"
        )),
    }
}

fn encode_word64(value: i128) -> Result<u64, String> {
    if value < i128::from(i64::MIN) || value > i128::from(u64::MAX) {
        return Err(format!(
            "ELF R_X86_64_64 value {value} does not fit a 64-bit word"
        ));
    }
    Ok(if value < 0 {
        (value as i64) as u64
    } else {
        value as u64
    })
}

fn encode_signed32(value: i128) -> Result<u64, String> {
    let value = i32::try_from(value)
        .map_err(|_| format!("ELF signed 32-bit relocation value {value} overflows"))?;
    Ok(u64::from(value as u32))
}

fn encode_unsigned32(value: i128) -> Result<u64, String> {
    let value = u32::try_from(value)
        .map_err(|_| format!("ELF unsigned 32-bit relocation value {value} overflows"))?;
    Ok(u64::from(value))
}

fn little_endian_bytes(value: u64, width_bytes: usize) -> Vec<u8> {
    value.to_le_bytes()[..width_bytes].to_vec()
}

fn count_status(applications: &[ElfAmd64RelocationApplication], status: &str) -> usize {
    applications
        .iter()
        .filter(|application| application.application_status == status)
        .count()
}

fn checked_usize(value: u64, label: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("{label} exceeds host address space"))
}

fn object_role_rank(role: &str) -> usize {
    match role {
        "program-llvm" => 0,
        "runtime-shim" => 1,
        _ => 2,
    }
}

#[cfg(test)]
#[path = "final_executable_elf_relocation_tests.rs"]
mod tests;
