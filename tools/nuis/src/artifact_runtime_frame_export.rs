use std::{fs, io::Read, path::Path};

use crate::artifact_doctor::ArtifactDoctorReport;

pub(super) fn validate(doctor: &ArtifactDoctorReport, output: &Path) -> Result<(), String> {
    let manifest = doctor
        .manifest_path
        .as_deref()
        .ok_or("--export-frame requires a verified build manifest beside the artifact")?;
    let report = nuisc::aot::verify_build_manifest(manifest)?;
    if report.packaging_mode != "window-aot-bundle" {
        return Err("--export-frame requires an embedded YIR window AOT bundle; this launch route does not support it".to_owned());
    }
    let bundle = fs::read_to_string(Path::new(&report.output_dir).join("bundle.txt"))
        .map_err(|error| format!("cannot inspect frame export capability: {error}"))?;
    let declaration = format!(
        "frame_export_contract={}",
        yir_runtime_host::FRAME_EXPORT_CONTRACT
    );
    if !bundle.lines().any(|line| line == declaration) {
        return Err("artifact does not declare frame export support; rebuild it with the current AOT packer".to_owned());
    }
    match fs::symlink_metadata(output) {
        Ok(_) => {
            return Err(format!(
                "frame output `{}` already exists; choose a new path",
                output.display()
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("cannot inspect frame output: {error}")),
    }
    if output.as_os_str().is_empty() {
        return Err("frame output path must not be empty".to_owned());
    }
    Ok(())
}

pub(super) fn verify_output(output: &Path) -> Result<(), String> {
    let mut header = [0; 3];
    fs::File::open(output)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|error| {
            format!(
                "artifact did not produce a frame at `{}`: {error}",
                output.display()
            )
        })?;
    if header != *b"P6\n" {
        return Err("artifact frame output is not a binary PPM image".to_owned());
    }
    if crate::success_logs_enabled() {
        println!(
            "  frame_export_contract: {}",
            yir_runtime_host::FRAME_EXPORT_CONTRACT
        );
        println!("  frame_export_execution: embedded-yir-lifecycle");
        println!("  frame_export_path: {}", output.display());
    }
    Ok(())
}
