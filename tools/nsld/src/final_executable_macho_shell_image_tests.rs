use super::*;
use crate::final_executable_macho_shell::tests::{
    build_shell, internal_got_shell_fixture, shell_fixture,
};

#[test]
fn serializes_a_deterministic_private_arm64_shell_image() {
    let fixture = shell_fixture(Some("_nuis_entry"), false);
    let shell = build_shell(&fixture).unwrap();
    let first = serialize(&fixture, &shell).unwrap();
    let second = serialize(&fixture, &shell).unwrap();

    assert_eq!(first.bytes, second.bytes);
    assert_eq!(first.report, second.report);
    assert_eq!(read_u32(&first.bytes, 0), 0xfeed_facf);
    assert_eq!(read_u32(&first.bytes, 12), 2);
    assert_eq!(
        read_u32(&first.bytes, 16) as usize,
        shell.load_command_count
    );
    assert_eq!(
        read_u32(&first.bytes, 20) as usize,
        shell.load_command_size_bytes
    );
    assert_eq!(
        first.bytes.len(),
        shell.code_signature_file_offset + first.report.code_signature.signature_payload_bytes
    );
    assert_eq!(first.report.shell_image_span_bytes, first.bytes.len());
    assert_eq!(first.report.status, "signed-private-image-validated");
    assert_eq!(
        first.report.code_signature_status,
        "ad-hoc-payload-validated"
    );
    assert_eq!(first.report.publication_status, "private-not-published");
    assert_eq!(
        first.report.code_signature.validation_status,
        "signed-private-image-structurally-valid"
    );
    assert!(!first.report.code_signature.publication_eligible);
    assert_eq!(
        first.report.code_signature.publication_blockers,
        ["independent-os-load-validation-pending"]
    );
    assert_eq!(
        first.report.code_signature.code_slot_count,
        first.report.code_signature.verified_code_slot_count
    );
    assert!(first.report.code_signature.linkedit_covers_signature);
    assert!(first.report.code_signature.signed_ranges_valid);
    assert!(first.report.code_signature.padding_valid);
    assert_eq!(first.report.relocation_rewrite_count, 1);
    assert_eq!(first.report.stub_rewrite_count, 1);
    assert_eq!(first.report.got_rewrite_count, 0);
    assert_eq!(
        first.report.rewrite_count,
        shell.required_address_rewrite_count
    );
    assert_eq!(first.report.rewrites.len(), first.report.rewrite_count);
    assert!(first
        .report
        .rewrites
        .iter()
        .any(|rewrite| rewrite.rewrite_kind == "stub-final-address"));
    assert_eq!(
        crate::fnv1a64_hex(&first.bytes),
        first.report.shell_image_hash
    );

    let signature = shell
        .load_commands
        .iter()
        .find(|command| command.command_kind == "code-signature")
        .unwrap();
    assert_eq!(
        read_u32(&first.bytes, signature.command_offset + 8) as usize,
        shell.code_signature_file_offset
    );
    assert_eq!(
        read_u32(&first.bytes, signature.command_offset + 12) as usize,
        first.report.code_signature.signature_payload_bytes
    );
    assert_eq!(
        read_be_u32(&first.bytes, shell.code_signature_file_offset),
        0xfade_0cc0
    );

    let relocation = first
        .report
        .rewrites
        .iter()
        .find(|rewrite| rewrite.rewrite_kind == "relocation-final-address")
        .unwrap();
    let word = read_u32(&first.bytes, relocation.file_offset);
    let displacement = sign_extend(u64::from(word & 0x03ff_ffff), 26) << 2;
    assert_eq!(
        relocation.vm_address as i128 + i128::from(displacement),
        i128::from(relocation.target_vm_address.unwrap())
    );
    assert_ne!(first.report.bind_stream_hash, crate::fnv1a64_hex(&[]));
    assert_eq!(first.report.rebase_stream_hash, crate::fnv1a64_hex(&[]));
    assert!(first.report.serialization_ledger_hash.len() >= 16);
}

#[test]
fn serializer_rejects_platform_image_byte_drift() {
    let mut fixture = shell_fixture(Some("_nuis_entry"), false);
    let shell = build_shell(&fixture).unwrap();
    fixture.applied.bytes[0] ^= 0xff;

    let error = serialize(&fixture, &shell).unwrap_err();
    assert!(error.contains("platform image drift"));
}

