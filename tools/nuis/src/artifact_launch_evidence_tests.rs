use super::*;

fn ready_prelaunch() -> crate::run_artifact::RunArtifactPrelaunchSummary {
    crate::run_artifact::RunArtifactPrelaunchSummary {
        kind: "nsld-host-entrypoint".to_owned(),
        status: "ready".to_owned(),
        evidence_status: "entrypoint-ready".to_owned(),
        command: Some("nuis-host-runner app.nsb".to_owned()),
        runner_command_present: true,
        entrypoint_path: Some("nuis.nsld.final-executable-launcher.toml".to_owned()),
        entrypoint_present: true,
        entrypoint_protocol: Some("nuis-nsld-launcher-v1".to_owned()),
        entrypoint_protocol_valid: Some(true),
        reason: "ready".to_owned(),
    }
}

fn ready_host_runner() -> HostRunnerJsonSurface {
    HostRunnerJsonSurface {
        invoked: true,
        status: "ready".to_owned(),
        program: Some("nuis-host-runner".to_owned()),
        exit_status: Some("0".to_owned()),
        error: None,
        ready: Some(true),
        would_enter_lifecycle_hook: Some(true),
        nsb_readable: Some(true),
        nsb_hash_matches: Some(true),
        nsb_payload_region_mapped: Some(true),
        nsb_payload_scan_kind: Some("nsld-container-toml".to_owned()),
        container_loader_status: Some("parsed".to_owned()),
        container_ready: Some(true),
        container_loader_entry_kind: Some("lifecycle-bootstrap".to_owned()),
        container_loader_entry_symbol: Some("main".to_owned()),
        container_loader_entry_section_id: Some("sec0000.compiled-artifact".to_owned()),
        container_loader_handoff_ready: Some(true),
        container_loader_handoff_status: Some("ready".to_owned()),
        container_loader_metadata_binding_count: Some(0),
        container_loader_metadata_binding_parsed_count: Some(0),
        container_loader_metadata_binding_table_hash: Some("0xcbf29ce484222325".to_owned()),
        container_loader_metadata_binding_validation_status: Some("not-applicable".to_owned()),
        container_loader_selected_provider_bundle_set_contract: None,
        container_loader_selected_provider_bundle_count: None,
        container_loader_selected_provider_bundle_set_hash: None,
        backend_artifact_payload_count: Some(0),
        backend_artifact_payload_parsed_count: Some(0),
        backend_artifact_payload_ready_count: Some(0),
        backend_artifact_payload_first_id: None,
        backend_artifact_payload_first_kind: None,
        backend_artifact_payload_first_role_status: None,
        backend_artifact_payload_table_hash: None,
    }
}

#[test]
fn launch_evidence_requires_host_runner_metadata_binding_proof() {
    let prelaunch = ready_prelaunch();
    let mut host_runner = ready_host_runner();
    let evidence = RunArtifactLaunchEvidence::from_surfaces(&prelaunch, &host_runner);
    let json = evidence.json_fields().join(",");
    assert!(json.contains("\"launch_evidence_status\":\"ready\""));
    assert!(json.contains("\"launch_evidence_payload_execution_trace_available\":true"));
    assert!(json.contains("\"trace_id\":\"payload-trace:container-loader:main\""));

    host_runner.container_loader_metadata_binding_count = None;
    let blocked = RunArtifactLaunchEvidence::from_surfaces(&prelaunch, &host_runner)
        .json_fields()
        .join(",");
    assert!(blocked.contains("\"launch_evidence_status\":\"blocked\""));
    assert!(blocked.contains(
        "\"launch_evidence_first_blocker\":\"container-loader-metadata-binding:count-missing\""
    ));
}

#[test]
fn launch_evidence_accepts_verified_selected_provider_binding() {
    let prelaunch = ready_prelaunch();
    let mut host_runner = ready_host_runner();
    host_runner.container_loader_metadata_binding_count = Some(1);
    host_runner.container_loader_metadata_binding_parsed_count = Some(1);
    host_runner.container_loader_metadata_binding_validation_status = Some("verified".to_owned());
    host_runner.container_loader_selected_provider_bundle_set_contract =
        Some("nuis-selected-provider-bundle-set-v1".to_owned());
    host_runner.container_loader_selected_provider_bundle_count = Some(2);
    host_runner.container_loader_selected_provider_bundle_set_hash =
        Some("fnv1a64:1234567890abcdef".to_owned());

    let evidence = RunArtifactLaunchEvidence::from_surfaces(&prelaunch, &host_runner);
    assert!(evidence
        .json_fields()
        .join(",")
        .contains("\"launch_evidence_status\":\"ready\""));

    let dir = std::env::temp_dir().join(format!(
        "nuis-launch-binding-handoff-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let persisted =
        crate::artifact_nsdb_handoff::persist_launch_evidence_nsdb_handoff(Some(&dir), &evidence);
    assert!(persisted.persisted());
    let path = dir.join("nuis.nsdb.payload-execution-handoff.toml");
    let source = std::fs::read_to_string(&path).unwrap();
    assert!(source
        .contains("final_image_binding_proof_contract = \"nuis-final-image-binding-proof-v1\""));
    assert!(source.contains("final_image_metadata_binding_count = 1"));
    assert!(source.contains(
        "final_image_selected_provider_bundle_set_contract = \
         \"nuis-selected-provider-bundle-set-v1\""
    ));
    assert!(source.contains("final_image_binding_proof_hash = \"fnv1a64:"));
    let verified = crate::artifact_nsdb_handoff::read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(verified.error(), None);
    assert!(verified
        .json_fields_with_prefix("handoff")
        .iter()
        .any(|field| {
            field == "\"handoff_final_image_binding_proof_verification_status\":\"verified\""
        }));

    std::fs::write(
        &path,
        source.replace("fnv1a64:1234567890abcdef", "fnv1a64:fedcba0987654321"),
    )
    .unwrap();
    let rejected = crate::artifact_nsdb_handoff::read_persisted_nsdb_handoff(Some(&dir));
    std::fs::remove_dir_all(dir).unwrap();
    assert_eq!(rejected.error(), Some("final-image-binding-proof-mismatch"));
}
