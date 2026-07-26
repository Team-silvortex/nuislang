use super::{
    main_test_support::empty_link_plan,
    seal::{nsld_seal_report, nsld_seal_report_json},
};
use std::{env, fs};

fn temp_dir(label: &str) -> std::path::PathBuf {
    env::temp_dir().join(format!("nsld-seal-{label}-{}", std::process::id()))
}

#[test]
fn seal_rejects_host_finalization_before_mutating_artifacts() {
    let dir = temp_dir("host-finalization");
    fs::create_dir_all(&dir).unwrap();
    let manifest = dir.join("nuis.build.manifest.toml");
    fs::write(&manifest, "schema = \"test\"\n").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_seal_report(&manifest, &plan);
    let json = nsld_seal_report_json(&report);

    assert!(!report.preflight_valid);
    assert!(!report.prepare_attempted);
    assert_eq!(report.completed_stage_count, 0);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.starts_with("packaging-mode:not-self-contained:")));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.starts_with("final-link-mode:not-self-contained:")));
    assert!(json.contains("\"kind\":\"nsld_seal\""));
    assert!(json.contains("\"bounded_stage_count\":3"));
    assert!(json.contains("\"completed\":false"));
    assert!(!dir.join("nuis.nsld.prepare.toml").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn seal_rejects_incomplete_provider_manifest_before_prepare() {
    let dir = temp_dir("pending-provider");
    fs::create_dir_all(&dir).unwrap();
    let manifest = dir.join("nuis.build.manifest.toml");
    fs::write(&manifest, "schema = \"test\"\n").unwrap();
    fs::write(
        dir.join("nuis.nsdb.device-provider-samples.toml"),
        concat!(
            "protocol = \"nuis-device-provider-samples-v1\"\n",
            "schema = \"nsdb-yir-device-provider-sample-v1\"\n",
            "ready_record_count = 0\n",
            "pending_record_count = 1\n",
            "[[device_provider_samples]]\n",
            "provider_family = \"registered-test-provider\"\n",
            "materialization_status = \"pending\"\n",
        ),
    )
    .unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.packaging_mode = "nuis-self-contained-image".to_owned();
    plan.final_stage.link_mode = "self-contained".to_owned();
    plan.final_stage.output_path = dir.join("demo.nsb").display().to_string();

    let report = nsld_seal_report(&manifest, &plan);

    assert!(!report.preflight_valid);
    assert!(!report.prepare_attempted);
    assert_eq!(report.provider_record_count, 1);
    assert_eq!(report.provider_pending_record_count, 1);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.starts_with("provider-manifest:not-ready:")));
    assert!(!dir.join("nuis.nsld.prepare.toml").exists());

    fs::remove_dir_all(dir).unwrap();
}
