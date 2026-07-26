use super::{
    json, main_test_support::empty_link_plan, nsld_artifact_chain_report, nsld_check_report,
    nsld_final_executable_output_report,
};
use std::{env, fs, path::Path};

#[test]
fn final_output_reports_pending_device_provider_sample_manifest_as_blocker() {
    let dir = temp_output_dir("pending");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-pending", 0, 1);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let report_json = json::nsld_final_executable_output_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.device_provider_sample_manifest_available);
    assert_eq!(
        report.device_provider_sample_manifest_status,
        "awaiting-provider-materialization"
    );
    assert_eq!(
        report.device_provider_sample_manifest_pending_record_count,
        1
    );
    assert_eq!(
        report
            .device_provider_sample_manifest_first_provider_family
            .as_deref(),
        Some("metal:apple-silicon-gpu")
    );
    assert_eq!(
        report
            .device_provider_sample_manifest_first_blocker
            .as_deref(),
        Some("device-provider-sample:metal:apple-silicon-gpu:pending:1")
    );
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "device-provider-sample:metal:apple-silicon-gpu:pending:1"));
    assert!(report_json.contains("\"device_provider_sample_manifest_available\":true"));
    assert!(report_json.contains(
        "\"device_provider_sample_manifest_status\":\"awaiting-provider-materialization\""
    ));
    assert!(report_json.contains(
        "\"device_provider_sample_manifest_first_blocker\":\"device-provider-sample:metal:apple-silicon-gpu:pending:1\""
    ));
}

#[test]
fn final_output_accepts_ready_device_provider_sample_manifest() {
    let dir = temp_output_dir("ready");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-materialized", 1, 0);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.device_provider_sample_manifest_available);
    assert_eq!(report.device_provider_sample_manifest_status, "ready");
    assert_eq!(report.device_provider_sample_manifest_ready_record_count, 1);
    assert_eq!(
        report
            .device_provider_sample_provider_bundle_registry_contract
            .as_deref(),
        Some("nuis-provider-bundle-registry-v1")
    );
    assert_eq!(
        report
            .device_provider_sample_provider_bundle_manifest_hash
            .as_deref(),
        Some("fnv1a64:08a971e5a543be2e")
    );
    assert_eq!(
        report
            .device_provider_sample_manifest_first_provider_bundle_package_id
            .as_deref(),
        Some("official.shader")
    );
    assert_eq!(
        report
            .device_provider_sample_manifest_first_provider_bundle_id
            .as_deref(),
        Some("metal.apple-silicon-gpu.bundle.v1")
    );
    assert_eq!(
        report
            .device_provider_sample_selected_provider_bundle_set_contract
            .as_deref(),
        Some("nuis-selected-provider-bundle-set-v1")
    );
    assert_eq!(
        report.device_provider_sample_selected_provider_bundle_count,
        Some(1)
    );
    assert_eq!(
        report
            .device_provider_sample_selected_provider_bundle_set_hash
            .as_deref(),
        Some("fnv1a64:5c7ac5158d84aa8b")
    );
    assert_eq!(
        report.device_provider_sample_selected_provider_bundle_set_validation_status,
        "verified"
    );
    assert_eq!(
        report.device_provider_sample_manifest_pending_record_count,
        0
    );
    assert_eq!(report.device_provider_sample_manifest_first_blocker, None);
    assert!(!report
        .blockers
        .iter()
        .any(|blocker| blocker.starts_with("device-provider-sample:")));
}

