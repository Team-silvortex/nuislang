use std::{fs, path::Path};

pub(super) fn validate(
    doctor: &crate::artifact_doctor::ArtifactDoctorReport,
) -> Result<(), String> {
    let manifest = doctor
        .manifest_path
        .as_deref()
        .ok_or("--window-session requires a verified build manifest")?;
    let report = nuisc::aot::verify_build_manifest(manifest)?;
    if report.packaging_mode != "window-aot-bundle" {
        return Err("--window-session requires an embedded YIR window AOT bundle".to_owned());
    }
    let bundle = fs::read_to_string(Path::new(&report.output_dir).join("bundle.txt"))
        .map_err(|error| format!("cannot inspect window session capability: {error}"))?;
    let contract = format!(
        "window_session_contract={}",
        yir_runtime_host::WINDOW_SESSION_CONTRACT
    );
    if !bundle.lines().any(|line| line == contract) {
        return Err("artifact does not declare window session support; rebuild it".to_owned());
    }
    Ok(())
}
