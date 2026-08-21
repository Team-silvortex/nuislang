use super::*;
use crate::{
    final_executable_elf_input::parse_elf64_amd64_object_linkage,
    final_executable_elf_layout::build_elf_amd64_placement_binding,
    final_executable_elf_test_fixture::{elf_program_object, elf_runtime_object, R_X86_64_PLT32},
};

#[test]
fn previews_real_cross_object_plt32_call() {
    let objects = fixture_objects();
    let placement = build_elf_amd64_placement_binding(&objects).unwrap();

    let report = build_elf_amd64_relocation_application(&objects, &placement).unwrap();

    assert_eq!(report.contract, ELF_AMD64_RELOCATION_APPLICATION_CONTRACT);
    assert_eq!(report.status, "ready-for-byte-preview");
    assert_eq!(report.placement_plan_hash, placement.plan_hash);
    assert_eq!(report.relocation_count, 1);
    assert_eq!(report.registered_kind_count, 1);
    assert_eq!(report.direct_preview_count, 1);
    assert_eq!(report.platform_structure_count, 0);
    let application = &report.applications[0];
    assert_eq!(application.relocation_id, "elf-amd64-reloc-000000");
    assert_eq!(application.source_offset, 1);
    assert_eq!(application.source_file_offset, 0x1001);
    assert_eq!(application.source_image_offset, 0x1001);
    assert_eq!(application.source_virtual_address, 0x401001);
    assert_eq!(application.relocation_kind, "x86_64-plt32");
    assert_eq!(
        application.target_symbol.as_deref(),
        Some("nuis_runtime_entry")
    );
    assert_eq!(
        application.target_object_id.as_deref(),
        Some("host.runtime-shim")
    );
    assert_eq!(application.target_virtual_address, Some(0x401010));
    assert_eq!(application.addend, -4);
    assert_eq!(application.computed_value, Some(11));
    assert_eq!(application.encoded_value, Some(11));
    assert_eq!(application.encoded_bytes, [0x0b, 0, 0, 0]);
    assert_eq!(application.application_status, "planned-direct");
}

#[test]
fn registered_shapes_produce_checked_little_endian_previews() {
    let cases = [
        (R_X86_64_NONE, 0, false, "x86_64-none", None, Vec::new()),
        (
            R_X86_64_64,
            8,
            false,
            "x86_64-64",
            Some(0x401010),
            vec![0x10, 0x10, 0x40, 0, 0, 0, 0, 0],
        ),
        (
            R_X86_64_PC32,
            4,
            true,
            "x86_64-pc32",
            Some(16),
            vec![0x10, 0, 0, 0],
        ),
        (
            R_X86_64_PLT32,
            4,
            true,
            "x86_64-plt32",
            Some(16),
            vec![0x10, 0, 0, 0],
        ),
        (
            R_X86_64_32,
            4,
            false,
            "x86_64-32",
            Some(0x401010),
            vec![0x10, 0x10, 0x40, 0],
        ),
        (
            R_X86_64_32S,
            4,
            false,
            "x86_64-32s",
            Some(0x401010),
            vec![0x10, 0x10, 0x40, 0],
        ),
    ];

    for (relocation_type, width, pc_relative, kind, computed, encoded_bytes) in cases {
        let mut objects = fixture_objects();
        let relocation = &mut objects[0].linkage.relocations[0];
        relocation.relocation_type = relocation_type;
        relocation.width_bytes = width;
        relocation.pc_relative = pc_relative;
        relocation.offset = 0;
        relocation.addend = 0;
        if relocation_type == R_X86_64_NONE {
            relocation.symbol_index = 0;
        }
        if width == 8 {
            objects[0].linkage.sections[0].size = 16;
            objects[0].linkage.symbols[1].size = 16;
        }
        let placement = build_elf_amd64_placement_binding(&objects).unwrap();

        let report = build_elf_amd64_relocation_application(&objects, &placement).unwrap();
        let application = &report.applications[0];

        assert_eq!(application.relocation_kind, kind);
        assert_eq!(application.computed_value, computed);
        assert_eq!(application.encoded_bytes, encoded_bytes);
        assert_eq!(
            application.application_status,
            if relocation_type == R_X86_64_NONE {
                "no-op"
            } else {
                "planned-direct"
            }
        );
    }
}

