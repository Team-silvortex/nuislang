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
    reports::{
        NsldMachOArm64PlatformStructurePlanReport, NsldMachOArm64RelocationApplicationReport,
        NsldMachOPlacementBindingReport,
    },
};
use std::collections::BTreeSet;

struct ShellFixture {
    program: ParsedMachOObjectLinkage,
    runtime: ParsedMachOObjectLinkage,
    placement: NsldMachOPlacementBindingReport,
    relocations: NsldMachOArm64RelocationApplicationReport,
    platform: NsldMachOArm64PlatformStructurePlanReport,
    applied: MachOArm64PlatformAppliedImage,
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
fn non_section_external_definition_is_not_silently_allocated() {
    let mut fixture = shell_fixture(Some("_nuis_entry"), false);
    fixture.program.symbols.push(ParsedMachOSymbol {
        index: 2,
        name: "_common".to_owned(),
        kind: "common".to_owned(),
        external: true,
        defined: true,
        section_ordinal: None,
        value: 8,
        common_alignment: Some(8),
        indirect_target: None,
    });
    fixture.program.symbol_count += 1;
    fixture.program.defined_symbol_count += 1;
    fixture
        .program
        .external_definitions
        .insert("_common".to_owned());
    let error = build_shell(&fixture).unwrap_err();

    assert!(error.contains("common/absolute allocation remains explicit"));
}

fn build_shell(
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

fn shell_fixture(program_entry: Option<&str>, runtime_main: bool) -> ShellFixture {
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
        platform,
        applied,
    }
}

fn linkage(
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
            size: 4,
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
