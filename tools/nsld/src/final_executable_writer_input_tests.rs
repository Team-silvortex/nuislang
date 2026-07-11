use crate::{
    final_executable_writer_input::nsld_emit_final_executable_writer_input_report,
    main_test_support::empty_link_plan, prepare::nsld_prepare_report,
};
use std::{fs, path::Path};

#[test]
fn final_executable_writer_input_command_args_use_pe_coff_native_object_path() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-final-executable-writer-input-pe-coff-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
    plan.cpu_target.machine_arch = "amd64".to_owned();
    plan.cpu_target.machine_os = "windows".to_owned();
    plan.cpu_target.object_format = "pe/coff".to_owned();
    fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
    fs::write(dir.join("nuis.nsld.pe-coff"), b"native-object").unwrap();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let source = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(source.contains("nuis.nsld.pe-coff"));
    assert!(!source.contains("nuis.nsld.mach-o"));
    assert!(source.contains("command_args = ["));
    assert!(source.contains("producer_phase = \"alpha-0.10.0\""));
}
