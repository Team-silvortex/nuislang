use std::{fs, path::Path};

use yir_runtime_host::{ApplicationScript, ApplicationScriptTermination};

pub(super) fn validate(
    doctor: &crate::artifact_doctor::ArtifactDoctorReport,
    binary: &Path,
    script: &ApplicationScript,
) -> Result<(), String> {
    script.validate()?;
    if matches!(
        script.termination,
        ApplicationScriptTermination::Cancel {
            drain_provider: false
        }
    ) {
        return Err("live application cancellation requires --drain-provider; host exit alone is not a provider retirement receipt".to_owned());
    }
    let manifest = doctor
        .manifest_path
        .as_deref()
        .ok_or("--application-session requires a verified build manifest")?;
    let report = nuisc::aot::verify_build_manifest(manifest)?;
    if report.packaging_mode != "headless-aot-bundle" {
        return Err("--application-session requires headless-aot-bundle packaging; rebuild with --packaging-mode headless-aot-bundle".to_owned());
    }
    nuisc::aot::verify_nuis_compiled_artifact(Path::new(&report.artifact_path))?;
    let output = Path::new(&report.output_dir);
    let expected = output.join(&report.artifact_binary_name);
    if fs::canonicalize(binary).map_err(|error| error.to_string())?
        != fs::canonicalize(&expected).map_err(|error| error.to_string())?
    {
        return Err(
            "application launch binary does not match the verified manifest identity".to_owned(),
        );
    }
    let bundle = fs::read_to_string(output.join("bundle.txt"))
        .map_err(|error| format!("cannot inspect application script capability: {error}"))?;
    validate_contracts(&bundle)?;
    let source = fs::read_to_string(output.join(format!("{}.yir", report.artifact_binary_name)))
        .map_err(|error| format!("cannot inspect application script registration: {error}"))?;
    require_declaration(
        &bundle,
        "application_yir_fnv1a64",
        &yir_core::provider_runtime_ipc::hash_bytes(source.as_bytes()),
    )?;
    script.validate_source(&source)
}

pub(super) fn require_explicit_script(
    doctor: &crate::artifact_doctor::ArtifactDoctorReport,
) -> Result<(), String> {
    if let Some(manifest) = &doctor.manifest_path {
        let report = nuisc::aot::verify_build_manifest(manifest)?;
        if nuisc::aot::native_session::registration_id(&report.packaging_mode)?.is_some() {
            return Err("native application artifact requires --native-session, --open-args and --close-args".to_owned());
        }
        if report.packaging_mode == "headless-aot-bundle" {
            return Err("headless application artifact requires --application-session, --open-args and an explicit close or cancellation script".to_owned());
        }
    }
    Ok(())
}

fn validate_contracts(bundle: &str) -> Result<(), String> {
    for (key, value) in [
        ("cpu_host_binary_mode", "embedded_yir_headless"),
        ("runtime_bootstrap_mode", "embedded_yir_session"),
        ("render_mode", "application_session_observation"),
        ("single_binary", "true"),
        (
            "application_script_contract",
            yir_runtime_host::APPLICATION_SCRIPT_CONTRACT,
        ),
        (
            "application_provider_drain_contract",
            yir_core::provider_runtime_ipc::SESSION_DRAIN_CONTRACT,
        ),
    ] {
        require_declaration(bundle, key, value)?;
    }
    if bundle.lines().any(|line| {
        line.split_once('=').is_some_and(|(key, _)| {
            matches!(
                key.trim(),
                "window_session_contract" | "fallback_frame_asset"
            )
        })
    }) {
        return Err(
            "headless application artifact declares an incompatible window or fallback capability"
                .to_owned(),
        );
    }
    Ok(())
}

fn require_declaration(bundle: &str, key: &str, value: &str) -> Result<(), String> {
    let mut declarations = bundle.lines().filter_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key).then_some(value)
    });
    if declarations.next() != Some(value) || declarations.next().is_some() {
        return Err(format!(
            "artifact must declare exactly one compatible `{key}`; rebuild it"
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "artifact_runtime_application_script_tests.rs"]
mod tests;
