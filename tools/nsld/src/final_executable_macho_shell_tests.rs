use super::*;
use crate::{
    final_executable_macho_application::apply_macho_arm64_patch_previews,
    final_executable_macho_input::{
        ParsedMachOObjectLinkage, ParsedMachORelocation, ParsedMachOSection, ParsedMachOSymbol,
    },
    final_executable_macho_layout::{build_macho_placement_binding_report, MachOLayoutObject},
    final_executable_macho_materialization::{
        build_macho_arm64_materialization_preview, MachOImageObject,
    },
    final_executable_macho_platform::build_macho_arm64_platform_structure_plan,
    final_executable_macho_platform_application::{
        apply_macho_arm64_platform_structure, MachOArm64PlatformAppliedImage,
    },
    final_executable_macho_relocation::build_macho_arm64_relocation_application_report,
    final_executable_macho_shell_image_linkedit::encode_shell_linkedit,
    reports::{
        NsldMachOArm64MaterializationPreviewReport, NsldMachOArm64PlatformStructurePlanReport,
        NsldMachOArm64RelocationApplicationReport, NsldMachOPlacementBindingReport,
    },
};
use std::collections::BTreeSet;

pub(crate) struct ShellFixture {
    pub(crate) program: ParsedMachOObjectLinkage,
    pub(crate) runtime: ParsedMachOObjectLinkage,
    pub(crate) placement: NsldMachOPlacementBindingReport,
    pub(crate) relocations: NsldMachOArm64RelocationApplicationReport,
    pub(crate) preview: NsldMachOArm64MaterializationPreviewReport,
    pub(crate) platform: NsldMachOArm64PlatformStructurePlanReport,
    pub(crate) applied: MachOArm64PlatformAppliedImage,
}

#[test]
fn plans_a_deterministic_arm64_shell_with_static_compatibility_metadata() {
    let fixture = shell_fixture(Some("_nuis_entry"), false);
    let first = build_shell(&fixture).unwrap();
    let second = build_shell(&fixture).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.contract, MACHO_ARM64_SHELL_LAYOUT_PLAN_CONTRACT);
    assert_eq!(first.status, "layout-planned-with-code-signature-boundary");
    assert_eq!(first.page_size, 0x4000);
    assert_eq!(first.image_base_vm_address, 0x1_0000_0000);
    assert_eq!(first.entry_rule_id, "arm64.macho.program-entry.v1");
    assert_eq!(first.entry_symbol, "_nuis_entry");
    assert!(first.entry_file_offset >= first.first_content_file_offset);
    assert_eq!(
        first.entry_vm_address,
        first.image_base_vm_address + first.entry_file_offset as u64
    );
    assert_eq!(first.required_address_rewrite_count, 2);

    let segment_names = first
        .segments
        .iter()
        .map(|segment| segment.segment_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        segment_names,
        ["__PAGEZERO", "__TEXT", "__DATA_CONST", "__LINKEDIT"]
    );
    assert_eq!(first.segment_count, first.segments.len());
    assert_eq!(first.section_count, 3);
    assert!(first
        .sections
        .iter()
        .any(|section| { section.segment_name == "__TEXT" && section.section_name == "__stubs" }));
    assert!(first.sections.iter().any(|section| {
        section.segment_name == "__DATA_CONST" && section.section_name == "__got"
    }));

    let symbol_names = first
        .symbols
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(symbol_names, ["_nuis_entry", "_runtime_anchor", "_puts"]);
    assert_eq!(first.defined_symbol_count, 2);
    assert_eq!(first.undefined_symbol_count, 1);
    assert_eq!(first.indirect_symbol_count, 2);
    assert_eq!(first.binds.len(), 1);
    assert!(first.rebases.is_empty());
    assert_eq!(first.binds[0].target_symbol, "_puts");
    assert_eq!(first.binds[0].dylib_ordinal, 1);
    let got_section = first
        .sections
        .iter()
        .find(|section| section.section_id == first.binds[0].shell_section_id)
        .unwrap();
    assert_eq!(got_section.segment_name, "__DATA_CONST");
    assert_eq!(Some(first.binds[0].file_offset), got_section.file_offset);

    assert!(first
        .load_commands
        .iter()
        .any(|command| command.command_kind == "load-dylib" && command.status == "registry-bound"));
    assert!(first
        .load_commands
        .iter()
        .any(|command| command.command_kind == "main" && command.status == "entry-bound"));
    assert!(first
        .load_commands
        .iter()
        .any(|command| command.command_kind == "uuid" && command.status == "image-bound"));
    assert!(first
        .load_commands
        .iter()
        .any(|command| command.command_kind == "code-signature"
            && command.status == "payload-pending"));
    assert_eq!(first.code_signature_status, "required-payload-pending");
    assert!(first.symbol_table_offset >= first.linkedit_file_offset);
    assert!(first.indirect_symbol_table_offset >= first.symbol_table_offset);
    assert!(first.string_table_offset >= first.indirect_symbol_table_offset);
    assert!(first.code_signature_file_offset >= first.planned_file_span_bytes);
    assert!(!first.plan_hash.is_empty());
}