#[test]
fn unresolved_system_target_remains_a_platform_structure_boundary() {
    let objects = vec![fixture_objects().into_iter().next().unwrap()];
    let placement = build_elf_amd64_placement_binding(&objects).unwrap();

    let report = build_elf_amd64_relocation_application(&objects, &placement).unwrap();

    assert_eq!(
        report.status,
        "preview-ready-with-platform-structure-boundary"
    );
    assert_eq!(report.direct_preview_count, 0);
    assert_eq!(report.platform_structure_count, 1);
    let application = &report.applications[0];
    assert_eq!(application.resolver_status, "external-compatibility");
    assert_eq!(application.application_status, "planned-platform-structure");
    assert_eq!(application.computed_value, None);
    assert!(application.encoded_bytes.is_empty());
}

#[test]
fn rejects_unsigned_and_signed_32_bit_overflow() {
    for (relocation_type, expected) in [
        (R_X86_64_32, "unsigned 32-bit relocation value"),
        (R_X86_64_32S, "signed 32-bit relocation value"),
    ] {
        let mut objects = fixture_objects();
        let relocation = &mut objects[0].linkage.relocations[0];
        relocation.relocation_type = relocation_type;
        relocation.width_bytes = 4;
        relocation.pc_relative = false;
        relocation.offset = 0;
        relocation.addend = 0;
        let target = &mut objects[1].linkage.symbols[1];
        target.section_index = None;
        target.absolute = true;
        target.value = 0x1_0000_0000;
        target.size = 0;
        let placement = build_elf_amd64_placement_binding(&objects).unwrap();

        let error = build_elf_amd64_relocation_application(&objects, &placement).unwrap_err();

        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn rejects_pc_relative_displacement_outside_i32() {
    let mut objects = fixture_objects();
    let relocation = &mut objects[0].linkage.relocations[0];
    relocation.relocation_type = R_X86_64_PC32;
    relocation.width_bytes = 4;
    relocation.pc_relative = true;
    relocation.offset = 0;
    relocation.addend = 0;
    let target = &mut objects[1].linkage.symbols[1];
    target.section_index = None;
    target.absolute = true;
    target.value = u64::MAX;
    target.size = 0;
    let placement = build_elf_amd64_placement_binding(&objects).unwrap();

    let error = build_elf_amd64_relocation_application(&objects, &placement).unwrap_err();

    assert!(error.contains("signed 32-bit relocation value"));
}

#[test]
fn placement_hash_drift_fails_before_relocation_planning() {
    let objects = fixture_objects();
    let mut placement = build_elf_amd64_placement_binding(&objects).unwrap();
    placement.plan_hash.push('0');

    let error = build_elf_amd64_relocation_application(&objects, &placement).unwrap_err();

    assert!(error.contains("placement hash mismatch"));
}

#[test]
fn relocation_plan_is_independent_of_input_object_order() {
    let mut objects = fixture_objects();
    let placement = build_elf_amd64_placement_binding(&objects).unwrap();
    let forward = build_elf_amd64_relocation_application(&objects, &placement).unwrap();
    objects.reverse();
    let reverse_placement = build_elf_amd64_placement_binding(&objects).unwrap();

    let reverse = build_elf_amd64_relocation_application(&objects, &reverse_placement).unwrap();

    assert_eq!(reverse.plan_hash, forward.plan_hash);
    assert_eq!(reverse.applications, forward.applications);
}

fn fixture_objects() -> Vec<ElfAmd64ObjectLinkage> {
    vec![
        ElfAmd64ObjectLinkage {
            object_id: "host.program-llvm".to_owned(),
            role: "program-llvm".to_owned(),
            linkage: parse_elf64_amd64_object_linkage(&elf_program_object(R_X86_64_PLT32)).unwrap(),
        },
        ElfAmd64ObjectLinkage {
            object_id: "host.runtime-shim".to_owned(),
            role: "runtime-shim".to_owned(),
            linkage: parse_elf64_amd64_object_linkage(&elf_runtime_object()).unwrap(),
        },
    ]
}
