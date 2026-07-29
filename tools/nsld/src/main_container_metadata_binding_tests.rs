use super::{
    main_test_support::empty_link_plan, nsld_container_report, nsld_emit_container_report,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_emit_final_stage_plan_report, nsld_final_executable_output_report, nsld_prepare_report,
    nsld_verify_container_report,
};
use std::{env, fs, path::Path};

const SELECTED_SET_HASH: &str = "fnv1a64:5c7ac5158d84aa8b";
const VULKAN_SELECTED_SET_HASH: &str = "fnv1a64:f8efa211643f7bcd";

#[test]
fn container_immutably_binds_verified_selected_provider_bundle_set() {
    let dir = temp_dir("verified");
    fs::create_dir_all(&dir).unwrap();
    let plan = plan_with_artifact(&dir);
    let unbound = nsld_container_report(Path::new("manifest.toml"), &plan);
    write_provider_sample_manifest(&dir, 1, SELECTED_SET_HASH);

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emitted = nsld_emit_container_report(Path::new("manifest.toml"), &plan).unwrap();
    let bound = nsld_container_report(Path::new("manifest.toml"), &plan);
    let source = fs::read_to_string(&emitted.output_path).unwrap();
    let verified = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(&dir).unwrap();

    assert!(bound
        .blockers
        .iter()
        .all(|blocker| !blocker.starts_with("metadata-binding:")));
    assert_eq!(bound.metadata_bindings.len(), 2);
    assert_eq!(
        bound.metadata_bindings[0].binding_id,
        "identity.selected-provider-bundle-set"
    );
    assert_eq!(
        bound.metadata_bindings[0].contract,
        "nuis-selected-provider-bundle-set-v1"
    );
    assert_eq!(bound.metadata_bindings[0].value_count, 1);
    assert_eq!(bound.metadata_bindings[0].value_hash, SELECTED_SET_HASH);
    assert_eq!(bound.metadata_bindings[0].validation_status, "verified");
    assert_eq!(
        bound.metadata_bindings[1].binding_id,
        "runtime.provider-dispatch-table"
    );
    assert_eq!(
        bound.metadata_bindings[1].contract,
        "nuis-final-image-provider-dispatch-v1"
    );
    assert_eq!(bound.provider_dispatch_validation_status, "verified");
    assert_eq!(bound.provider_dispatches.len(), 1);
    assert_eq!(
        bound.provider_dispatches[0].bundle_id,
        "metal.apple-silicon-gpu.bundle.v1"
    );
    assert_ne!(bound.metadata_table_hash, unbound.metadata_table_hash);
    assert_ne!(bound.container_hash, unbound.container_hash);
    assert_eq!(emitted.metadata_binding_count, 2);
    assert_eq!(
        emitted.metadata_binding_table_hash,
        bound.metadata_binding_table_hash
    );
    assert!(source.contains("[[metadata_binding]]"));
    assert!(source.contains("binding_id = \"identity.selected-provider-bundle-set\""));
    assert!(source.contains(&format!("value_hash = \"{SELECTED_SET_HASH}\"")));
    assert!(verified.valid, "{:?}", verified.issues);
}

#[test]
fn container_emit_rejects_mismatched_selected_provider_bundle_set() {
    let dir = temp_dir("mismatch");
    fs::create_dir_all(&dir).unwrap();
    let plan = plan_with_artifact(&dir);
    write_provider_sample_manifest(&dir, 2, SELECTED_SET_HASH);

    let preview = nsld_container_report(Path::new("manifest.toml"), &plan);
    let error = nsld_emit_container_report(Path::new("manifest.toml"), &plan).unwrap_err();
    let container_path = dir.join("nuis.nsld.container");
    let payload_path = dir.join("nuis.nsld.container.payload");
    let container_present = container_path.exists();
    let payload_present = payload_path.exists();
    fs::remove_dir_all(&dir).unwrap();

    assert!(!preview.ready);
    assert!(preview.metadata_bindings.is_empty());
    assert!(preview
        .blockers
        .iter()
        .any(|blocker| blocker.contains("selected-provider-bundle-set")));
    assert!(error.contains("invalid immutable metadata binding"));
    assert!(!container_present);
    assert!(!payload_present);
}

#[test]
fn container_verify_rejects_tampered_embedded_metadata_binding() {
    let dir = temp_dir("tampered");
    fs::create_dir_all(&dir).unwrap();
    let plan = plan_with_artifact(&dir);
    write_provider_sample_manifest(&dir, 1, SELECTED_SET_HASH);
    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let path = dir.join("nuis.nsld.container");
    let source = fs::read_to_string(&path).unwrap().replace(
        &format!("value_hash = \"{SELECTED_SET_HASH}\""),
        "value_hash = \"fnv1a64:0000000000000000\"",
    );
    fs::write(&path, source).unwrap();

    let verified = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(&dir).unwrap();

    assert!(!verified.valid);
    assert!(verified
        .issues
        .iter()
        .any(|issue| issue == "container-content-mismatch"));
}

