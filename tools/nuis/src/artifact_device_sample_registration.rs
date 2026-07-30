use std::path::Path;

pub(crate) const DEVICE_SAMPLE_INPUT_REGISTRATION_CONTRACT: &str =
    "nuis-device-sample-input-registration-v1";

pub(crate) struct DeviceSampleInputRegistration {
    pub(crate) package_id: &'static str,
    pub(crate) registration_id: &'static str,
    pub(crate) provider_family: &'static str,
    pub(crate) supports: fn(&str, &str) -> bool,
    pub(crate) metadata_selector: Option<fn(&str) -> bool>,
    pub(crate) enrich_evidence: fn(&str) -> String,
    pub(crate) resolve_evidence: Option<fn(&Path, &str) -> Result<String, String>>,
    pub(crate) persist_payloads: fn(&Path, &[&str]) -> Result<(), String>,
}

pub(crate) fn enrich_registered_input_evidence(
    backend_family: &str,
    target_device: &str,
    base: &str,
) -> Option<String> {
    selected_registration(backend_family, target_device, base).map(|registration| {
            format!(
                "{};provider_sample_registration_contract={DEVICE_SAMPLE_INPUT_REGISTRATION_CONTRACT};provider_sample_registration_package={};provider_sample_registration_id={};{}",
                base,
                registration.package_id,
                registration.registration_id,
                (registration.enrich_evidence)(base)
            )
        })
}

pub(crate) fn registered_provider_family(
    backend_family: &str,
    target_device: &str,
) -> Option<&'static str> {
    registrations()
        .iter()
        .find(|registration| (registration.supports)(backend_family, target_device))
        .map(|registration| registration.provider_family)
}

pub(crate) fn persist_registered_input_payloads(
    output_dir: &Path,
    evidence: &[&str],
) -> Result<(), String> {
    for registration in registrations() {
        if evidence
            .iter()
            .any(|item| evidence_matches_registration(item, &registration))
        {
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
        if evidence_matches_registration(&resolved, &registration) {
            if let Some(resolve) = registration.resolve_evidence {
                resolved = resolve(output_dir, &resolved)?;
            }
        }
    }
    Ok(resolved)
}

fn selected_registration(
    backend_family: &str,
    target_device: &str,
    base: &str,
) -> Option<DeviceSampleInputRegistration> {
    registrations()
        .into_iter()
        .find(|registration| {
            (registration.supports)(backend_family, target_device)
                && registration
                    .metadata_selector
                    .is_some_and(|selector| selector(base))
        })
        .or_else(|| {
            registrations().into_iter().find(|registration| {
                (registration.supports)(backend_family, target_device)
                    && registration.metadata_selector.is_none()
            })
        })
}

fn evidence_matches_registration(
    evidence: &str,
    registration: &DeviceSampleInputRegistration,
) -> bool {
    if !evidence.split(';').any(|field| {
        field
            == format!(
                "provider_sample_registration_package={}",
                registration.package_id
            )
    }) {
        return false;
    }
    let explicit_id = evidence
        .split(';')
        .filter_map(|field| field.split_once('='))
        .find_map(|(key, value)| (key == "provider_sample_registration_id").then_some(value));
    explicit_id
        .map(|id| id == registration.registration_id)
        .unwrap_or_else(|| registration.metadata_selector.is_none())
}

fn registrations() -> [DeviceSampleInputRegistration; 4] {
    [
        crate::artifact_device_sample_pixelmagic::registration(),
        crate::artifact_device_sample_kernel::registration(),
        crate::artifact_device_sample_shader_metal::registration(),
        crate::artifact_device_sample_shader_vulkan::registration(),
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
        assert!(evidence.contains("provider_sample_registration_id=nuis.pixelmagic.gray8-filter"));
        assert!(evidence.contains("provider_buffer_descriptor_contract="));
        assert_eq!(
            registered_provider_family("metal", "apple-silicon-gpu"),
            Some("metal:apple-silicon-gpu")
        );
        let cuda = enrich_registered_input_evidence("cuda", "nvidia-gpu", "base").unwrap();
        assert!(cuda.contains("provider_sample_registration_package=official.kernel"));
        assert!(cuda.contains("provider_sample_registration_id=official.kernel.cuda-vector"));
        assert!(cuda.contains("provider_code_asset_descriptor_contract="));
        assert_eq!(
            registered_provider_family("cuda", "nvidia-gpu"),
            Some("cuda:nvidia-gpu")
        );
        let vulkan =
            enrich_registered_input_evidence("vulkan", "discrete-or-integrated-gpu", "base")
                .unwrap();
        assert!(vulkan.contains("provider_sample_registration_package=official.shader"));
        assert!(vulkan.contains("provider_sample_registration_id=official.shader.vulkan-copy-u32"));
        assert!(vulkan.contains("provider_code_asset_format=spirv-binary"));
        assert!(vulkan.contains("provider_adapter_binding_provider_family=spirv:vulkan-gpu"));
        assert_eq!(
            registered_provider_family("vulkan", "discrete-or-integrated-gpu"),
            Some("spirv:vulkan-gpu")
        );
        assert!(
            enrich_registered_input_evidence("unknown", "unknown", "base").is_none(),
            "unregistered backends must remain generic"
        );
    }

    #[test]
    fn explicit_shader_metal_metadata_does_not_steal_default_metal_registration() {
        let default =
            enrich_registered_input_evidence("metal", "apple-silicon-gpu", "base").unwrap();
        assert!(default.contains("provider_sample_registration_package=nuis.pixelmagic"));

        let selected = enrich_registered_input_evidence(
            "metal",
            "apple-silicon-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=metal-copy-u32",
        )
        .unwrap();
        assert!(selected.contains("provider_sample_registration_package=official.shader"));
        assert!(selected.contains("provider_sample_registration_id=official.shader.metal-copy-u32"));
        assert!(selected.contains("provider_code_asset_id=shader.metal.copy-u32.msl"));
    }
}
