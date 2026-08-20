use crate::reports::{
    NsldMachOArm64PlatformStructurePlanReport, NsldMachOArm64ShellLoadCommandPlan,
    NsldMachOArm64ShellSectionPlan, NsldMachOArm64ShellSegmentPlan,
    NsldMachOPlacementBindingReport,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_PAGE_SIZE: usize = 0x4000;
pub(crate) const MACHO_ARM64_IMAGE_BASE: u64 = 0x1_0000_0000;
pub(crate) const MACHO_HEADER_SIZE: usize = 32;
pub(crate) const SYSTEM_DYLIB_PATH: &str = "/usr/lib/libSystem.B.dylib";
pub(crate) const DYLINKER_PATH: &str = "/usr/lib/dyld";
const SEGMENT_COMMAND_SIZE: usize = 72;
const SECTION_COMMAND_SIZE: usize = 80;

const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x2;
const LC_DYSYMTAB: u32 = 0xb;
const LC_LOAD_DYLINKER: u32 = 0xe;
const LC_LOAD_DYLIB: u32 = 0xc;
const LC_DYLD_INFO_ONLY: u32 = 0x8000_0022;
const LC_MAIN: u32 = 0x8000_0028;
const LC_BUILD_VERSION: u32 = 0x32;
const LC_CODE_SIGNATURE: u32 = 0x1d;

const STUB_FLAGS: u32 = 0x8000_0408;
const GOT_FLAGS: u32 = 0x6;

#[derive(Clone)]
struct SectionSeed {
    source_kind: &'static str,
    source_id: String,
    segment_name: String,
    section_name: String,
    source_image_offset: usize,
    source_size_bytes: usize,
    alignment: usize,
    flags: u32,
    zero_fill: bool,
    reserved1: u32,
    reserved2: u32,
}

pub(crate) struct ShellLayoutDraft {
    pub(crate) load_command_count: usize,
    pub(crate) load_command_size_bytes: usize,
    pub(crate) first_content_file_offset: usize,
    pub(crate) linkedit_file_offset: usize,
    pub(crate) linkedit_vm_address: u64,
    pub(crate) has_dylib: bool,
    pub(crate) segments: Vec<NsldMachOArm64ShellSegmentPlan>,
    pub(crate) sections: Vec<NsldMachOArm64ShellSectionPlan>,
}

pub(crate) struct LocatedShellAddress {
    pub(crate) section_id: String,
    pub(crate) file_offset: Option<usize>,
    pub(crate) vm_address: u64,
    pub(crate) segment_index: usize,
    pub(crate) segment_offset: usize,
}

pub(crate) struct FinalizedShellLayout {
    pub(crate) segments: Vec<NsldMachOArm64ShellSegmentPlan>,
    pub(crate) load_commands: Vec<NsldMachOArm64ShellLoadCommandPlan>,
    pub(crate) planned_file_span_bytes: usize,
    pub(crate) code_signature_file_offset: usize,
}

pub(crate) fn build_shell_layout_draft(
    placement: &NsldMachOPlacementBindingReport,
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    has_dylib: bool,
) -> Result<ShellLayoutDraft, String> {
    let seeds = section_seeds(placement, platform)?;
    let groups = group_seeds(seeds)?;
    let segment_count = groups
        .len()
        .checked_add(2)
        .ok_or_else(|| "Mach-O shell segment count overflows".to_owned())?;
    let load_command_count = segment_count
        .checked_add(7)
        .and_then(|count| count.checked_add(usize::from(has_dylib)))
        .ok_or_else(|| "Mach-O shell load-command count overflows".to_owned())?;
    let load_command_size_bytes = load_command_size(&groups, has_dylib)?;
    let first_content_file_offset = align_up(
        MACHO_HEADER_SIZE
            .checked_add(load_command_size_bytes)
            .ok_or_else(|| "Mach-O shell command span overflows".to_owned())?,
        16,
    )?;

    let mut sections = Vec::new();
    let mut segments = vec![pagezero_segment()];
    let mut file_cursor = first_content_file_offset;
    let mut vm_cursor = MACHO_ARM64_IMAGE_BASE
        .checked_add(
            u64::try_from(first_content_file_offset)
                .map_err(|_| "Mach-O first content offset exceeds 64-bit VM space".to_owned())?,
        )
        .ok_or_else(|| "Mach-O first content VM address overflows".to_owned())?;
    let mut section_ordinal = 1usize;

    for (segment_name, seeds) in groups {
        let text = segment_name == "__TEXT";
        let segment_file_offset = if text {
            0
        } else {
            align_up(file_cursor, MACHO_ARM64_PAGE_SIZE)?
        };
        let segment_vm_address = if text {
            MACHO_ARM64_IMAGE_BASE
        } else {
            align_up_u64(vm_cursor, MACHO_ARM64_PAGE_SIZE)?
        };
        let mut section_file_cursor = if text {
            first_content_file_offset
        } else {
            segment_file_offset
        };
        let mut section_vm_cursor =
            if text {
                MACHO_ARM64_IMAGE_BASE
                    .checked_add(u64::try_from(first_content_file_offset).map_err(|_| {
                        "Mach-O text content offset exceeds 64-bit VM space".to_owned()
                    })?)
                    .ok_or_else(|| "Mach-O text content VM address overflows".to_owned())?
            } else {
                segment_vm_address
            };
        let mut section_ids = Vec::with_capacity(seeds.len());
        for seed in seeds {
            section_file_cursor = align_up(section_file_cursor, seed.alignment)?;
            section_vm_cursor = align_up_u64(section_vm_cursor, seed.alignment)?;
            let file_offset = (!seed.zero_fill).then_some(section_file_cursor);
            let file_size_bytes = if seed.zero_fill {
                0
            } else {
                seed.source_size_bytes
            };
            let section_id = format!("macho-arm64-shell-section-{:04}", sections.len());
            let audit_hash = section_audit_hash(
                &section_id,
                &seed,
                section_ordinal,
                file_offset,
                section_vm_cursor,
            );
            sections.push(NsldMachOArm64ShellSectionPlan {
                section_id: section_id.clone(),
                source_kind: seed.source_kind.to_owned(),
                source_id: seed.source_id,
                segment_name: seed.segment_name,
                section_name: seed.section_name,
                section_ordinal,
                source_image_offset: Some(seed.source_image_offset),
                source_size_bytes: seed.source_size_bytes,
                alignment: seed.alignment,
                file_offset,
                file_size_bytes,
                vm_address: section_vm_cursor,
                vm_size_bytes: seed.source_size_bytes,
                flags: seed.flags,
                reserved1: seed.reserved1,
                reserved2: seed.reserved2,
                audit_hash,
            });
            section_ids.push(section_id);
            section_ordinal = section_ordinal
                .checked_add(1)
                .ok_or_else(|| "Mach-O shell section ordinal overflows".to_owned())?;
            if !seed.zero_fill {
                section_file_cursor = section_file_cursor
                    .checked_add(seed.source_size_bytes)
                    .ok_or_else(|| "Mach-O shell section file span overflows".to_owned())?;
            }
            section_vm_cursor = section_vm_cursor
                .checked_add(
                    u64::try_from(seed.source_size_bytes)
                        .map_err(|_| "Mach-O shell section VM size overflows".to_owned())?,
                )
                .ok_or_else(|| "Mach-O shell section VM span overflows".to_owned())?;
        }
        let segment_file_size = section_file_cursor
            .checked_sub(segment_file_offset)
            .ok_or_else(|| "Mach-O shell segment file span underflows".to_owned())?;
        let raw_vm_size = section_vm_cursor
            .checked_sub(segment_vm_address)
            .ok_or_else(|| "Mach-O shell segment VM span underflows".to_owned())?;
        let segment_vm_size = align_up_u64(raw_vm_size, MACHO_ARM64_PAGE_SIZE)?;
        let (max_protection, initial_protection) = segment_protections(&segment_name);
        let segment_id = format!("macho-arm64-shell-segment-{:04}", segments.len());
        let segment_index = segments.len();
        let audit_hash = segment_audit_hash(
            &segment_id,
            &segment_name,
            segment_index,
            segment_file_offset,
            segment_file_size,
            segment_vm_address,
            segment_vm_size,
            &section_ids,
        );
        segments.push(NsldMachOArm64ShellSegmentPlan {
            segment_id,
            segment_name,
            segment_index,
            file_offset: segment_file_offset,
            file_size_bytes: segment_file_size,
            vm_address: segment_vm_address,
            vm_size_bytes: usize::try_from(segment_vm_size)
                .map_err(|_| "Mach-O shell segment VM size exceeds host space".to_owned())?,
            max_protection,
            initial_protection,
            section_ids,
            audit_hash,
        });
        file_cursor = section_file_cursor;
        vm_cursor = segment_vm_address
            .checked_add(segment_vm_size)
            .ok_or_else(|| "Mach-O shell next segment VM address overflows".to_owned())?;
    }
    let linkedit_file_offset = align_up(file_cursor, MACHO_ARM64_PAGE_SIZE)?;
    let linkedit_vm_address = align_up_u64(vm_cursor, MACHO_ARM64_PAGE_SIZE)?;
    Ok(ShellLayoutDraft {
        load_command_count,
        load_command_size_bytes,
        first_content_file_offset,
        linkedit_file_offset,
        linkedit_vm_address,
        has_dylib,
        segments,
        sections,
    })
}

pub(crate) fn finalize_shell_layout(
    draft: &ShellLayoutDraft,
    linkedit_bytes: usize,
) -> Result<FinalizedShellLayout, String> {
    let mut segments = draft.segments.clone();
    let segment_id = format!("macho-arm64-shell-segment-{:04}", segments.len());
    let segment_index = segments.len();
    let vm_size = align_up(linkedit_bytes, MACHO_ARM64_PAGE_SIZE)?;
    let audit_hash = segment_audit_hash(
        &segment_id,
        "__LINKEDIT",
        segment_index,
        draft.linkedit_file_offset,
        linkedit_bytes,
        draft.linkedit_vm_address,
        u64::try_from(vm_size).map_err(|_| "Mach-O linkedit VM size overflows".to_owned())?,
        &[],
    );
    segments.push(NsldMachOArm64ShellSegmentPlan {
        segment_id,
        segment_name: "__LINKEDIT".to_owned(),
        segment_index,
        file_offset: draft.linkedit_file_offset,
        file_size_bytes: linkedit_bytes,
        vm_address: draft.linkedit_vm_address,
        vm_size_bytes: vm_size,
        max_protection: 1,
        initial_protection: 1,
        section_ids: Vec::new(),
        audit_hash,
    });
    let load_commands = build_load_commands(&segments, &draft.sections, draft.has_dylib)?;
    let observed_size = load_commands.iter().try_fold(0usize, |size, command| {
        size.checked_add(command.command_size_bytes)
            .ok_or_else(|| "Mach-O shell observed command span overflows".to_owned())
    })?;
    if load_commands.len() != draft.load_command_count
        || observed_size != draft.load_command_size_bytes
    {
        return Err("Mach-O shell load-command coverage drift".to_owned());
    }
    let planned_file_span_bytes = draft
        .linkedit_file_offset
        .checked_add(linkedit_bytes)
        .ok_or_else(|| "Mach-O shell planned file span overflows".to_owned())?;
    let code_signature_file_offset = align_up(planned_file_span_bytes, 16)?;
    Ok(FinalizedShellLayout {
        segments,
        load_commands,
        planned_file_span_bytes,
        code_signature_file_offset,
    })
}

pub(crate) fn locate_source_address(
    source_image_offset: usize,
    sections: &[NsldMachOArm64ShellSectionPlan],
    segments: &[NsldMachOArm64ShellSegmentPlan],
) -> Result<LocatedShellAddress, String> {
    let matches = sections
        .iter()
        .filter(|section| {
            let Some(start) = section.source_image_offset else {
                return false;
            };
            start
                .checked_add(section.source_size_bytes)
                .is_some_and(|end| (start..end).contains(&source_image_offset))
        })
        .collect::<Vec<_>>();
    let [section] = matches.as_slice() else {
        return Err(format!(
            "Mach-O shell source offset {source_image_offset} maps to {} sections",
            matches.len()
        ));
    };
    let source_start = section.source_image_offset.unwrap();
    let relative = source_image_offset - source_start;
    let file_offset = match section.file_offset {
        Some(offset) => Some(
            offset
                .checked_add(relative)
                .ok_or_else(|| "Mach-O shell source file offset overflows".to_owned())?,
        ),
        None => None,
    };
    let vm_address = section
        .vm_address
        .checked_add(
            u64::try_from(relative)
                .map_err(|_| "Mach-O shell source VM offset overflows".to_owned())?,
        )
        .ok_or_else(|| "Mach-O shell source VM address overflows".to_owned())?;
    let segment = segments
        .iter()
        .find(|segment| segment.section_ids.contains(&section.section_id))
        .ok_or_else(|| {
            format!(
                "Mach-O shell section `{}` has no owning segment",
                section.section_id
            )
        })?;
    let segment_offset = usize::try_from(vm_address - segment.vm_address)
        .map_err(|_| "Mach-O shell segment offset exceeds host space".to_owned())?;
    Ok(LocatedShellAddress {
        section_id: section.section_id.clone(),
        file_offset,
        vm_address,
        segment_index: segment.segment_index,
        segment_offset,
    })
}

fn section_seeds(
    placement: &NsldMachOPlacementBindingReport,
    platform: &NsldMachOArm64PlatformStructurePlanReport,
) -> Result<Vec<SectionSeed>, String> {
    let mut seeds = placement
        .merged_sections
        .iter()
        .map(|section| SectionSeed {
            source_kind: "merged-section",
            source_id: section.section_id.clone(),
            segment_name: section.segment_name.clone(),
            section_name: section.section_name.clone(),
            source_image_offset: section.output_offset,
            source_size_bytes: section.size_bytes,
            alignment: section.alignment,
            flags: section.flags,
            zero_fill: section.zero_fill,
            reserved1: 0,
            reserved2: 0,
        })
        .collect::<Vec<_>>();
    if platform.stub_region_bytes > 0 {
        seeds.push(SectionSeed {
            source_kind: "platform-stubs",
            source_id: "macho-arm64-platform-stubs".to_owned(),
            segment_name: "__TEXT".to_owned(),
            section_name: "__stubs".to_owned(),
            source_image_offset: platform.stub_region_offset,
            source_size_bytes: platform.stub_region_bytes,
            alignment: platform.stub_alignment,
            flags: STUB_FLAGS,
            zero_fill: false,
            reserved1: 0,
            reserved2: u32::try_from(platform.stub_entry_size)
                .map_err(|_| "Mach-O stub entry size exceeds u32".to_owned())?,
        });
    }
    if platform.got_region_bytes > 0 {
        seeds.push(SectionSeed {
            source_kind: "platform-got",
            source_id: "macho-arm64-platform-got".to_owned(),
            segment_name: "__DATA_CONST".to_owned(),
            section_name: "__got".to_owned(),
            source_image_offset: platform.got_region_offset,
            source_size_bytes: platform.got_region_bytes,
            alignment: platform.got_alignment,
            flags: GOT_FLAGS,
            zero_fill: false,
            reserved1: u32::try_from(platform.stub_entry_count)
                .map_err(|_| "Mach-O stub indirect-symbol count exceeds u32".to_owned())?,
            reserved2: 0,
        });
    }
    validate_seed_ranges(&seeds, platform.planned_image_span_bytes)?;
    Ok(seeds)
}

fn group_seeds(seeds: Vec<SectionSeed>) -> Result<Vec<(String, Vec<SectionSeed>)>, String> {
    let mut groups = BTreeMap::<String, Vec<SectionSeed>>::new();
    let mut identities = BTreeSet::new();
    for seed in seeds {
        validate_macho_name(&seed.segment_name, "segment")?;
        validate_macho_name(&seed.section_name, "section")?;
        if !identities.insert((seed.segment_name.clone(), seed.section_name.clone())) {
            return Err(format!(
                "Mach-O shell repeats section `{},{}`",
                seed.segment_name, seed.section_name
            ));
        }
        groups
            .entry(seed.segment_name.clone())
            .or_default()
            .push(seed);
    }
    if !groups.contains_key("__TEXT") {
        return Err("Mach-O shell layout requires a __TEXT segment".to_owned());
    }
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    groups.sort_by(|lhs, rhs| {
        segment_rank(&lhs.0)
            .cmp(&segment_rank(&rhs.0))
            .then(lhs.0.cmp(&rhs.0))
    });
    for (_, seeds) in &mut groups {
        seeds.sort_by(|lhs, rhs| {
            lhs.zero_fill
                .cmp(&rhs.zero_fill)
                .then(lhs.source_image_offset.cmp(&rhs.source_image_offset))
                .then(lhs.section_name.cmp(&rhs.section_name))
        });
    }
    Ok(groups)
}

fn load_command_size(
    groups: &[(String, Vec<SectionSeed>)],
    has_dylib: bool,
) -> Result<usize, String> {
    let section_count = groups.iter().try_fold(0usize, |count, (_, sections)| {
        count
            .checked_add(sections.len())
            .ok_or_else(|| "Mach-O section count overflows".to_owned())
    })?;
    if section_count > usize::from(u8::MAX) {
        return Err("Mach-O shell section count exceeds nlist_64 ordinal space".to_owned());
    }
    let segment_count = groups
        .len()
        .checked_add(2)
        .ok_or_else(|| "Mach-O segment count overflows".to_owned())?;
    let section_bytes = section_count
        .checked_mul(SECTION_COMMAND_SIZE)
        .ok_or_else(|| "Mach-O section command size overflows".to_owned())?;
    let segment_bytes = segment_count
        .checked_mul(SEGMENT_COMMAND_SIZE)
        .and_then(|value| value.checked_add(section_bytes))
        .ok_or_else(|| "Mach-O segment command size overflows".to_owned())?;
    let fixed = 48usize + 24 + 80 + 24 + 24 + 16;
    let dylinker = path_command_size(12, DYLINKER_PATH)?;
    let dylib = if has_dylib {
        path_command_size(24, SYSTEM_DYLIB_PATH)?
    } else {
        0
    };
    segment_bytes
        .checked_add(fixed)
        .and_then(|value| value.checked_add(dylinker))
        .and_then(|value| value.checked_add(dylib))
        .ok_or_else(|| "Mach-O load-command size overflows".to_owned())
}

fn build_load_commands(
    segments: &[NsldMachOArm64ShellSegmentPlan],
    sections: &[NsldMachOArm64ShellSectionPlan],
    has_dylib: bool,
) -> Result<Vec<NsldMachOArm64ShellLoadCommandPlan>, String> {
    let mut specs = Vec::<(&str, u32, usize, Option<String>, &str)>::new();
    for segment in segments {
        let section_count = sections
            .iter()
            .filter(|section| segment.section_ids.contains(&section.section_id))
            .count();
        let section_bytes = section_count
            .checked_mul(SECTION_COMMAND_SIZE)
            .ok_or_else(|| "Mach-O section command span overflows".to_owned())?;
        let command_size = SEGMENT_COMMAND_SIZE
            .checked_add(section_bytes)
            .ok_or_else(|| "Mach-O segment command span overflows".to_owned())?;
        specs.push((
            "segment-64",
            LC_SEGMENT_64,
            command_size,
            Some(segment.segment_id.clone()),
            "layout-bound",
        ));
    }
    specs.extend([
        (
            "dyld-info-only",
            LC_DYLD_INFO_ONLY,
            48,
            None,
            "layout-bound",
        ),
        ("symtab", LC_SYMTAB, 24, None, "layout-bound"),
        ("dysymtab", LC_DYSYMTAB, 80, None, "layout-bound"),
        (
            "load-dylinker",
            LC_LOAD_DYLINKER,
            path_command_size(12, DYLINKER_PATH)?,
            None,
            "registry-bound",
        ),
    ]);
    if has_dylib {
        specs.push((
            "load-dylib",
            LC_LOAD_DYLIB,
            path_command_size(24, SYSTEM_DYLIB_PATH)?,
            None,
            "registry-bound",
        ));
    }
    specs.extend([
        ("main", LC_MAIN, 24, None, "entry-bound"),
        (
            "build-version",
            LC_BUILD_VERSION,
            24,
            None,
            "platform-bound",
        ),
        (
            "code-signature",
            LC_CODE_SIGNATURE,
            16,
            None,
            "payload-pending",
        ),
    ]);
    let mut offset = MACHO_HEADER_SIZE;
    let mut commands = Vec::with_capacity(specs.len());
    for (index, (kind, value, size, segment_id, status)) in specs.into_iter().enumerate() {
        let command_id = format!("macho-arm64-shell-command-{index:04}");
        let audit_hash = command_audit_hash(
            &command_id,
            kind,
            value,
            offset,
            size,
            segment_id.as_deref(),
            status,
        );
        commands.push(NsldMachOArm64ShellLoadCommandPlan {
            command_id,
            command_kind: kind.to_owned(),
            command_value: value,
            command_offset: offset,
            command_size_bytes: size,
            segment_id,
            status: status.to_owned(),
            audit_hash,
        });
        offset = offset
            .checked_add(size)
            .ok_or_else(|| "Mach-O load-command offset overflows".to_owned())?;
    }
    Ok(commands)
}

fn pagezero_segment() -> NsldMachOArm64ShellSegmentPlan {
    let segment_id = "macho-arm64-shell-segment-0000".to_owned();
    let audit_hash = segment_audit_hash(
        &segment_id,
        "__PAGEZERO",
        0,
        0,
        0,
        0,
        MACHO_ARM64_IMAGE_BASE,
        &[],
    );
    NsldMachOArm64ShellSegmentPlan {
        segment_id,
        segment_name: "__PAGEZERO".to_owned(),
        segment_index: 0,
        file_offset: 0,
        file_size_bytes: 0,
        vm_address: 0,
        vm_size_bytes: MACHO_ARM64_IMAGE_BASE as usize,
        max_protection: 0,
        initial_protection: 0,
        section_ids: Vec::new(),
        audit_hash,
    }
}

fn validate_seed_ranges(seeds: &[SectionSeed], image_span: usize) -> Result<(), String> {
    let mut occupied = Vec::<(usize, usize, &str)>::new();
    for seed in seeds {
        if seed.alignment == 0 || !seed.alignment.is_power_of_two() {
            return Err(format!(
                "Mach-O shell source `{}` has invalid alignment {}",
                seed.source_id, seed.alignment
            ));
        }
        let end = seed
            .source_image_offset
            .checked_add(seed.source_size_bytes)
            .ok_or_else(|| "Mach-O shell source range overflows".to_owned())?;
        if end > image_span {
            return Err(format!(
                "Mach-O shell source `{}` exceeds platform image span {image_span}",
                seed.source_id
            ));
        }
        if let Some((_, _, previous)) = occupied.iter().find(|(start, previous_end, _)| {
            seed.source_image_offset < *previous_end && *start < end
        }) {
            return Err(format!(
                "Mach-O shell source `{}` overlaps `{previous}`",
                seed.source_id
            ));
        }
        occupied.push((seed.source_image_offset, end, &seed.source_id));
    }
    Ok(())
}

fn validate_macho_name(name: &str, label: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 16 || !name.is_ascii() || !name.starts_with("__") {
        return Err(format!(
            "Mach-O shell {label} name `{name}` is not a canonical 1..16 byte Mach-O name"
        ));
    }
    Ok(())
}

fn segment_rank(name: &str) -> usize {
    match name {
        "__TEXT" => 0,
        "__DATA_CONST" => 1,
        "__DATA" => 2,
        _ => 3,
    }
}

fn segment_protections(name: &str) -> (u32, u32) {
    if name == "__TEXT" {
        (7, 5)
    } else {
        (3, 3)
    }
}

fn path_command_size(prefix: usize, path: &str) -> Result<usize, String> {
    align_up(
        prefix
            .checked_add(path.len() + 1)
            .ok_or_else(|| "Mach-O path command size overflows".to_owned())?,
        8,
    )
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err("Mach-O shell alignment must be a nonzero power of two".to_owned());
    }
    let mask = alignment - 1;
    value
        .checked_add(mask)
        .map(|value| value & !mask)
        .ok_or_else(|| "Mach-O shell alignment overflows".to_owned())
}

