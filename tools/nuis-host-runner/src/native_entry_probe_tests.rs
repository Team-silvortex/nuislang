use super::*;

#[cfg(target_arch = "aarch64")]
const NONZERO_NATIVE_ENTRY_CODE: [u8; 8] = [0x40, 0x05, 0x80, 0xd2, 0xc0, 0x03, 0x5f, 0xd6];
#[cfg(target_arch = "x86_64")]
const NONZERO_NATIVE_ENTRY_CODE: [u8; 8] = [0xb8, 42, 0, 0, 0, 0xc3, 0x90, 0x90];

fn nsb_with_runtime_dispatch_import(provider: &str) -> Vec<u8> {
    let capsule = String::from_utf8(container_capsule())
        .expect("container capsule is utf-8")
        .replace("external_import_count = 0", "external_import_count = 1")
        .replace(
            CONTAINER_CAPSULE_END_MARKER,
            &format!(
                "\n[[external_import]]\nimport_id = \"imp0000.runtime-service-dispatch\"\nimport_kind = \"{}\"\nimport_name = \"{}\"\nprovider = \"{provider}\"\nrequired = true\n{CONTAINER_CAPSULE_END_MARKER}",
                nuis_runtime::NATIVE_RUNTIME_DISPATCH_IMPORT_KIND,
                nuis_runtime::NATIVE_RUNTIME_DISPATCH_IMPORT_NAME,
            ),
        );
    nsb_bytes_from_payload(&image_payload_from_capsule(capsule.as_bytes()))
}

#[test]
fn explicit_probe_invokes_verified_final_image_entry_once() {
    let bytes = nsb_bytes();
    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-native-entry-probe-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff_with_probe(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
        true,
    );

    assert!(report.ready, "{:?}", report.blockers);
    assert!(report.would_enter_lifecycle_hook);
    assert_eq!(report.native_entry_handoff.status, "invoked");
    assert!(report.native_entry_handoff.ready);
    assert!(report.native_entry_handoff.preparation_ready);
    assert_eq!(
        report.native_entry_handoff.context_protocol.as_deref(),
        Some(nuis_runtime::NATIVE_LIFECYCLE_ENTRY_CONTEXT_PROTOCOL)
    );
    assert_eq!(report.native_entry_handoff.context_status, "verified");
    assert_eq!(report.native_entry_handoff.context_version, Some(1));
    assert_eq!(report.native_entry_handoff.context_size_bytes, Some(64));
    assert!(report.native_entry_handoff.context_identity_hash.is_some());
    assert!(report
        .native_entry_handoff
        .context_clock_root_handle
        .is_some_and(|handle| handle != 0));
    assert!(report
        .native_entry_handoff
        .context_glm_root_handle
        .is_some_and(|handle| handle != 0));
    assert!(report.native_entry_handoff.invocation_requested);
    assert_eq!(
        report
            .native_entry_handoff
            .invocation_permit_protocol
            .as_deref(),
        Some(nuis_runtime::NATIVE_ENTRY_INVOCATION_PERMIT_PROTOCOL)
    );
    assert_eq!(
        report.native_entry_handoff.invocation_protocol.as_deref(),
        Some(nuis_runtime::NATIVE_ENTRY_INVOCATION_PROTOCOL)
    );
    assert_eq!(report.native_entry_handoff.invocation_status, "invoked");
    assert!(report.native_entry_handoff.invoked);
    assert_eq!(report.native_entry_handoff.invocation_return_value, Some(0));
    assert_eq!(
        report.native_entry_handoff.invocation_return_status,
        "verified"
    );
    assert!(report.native_entry_handoff.blockers.is_empty());
    assert!(report
        .launch_steps
        .contains(&"native-entry-invocation:invoked".to_owned()));

    let json = report::render_json_report(&report);
    assert!(json.contains("\"invocation_requested\":true"));
    assert!(json.contains("\"context_protocol\":\"nuis-runtime-lifecycle-entry-context-v1\""));
    assert!(json.contains("\"context_status\":\"verified\""));
    assert!(json.contains("\"context_version\":1"));
    assert!(json.contains("\"context_size_bytes\":64"));
    assert!(json.contains("\"context_identity_hash\":\"0x"));
    assert!(
        json.contains("\"invocation_permit_protocol\":\"nuis-native-entry-invocation-permit-v1\"")
    );
    assert!(json.contains("\"invocation_protocol\":\"nuis-native-entry-invocation-v1\""));
    assert!(json.contains("\"invocation_return_value\":0"));
    assert!(json.contains("\"invocation_return_status\":\"verified\""));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn explicit_probe_does_not_issue_permit_when_lifecycle_gate_is_blocked() {
    let bytes = nsb_bytes();
    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-native-entry-gate-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff_with_probe(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "wrong_lifecycle_hook",
        &manifest,
        true,
    );

    assert!(!report.ready);
    assert!(!report.would_enter_lifecycle_hook);
    assert!(report.native_entry_handoff.invocation_requested);
    assert_eq!(report.native_entry_handoff.context_status, "not-verified");
    assert!(report
        .native_entry_handoff
        .invocation_permit_protocol
        .is_none());
    assert!(!report.native_entry_handoff.invoked);
    assert_eq!(report.native_entry_handoff.invocation_return_value, None);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "lifecycle-hook:mismatch"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn explicit_probe_rejects_unregistered_runtime_dispatch_provider() {
    let bytes = nsb_with_runtime_dispatch_import("host-special-case");
    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-dispatch-provider-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff_with_probe(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
        true,
    );

    assert!(!report.ready);
    assert_eq!(
        report.native_entry_handoff.dispatch_resolution_status,
        "blocked"
    );
    assert!(report.native_entry_handoff.dispatch_import_declared);
    assert!(!report.native_entry_handoff.invoked);
    assert!(report
        .blockers
        .contains(&"runtime-dispatch-import:provider-unsupported".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[test]
fn explicit_probe_rejects_nonzero_bootstrap_return() {
    let mut asset = [0u8; 16];
    asset[8..].copy_from_slice(&NONZERO_NATIVE_ENTRY_CODE);
    let old_asset_hash = fnv1a64_hex(&NATIVE_ENTRY_ASSET);
    let old_code_hash = fnv1a64_hex(&NATIVE_ENTRY_ASSET[8..]);
    let capsule = String::from_utf8(container_capsule())
        .expect("container capsule is utf-8")
        .replace(&old_asset_hash, &fnv1a64_hex(&asset))
        .replace(&old_code_hash, &fnv1a64_hex(&asset[8..]));
    let mut image = asset;
    image[0] = 0x48;
    let aligned = capsule.len().next_multiple_of(16);
    let mut payload = capsule.into_bytes();
    payload.resize(aligned, 0);
    payload.extend_from_slice(&image);
    let bytes = nsb_bytes_from_payload(&payload);
    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-native-entry-mismatch-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff_with_probe(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
        true,
    );

    assert!(!report.ready);
    assert!(!report.would_enter_lifecycle_hook);
    assert!(!report.native_entry_handoff.ready);
    assert_eq!(report.native_entry_handoff.context_status, "verified");
    assert!(report.native_entry_handoff.invoked);
    assert_eq!(
        report.native_entry_handoff.invocation_return_value,
        Some(42)
    );
    assert_eq!(
        report.native_entry_handoff.invocation_return_status,
        "mismatch"
    );
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "native-entry:bootstrap-return-mismatch"));
    let _ = fs::remove_dir_all(&dir);
}
