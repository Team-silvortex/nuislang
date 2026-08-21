use super::{tests::Fixture, *};
use crate::{
    final_executable_elf_materialization::application::platform::application::ElfAmd64PlatformAppliedImage,
    final_executable_elf_test_fixture::{
        elf_runtime_object, elf_runtime_object_with_bss, elf_unrelated_runtime_object,
    },
};

struct ValidationFixture {
    platform: ElfAmd64PlatformAppliedImage,
    shell: ElfAmd64ShellLayoutPlanReport,
    image: super::image::ElfAmd64SerializedShellImage,
}

#[test]
fn independently_validates_static_private_image() {
    let fixture = validation_fixture(elf_runtime_object());

    let report = validate_elf_amd64_shell_image(
        &fixture.image.bytes,
        &fixture.platform,
        &fixture.shell,
        &fixture.image.report,
    )
    .unwrap();

    assert_eq!(
        report.contract,
        super::validation::ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT
    );
    assert_eq!(report.status, "independently-validated-private-image");
    assert!(report.header_valid);
    assert_eq!(report.expected_table_count, 4);
    assert_eq!(report.verified_table_count, 4);
    assert_eq!(report.program_header_count, 3);
    assert_eq!(report.load_segment_count, 2);
    assert_eq!(report.dynamic_segment_count, 0);
    assert_eq!(report.dynamic_entry_count, 0);
    assert_eq!(report.section_header_count, 3);
    assert_eq!(report.section_name_count, 3);
    assert_eq!(report.expected_source_validation_count, 1);
    assert_eq!(report.verified_source_validation_count, 1);
    assert_eq!(
        report.publication_eligibility_contract,
        super::validation::ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT
    );
    assert_eq!(
        report.publication_eligibility_status,
        "blocked-os-loader-probe-pending"
    );
    assert!(!report.publication_eligible);
    assert_eq!(report.publication_blockers, ["os-loader-probe-pending"]);
    assert_eq!(
        report.validation_ledger_hash,
        crate::fnv1a64_hex(report.canonical_ledger().as_bytes())
    );
    assert!(report
        .tables
        .iter()
        .all(|table| table.status == "parsed-and-write-audit-verified"));
    assert!(report
        .sources
        .iter()
        .all(|source| source.status == "independently-preserved"));
}

#[test]
fn validates_dynamic_image_but_keeps_external_boundary_blocked() {
    let fixture = validation_fixture(elf_unrelated_runtime_object());

    let report = validate_elf_amd64_shell_image(
        &fixture.image.bytes,
        &fixture.platform,
        &fixture.shell,
        &fixture.image.report,
    )
    .unwrap();

    assert_eq!(report.expected_table_count, 5);
    assert_eq!(report.verified_table_count, 5);
    assert_eq!(report.dynamic_segment_count, 1);
    assert_eq!(report.dynamic_entry_count, 12);
    assert_eq!(report.section_header_count, 9);
    assert_eq!(report.verified_source_validation_count, 6);
    assert_eq!(
        report.publication_eligibility_status,
        "blocked-os-loader-and-external-resolution-pending"
    );
    assert_eq!(
        report.publication_blockers,
        [
            "os-loader-probe-pending",
            "registered-external-resolution-provenance-pending"
        ]
    );
    assert!(report
        .tables
        .iter()
        .any(|table| table.table_kind == "dynamic-table" && table.verified_record_count == 12));
}

#[test]
fn independently_validates_nobits_source_semantics() {
    let fixture = validation_fixture(elf_runtime_object_with_bss());

    let report = validate_elf_amd64_shell_image(
        &fixture.image.bytes,
        &fixture.platform,
        &fixture.shell,
        &fixture.image.report,
    )
    .unwrap();

    let bss = report
        .sources
        .iter()
        .find(|source| source.section_name == ".bss")
        .unwrap();
    assert_eq!(bss.preservation_kind, "nobits-zero-fill-span");
    assert_eq!(bss.source_size_bytes, 32);
    assert_eq!(bss.result_file_offset, None);
    assert_eq!(bss.source_bytes_hash, bss.result_bytes_hash);
}