#[test]
fn serializer_rejects_structure_write_encoding_drift() {
    let mut fixture = shell_fixture(Some("_nuis_entry"), false);
    let shell = build_shell(&fixture).unwrap();
    fixture.applied.report.structure_writes[0]
        .encoded_bytes_hex
        .replace_range(0..2, "ff");

    let error = serialize(&fixture, &shell).unwrap_err();
    assert!(error.contains("encoded byte drift"));
}

#[test]
fn serializes_internal_got_rebase_with_final_pointer_value() {
    let fixture = internal_got_shell_fixture();
    let shell = build_shell(&fixture).unwrap();
    let output = serialize(&fixture, &shell).unwrap();

    assert!(shell.binds.is_empty());
    assert_eq!(shell.rebases.len(), 1);
    assert_eq!(output.report.relocation_rewrite_count, 2);
    assert_eq!(output.report.stub_rewrite_count, 0);
    assert_eq!(output.report.got_rewrite_count, 1);
    assert_eq!(output.report.rewrite_count, 3);
    let rebase = &shell.rebases[0];
    assert_eq!(
        read_u64(&output.bytes, rebase.file_offset),
        rebase.target_vm_address
    );
    assert_ne!(output.report.rebase_stream_hash, crate::fnv1a64_hex(&[]));
    assert_eq!(output.report.bind_stream_hash, crate::fnv1a64_hex(&[]));
    assert!(output.report.rewrites.iter().any(|rewrite| {
        rewrite.rewrite_kind == "internal-got-final-address"
            && rewrite.target_vm_address == Some(rebase.target_vm_address)
    }));
}

#[test]
fn serializer_rejects_shell_upstream_hash_drift() {
    let fixture = shell_fixture(Some("_nuis_entry"), false);
    let mut shell = build_shell(&fixture).unwrap();
    shell.platform_image_hash.push('0');

    let error = serialize(&fixture, &shell).unwrap_err();
    assert!(error.contains("input hash drift"));
}

#[test]
fn signed_image_validator_rejects_signed_content_drift() {
    let fixture = shell_fixture(Some("_nuis_entry"), false);
    let shell = build_shell(&fixture).unwrap();
    let mut output = serialize(&fixture, &shell).unwrap();
    let plan =
        crate::final_executable_macho_shell_signature::plan_macho_arm64_ad_hoc_signature(&shell)
            .unwrap();
    output.bytes[shell.first_content_file_offset] ^= 0x01;

    let error = crate::final_executable_macho_shell_signature_validation::validate_macho_arm64_signed_shell_image(
        &output.bytes,
        &shell,
        &plan,
    )
    .unwrap_err();
    assert!(error.contains("code slot") && error.contains("digest drift"));
}

#[test]
fn signed_image_validator_rejects_load_command_boundary_drift() {
    let fixture = shell_fixture(Some("_nuis_entry"), false);
    let shell = build_shell(&fixture).unwrap();
    let mut output = serialize(&fixture, &shell).unwrap();
    let plan =
        crate::final_executable_macho_shell_signature::plan_macho_arm64_ad_hoc_signature(&shell)
            .unwrap();
    output.bytes[36..40].copy_from_slice(&0u32.to_le_bytes());

    let error = crate::final_executable_macho_shell_signature_validation::validate_macho_arm64_signed_shell_image(
        &output.bytes,
        &shell,
        &plan,
    )
    .unwrap_err();
    assert!(error.contains("invalid boundary"));
}

#[test]
fn signed_image_validator_rejects_signature_padding_drift() {
    let fixture = shell_fixture(Some("_nuis_entry"), false);
    let shell = build_shell(&fixture).unwrap();
    let mut output = serialize(&fixture, &shell).unwrap();
    let plan =
        crate::final_executable_macho_shell_signature::plan_macho_arm64_ad_hoc_signature(&shell)
            .unwrap();
    assert!(plan.signature_blob_bytes < plan.signature_payload_bytes);
    output.bytes[shell.code_signature_file_offset + plan.signature_blob_bytes] = 1;

    let error = crate::final_executable_macho_shell_signature_validation::validate_macho_arm64_signed_shell_image(
        &output.bytes,
        &shell,
        &plan,
    )
    .unwrap_err();
    assert!(error.contains("alignment padding"));
}

fn serialize(
    fixture: &crate::final_executable_macho_shell::tests::ShellFixture,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<MachOArm64SerializedShellImage, String> {
    serialize_macho_arm64_shell_image(
        &fixture.relocations,
        &fixture.preview,
        &fixture.platform,
        &fixture.applied,
        shell,
    )
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn read_be_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn sign_extend(value: u64, bits: u32) -> i64 {
    let shift = 64 - bits;
    ((value << shift) as i64) >> shift
}