#[test]
fn final_output_rejects_provider_sample_without_bundle_provenance() {
    let dir = temp_output_dir("missing-bundle-provenance");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-materialized", 1, 0);
    let path = dir.join("nuis.nsdb.device-provider-samples.toml");
    let source = fs::read_to_string(&path).unwrap();
    let source = source
        .lines()
        .filter(|line| !line.starts_with("provider_bundle_"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, source).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(
        report.device_provider_sample_manifest_status,
        "provider-bundle-evidence-invalid"
    );
    assert!(report
        .device_provider_sample_manifest_first_blocker
        .as_deref()
        .is_some_and(|blocker| blocker.contains("provider-bundle-evidence-invalid")));
}

#[test]
fn final_output_rejects_mismatched_selected_provider_bundle_set() {
    let dir = temp_output_dir("mismatched-bundle-set");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-materialized", 1, 0);
    let path = dir.join("nuis.nsdb.device-provider-samples.toml");
    let source = fs::read_to_string(&path).unwrap().replace(
        "selected_provider_bundle_count = 1",
        "selected_provider_bundle_count = 2",
    );
    fs::write(&path, source).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(
        report.device_provider_sample_manifest_status,
        "provider-bundle-evidence-invalid"
    );
    assert_eq!(
        report.device_provider_sample_selected_provider_bundle_set_validation_status,
        "mismatch"
    );
}

#[test]
fn final_output_reports_blocked_device_provider_sample_manifest() {
    let dir = temp_output_dir("blocked");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-blocked", 0, 0);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let report_json = json::nsld_final_executable_output_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.device_provider_sample_manifest_available);
    assert_eq!(
        report.device_provider_sample_manifest_status,
        "blocked-provider-sample"
    );
    assert_eq!(
        report
            .device_provider_sample_manifest_first_blocker
            .as_deref(),
        Some("device-provider-sample:metal:apple-silicon-gpu:blocked:1:provider-sample-blocked")
    );
    assert_eq!(
        report.device_provider_sample_manifest_blocked_record_count,
        1
    );
    assert!(report.blockers.iter().any(|blocker| {
        blocker
            == "device-provider-sample:metal:apple-silicon-gpu:blocked:1:provider-sample-blocked"
    }));
    assert!(report_json
        .contains("\"device_provider_sample_manifest_status\":\"blocked-provider-sample\""));
    assert!(report_json.contains("\"device_provider_sample_manifest_blocked_record_count\":1"));
    assert!(report_json.contains(
        "\"device_provider_sample_manifest_first_blocker\":\"device-provider-sample:metal:apple-silicon-gpu:blocked:1:provider-sample-blocked\""
    ));
}

#[test]
fn check_report_exposes_device_provider_sample_manifest_summary() {
    let dir = temp_output_dir("check");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-pending", 0, 1);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.final_executable_output_device_provider_sample_manifest_available);
    assert_eq!(
        report.final_executable_output_device_provider_sample_manifest_status,
        "awaiting-provider-materialization"
    );
    assert_eq!(
        report.final_executable_output_device_provider_sample_manifest_pending_record_count,
        1
    );
    assert_eq!(
        report.final_executable_output_device_provider_sample_manifest_blocked_record_count,
        0
    );
    assert_eq!(
        report
            .final_executable_output_device_provider_sample_manifest_first_blocker
            .as_deref(),
        Some("device-provider-sample:metal:apple-silicon-gpu:pending:1")
    );
    assert!(report_json
        .contains("\"final_executable_output_device_provider_sample_manifest_available\":true"));
    assert!(report_json.contains(
        "\"final_executable_output_device_provider_sample_manifest_status\":\"awaiting-provider-materialization\""
    ));
    assert!(report_json.contains(
        "\"final_executable_output_device_provider_sample_manifest_blocked_record_count\":0"
    ));
    assert!(report_json.contains(
        "\"final_executable_output_device_provider_sample_manifest_first_blocker\":\"device-provider-sample:metal:apple-silicon-gpu:pending:1\""
    ));
}

#[test]
fn check_report_exposes_blocked_device_provider_sample_manifest_count() {
    let dir = temp_output_dir("check-blocked");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-blocked", 0, 0);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(
        report.final_executable_output_device_provider_sample_manifest_status,
        "blocked-provider-sample"
    );
    assert_eq!(
        report.final_executable_output_device_provider_sample_manifest_blocked_record_count,
        1
    );
    assert!(report_json.contains(
        "\"final_executable_output_device_provider_sample_manifest_blocked_record_count\":1"
    ));
}