#[test]
fn rejects_header_identity_tamper_before_ledger_fallback() {
    let mut fixture = validation_fixture(elf_runtime_object());
    fixture.image.bytes[18] ^= 1;

    let error = validate_fixture(&fixture).unwrap_err();

    assert!(error.contains("executable header identity"), "{error}");
}

#[test]
fn rejects_dynamic_record_tamper_during_parsing() {
    let mut fixture = validation_fixture(elf_unrelated_runtime_object());
    fixture.image.bytes[fixture.shell.dynamic_table_file_offset.unwrap()] ^= 1;

    let error = validate_fixture(&fixture).unwrap_err();

    assert!(error.contains("dynamic entry 0"), "{error}");
}

#[test]
fn rejects_invalid_utf8_section_name_during_parsing() {
    let mut fixture = validation_fixture(elf_runtime_object());
    let text = fixture
        .shell
        .sections
        .iter()
        .find(|section| section.section_name == ".text")
        .unwrap();
    let offset = fixture.shell.section_name_table_file_offset + text.section_name_offset;
    fixture.image.bytes[offset] = 0xff;

    let error = validate_fixture(&fixture).unwrap_err();

    assert!(error.contains("section name is not UTF-8"), "{error}");
}

#[test]
fn rejects_unexplained_platform_prefix_change() {
    let mut fixture = validation_fixture(elf_runtime_object());
    fixture.image.bytes[0x800] = 1;

    let error = validate_fixture(&fixture).unwrap_err();

    assert!(
        error.contains("unexplained platform-prefix changes"),
        "{error}"
    );
}

#[test]
fn rejects_unexplained_zero_tail_change() {
    let mut fixture = validation_fixture(elf_runtime_object());
    let offset = (fixture.platform.report.applied_file_span_bytes..fixture.image.bytes.len())
        .find(|offset| {
            !fixture.image.report.writes.iter().any(|write| {
                (write.file_offset..write.file_offset + write.width_bytes).contains(offset)
            })
        })
        .expect("static shell fixture should contain reserved zero tail bytes");
    fixture.image.bytes[offset] = 1;

    let error = validate_fixture(&fixture).unwrap_err();

    assert!(error.contains("zero-tail bytes"), "{error}");
}

#[test]
fn rejects_serialization_write_audit_tamper() {
    let mut fixture = validation_fixture(elf_runtime_object());
    fixture.image.report.writes[0].audit_hash.push('0');

    let error = validate_fixture(&fixture).unwrap_err();

    assert!(error.contains("rejects write audit"), "{error}");
}

#[test]
fn rejects_serialization_source_audit_tamper() {
    let mut fixture = validation_fixture(elf_runtime_object());
    fixture.image.report.source_preservations[0]
        .audit_hash
        .push('0');

    let error = validate_fixture(&fixture).unwrap_err();

    assert!(error.contains("rejects source preservation"), "{error}");
}

fn validation_fixture(runtime: Vec<u8>) -> ValidationFixture {
    let fixture = Fixture::new(runtime);
    let (placement, relocations, platform_plan, platform) = fixture.pipeline();
    let shell = build_elf_amd64_shell_layout_plan(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform,
    )
    .unwrap();
    let image = serialize_elf_amd64_shell_image(
        &fixture.objects,
        &placement,
        &relocations,
        &platform_plan,
        &platform,
        &shell,
    )
    .unwrap();
    ValidationFixture {
        platform,
        shell,
        image,
    }
}

fn validate_fixture(
    fixture: &ValidationFixture,
) -> Result<ElfAmd64ShellImageValidationReport, String> {
    validate_elf_amd64_shell_image(
        &fixture.image.bytes,
        &fixture.platform,
        &fixture.shell,
        &fixture.image.report,
    )
}
