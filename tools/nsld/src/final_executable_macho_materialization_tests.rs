use super::*;
use crate::{
    final_executable_macho_application::apply_macho_arm64_patch_previews,
    final_executable_macho_input::{
        ParsedMachOObjectLinkage, ParsedMachORelocation, ParsedMachOSection, ParsedMachOSymbol,
    },
    final_executable_macho_layout::{build_macho_placement_binding_report, MachOLayoutObject},
    final_executable_macho_relocation::build_macho_arm64_relocation_application_report,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn merged_image_and_direct_patch_previews_are_deterministic() {
    let program_bytes = program_bytes();
    let runtime_bytes = 0xd65f_03c0u32.to_le_bytes();
    let program = program_linkage();
    let runtime = runtime_linkage();
    let layouts = [
        layout("program", "program-llvm", &program),
        layout("runtime", "runtime-shim", &runtime),
    ];
    let placement = build_macho_placement_binding_report(&layouts).unwrap();
    let relocations =
        build_macho_arm64_relocation_application_report(&layouts, &placement).unwrap();
    let images = [
        image("program", "program-llvm", &program_bytes, &program),
        image("runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let report =
        build_macho_arm64_materialization_preview(&images, &placement, &relocations).unwrap();

    assert_eq!(report.status, "preview-ready");
    assert_eq!(report.image_span_bytes, 28);
    assert_eq!(report.copied_bytes, 24);
    assert_eq!(report.zero_fill_bytes, 4);
    assert_eq!(report.planned_direct_count, 4);
    assert_eq!(report.previewed_patch_count, 4);
    assert_eq!(report.deferred_patch_count, 0);
    assert_eq!(report.section_audits.len(), 1);
    assert_eq!(report.section_audits[0].zero_fill_bytes, 4);
    assert_eq!(report.patches[0].source_bytes_hex, "00000094");
    assert_eq!(report.patches[0].encoded_bytes_hex, "06000094");
    assert_eq!(report.patches[1].encoded_bytes_hex, "00000090");
    assert_eq!(report.patches[2].encoded_bytes_hex, "00600091");
    assert_eq!(report.patches[3].encoded_bytes_hex, "1800000000000000");
    assert_ne!(
        report.patches[0].source_bytes_hash,
        report.patches[0].encoded_bytes_hash
    );
    let applied =
        apply_macho_arm64_patch_previews(&images, &placement, &relocations, &report).unwrap();
    assert_eq!(applied.report.status, "direct-patches-applied");
    assert_eq!(applied.report.expected_patch_count, 4);
    assert_eq!(applied.report.applied_patch_count, 4);
    assert_eq!(applied.report.write_once_span_count, 4);
    assert_ne!(
        applied.report.original_image_hash,
        applied.report.applied_image_hash
    );
    assert_eq!(
        crate::fnv1a64_hex(&applied.bytes),
        applied.report.applied_image_hash
    );
    assert_eq!(&applied.bytes[0..4], &0x9400_0006u32.to_le_bytes());
    assert_eq!(&applied.bytes[8..12], &0x9100_6000u32.to_le_bytes());
    assert_eq!(&applied.bytes[12..20], &24u64.to_le_bytes());

    let reversed_layouts = [
        layout("runtime", "runtime-shim", &runtime),
        layout("program", "program-llvm", &program),
    ];
    let reversed_placement = build_macho_placement_binding_report(&reversed_layouts).unwrap();
    let reversed_relocations =
        build_macho_arm64_relocation_application_report(&reversed_layouts, &reversed_placement)
            .unwrap();
    let reversed_images = [
        image("runtime", "runtime-shim", &runtime_bytes, &runtime),
        image("program", "program-llvm", &program_bytes, &program),
    ];
    let reversed = build_macho_arm64_materialization_preview(
        &reversed_images,
        &reversed_placement,
        &reversed_relocations,
    )
    .unwrap();
    assert_eq!(report, reversed);
    let reversed_applied = apply_macho_arm64_patch_previews(
        &reversed_images,
        &reversed_placement,
        &reversed_relocations,
        &reversed,
    )
    .unwrap();
    assert_eq!(applied.report, reversed_applied.report);
    assert_eq!(applied.bytes, reversed_applied.bytes);
}

#[test]
fn patch_application_rejects_source_and_preview_audit_drift() {
    let mut drifted_program_bytes = program_bytes();
    let runtime_bytes = 0xd65f_03c0u32.to_le_bytes();
    let program = program_linkage();
    let runtime = runtime_linkage();
    let layouts = [
        layout("program", "program-llvm", &program),
        layout("runtime", "runtime-shim", &runtime),
    ];
    let placement = build_macho_placement_binding_report(&layouts).unwrap();
    let relocations =
        build_macho_arm64_relocation_application_report(&layouts, &placement).unwrap();
    let images = [
        image("program", "program-llvm", &drifted_program_bytes, &program),
        image("runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let report =
        build_macho_arm64_materialization_preview(&images, &placement, &relocations).unwrap();

    drifted_program_bytes[0] ^= 1;
    let drifted_images = [
        image("program", "program-llvm", &drifted_program_bytes, &program),
        image("runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let source_error =
        apply_macho_arm64_patch_previews(&drifted_images, &placement, &relocations, &report)
            .unwrap_err();
    assert!(source_error.contains("source image drift"));

    let stable_program_bytes = program_bytes();
    let stable_images = [
        image("program", "program-llvm", &stable_program_bytes, &program),
        image("runtime", "runtime-shim", &runtime_bytes, &runtime),
    ];
    let mut tampered =
        build_macho_arm64_materialization_preview(&stable_images, &placement, &relocations)
            .unwrap();
    tampered.patches[0].encoded_bytes_hex = "07000094".to_owned();
    tampered.patch_plan_hash = materialization_patch_plan_hash(
        &placement.plan_hash,
        &relocations.plan_hash,
        &tampered.image_hash,
        &tampered.patches,
    );
    let audit_error =
        apply_macho_arm64_patch_previews(&stable_images, &placement, &relocations, &tampered)
            .unwrap_err();
    assert!(audit_error.contains("encoded byte hash drift"));

    let mut source_hash_tampered =
        build_macho_arm64_materialization_preview(&stable_images, &placement, &relocations)
            .unwrap();
    let patch = &mut source_hash_tampered.patches[0];
    patch.source_bytes_hash = "0000000000000000".to_owned();
    let application = relocations
        .applications
        .iter()
        .find(|item| item.relocation_id == patch.relocation_id)
        .unwrap();
    patch.audit_hash = patch_audit_hash(
        application,
        patch.target_output_offset,
        patch.effective_addend,
        &patch.source_bytes_hash,
        &patch.encoded_bytes_hash,
    );
    source_hash_tampered.patch_plan_hash = materialization_patch_plan_hash(
        &placement.plan_hash,
        &relocations.plan_hash,
        &source_hash_tampered.image_hash,
        &source_hash_tampered.patches,
    );
    let source_hash_error = apply_macho_arm64_patch_previews(
        &stable_images,
        &placement,
        &relocations,
        &source_hash_tampered,
    )
    .unwrap_err();
    assert!(source_hash_error.contains("source byte hash drift"));

    let mut reordered =
        build_macho_arm64_materialization_preview(&stable_images, &placement, &relocations)
            .unwrap();
    reordered.patches.reverse();
    reordered.patch_plan_hash = materialization_patch_plan_hash(
        &placement.plan_hash,
        &relocations.plan_hash,
        &reordered.image_hash,
        &reordered.patches,
    );
    let order_error =
        apply_macho_arm64_patch_previews(&stable_images, &placement, &relocations, &reordered)
            .unwrap_err();
    assert!(order_error.contains("preview order drift"));
}

#[test]
fn subtractor_pair_contributes_to_absolute_patch_without_mutation() {
    let metadata = application(
        "metadata",
        "arm64-subtractor",
        "paired-metadata",
        10,
        Some("direct"),
    );
    let direct = application(
        "direct",
        "arm64-unsigned",
        "planned-direct",
        30,
        Some("metadata"),
    );
    let applications = BTreeMap::from([
        (metadata.relocation_id.as_str(), &metadata),
        (direct.relocation_id.as_str(), &direct),
    ]);
    let source = 5i64.to_le_bytes();
    let (encoded, effective) = encode_unsigned(&direct, &source, 30, &applications).unwrap();
    assert_eq!(effective, -5);
    assert_eq!(encoded, 25u64.to_le_bytes());
    assert_eq!(source, 5i64.to_le_bytes());
}

#[test]
fn explicit_addend_is_applied_to_pageoff_preview() {
    let mut metadata = application(
        "metadata",
        "arm64-addend",
        "paired-metadata",
        0,
        Some("direct"),
    );
    metadata.explicit_addend = Some(8);
    let direct = application(
        "direct",
        "arm64-pageoff12",
        "planned-direct",
        24,
        Some("metadata"),
    );
    let applications = BTreeMap::from([
        (metadata.relocation_id.as_str(), &metadata),
        (direct.relocation_id.as_str(), &direct),
    ]);
    let source = 0x9100_0000u32.to_le_bytes();
    let (encoded, effective) = encode_pageoff12(&direct, &source, 24, &applications).unwrap();
    assert_eq!(effective, 8);
    assert_eq!(encoded, 0x9100_8000u32.to_le_bytes());
}

#[test]
fn invalid_or_unreachable_arm64_instruction_fails_closed() {
    let branch = application("branch", "arm64-branch26", "planned-direct", 2, None);
    let unaligned = encode_branch26(&branch, &0x9400_0000u32.to_le_bytes(), 2).unwrap_err();
    assert!(unaligned.contains("unaligned or out of range"));

    let invalid = encode_branch26(&branch, &0xd503_201fu32.to_le_bytes(), 4).unwrap_err();
    assert!(invalid.contains("is not B/BL"));

    let pageoff = application("pageoff", "arm64-pageoff12", "planned-direct", 8, None);
    let empty = BTreeMap::new();
    let unsupported =
        encode_pageoff12(&pageoff, &0xd503_201fu32.to_le_bytes(), 8, &empty).unwrap_err();
    assert!(unsupported.contains("is not supported ADD/load/store"));
}

fn program_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0x9400_0000u32.to_le_bytes());
    bytes.extend_from_slice(&0x9000_0000u32.to_le_bytes());
    bytes.extend_from_slice(&0x9100_0000u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes
}

fn program_linkage() -> ParsedMachOObjectLinkage {
    linkage(
        20,
        4,
        vec![undefined_symbol("_target")],
        vec![
            relocation(0, 4, true, 2),
            relocation(4, 4, true, 3),
            relocation(8, 4, false, 4),
            relocation(12, 8, false, 0),
        ],
    )
}

fn runtime_linkage() -> ParsedMachOObjectLinkage {
    linkage(4, 8, vec![defined_symbol("_target")], Vec::new())
}

fn linkage(
    size: u64,
    alignment: u64,
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
            size,
            alignment,
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

fn undefined_symbol(name: &str) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index: 0,
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

fn defined_symbol(name: &str) -> ParsedMachOSymbol {
    ParsedMachOSymbol {
        index: 0,
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

fn relocation(
    offset: u32,
    width_bytes: u64,
    pc_relative: bool,
    relocation_type: u32,
) -> ParsedMachORelocation {
    ParsedMachORelocation {
        section_ordinal: 1,
        offset,
        symbol_number: 0,
        width_bytes,
        pc_relative,
        external: true,
        relocation_type,
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

fn application(
    id: &str,
    kind: &str,
    status: &str,
    target: usize,
    pair: Option<&str>,
) -> NsldMachOArm64RelocationApplication {
    NsldMachOArm64RelocationApplication {
        relocation_id: id.to_owned(),
        object_id: "object".to_owned(),
        object_role: "program-llvm".to_owned(),
        input_section_ordinal: 1,
        source_section_id: "macho-section-0000".to_owned(),
        source_offset: 0,
        source_output_offset: 0,
        width_bytes: 8,
        pc_relative: false,
        external: true,
        relocation_type: 0,
        relocation_kind: kind.to_owned(),
        action_kind: "preview".to_owned(),
        target_symbol: Some("_target".to_owned()),
        target_symbol_index: Some(0),
        target_object_id: Some("object".to_owned()),
        target_section_id: Some("macho-section-0000".to_owned()),
        target_output_offset: Some(target),
        explicit_addend: None,
        pair_relocation_id: pair.map(str::to_owned),
        resolver_status: "internal".to_owned(),
        application_status: status.to_owned(),
    }
}
