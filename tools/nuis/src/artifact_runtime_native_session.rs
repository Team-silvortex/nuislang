use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// An explicit static profile never enters provider preparation or Nsld fallback.
pub(crate) fn handle_run_artifact_with_native_session(
    input: PathBuf,
    arguments: Vec<String>,
) -> Result<(), String> {
    let doctor = super::probe_artifact_doctor(&input);
    let manifest = doctor.manifest_path.as_deref().ok_or(
        "--native-session requires a verified build manifest; unpack standalone artifacts first",
    )?;
    let report = nuisc::aot::verify_build_manifest(manifest)?;
    let id = nuisc::aot::native_session::registration_id(&report.packaging_mode)?
        .ok_or("--native-session requires native-session-aot-bundle:<registration-id> packaging")?;
    nuisc::aot::verify_nuis_compiled_artifact(Path::new(&report.artifact_path))?;
    let binary = super::resolve_run_artifact_binary_path(&input)?;
    let output = Path::new(&report.output_dir);
    let expected = output.join(&report.artifact_binary_name);
    if fs::canonicalize(&binary).map_err(|e| e.to_string())?
        != fs::canonicalize(&expected).map_err(|e| e.to_string())?
    {
        return Err(
            "native launch binary does not match the verified manifest identity".to_owned(),
        );
    }
    let read = |name: String| fs::read_to_string(output.join(name)).map_err(|e| e.to_string());
    let module =
        yir_syntax::parse_explicit_module(&read(format!("{}.yir", report.artifact_binary_name))?)?;
    let layout = nuisc::aot::native_session::verify_checkpoint(
        &module,
        id,
        &read(format!("{}.ll", report.artifact_binary_name))?,
        &read("bundle.txt".to_owned())?,
    )?;
    yir_runtime_host::validate_native_application_script(
        &module,
        id,
        &layout,
        &arguments.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    let status = Command::new(&binary)
        .args(arguments)
        .status()
        .map_err(|error| format!("failed to launch native session binary: {error}"))?;
    if !status.success() {
        return Err(format!(
            "native session process failed: {status}; no fallback was attempted"
        ));
    }
    Ok(())
}
