use super::*;
use crate::{
    final_executable_macho_input::{
        parse_macho_arm64_object_linkage, ParsedMachOObjectLinkage, ParsedMachORelocation,
        ParsedMachOSection, ParsedMachOSymbol,
    },
    final_executable_macho_layout::build_macho_placement_binding_report,
    final_executable_macho_materialization::{
        build_macho_arm64_materialization_preview, MachOImageObject,
    },
};

#[test]
fn relocation_application_is_deterministic_and_resolves_placed_targets() {
    let program = linkage(
        16,
        vec![
            defined_symbol(0, "_entry", 0),
            undefined_symbol(1, "_runtime"),
        ],
        vec![
            relocation(0, 1, 4, true, true, ARM64_RELOC_BRANCH26),
            relocation(8, 1, 8, false, false, ARM64_RELOC_UNSIGNED),
        ],
    );
    let runtime = linkage(16, vec![defined_symbol(0, "_runtime", 4)], vec![]);
    let forward = [
        layout_object("host.program", "program-llvm", &program),
        layout_object("host.runtime", "runtime-shim", &runtime),
    ];
    let reversed = [
        layout_object("host.runtime", "runtime-shim", &runtime),
        layout_object("host.program", "program-llvm", &program),
    ];
    let placement = build_macho_placement_binding_report(&forward).unwrap();
    let reversed_placement = build_macho_placement_binding_report(&reversed).unwrap();

    let report = build_macho_arm64_relocation_application_report(&forward, &placement).unwrap();
    let reordered =
        build_macho_arm64_relocation_application_report(&reversed, &reversed_placement).unwrap();

    assert_eq!(report, reordered);
    assert_eq!(report.contract, MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT);
    assert_eq!(report.status, "ready-for-byte-encoding");
    assert_eq!(report.relocation_count, 2);
    assert_eq!(report.registered_kind_count, 2);
    assert_eq!(report.ready_application_count, 2);
    assert_eq!(report.platform_structure_count, 0);
    assert_eq!(report.applications[0].relocation_kind, "arm64-branch26");
    assert_eq!(report.applications[0].source_output_offset, 0);
    assert_eq!(
        report.applications[0].target_symbol.as_deref(),
        Some("_runtime")
    );
    assert_eq!(report.applications[0].target_output_offset, Some(20));
    assert_eq!(report.applications[1].resolver_status, "local-section");
    assert_eq!(report.applications[1].target_output_offset, Some(0));
}

#[test]
fn direct_common_relocations_bind_to_the_allocated_zero_fill_address() {
    let object = linkage(
        8,
        vec![
            defined_symbol(0, "_entry", 0),
            common_symbol(1, "_state", 16, 8),
        ],
        vec![
            relocation(0, 1, 4, true, true, ARM64_RELOC_PAGE21),
            relocation(4, 1, 4, false, true, ARM64_RELOC_PAGEOFF12),
        ],
    );
    let bytes = [0x9000_0000u32.to_le_bytes(), 0x9100_0000u32.to_le_bytes()].concat();
    let objects = [layout_object("host.program", "program-llvm", &object)];
    let placement = build_macho_placement_binding_report(&objects).unwrap();

    let report = build_macho_arm64_relocation_application_report(&objects, &placement).unwrap();

    assert_eq!(placement.common_allocations[0].output_offset, 8);
    assert_eq!(report.ready_application_count, 2);
    assert!(report
        .applications
        .iter()
        .all(|application| application.target_output_offset == Some(8)));
    assert!(report
        .applications
        .iter()
        .all(|application| application.resolver_status == "internal"));
    let images = [MachOImageObject {
        object_id: "host.program",
        role: "program-llvm",
        bytes: &bytes,
        linkage: &object,
    }];
    let preview = build_macho_arm64_materialization_preview(&images, &placement, &report).unwrap();
    assert_eq!(preview.zero_fill_bytes, 16);
    assert_eq!(preview.patches.len(), 2);
    assert_eq!(preview.patches[1].encoded_bytes_hex, "00200091");
}

#[test]
fn absolute_unsigned_relocation_preserves_value_semantics() {
    let object = linkage(
        8,
        vec![absolute_symbol(0, "_constant", 0x1122_3344_5566_7788)],
        vec![relocation(0, 0, 8, false, true, ARM64_RELOC_UNSIGNED)],
    );
    let bytes = [0u8; 8];
    let objects = [layout_object("host.program", "program-llvm", &object)];
    let placement = build_macho_placement_binding_report(&objects).unwrap();

    let report = build_macho_arm64_relocation_application_report(&objects, &placement).unwrap();

    assert_eq!(report.applications[0].target_output_offset, None);
    assert_eq!(
        report.applications[0].target_absolute_value,
        Some(0x1122_3344_5566_7788)
    );
    let images = [MachOImageObject {
        object_id: "host.program",
        role: "program-llvm",
        bytes: &bytes,
        linkage: &object,
    }];
    let preview = build_macho_arm64_materialization_preview(&images, &placement, &report).unwrap();
    assert_eq!(preview.patches[0].target_output_offset, None);
    assert_eq!(
        preview.patches[0].target_absolute_value,
        Some(0x1122_3344_5566_7788)
    );
    assert_eq!(preview.patches[0].encoded_bytes_hex, "8877665544332211");
}

