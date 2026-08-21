use super::*;
use crate::{
    final_executable_elf_shell::{
        build_elf_amd64_shell_layout_plan, serialize_elf_amd64_shell_image, tests::Fixture,
        validate_elf_amd64_shell_image,
    },
    final_executable_elf_test_fixture::{
        elf_exit_program_object, elf_linux_exit_runtime_object, elf_program_object,
        elf_runtime_object, elf_unrelated_runtime_object, R_X86_64_PLT32,
    },
};
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::fs;
use std::{path::PathBuf, time::SystemTime};

struct ProbeFixture {
    bytes: Vec<u8>,
    validation: ElfAmd64ShellImageValidationReport,
    unresolved_external_symbol_count: usize,
}

#[test]
fn static_plan_only_probe_never_materializes() {
    let fixture = probe_fixture(elf_program_object(R_X86_64_PLT32), elf_runtime_object(), 0);
    let root = unique_temp_dir("nsld-elf-loader-probe-plan");

    let report = probe(&fixture, &root, false).unwrap();

    assert_eq!(report.contract, ELF_AMD64_LOADER_PROBE_CONTRACT);
    assert_eq!(report.probe_mode, "plan-only");
    assert!(report.input_eligible);
    assert!(!report.attempted);
    assert!(!report.materialized);
    assert!(!report.publication_eligible);
    let expected_blocker = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "explicit-loader-probe-apply-required"
    } else {
        "unsupported-probe-host"
    };
    assert_eq!(report.publication_blockers, [expected_blocker]);
    assert_eq!(
        report.probe_ledger_hash,
        crate::fnv1a64_hex(report.canonical_ledger().as_bytes())
    );
    assert!(!root.exists());
}

#[test]
fn external_boundary_is_rejected_before_filesystem_access() {
    let fixture = probe_fixture(
        elf_program_object(R_X86_64_PLT32),
        elf_unrelated_runtime_object(),
        1,
    );
    let root = unique_temp_dir("nsld-elf-loader-probe-external");

    let report = probe(&fixture, &root, true).unwrap();

    assert_eq!(report.status, "blocked-external-compatibility-input");
    assert!(!report.input_eligible);
    assert!(!report.attempted);
    assert!(!report.materialized);
    assert_eq!(report.dynamic_segment_count, 1);
    assert_eq!(report.dynamic_entry_count, 12);
    assert_eq!(
        report.publication_blockers,
        ["private-image-has-external-compatibility-bindings"]
    );
    assert!(!root.exists());
}

#[test]
fn private_image_drift_fails_before_filesystem_access() {
    let mut fixture = probe_fixture(elf_program_object(R_X86_64_PLT32), elf_runtime_object(), 0);
    let root = unique_temp_dir("nsld-elf-loader-probe-image-drift");
    fixture.bytes[0] ^= 1;

    let error = probe(&fixture, &root, false).unwrap_err();

    assert!(error.contains("private image drift"), "{error}");
    assert!(!root.exists());
}

#[test]
fn validation_ledger_drift_fails_before_filesystem_access() {
    let mut fixture = probe_fixture(elf_program_object(R_X86_64_PLT32), elf_runtime_object(), 0);
    let root = unique_temp_dir("nsld-elf-loader-probe-ledger-drift");
    fixture.validation.validation_ledger_hash.push('0');

    let error = probe(&fixture, &root, false).unwrap_err();

    assert!(error.contains("validation report ledger drift"), "{error}");
    assert!(!root.exists());
}

#[test]
fn validation_table_audit_drift_cannot_hide_behind_a_refreshed_ledger() {
    let mut fixture = probe_fixture(elf_program_object(R_X86_64_PLT32), elf_runtime_object(), 0);
    let root = unique_temp_dir("nsld-elf-loader-probe-table-drift");
    fixture.validation.tables[0].bytes_hash.push('0');
    fixture.validation.validation_ledger_hash =
        crate::fnv1a64_hex(fixture.validation.canonical_ledger().as_bytes());

    let error = probe(&fixture, &root, false).unwrap_err();

    assert!(error.contains("validation report table 0 drift"), "{error}");
    assert!(!root.exists());
}

#[test]
fn linux_exit_fixture_closes_static_plan_without_execution() {
    let fixture = probe_fixture(
        elf_exit_program_object(),
        elf_linux_exit_runtime_object(),
        0,
    );
    let root = unique_temp_dir("nsld-elf-loader-probe-exit-plan");

    let report = probe(&fixture, &root, false).unwrap();

    assert!(report.input_eligible);
    assert_eq!(report.dynamic_segment_count, 0);
    assert_eq!(report.dynamic_entry_count, 0);
    assert!(!report.attempted);
    assert!(!report.materialized);
    assert!(!root.exists());
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn real_linux_loader_executes_cross_object_static_image_and_cleans_up() {
    let fixture = probe_fixture(
        elf_exit_program_object(),
        elf_linux_exit_runtime_object(),
        0,
    );
    let root = unique_temp_dir("nsld-elf-loader-probe-execute");
    fs::create_dir_all(&root).unwrap();

    let report = probe(&fixture, &root, true).unwrap();

    assert!(report.attempted, "{report:#?}");
    assert!(report.materialized, "{report:#?}");
    assert!(report.materialized_hash_matches, "{report:#?}");
    assert!(report.kernel_accepted, "{report:#?}");
    assert!(report.process_completed, "{report:#?}");
    assert!(!report.timed_out, "{report:#?}");
    assert_eq!(report.exit_code, Some(0), "{report:#?}");
    assert_eq!(report.termination_signal, None, "{report:#?}");
    assert_eq!(report.stdout_captured_bytes, 0, "{report:#?}");
    assert_eq!(report.stderr_captured_bytes, 0, "{report:#?}");
    assert!(report.cleanup_attempted, "{report:#?}");
    assert!(report.cleanup_succeeded, "{report:#?}");
    assert!(report.publication_eligible, "{report:#?}");
    assert!(report.publication_blockers.is_empty(), "{report:#?}");
    validate_successful_elf_amd64_loader_probe(&report).unwrap();
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    fs::remove_dir(root).unwrap();
}

fn probe_fixture(
    program: Vec<u8>,
    runtime: Vec<u8>,
    unresolved_external_symbol_count: usize,
) -> ProbeFixture {
    let fixture = Fixture::from_bytes(program, runtime);
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
    let validation =
        validate_elf_amd64_shell_image(&image.bytes, &platform, &shell, &image.report).unwrap();
    ProbeFixture {
        bytes: image.bytes,
        validation,
        unresolved_external_symbol_count,
    }
}

fn probe(
    fixture: &ProbeFixture,
    root: &std::path::Path,
    execute: bool,
) -> Result<ElfAmd64LoaderProbeReport, String> {
    probe_elf_amd64_private_shell_image(
        ElfAmd64LoaderProbeInput {
            bytes: &fixture.bytes,
            validation: &fixture.validation,
            unresolved_external_symbol_count: fixture.unresolved_external_symbol_count,
        },
        root,
        execute,
    )
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
