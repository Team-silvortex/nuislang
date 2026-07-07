use crate::{final_stage::nsld_final_stage_plan_report, main_test_support::empty_link_plan};
use std::{fs, path::Path};

#[test]
fn final_stage_uses_plan_specific_native_object_path_for_pe_coff_alias() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-final-stage-pe-coff-native-object-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.cpu_target.machine_arch = "amd64".to_owned();
    plan.cpu_target.machine_os = "windows".to_owned();
    plan.cpu_target.object_format = "pe/coff".to_owned();
    fs::write(dir.join("nuis.nsld.pe-coff"), b"native-object").unwrap();

    let report = nsld_final_stage_plan_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    let native_object = report
        .inputs
        .iter()
        .find(|input| input.input_id == "fsi0003.native-object")
        .expect("final-stage native object input should be registered");
    assert!(native_object.path.ends_with("nuis.nsld.pe-coff"));
    assert!(native_object.present);
    assert!(report.native_object_present);
    assert!(!native_object.path.ends_with("nuis.nsld.mach-o"));
    assert!(report
        .notes
        .iter()
        .any(|note| note == "object-format:pe/coff"));
}