#[test]
fn runtime_main_registry_rule_has_priority_over_program_entry() {
    let fixture = shell_fixture(Some("_nuis_entry"), true);
    let report = build_shell(&fixture).unwrap();

    assert_eq!(report.entry_rule_id, "arm64.macho.runtime-main.v1");
    assert_eq!(report.entry_symbol, "_main");
}

#[test]
fn missing_registered_entry_fails_closed() {
    let fixture = shell_fixture(None, false);
    let error = build_shell(&fixture).unwrap_err();

    assert!(error.contains("entry registry found no supported definition"));
}

#[test]
fn platform_image_drift_fails_before_shell_publication() {
    let mut fixture = shell_fixture(Some("_nuis_entry"), false);
    fixture.applied.bytes[0] ^= 0xff;
    let error = build_shell(&fixture).unwrap_err();

    assert!(error.contains("platform application drift"));
}

#[test]
fn common_definition_becomes_a_vm_only_shell_section_and_defined_symbol() {
    let fixture = common_shell_fixture();
    let report = build_shell(&fixture).unwrap();

    let section = report
        .sections
        .iter()
        .find(|section| section.section_name == "__nuis_common")
        .unwrap();
    assert_eq!(section.segment_name, "__DATA");
    assert_eq!(section.flags, 1);
    assert_eq!(section.file_offset, None);
    assert_eq!(section.file_size_bytes, 0);
    assert_eq!(section.vm_size_bytes, 16);
    let symbol = report
        .symbols
        .iter()
        .find(|symbol| symbol.name == "_state")
        .unwrap();
    assert_eq!(
        symbol.shell_section_id.as_deref(),
        Some(section.section_id.as_str())
    );
    assert_eq!(symbol.vm_address, Some(section.vm_address));
    assert_eq!(report.undefined_symbol_count, 0);
}

#[test]
fn absolute_and_alias_definitions_emit_resolved_shell_symbols() {
    let fixture = symbol_resolution_shell_fixture();
    let report = build_shell(&fixture).unwrap();

    let absolute = report
        .symbols
        .iter()
        .find(|symbol| symbol.name == "_constant")
        .unwrap();
    assert_eq!(absolute.record_kind, "external-absolute");
    assert_eq!(absolute.shell_section_id, None);
    assert_eq!(absolute.source_image_offset, None);
    assert_eq!(absolute.vm_address, Some(0x1122_3344_5566_7788));
    let absolute_alias = report
        .symbols
        .iter()
        .find(|symbol| symbol.name == "_constant_alias")
        .unwrap();
    assert_eq!(absolute_alias.record_kind, "external-absolute-alias");
    assert_eq!(absolute_alias.vm_address, absolute.vm_address);
    let section_alias = report
        .symbols
        .iter()
        .find(|symbol| symbol.name == "_entry_alias")
        .unwrap();
    assert_eq!(section_alias.record_kind, "external-defined-alias");
    assert_eq!(section_alias.vm_address, Some(report.entry_vm_address));

    let encoded = encode_shell_linkedit(&report).unwrap();
    for symbol in [absolute, absolute_alias] {
        let offset = symbol.symbol_table_index * 16;
        assert_eq!(encoded.symbol_table[offset + 4], 0x03);
        assert_eq!(encoded.symbol_table[offset + 5], 0);
        assert_eq!(
            u64::from_le_bytes(
                encoded.symbol_table[offset + 8..offset + 16]
                    .try_into()
                    .unwrap()
            ),
            0x1122_3344_5566_7788
        );
    }
}

