use crate::{
    final_executable_host::{
        nsld_emit_final_executable_host_invoke_plan_report,
        nsld_final_executable_host_dry_run_report, nsld_final_executable_host_invoke_plan_report,
    },
    final_executable_writer_input::nsld_emit_final_executable_writer_input_report,
    main_test_support::empty_link_plan,
    prepare::nsld_prepare_report,
};
use std::{fs, path::Path};

#[test]
fn host_finalizer_reports_keep_pe_coff_native_object_command_arg() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-final-executable-host-pe-coff-command-{}",
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
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let dry_run = nsld_final_executable_host_dry_run_report(Path::new("manifest.toml"), &plan);
    let invoke_plan =
        nsld_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(dry_run
        .command_args
        .iter()
        .any(|arg| arg.ends_with("nuis.nsld.pe-coff")));
    assert!(invoke_plan
        .command_args
        .iter()
        .any(|arg| arg.ends_with("nuis.nsld.pe-coff")));
    assert!(dry_run
        .command_args
        .iter()
        .all(|arg| !arg.ends_with("nuis.nsld.mach-o")));
    assert!(invoke_plan
        .command_args
        .iter()
        .all(|arg| !arg.ends_with("nuis.nsld.mach-o")));
}

#[test]
fn host_invoke_plan_records_alpha_0_10_producer_phase() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-final-executable-host-invoke-phase-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
    fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
    fs::write(dir.join("nuis.nsld.mach-o"), b"native-object").unwrap();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan)
            .unwrap();
    let source = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(source.contains("producer_phase = \"alpha-0.10.0\""));
}
