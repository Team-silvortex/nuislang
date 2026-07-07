use crate::{artifact_chain::nsld_artifact_chain_report, main_test_support::empty_link_plan};
use std::{fs, path::Path};

#[test]
fn artifact_chain_uses_plan_specific_object_output_file_name() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-artifact-chain-elf-object-output-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.cpu_target.object_format = "elf".to_owned();
    fs::write(dir.join("nuis.nsld.elf"), b"native-object").unwrap();

    let report = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    let object_output = report
        .stages
        .iter()
        .find(|stage| stage.stage_id == "object-output")
        .expect("object-output stage should be registered");
    assert_eq!(object_output.file_name, "nuis.nsld.elf");
    assert!(object_output.path.ends_with("nuis.nsld.elf"));
    assert!(object_output.present);
}
