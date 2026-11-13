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
        "native session launch requires a verified build manifest; unpack standalone artifacts first",
    )?;
    let report = nuisc::aot::verify_build_manifest(manifest)?;
    let resource = nuisc::aot::resource_session::registration_id(&report.packaging_mode)?;
    let scalar = nuisc::aot::native_session::registration_id(&report.packaging_mode)?;
    if resource.is_none() && scalar.is_none() {
        return Err("native launch requires a matching native-session or native-resource-session packaging profile".to_owned());
    }
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
    let llvm = read(format!("{}.ll", report.artifact_binary_name))?;
    let bundle = read("bundle.txt".to_owned())?;
    let args = arguments.iter().map(String::as_str).collect::<Vec<_>>();
    if let Some(id) = resource {
        if args.chunks_exact(2).any(|pair| {
            matches!(
                pair[0],
                "--resource-frame" | "--resource-frame-output" | "--resource-frame-fields"
            )
        }) {
            nuisc::aot::resource_session::verify_frame_export_capability(&bundle)?;
        }
        nuisc::aot::resource_session::verify_checkpoint(
            &module,
            &report.packaging_mode,
            &llvm,
            &bundle,
        )?;
        yir_runtime_host::native_resource_session::validate_resource_script(&module, id, &args)?;
    } else {
        let layout = nuisc::aot::native_session::verify_checkpoint_for_packaging_mode(
            &module,
            &report.packaging_mode,
            &llvm,
            &bundle,
        )?;
        yir_runtime_host::validate_native_application_script(
            &module,
            scalar.unwrap(),
            &layout,
            &args,
        )?;
    }
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
