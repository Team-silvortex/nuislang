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

fn registrations() -> [DeviceSampleInputRegistration; 20] {
    [
        crate::artifact_device_sample_pixelmagic::registration(),
        crate::artifact_device_sample_kernel::registration(),
        crate::artifact_device_sample_shader_metal::registration(),
        crate::artifact_device_sample_shader_metal::add_registration(),
        crate::artifact_device_sample_shader_metal::add_pair_registration(),
        crate::artifact_device_sample_shader_metal_fan_out::registration(),
        crate::artifact_device_sample_shader_metal::sub_registration(),
        crate::artifact_device_sample_shader_metal::mul_registration(),
        crate::artifact_device_sample_shader_metal::xor_registration(),
        crate::artifact_device_sample_shader_metal::chain_registration(),
        crate::artifact_device_sample_shader_vulkan::registration(),
        crate::artifact_device_sample_shader_vulkan::add_registration(),
        crate::artifact_device_sample_shader_vulkan::add_pair_registration(),
        crate::artifact_device_sample_shader_vulkan_fan_out::registration(),
        crate::artifact_device_sample_shader_vulkan_fan_out::padded_registration(),
        crate::artifact_device_sample_shader_vulkan_fan_out::reduced_registration(),
        crate::artifact_device_sample_shader_vulkan::sub_registration(),
        crate::artifact_device_sample_shader_vulkan::mul_registration(),
        crate::artifact_device_sample_shader_vulkan::xor_registration(),
        crate::artifact_device_sample_shader_vulkan::chain_registration(),
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
        let vulkan_add = enrich_registered_input_evidence(
            "vulkan",
            "discrete-or-integrated-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=vulkan-add-u32",
        )
        .unwrap();
        assert!(
            vulkan_add.contains("provider_sample_registration_id=official.shader.vulkan-add-u32")
        );
        assert!(vulkan_add.contains("provider_code_asset_id=shader.vulkan.add-u32.spirv"));
        assert!(vulkan_add.contains("provider_kernel_operation=add-u32"));
        let vulkan_add_pair = enrich_registered_input_evidence(
            "vulkan",
            "discrete-or-integrated-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=vulkan-add-pair-u32",
        )
        .unwrap();
        assert!(vulkan_add_pair
            .contains("provider_sample_registration_id=official.shader.vulkan-add-pair-u32"));
        assert!(vulkan_add_pair.contains("provider_code_asset_id=shader.vulkan.add-pair-u32.spirv"));
        assert!(vulkan_add_pair.contains("provider_kernel_operation=add-pair-u32"));
        assert!(vulkan_add_pair.contains("provider_input_binding_count=2"));
        let vulkan_sub = enrich_registered_input_evidence(
            "vulkan",
            "discrete-or-integrated-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=vulkan-sub-u32",
        )
        .unwrap();
        assert!(
            vulkan_sub.contains("provider_sample_registration_id=official.shader.vulkan-sub-u32")
        );
        assert!(vulkan_sub.contains("provider_code_asset_id=shader.vulkan.sub-u32.spirv"));
        assert!(vulkan_sub.contains("provider_kernel_operation=sub-u32"));
        let vulkan_mul = enrich_registered_input_evidence(
            "vulkan",
            "discrete-or-integrated-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=vulkan-mul-u32",
        )
        .unwrap();
        assert!(
            vulkan_mul.contains("provider_sample_registration_id=official.shader.vulkan-mul-u32")
        );
        assert!(vulkan_mul.contains("provider_code_asset_id=shader.vulkan.mul-u32.spirv"));
        assert!(vulkan_mul.contains("provider_kernel_operation=mul-u32"));
        let vulkan_xor = enrich_registered_input_evidence(
            "vulkan",
            "discrete-or-integrated-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=vulkan-xor-u32",
        )
        .unwrap();
        assert!(
            vulkan_xor.contains("provider_sample_registration_id=official.shader.vulkan-xor-u32")
        );
        assert!(vulkan_xor.contains("provider_code_asset_id=shader.vulkan.xor-u32.spirv"));
        assert!(vulkan_xor.contains("provider_kernel_operation=xor-u32"));
        let vulkan_chain = enrich_registered_input_evidence(
            "vulkan",
            "discrete-or-integrated-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=vulkan-u32-chain",
        )
        .unwrap();
        assert!(vulkan_chain
            .contains("provider_sample_registration_id=official.shader.vulkan-u32-chain"));
        assert!(vulkan_chain.contains("provider_request_collection_contract="));
        assert!(vulkan_chain.contains("provider_request_count=2"));
        assert!(
            vulkan_chain.contains("provider_request_0_code_asset_id=shader.vulkan.add-u32.spirv")
        );
        assert!(
            vulkan_chain.contains("provider_request_1_code_asset_id=shader.vulkan.xor-u32.spirv")
        );
        assert!(vulkan_chain.contains(
            "provider_request_1_dependency_0_transport_contract=nuis-provider-edge-transport-v1"
        ));
        assert!(vulkan_chain.contains("provider_code_asset_identity_set_count=2"));
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

        let add = enrich_registered_input_evidence(
            "metal",
            "apple-silicon-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=metal-add-u32",
        )
        .unwrap();
        assert!(add.contains("provider_sample_registration_id=official.shader.metal-add-u32"));
        assert!(add.contains("provider_code_asset_id=shader.metal.add-u32.msl"));
        assert!(add.contains("provider_kernel_operation=add-u32"));

        let add_pair = enrich_registered_input_evidence(
            "metal",
            "apple-silicon-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=metal-add-pair-u32",
        )
        .unwrap();
        assert!(
            add_pair.contains("provider_sample_registration_id=official.shader.metal-add-pair-u32")
        );
        assert!(add_pair.contains("provider_code_asset_id=shader.metal.add-pair-u32.msl"));
        assert!(add_pair.contains("provider_kernel_operation=add-pair-u32"));
        assert!(add_pair.contains("provider_input_binding_count=2"));

        let sub = enrich_registered_input_evidence(
            "metal",
            "apple-silicon-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=metal-sub-u32",
        )
        .unwrap();
        assert!(sub.contains("provider_sample_registration_id=official.shader.metal-sub-u32"));
        assert!(sub.contains("provider_code_asset_id=shader.metal.sub-u32.msl"));
        assert!(sub.contains("provider_kernel_operation=sub-u32"));

        let mul = enrich_registered_input_evidence(
            "metal",
            "apple-silicon-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=metal-mul-u32",
        )
        .unwrap();
        assert!(mul.contains("provider_sample_registration_id=official.shader.metal-mul-u32"));
        assert!(mul.contains("provider_code_asset_id=shader.metal.mul-u32.msl"));
        assert!(mul.contains("provider_kernel_operation=mul-u32"));

        let xor = enrich_registered_input_evidence(
            "metal",
            "apple-silicon-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=metal-xor-u32",
        )
        .unwrap();
        assert!(xor.contains("provider_sample_registration_id=official.shader.metal-xor-u32"));
        assert!(xor.contains("provider_code_asset_id=shader.metal.xor-u32.msl"));
        assert!(xor.contains("provider_kernel_operation=xor-u32"));

        let chain = enrich_registered_input_evidence(
            "metal",
            "apple-silicon-gpu",
            "artifact_provider_metadata_0=official.shader:provider-sample=metal-u32-chain",
        )
        .unwrap();
        assert!(chain.contains("provider_sample_registration_id=official.shader.metal-u32-chain"));
        assert!(chain.contains("provider_request_collection_contract="));
        assert!(chain.contains("provider_request_count=2"));
        assert!(chain.contains("provider_request_1_input_binding_0_source=dependency"));
        assert!(chain.contains("provider_request_1_code_asset_id=shader.metal.xor-u32.msl"));
        assert!(chain.contains("provider_code_asset_identity_set_count=2"));
    }
}