#[test]
fn indirect_relocation_consumes_the_layout_alias_resolution() {
    let object = linkage(
        12,
        vec![
            indirect_symbol(0, "_alias", "_target"),
            defined_symbol(1, "_target", 8),
        ],
        vec![relocation(0, 0, 4, true, true, ARM64_RELOC_BRANCH26)],
    );
    let objects = [layout_object("host.program", "program-llvm", &object)];
    let placement = build_macho_placement_binding_report(&objects).unwrap();

    let report = build_macho_arm64_relocation_application_report(&objects, &placement).unwrap();

    assert_eq!(report.applications[0].target_output_offset, Some(8));
    assert_eq!(report.applications[0].target_absolute_value, None);
    assert_eq!(
        report.applications[0].target_alias_chain,
        ["_alias", "_target"]
    );
}

#[test]
fn paired_addend_subtractor_and_got_records_remain_explicit() {
    let object = linkage(
        32,
        vec![defined_symbol(0, "_low", 0), defined_symbol(1, "_high", 8)],
        vec![
            relocation(0, 0, 8, false, true, ARM64_RELOC_SUBTRACTOR),
            relocation(0, 1, 8, false, true, ARM64_RELOC_UNSIGNED),
            relocation(8, 0x00ff_fffc, 4, false, false, ARM64_RELOC_ADDEND),
            relocation(8, 1, 4, false, true, ARM64_RELOC_PAGEOFF12),
            relocation(12, 1, 4, true, true, ARM64_RELOC_GOT_LOAD_PAGE21),
            relocation(16, 1, 4, false, true, ARM64_RELOC_GOT_LOAD_PAGEOFF12),
        ],
    );
    let objects = [layout_object("host.program", "program-llvm", &object)];
    let placement = build_macho_placement_binding_report(&objects).unwrap();

    let report = build_macho_arm64_relocation_application_report(&objects, &placement).unwrap();

    assert_eq!(report.relocation_count, 6);
    assert_eq!(report.metadata_record_count, 2);
    assert_eq!(report.platform_structure_count, 2);
    assert_eq!(report.status, "planned-with-platform-structure-boundary");
    assert_eq!(
        report.applications[0].pair_relocation_id.as_deref(),
        Some("macho-arm64-reloc-000001")
    );
    assert_eq!(
        report.applications[1].pair_relocation_id.as_deref(),
        Some("macho-arm64-reloc-000000")
    );
    assert_eq!(report.applications[2].explicit_addend, Some(-4));
    assert_eq!(
        report.applications[2].pair_relocation_id.as_deref(),
        Some("macho-arm64-reloc-000003")
    );
    assert_eq!(
        report.applications[3].pair_relocation_id.as_deref(),
        Some("macho-arm64-reloc-000002")
    );
    assert_eq!(
        report.applications[4].application_status,
        "planned-platform-structure"
    );
    assert_eq!(
        report.applications[5].application_status,
        "planned-platform-structure"
    );
}

#[test]
fn unregistered_relocation_type_fails_closed() {
    let object = linkage(
        8,
        vec![defined_symbol(0, "_target", 0)],
        vec![relocation(0, 0, 4, true, true, 7)],
    );
    let objects = [layout_object("host.program", "program-llvm", &object)];
    let placement = build_macho_placement_binding_report(&objects).unwrap();

    let error = build_macho_arm64_relocation_application_report(&objects, &placement).unwrap_err();
    assert!(error.contains("unregistered ARM64 Mach-O relocation type 7"));
    assert!(error.contains("fails closed"));
}