#[test]
fn final_image_loader_verifies_embedded_selected_provider_bundle_binding() {
    let dir = temp_dir("final-image");
    fs::create_dir_all(&dir).unwrap();
    let mut plan = plan_with_artifact(&dir);
    plan.final_stage.kind = "nuis-self-contained-image".to_owned();
    plan.final_stage.driver = "nsld-internal-image-writer".to_owned();
    plan.final_stage.link_mode = "self-contained".to_owned();
    plan.final_stage.output_path = dir.join("nuis-app.nsb").display().to_string();
    write_provider_sample_manifest(&dir, 1, SELECTED_SET_HASH);

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();

    let verified = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let verified_json = super::json::nsld_final_executable_output_report_json(&verified);
    assert!(
        verified.container_loader_handoff_ready,
        "{:?}",
        verified.container_loader_handoff_first_blocker
    );
    assert_eq!(
        verified.container_loader_metadata_binding_validation_status,
        "verified"
    );
    assert_eq!(verified.container_loader_metadata_binding_count, Some(2));
    assert_eq!(
        verified
            .container_loader_selected_provider_bundle_set_contract
            .as_deref(),
        Some("nuis-selected-provider-bundle-set-v1")
    );
    assert_eq!(
        verified.container_loader_selected_provider_bundle_count,
        Some(1)
    );
    assert_eq!(
        verified
            .container_loader_selected_provider_bundle_set_hash
            .as_deref(),
        Some(SELECTED_SET_HASH)
    );
    assert_eq!(
        verified.container_loader_provider_dispatch_status,
        "verified"
    );
    assert_eq!(verified.container_loader_provider_dispatch_count, 1);
    assert_eq!(
        verified
            .container_loader_provider_dispatch_first_bundle_id
            .as_deref(),
        Some("metal.apple-silicon-gpu.bundle.v1")
    );
    assert!(verified_json
        .contains("\"container_loader_metadata_binding_validation_status\":\"verified\""));
    assert!(verified_json.contains("\"container_loader_metadata_binding_count\":2"));
    assert!(verified_json.contains("\"container_loader_provider_dispatch_status\":\"verified\""));
    assert!(verified_json.contains("\"container_loader_provider_dispatch_count\":1"));
    assert!(verified_json.contains(
        "\"container_loader_selected_provider_bundle_set_contract\":\"nuis-selected-provider-bundle-set-v1\""
    ));
    assert!(verified_json.contains(&format!(
        "\"container_loader_selected_provider_bundle_set_hash\":\"{SELECTED_SET_HASH}\""
    )));

    let original = fs::read(&plan.final_stage.output_path).unwrap();
    let mut dispatch_tampered = original.clone();
    replace_bytes_once(
        &mut dispatch_tampered,
        b"runner_adapter_id = \"metal-gray8-invert\"",
        b"runner_adapter_id = \"metal-gray8-driftt\"",
    );
    fs::write(&plan.final_stage.output_path, dispatch_tampered).unwrap();
    let dispatch_rejected = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    assert!(!dispatch_rejected.container_loader_handoff_ready);
    assert_eq!(
        dispatch_rejected.container_loader_provider_dispatch_status,
        "mismatch"
    );
    assert!(dispatch_rejected
        .container_loader_handoff_first_blocker
        .as_deref()
        .is_some_and(|blocker| blocker.contains("provider-dispatch-table-hash-mismatch")));

    let mut tampered = original;
    replace_bytes_once(
        &mut tampered,
        format!("value_hash = \"{SELECTED_SET_HASH}\"").as_bytes(),
        b"value_hash = \"fnv1a64:0000000000000000\"",
    );
    fs::write(&plan.final_stage.output_path, tampered).unwrap();
    let rejected = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(&dir).unwrap();

    assert!(!rejected.container_loader_handoff_ready);
    assert_eq!(
        rejected.container_loader_metadata_binding_validation_status,
        "mismatch"
    );
    assert!(rejected
        .container_loader_handoff_first_blocker
        .as_deref()
        .is_some_and(|blocker| blocker.contains("metadata-binding-table-hash-mismatch")));
}