fn align_up_u64(value: u64, alignment: usize) -> Result<u64, String> {
    let alignment =
        u64::try_from(alignment).map_err(|_| "Mach-O shell VM alignment exceeds u64".to_owned())?;
    let mask = alignment - 1;
    value
        .checked_add(mask)
        .map(|value| value & !mask)
        .ok_or_else(|| "Mach-O shell VM alignment overflows".to_owned())
}

fn section_audit_hash(
    section_id: &str,
    seed: &SectionSeed,
    ordinal: usize,
    file_offset: Option<usize>,
    vm_address: u64,
) -> String {
    let mut out = String::new();
    append_text(&mut out, section_id);
    append_text(&mut out, seed.source_kind);
    append_text(&mut out, &seed.source_id);
    append_text(&mut out, &seed.segment_name);
    append_text(&mut out, &seed.section_name);
    writeln!(
        out,
        "facts={}|{}|{}|{}|{}|{}|{:08x}|{}|{}|{}",
        ordinal,
        seed.source_image_offset,
        seed.source_size_bytes,
        seed.alignment,
        file_offset.map_or("none".to_owned(), |value| value.to_string()),
        vm_address,
        seed.flags,
        seed.zero_fill,
        seed.reserved1,
        seed.reserved2
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn segment_audit_hash(
    segment_id: &str,
    name: &str,
    index: usize,
    file_offset: usize,
    file_size: usize,
    vm_address: u64,
    vm_size: u64,
    section_ids: &[String],
) -> String {
    let mut out = String::new();
    append_text(&mut out, segment_id);
    append_text(&mut out, name);
    writeln!(
        out,
        "facts={index}|{file_offset}|{file_size}|{vm_address}|{vm_size}"
    )
    .unwrap();
    for section_id in section_ids {
        append_text(&mut out, section_id);
    }
    crate::fnv1a64_hex(out.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn command_audit_hash(
    command_id: &str,
    kind: &str,
    value: u32,
    offset: usize,
    size: usize,
    segment_id: Option<&str>,
    status: &str,
) -> String {
    let mut out = String::new();
    append_text(&mut out, command_id);
    append_text(&mut out, kind);
    append_text(&mut out, segment_id.unwrap_or("none"));
    append_text(&mut out, status);
    writeln!(out, "facts={value:08x}|{offset}|{size}").unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}