#[test]
fn malformed_paired_relocation_fails_closed() {
    let object = linkage(
        8,
        vec![defined_symbol(0, "_target", 0)],
        vec![relocation(0, 7, 4, false, false, ARM64_RELOC_ADDEND)],
    );
    let objects = [layout_object("host.program", "program-llvm", &object)];
    let placement = build_macho_placement_binding_report(&objects).unwrap();

    let error = build_macho_arm64_relocation_application_report(&objects, &placement).unwrap_err();
    assert!(error.contains("ADDEND relocation 0 has no paired record"));
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn plans_every_relocation_from_real_nuisc_program_and_runtime_objects() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nsld-real-relocation-plan-{stamp}"));
    let input = dir.join("main.ns");
    let output = dir.join("out");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&input, "mod cpu Main { fn main() -> i64 { return 0; } }\n").unwrap();
    nuisc::run(nuisc::cli::CommandKind::Compile {
        input,
        output_dir: output.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: Some("native-cpu-llvm".to_owned()),
    })
    .unwrap();
    let artifact =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    let parsed = artifact
        .host_objects
        .iter()
        .map(|object| parse_macho_arm64_object_linkage(&object.bytes).unwrap())
        .collect::<Vec<_>>();
    let objects = artifact
        .host_objects
        .iter()
        .zip(&parsed)
        .map(|(object, linkage)| MachOLayoutObject {
            object_id: &object.object_id,
            role: &object.role,
            linkage,
        })
        .collect::<Vec<_>>();
    let placement = build_macho_placement_binding_report(&objects).unwrap();
    let report = build_macho_arm64_relocation_application_report(&objects, &placement).unwrap();
    let images = artifact
        .host_objects
        .iter()
        .zip(&parsed)
        .map(|(object, linkage)| MachOImageObject {
            object_id: &object.object_id,
            role: &object.role,
            bytes: &object.bytes,
            linkage,
        })
        .collect::<Vec<_>>();
    let preview = build_macho_arm64_materialization_preview(&images, &placement, &report).unwrap();
    let parsed_count = parsed
        .iter()
        .map(|object| object.relocation_count)
        .sum::<usize>();
    std::fs::remove_dir_all(dir).unwrap();

    assert!(parsed_count > 0);
    assert_eq!(report.relocation_count, parsed_count);
    assert_eq!(report.applications.len(), parsed_count);
    assert_eq!(preview.image_span_bytes, placement.image_span_bytes);
    assert_eq!(
        preview.previewed_patch_count,
        report.ready_application_count
    );
    assert!(!preview.image_hash.is_empty());
    assert!(!preview.patch_plan_hash.is_empty());
    assert!(report.registered_kind_count >= 6);
    assert!(report
        .applications
        .iter()
        .all(|item| item.application_status.starts_with("planned-")
            || item.application_status == "paired-metadata"));
}

fn layout_object<'a>(
    object_id: &'a str,
    role: &'a str,
    linkage: &'a ParsedMachOObjectLinkage,
) -> MachOLayoutObject<'a> {
    MachOLayoutObject {
        object_id,
        role,
        linkage,
    }
}

fn linkage(
    section_size: u64,
    symbols: Vec<ParsedMachOSymbol>,
    relocations: Vec<ParsedMachORelocation>,
) -> ParsedMachOObjectLinkage {
    let external_definitions = symbols
        .iter()
        .filter(|symbol| symbol.external && symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect();
    let external_undefined = symbols
        .iter()
        .filter(|symbol| symbol.external && !symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect();
    ParsedMachOObjectLinkage {
        section_count: 1,
        symbol_count: symbols.len(),
        relocation_count: relocations.len(),
        defined_symbol_count: symbols.iter().filter(|symbol| symbol.defined).count(),
        undefined_symbol_count: symbols.iter().filter(|symbol| !symbol.defined).count(),
        external_definitions,
        external_undefined,
        sections: vec![ParsedMachOSection {
            ordinal: 1,
            segment_name: "__TEXT".to_owned(),
            name: "__text".to_owned(),
            address: 0,
            size: section_size,
            alignment: 4,
            flags: 0,
            zero_fill: false,
            payload_offset: 0,
            relocation_offset: 0,
            relocation_count: relocations.len(),
        }],
        symbols,
        relocations,
    }
}

fn defined_symbol(index: usize, name: &str, value: u64) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "section".to_owned(),
        external: true,
        defined: true,
        section_ordinal: Some(1),
        value,
        common_alignment: None,
        indirect_target: None,
    }
}

fn undefined_symbol(index: usize, name: &str) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "undefined".to_owned(),
        external: true,
        defined: false,
        section_ordinal: None,
        value: 0,
        common_alignment: None,
        indirect_target: None,
    }
}

fn common_symbol(index: usize, name: &str, size: u64, alignment: u64) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "common".to_owned(),
        external: true,
        defined: true,
        section_ordinal: None,
        value: size,
        common_alignment: Some(alignment),
        indirect_target: None,
    }
}

fn absolute_symbol(index: usize, name: &str, value: u64) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "absolute".to_owned(),
        external: true,
        defined: true,
        section_ordinal: None,
        value,
        common_alignment: None,
        indirect_target: None,
    }
}

fn indirect_symbol(index: usize, name: &str, target: &str) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "indirect".to_owned(),
        external: true,
        defined: true,
        section_ordinal: None,
        value: 0,
        common_alignment: None,
        indirect_target: Some(target.to_owned()),
    }
}

fn relocation(
    offset: u32,
    symbol_number: usize,
    width_bytes: u64,
    pc_relative: bool,
    external: bool,
    relocation_type: u32,
) -> ParsedMachORelocation {
    ParsedMachORelocation {
        section_ordinal: 1,
        offset,
        symbol_number,
        width_bytes,
        pc_relative,
        external,
        relocation_type,
    }
}