#[test]
fn final_image_loader_verifies_vulkan_provider_dispatch_without_backend_branch() {
    let dir = temp_dir("final-image-vulkan");
    fs::create_dir_all(&dir).unwrap();
    let mut plan = plan_with_artifact(&dir);
    plan.final_stage.kind = "nuis-self-contained-image".to_owned();
    plan.final_stage.driver = "nsld-internal-image-writer".to_owned();
    plan.final_stage.link_mode = "self-contained".to_owned();
    plan.final_stage.output_path = dir.join("nuis-app.nsb").display().to_string();
    write_vulkan_provider_sample_manifest(&dir);

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();

    let verified = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let verified_json = super::json::nsld_final_executable_output_report_json(&verified);
    assert!(
        verified.container_loader_handoff_ready,
        "{:?}",
        verified.container_loader_handoff_first_blocker
    );
    assert_eq!(
        verified.container_loader_metadata_binding_validation_status,
        "verified"
    );
    assert_eq!(
        verified
            .container_loader_selected_provider_bundle_set_hash
            .as_deref(),
        Some(VULKAN_SELECTED_SET_HASH)
    );
    assert_eq!(
        verified.container_loader_provider_dispatch_status,
        "verified"
    );
    assert_eq!(verified.container_loader_provider_dispatch_count, 1);
    assert_eq!(
        verified
            .container_loader_provider_dispatch_first_bundle_id
            .as_deref(),
        Some("spirv.vulkan-gpu.bundle.v1")
    );
    assert_eq!(
        verified
            .container_loader_provider_dispatch_first_provider_family
            .as_deref(),
        Some("spirv:vulkan-gpu")
    );
    assert_eq!(
        verified
            .device_provider_sample_manifest_first_provider_family
            .as_deref(),
        Some("spirv:vulkan-gpu")
    );
    assert!(verified_json.contains(
        "\"container_loader_provider_dispatch_first_provider_family\":\"spirv:vulkan-gpu\""
    ));

    let original = fs::read(&plan.final_stage.output_path).unwrap();
    let mut dispatch_tampered = original;
    replace_bytes_once(
        &mut dispatch_tampered,
        b"runner_adapter_id = \"spirv.vulkan.real-device\"",
        b"runner_adapter_id = \"spirv.vulkan.fake-device\"",
    );
    fs::write(&plan.final_stage.output_path, dispatch_tampered).unwrap();
    let rejected = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(&dir).unwrap();

    assert!(!rejected.container_loader_handoff_ready);
    assert_eq!(
        rejected.container_loader_provider_dispatch_status,
        "mismatch"
    );
    assert!(rejected
        .container_loader_handoff_first_blocker
        .as_deref()
        .is_some_and(|blocker| blocker.contains("provider-dispatch-table-hash-mismatch")));
}

fn plan_with_artifact(dir: &Path) -> nuisc::linker::LinkPlan {
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    env::temp_dir().join(format!(
        "nsld-container-metadata-binding-{label}-{}",
        std::process::id()
    ))
}

fn write_vulkan_provider_sample_manifest(output_dir: &Path) {
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        format!(
            r#"protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
record_count = 1
ready_record_count = 1
pending_record_count = 0
provider_bundle_registry_contract = "nuis-provider-bundle-registry-v1"
provider_bundle_manifest_contract = "nuis-provider-bundle-manifest-v1"
provider_bundle_manifest_hash = "fnv1a64:9831a33035211556"
provider_bundle_manifest_entry_count = 2
selected_provider_bundle_set_contract = "nuis-selected-provider-bundle-set-v1"
selected_provider_bundle_count = 1
selected_provider_bundle_set_hash = "{VULKAN_SELECTED_SET_HASH}"

[[device_provider_samples]]
provider_family = "spirv:vulkan-gpu"
provider_bundle_package_id = "official.shader"
provider_bundle_id = "spirv.vulkan-gpu.bundle.v1"
requested_runner_contract = "nuis-provider-runner-v1"
requested_runner_adapter_contract = "nuis-provider-runner-adapter-v1"
requested_runner_adapter_id = "spirv.vulkan.real-device"
materialization_status = "provider-sample-materialized"
"#
        ),
    )
    .unwrap();
}

fn write_provider_sample_manifest(output_dir: &Path, count: usize, hash: &str) {
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        format!(
            r#"protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
record_count = 1
ready_record_count = 1
pending_record_count = 0
provider_bundle_registry_contract = "nuis-provider-bundle-registry-v1"
provider_bundle_manifest_contract = "nuis-provider-bundle-manifest-v1"
provider_bundle_manifest_hash = "fnv1a64:08a971e5a543be2e"
provider_bundle_manifest_entry_count = 3
selected_provider_bundle_set_contract = "nuis-selected-provider-bundle-set-v1"
selected_provider_bundle_count = {count}
selected_provider_bundle_set_hash = "{hash}"

[[device_provider_samples]]
provider_family = "metal:apple-silicon-gpu"
provider_bundle_package_id = "official.shader"
provider_bundle_id = "metal.apple-silicon-gpu.bundle.v1"
requested_runner_contract = "nuis-provider-runner-v1"
requested_runner_adapter_contract = "nuis-provider-runner-adapter-v1"
requested_runner_adapter_id = "metal-gray8-invert"
materialization_status = "provider-sample-materialized"
"#
        ),
    )
    .unwrap();
}

fn replace_bytes_once(bytes: &mut [u8], needle: &[u8], replacement: &[u8]) {
    assert_eq!(needle.len(), replacement.len());
    let offset = bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("embedded metadata binding should be present");
    bytes[offset..offset + needle.len()].copy_from_slice(replacement);
}
