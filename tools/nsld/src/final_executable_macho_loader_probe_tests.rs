use super::*;
use crate::{
    final_executable_macho_shell::tests::{
        build_shell, loader_probe_shell_fixture, shell_fixture, ShellFixture,
    },
    final_executable_macho_shell_image::{
        serialize_macho_arm64_shell_image, MachOArm64SerializedShellImage,
    },
};
use std::{fs, path::PathBuf, time::SystemTime};

#[test]
fn plan_only_probe_requires_explicit_apply_without_materializing() {
    let fixture = loader_probe_shell_fixture();
    let image = serialize(&fixture);
    let root = unique_temp_dir("nsld-loader-probe-plan");
    fs::create_dir_all(&root).unwrap();

    let report = probe(&image, &fixture, &root, false).unwrap();
    let host_supported = cfg!(all(target_os = "macos", target_arch = "aarch64"));

    assert_eq!(report.contract, MACHO_ARM64_LOADER_PROBE_CONTRACT);
    assert_eq!(report.host_supported, host_supported);
    assert_eq!(
        report.status,
        if host_supported {
            "ready-explicit-apply-required"
        } else {
            "blocked-unsupported-probe-host"
        }
    );
    assert!(report.input_eligible);
    assert!(!report.attempted);
    assert!(!report.materialized);
    assert!(report.cleanup_succeeded);
    assert!(!report.publication_eligible);
    assert_eq!(
        report.publication_blockers,
        [if host_supported {
            "explicit-loader-probe-apply-required"
        } else {
            "unsupported-probe-host"
        }]
    );
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    fs::remove_dir(root).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn isolated_probe_is_accepted_by_the_os_loader_and_cleans_up() {
    let fixture = loader_probe_shell_fixture();
    let image = serialize(&fixture);
    let shell = build_shell(&fixture).unwrap();
    let root = unique_temp_dir("nsld-loader-probe-execute");
    fs::create_dir_all(&root).unwrap();

    assert!(shell
        .load_commands
        .iter()
        .any(|command| command.command_kind == "load-dylib"));

    let report = probe(&image, &fixture, &root, true).unwrap();

    assert!(report.attempted, "{report:#?}");
    assert!(report.materialized, "{report:#?}");
    assert!(report.materialized_hash_matches, "{report:#?}");
    assert!(report.kernel_accepted, "{report:#?}");
    assert!(report.process_completed, "{report:#?}");
    assert!(!report.timed_out, "{report:#?}");
    assert_eq!(report.exit_code, Some(0), "{report:#?}");
    assert_eq!(report.stdout_captured_bytes, 0, "{report:#?}");
    assert_eq!(report.stderr_captured_bytes, 0, "{report:#?}");
    assert!(report.cleanup_attempted, "{report:#?}");
    assert!(report.cleanup_succeeded, "{report:#?}");
    assert!(report.publication_eligible, "{report:#?}");
    assert!(report.publication_blockers.is_empty(), "{report:#?}");
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    fs::remove_dir(root).unwrap();
}

#[test]
fn probe_rejects_external_compatibility_input_before_materialization() {
    let fixture = shell_fixture(Some("_nuis_entry"), false);
    let image = serialize(&fixture);
    let root = unique_temp_dir("nsld-loader-probe-external");
    fs::create_dir_all(&root).unwrap();

    let report = probe(&image, &fixture, &root, true).unwrap();

    assert_eq!(report.status, "blocked-external-compatibility-input");
    assert!(!report.input_eligible);
    assert!(!report.attempted);
    assert!(!report.materialized);
    assert!(!report.publication_eligible);
    assert_eq!(
        report.publication_blockers,
        ["private-image-has-external-compatibility-bindings"]
    );
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    fs::remove_dir(root).unwrap();
}

#[test]
fn probe_rejects_private_image_byte_drift() {
    let fixture = loader_probe_shell_fixture();
    let mut image = serialize(&fixture);
    let root = unique_temp_dir("nsld-loader-probe-drift");
    fs::create_dir_all(&root).unwrap();
    image.bytes[image.report.header_bytes] ^= 1;

    let error = probe(&image, &fixture, &root, false).unwrap_err();

    assert!(error.contains("private image drift"));
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    fs::remove_dir(root).unwrap();
}

fn probe(
    image: &MachOArm64SerializedShellImage,
    fixture: &ShellFixture,
    root: &std::path::Path,
    execute: bool,
) -> Result<crate::reports::NsldMachOArm64LoaderProbeReport, String> {
    probe_macho_arm64_signed_shell_image(
        MachOArm64LoaderProbeInput {
            bytes: &image.bytes,
            serialization: &image.report,
            unresolved_external_symbol_count: fixture.program.undefined_symbol_count
                + fixture.runtime.undefined_symbol_count,
            bind_count: build_shell(fixture).unwrap().binds.len(),
        },
        root,
        execute,
    )
}

fn serialize(fixture: &ShellFixture) -> MachOArm64SerializedShellImage {
    let shell = build_shell(fixture).unwrap();
    serialize_macho_arm64_shell_image(
        &fixture.relocations,
        &fixture.preview,
        &fixture.platform,
        &fixture.applied,
        &shell,
    )
    .unwrap()
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
