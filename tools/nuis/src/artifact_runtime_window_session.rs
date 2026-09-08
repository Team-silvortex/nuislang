use std::{fs, path::Path};

pub(super) fn validate(
    doctor: &crate::artifact_doctor::ArtifactDoctorReport,
    options: &crate::cli::WindowSessionOptions,
) -> Result<(), String> {
    if options.cancel_after_events && (options.events.is_none() || options.parent.is_some()) {
        return Err(
            "--window-cancel-after-events requires --window-events and no parent session"
                .to_owned(),
        );
    }
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
    validate_contracts(&bundle, options)
}

fn validate_contracts(
    bundle: &str,
    options: &crate::cli::WindowSessionOptions,
) -> Result<(), String> {
    let contract = format!(
        "window_session_contract={}",
        yir_runtime_host::WINDOW_SESSION_CONTRACT
    );
    if !bundle.lines().any(|line| line == contract) {
        return Err("artifact does not declare window session support; rebuild it".to_owned());
    }
    if options.parent.is_some() {
        let contract = format!(
            "window_parent_contract={}",
            yir_runtime_host::APPLICATION_OUTCOME_PUMP_CONTRACT
        );
        if !bundle.lines().any(|line| line == contract) {
            return Err("artifact does not declare window parent support; rebuild it".to_owned());
        }
    }
    if options.cancel_after_events {
        let contract = format!(
            "window_cancellation_contract={}",
            yir_runtime_host::APPLICATION_CANCELLATION_CONTRACT
        );
        if !bundle.lines().any(|line| line == contract) {
            return Err(
                "artifact does not declare window cancellation support; rebuild it".to_owned(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_requires_a_declared_bundle_capability_without_breaking_ordinary_quit() {
        let mut options = crate::cli::WindowSessionOptions {
            id: "window".to_owned(),
            events: Some(String::new()),
            parent: None,
            cancel_after_events: false,
        };
        let old = format!(
            "window_session_contract={}\n",
            yir_runtime_host::WINDOW_SESSION_CONTRACT
        );
        assert!(validate_contracts(&old, &options).is_ok());
        options.cancel_after_events = true;
        assert!(validate_contracts(&old, &options)
            .unwrap_err()
            .contains("rebuild"));
        let new = format!(
            "{old}window_cancellation_contract={}\n",
            yir_runtime_host::APPLICATION_CANCELLATION_CONTRACT
        );
        assert!(validate_contracts(&new, &options).is_ok());
        for bad in [
            "nuis-yir-application-cancellation-v0",
            "nuis-yir-application-cancellation-v1-extra",
        ] {
            assert!(validate_contracts(
                &format!("{old}window_cancellation_contract={bad}\n"),
                &options
            )
            .is_err());
        }
    }
}
