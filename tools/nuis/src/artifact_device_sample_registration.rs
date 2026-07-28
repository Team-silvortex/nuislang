use std::path::Path;

pub(crate) const DEVICE_SAMPLE_INPUT_REGISTRATION_CONTRACT: &str =
    "nuis-device-sample-input-registration-v1";

pub(crate) struct DeviceSampleInputRegistration {
    pub(crate) package_id: &'static str,
    pub(crate) supports: fn(&str, &str) -> bool,
    pub(crate) enrich_evidence: fn(&str) -> String,
    pub(crate) resolve_evidence: Option<fn(&Path, &str) -> Result<String, String>>,
    pub(crate) persist_payloads: fn(&Path, &[&str]) -> Result<(), String>,
}

pub(crate) fn enrich_registered_input_evidence(
    backend_family: &str,
    target_device: &str,
    base: &str,
) -> Option<String> {
    registrations()
        .iter()
        .find(|registration| (registration.supports)(backend_family, target_device))
        .map(|registration| {
            format!(
                "{};provider_sample_registration_contract={DEVICE_SAMPLE_INPUT_REGISTRATION_CONTRACT};provider_sample_registration_package={};{}",
                base,
                registration.package_id,
                (registration.enrich_evidence)(base)
            )
        })
}

pub(crate) fn persist_registered_input_payloads(
    output_dir: &Path,
    evidence: &[&str],
) -> Result<(), String> {
    for registration in registrations() {
        if evidence.iter().any(|item| {
            item.contains(&format!(
                "provider_sample_registration_package={}",
                registration.package_id
            ))
        }) {
            (registration.persist_payloads)(output_dir, evidence)?;
        }
    }
    Ok(())
}

pub(crate) fn resolve_registered_input_evidence(
    output_dir: &Path,
    evidence: &str,
) -> Result<String, String> {
    let mut resolved = evidence.to_owned();
    for registration in registrations() {
        if resolved.contains(&format!(
            "provider_sample_registration_package={}",
            registration.package_id
        )) {
            if let Some(resolve) = registration.resolve_evidence {
                resolved = resolve(output_dir, &resolved)?;
            }
        }
    }
    Ok(resolved)
}

fn registrations() -> [DeviceSampleInputRegistration; 2] {
    [
        crate::artifact_device_sample_pixelmagic::registration(),
        crate::artifact_device_sample_kernel::registration(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_input_evidence_is_open_and_provider_neutral() {
        let evidence =
            enrich_registered_input_evidence("metal", "apple-silicon-gpu", "base").unwrap();

        assert!(evidence.contains(DEVICE_SAMPLE_INPUT_REGISTRATION_CONTRACT));
        assert!(evidence.contains("provider_sample_registration_package=nuis.pixelmagic"));
        assert!(evidence.contains("provider_buffer_descriptor_contract="));
        let cuda = enrich_registered_input_evidence("cuda", "nvidia-gpu", "base").unwrap();
        assert!(cuda.contains("provider_sample_registration_package=official.kernel"));
        assert!(cuda.contains("provider_code_asset_descriptor_contract="));
        assert!(
            enrich_registered_input_evidence("unknown", "unknown", "base").is_none(),
            "unregistered backends must remain generic"
        );
    }
}