pub(crate) fn build_shell(
    fixture: &ShellFixture,
) -> Result<crate::reports::NsldMachOArm64ShellLayoutPlanReport, String> {
    let objects = [
        layout("host.program", "program-llvm", &fixture.program),
        layout("host.runtime", "runtime-shim", &fixture.runtime),
    ];
    build_macho_arm64_shell_layout_plan(
        &objects,
        &fixture.placement,
        &fixture.relocations,
        &fixture.platform,
        &fixture.applied,
    )
}

pub(crate) fn shell_fixture(program_entry: Option<&str>, runtime_main: bool) -> ShellFixture {
    let mut program_symbols = Vec::new();
    if let Some(entry) = program_entry {
        program_symbols.push(defined_symbol(program_symbols.len(), entry));
    }
    let puts_index = program_symbols.len();
    program_symbols.push(undefined_symbol(puts_index, "_puts"));
    let program = linkage(
        program_symbols,
        vec![ParsedMachORelocation {
            section_ordinal: 1,
            offset: 0,
            symbol_number: puts_index,
            width_bytes: 4,
            pc_relative: true,
            external: true,
            relocation_type: 2,
        }],
    );
    let mut runtime_symbols = vec![defined_symbol(0, "_runtime_anchor")];
    if runtime_main {
        runtime_symbols.push(defined_symbol(1, "_main"));
    }
    let runtime = linkage(runtime_symbols, Vec::new());
    let program_bytes = 0x9400_0000u32.to_le_bytes();
    let runtime_bytes = 0xd65f_03c0u32.to_le_bytes();
    let layouts = [
        layout("host.program", "program-llvm", &program),
        layout("host.runtime", "runtime-shim", &runtime),
    ];
    let placement = build_macho_placement_binding_report(&layouts).unwrap();
    let relocations =
        build_macho_arm64_relocation_application_report(&layouts, &placement).unwrap();
    let images = [
        image("host.program", "program-llvm", &program_bytes, &program),
        image("host.runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let preview =
        build_macho_arm64_materialization_preview(&images, &placement, &relocations).unwrap();
    let applied =
        apply_macho_arm64_patch_previews(&images, &placement, &relocations, &preview).unwrap();
    let platform =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
            .unwrap();
    let applied =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &platform)
            .unwrap();
    ShellFixture {
        program,
        runtime,
        placement,
        relocations,
        preview,
        platform,
        applied,
    }
}

pub(crate) fn internal_got_shell_fixture() -> ShellFixture {
    let program = linkage_sized(
        8,
        vec![
            defined_symbol(0, "_nuis_entry"),
            undefined_symbol(1, "_runtime_anchor"),
        ],
        vec![
            ParsedMachORelocation {
                section_ordinal: 1,
                offset: 0,
                symbol_number: 1,
                width_bytes: 4,
                pc_relative: true,
                external: true,
                relocation_type: 5,
            },
            ParsedMachORelocation {
                section_ordinal: 1,
                offset: 4,
                symbol_number: 1,
                width_bytes: 4,
                pc_relative: false,
                external: true,
                relocation_type: 6,
            },
        ],
    );
    let runtime = linkage(vec![defined_symbol(0, "_runtime_anchor")], Vec::new());
    let mut program_bytes = Vec::new();
    program_bytes.extend_from_slice(&0x9000_0000u32.to_le_bytes());
    program_bytes.extend_from_slice(&0xf940_0000u32.to_le_bytes());
    let runtime_bytes = 0xd65f_03c0u32.to_le_bytes();
    let layouts = [
        layout("host.program", "program-llvm", &program),
        layout("host.runtime", "runtime-shim", &runtime),
    ];
    let placement = build_macho_placement_binding_report(&layouts).unwrap();
    let relocations =
        build_macho_arm64_relocation_application_report(&layouts, &placement).unwrap();
    let images = [
        image("host.program", "program-llvm", &program_bytes, &program),
        image("host.runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let preview =
        build_macho_arm64_materialization_preview(&images, &placement, &relocations).unwrap();
    let applied =
        apply_macho_arm64_patch_previews(&images, &placement, &relocations, &preview).unwrap();
    let platform =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
            .unwrap();
    let applied =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &platform)
            .unwrap();
    ShellFixture {
        program,
        runtime,
        placement,
        relocations,
        preview,
        platform,
        applied,
    }
}

pub(crate) fn loader_probe_shell_fixture() -> ShellFixture {
    let program = linkage_sized(8, vec![defined_symbol(0, "_nuis_entry")], Vec::new());
    let runtime = linkage(vec![defined_symbol(0, "_runtime_anchor")], Vec::new());
    let mut program_bytes = Vec::new();
    program_bytes.extend_from_slice(&0x5280_0000u32.to_le_bytes());
    program_bytes.extend_from_slice(&0xd65f_03c0u32.to_le_bytes());
    let runtime_bytes = 0xd65f_03c0u32.to_le_bytes();
    let layouts = [
        layout("host.program", "program-llvm", &program),
        layout("host.runtime", "runtime-shim", &runtime),
    ];
    let placement = build_macho_placement_binding_report(&layouts).unwrap();
    let relocations =
        build_macho_arm64_relocation_application_report(&layouts, &placement).unwrap();
    let images = [
        image("host.program", "program-llvm", &program_bytes, &program),
        image("host.runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let preview =
        build_macho_arm64_materialization_preview(&images, &placement, &relocations).unwrap();
    let applied =
        apply_macho_arm64_patch_previews(&images, &placement, &relocations, &preview).unwrap();
    let platform =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
            .unwrap();
    let applied =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &platform)
            .unwrap();
    ShellFixture {
        program,
        runtime,
        placement,
        relocations,
        preview,
        platform,
        applied,
    }
}

fn common_shell_fixture() -> ShellFixture {
    let program = linkage_sized(
        12,
        vec![
            defined_symbol(0, "_nuis_entry"),
            common_symbol(1, "_state", 16, 8),
        ],
        vec![
            ParsedMachORelocation {
                section_ordinal: 1,
                offset: 0,
                symbol_number: 1,
                width_bytes: 4,
                pc_relative: true,
                external: true,
                relocation_type: 3,
            },
            ParsedMachORelocation {
                section_ordinal: 1,
                offset: 4,
                symbol_number: 1,
                width_bytes: 4,
                pc_relative: false,
                external: true,
                relocation_type: 4,
            },
        ],
    );
    let runtime = linkage(vec![defined_symbol(0, "_runtime_anchor")], Vec::new());
    let program_bytes = [
        0x9000_0000u32.to_le_bytes(),
        0x9100_0000u32.to_le_bytes(),
        0xd65f_03c0u32.to_le_bytes(),
    ]
    .concat();
    let runtime_bytes = 0xd65f_03c0u32.to_le_bytes();
    let layouts = [
        layout("host.program", "program-llvm", &program),
        layout("host.runtime", "runtime-shim", &runtime),
    ];
    let placement = build_macho_placement_binding_report(&layouts).unwrap();
    let relocations =
        build_macho_arm64_relocation_application_report(&layouts, &placement).unwrap();
    let images = [
        image("host.program", "program-llvm", &program_bytes, &program),
        image("host.runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let preview =
        build_macho_arm64_materialization_preview(&images, &placement, &relocations).unwrap();
    let applied =
        apply_macho_arm64_patch_previews(&images, &placement, &relocations, &preview).unwrap();
    let platform =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
            .unwrap();
    let applied =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &platform)
            .unwrap();
    ShellFixture {
        program,
        runtime,
        placement,
        relocations,
        preview,
        platform,
        applied,
    }
}

pub(crate) fn symbol_resolution_shell_fixture() -> ShellFixture {
    let program = linkage_sized(
        12,
        vec![
            defined_symbol(0, "_nuis_entry"),
            absolute_symbol(1, "_constant", 0x1122_3344_5566_7788),
            indirect_symbol(2, "_constant_alias", "_constant"),
            indirect_symbol(3, "_entry_alias", "_nuis_entry"),
        ],
        vec![ParsedMachORelocation {
            section_ordinal: 1,
            offset: 4,
            symbol_number: 2,
            width_bytes: 8,
            pc_relative: false,
            external: true,
            relocation_type: 0,
        }],
    );
    let runtime = linkage(vec![defined_symbol(0, "_runtime_anchor")], Vec::new());
    let program_bytes = [0xd65f_03c0u32.to_le_bytes().as_slice(), [0u8; 8].as_slice()].concat();
    let runtime_bytes = 0xd65f_03c0u32.to_le_bytes();
    let layouts = [
        layout("host.program", "program-llvm", &program),
        layout("host.runtime", "runtime-shim", &runtime),
    ];
    let placement = build_macho_placement_binding_report(&layouts).unwrap();
    let relocations =
        build_macho_arm64_relocation_application_report(&layouts, &placement).unwrap();
    let images = [
        image("host.program", "program-llvm", &program_bytes, &program),
        image("host.runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let preview =
        build_macho_arm64_materialization_preview(&images, &placement, &relocations).unwrap();
    let applied =
        apply_macho_arm64_patch_previews(&images, &placement, &relocations, &preview).unwrap();
    let platform =
        build_macho_arm64_platform_structure_plan(&placement, &relocations, &applied.report)
            .unwrap();
    let applied =
        apply_macho_arm64_platform_structure(&placement, &relocations, &applied, &platform)
            .unwrap();
    ShellFixture {
        program,
        runtime,
        placement,
        relocations,
        preview,
        platform,
        applied,
    }
}

fn linkage(
    symbols: Vec<ParsedMachOSymbol>,
    relocations: Vec<ParsedMachORelocation>,
) -> ParsedMachOObjectLinkage {
    linkage_sized(4, symbols, relocations)
}

fn linkage_sized(
    section_size: u64,
    symbols: Vec<ParsedMachOSymbol>,
    relocations: Vec<ParsedMachORelocation>,
) -> ParsedMachOObjectLinkage {
    let external_definitions = symbols
        .iter()
        .filter(|symbol| symbol.external && symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect::<BTreeSet<_>>();
    let external_undefined = symbols
        .iter()
        .filter(|symbol| symbol.external && !symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect::<BTreeSet<_>>();
    ParsedMachOObjectLinkage {
        section_count: 1,
        symbol_count: symbols.len(),
        relocation_count: relocations.len(),
        defined_symbol_count: external_definitions.len(),
        undefined_symbol_count: external_undefined.len(),
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

fn defined_symbol(index: usize, name: &str) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index,
        name: name.to_owned(),
        kind: "section".to_owned(),
        external: true,
        defined: true,
        section_ordinal: Some(1),
        value: 0,
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

fn layout<'a>(
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

fn image<'a>(
    object_id: &'a str,
    role: &'a str,
    bytes: &'a [u8],
    linkage: &'a ParsedMachOObjectLinkage,
) -> MachOImageObject<'a> {
    MachOImageObject {
        object_id,
        role,
        bytes,
        linkage,
    }
}