#[test]
fn artifact_chain_points_provider_sample_boundary_to_nsdb_materializer() {
    let dir = temp_output_dir("chain");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-pending", 0, 1);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = json::nsld_artifact_chain_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.final_output_boundary_ready);
    assert_eq!(
        report.final_output_boundary_command_id.as_deref(),
        Some("materialize-provider-samples")
    );
    assert_eq!(
        report.final_output_boundary_command.as_deref(),
        Some("nsdb materialize-provider-samples <artifact-output-dir> --json")
    );
    assert!(report
        .final_output_boundary_command_resolved
        .as_deref()
        .is_some_and(|command| command.contains("nsdb materialize-provider-samples")));
    assert!(report
        .final_output_boundary_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("materialize provider samples")));
    assert_eq!(check.next_action_command_id.as_deref(), Some("emit-inputs"));
    assert_eq!(check.next_action_source.as_deref(), Some("required"));
    assert_eq!(
        check
            .artifact_chain_final_output_boundary_command_id
            .as_deref(),
        Some("materialize-provider-samples")
    );
    assert!(check
        .artifact_chain_final_output_boundary_command_resolved
        .as_deref()
        .is_some_and(|command| command.contains("nsdb materialize-provider-samples")));
    assert!(check.next_action_available);
    assert!(report_json
        .contains("\"final_output_boundary_command_id\":\"materialize-provider-samples\""));
    assert!(report_json.contains("nsdb materialize-provider-samples"));
}

#[test]
fn artifact_chain_points_blocked_provider_sample_boundary_to_payload_repair() {
    let dir = temp_output_dir("chain-blocked");
    fs::create_dir_all(&dir).unwrap();
    write_provider_sample_manifest(&dir, "provider-sample-blocked", 0, 0);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = json::nsld_artifact_chain_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.final_output_boundary_ready);
    assert_eq!(
        report.final_output_boundary_command_id.as_deref(),
        Some("repair-provider-output-payload")
    );
    assert_eq!(
        report.final_output_boundary_command.as_deref(),
        Some("nsdb materialize-provider-samples <artifact-output-dir> --json")
    );
    assert!(report
        .final_output_boundary_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("repair provider output payload diagnostics")));
    assert_eq!(
        check
            .artifact_chain_final_output_boundary_command_id
            .as_deref(),
        Some("repair-provider-output-payload")
    );
    assert!(check
        .artifact_chain_final_output_boundary_command_resolved
        .as_deref()
        .is_some_and(|command| command.contains("nsdb materialize-provider-samples")));
    assert!(report_json
        .contains("\"final_output_boundary_command_id\":\"repair-provider-output-payload\""));
    assert!(report_json.contains("repair provider output payload diagnostics"));
}

fn temp_output_dir(label: &str) -> std::path::PathBuf {
    env::temp_dir().join(format!(
        "nsld-provider-sample-{label}-{}",
        std::process::id()
    ))
}

fn write_provider_sample_manifest(
    output_dir: &Path,
    materialization_status: &str,
    ready_count: usize,
    pending_count: usize,
) {
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        format!(
            r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "nsdb-materialize-provider-samples"
status = "ready"
record_count = 1
ready_record_count = {ready_count}
pending_record_count = {pending_count}
provider_bundle_registry_contract = "nuis-provider-bundle-registry-v1"
provider_bundle_manifest_contract = "nuis-provider-bundle-manifest-v1"
provider_bundle_manifest_hash = "fnv1a64:08a971e5a543be2e"
provider_bundle_manifest_entry_count = 3
selected_provider_bundle_set_contract = "nuis-selected-provider-bundle-set-v1"
selected_provider_bundle_count = 1
selected_provider_bundle_set_hash = "fnv1a64:5c7ac5158d84aa8b"

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
provider_bundle_package_id = "official.shader"
provider_bundle_id = "metal.apple-silicon-gpu.bundle.v1"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "provider-execution-ready"
validation_status = "provider-execution-validated"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "metallib:pixelmagic.metallib"
materialization_status = "{materialization_status}"
materialization_detail = "mock-provider-runtime-result-materialized"
next_action = "replay-device-sample"
"#
        ),
    )
    .unwrap();
}
