use super::*;

#[test]
fn blocks_native_entry_before_mapping_when_target_arch_differs_from_host() {
    let host_arch = nuis_runtime::native_host_machine_arch().expect("supported test host");
    let other_arch = match host_arch {
        nuis_runtime::NUIS_MACHINE_ARCH_AARCH64 => nuis_runtime::NUIS_MACHINE_ARCH_X86_64,
        nuis_runtime::NUIS_MACHINE_ARCH_X86_64 => nuis_runtime::NUIS_MACHINE_ARCH_AARCH64,
        _ => unreachable!("supported host architecture"),
    };
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace(
            &format!("loader_entry_machine_arch = \"{host_arch}\""),
            &format!("loader_entry_machine_arch = \"{other_arch}\""),
        )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-native-entry-arch-drift-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(report.nsb_hash_matches);
    assert!(report.container_loader_handoff_ready);
    assert!(!report.ready);
    assert!(!report.native_entry_handoff.preparation_ready);
    assert_eq!(report.native_entry_handoff.mapping_size_bytes, 0);
    assert_eq!(
        report.native_entry_handoff.target_machine_arch.as_deref(),
        Some(other_arch)
    );
    assert_eq!(
        report.native_entry_handoff.machine_arch_status,
        "host-mismatch"
    );
    assert!(report
        .blockers
        .contains(&"executable-memory:host-machine-arch-mismatch".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_native_entry_when_final_image_code_bytes_drift() {
    let mut bytes = nsb_bytes();
    let code_offset = IMAGE_HEADER_SIZE + container_capsule().len().next_multiple_of(16) + 8;
    bytes[code_offset] ^= 0x01;

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-native-entry-code-drift-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(report.nsb_hash_matches);
    assert!(report.container_loader_handoff_ready);
    assert!(!report.ready);
    assert!(!report.would_enter_lifecycle_hook);
    assert_eq!(report.native_entry_handoff.status, "blocked");
    assert_eq!(report.native_entry_handoff.code_hash_status, "not-verified");
    assert_eq!(
        report.native_entry_handoff.preparation_status,
        "not-attempted"
    );
    assert!(!report.native_entry_handoff.preparation_ready);
    assert_eq!(report.native_entry_handoff.mapping_size_bytes, 0);
    assert_eq!(report.native_entry_handoff.invocation_status, "not-invoked");
    assert!(report
        .blockers
        .contains(&"native-entry:section-hash-mismatch".to_owned()));
    assert!(report
        .blockers
        .contains(&"native-entry:code-hash-mismatch".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_handoff_when_container_blockers_are_declared() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace("\nblockers = []", "\nblockers = [\"payload-not-sealed\"]")
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-container-blocker-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert_eq!(
        report.container_blockers,
        vec!["payload-not-sealed".to_owned()]
    );
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container:blocker:payload-not-sealed".to_owned()));
    assert!(report
        .blockers
        .contains(&"container:blocker:payload-not-sealed".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_loader_handoff_when_entry_kind_mismatches_symbol_kind() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace(
            "loader_entry_kind = \"lifecycle-bootstrap\"",
            "loader_entry_kind = \"host-entry-bootstrap\"",
        )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-entry-kind-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert_eq!(
        report.container_loader_entry_kind.as_deref(),
        Some("host-entry-bootstrap")
    );
    assert_eq!(
        report.container_loader_symbol_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container-loader:entry-kind-mismatch".to_owned()));
    assert!(report
        .blockers
        .contains(&"container-loader:entry-kind-mismatch".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}
