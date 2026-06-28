use std::path::Path;

use nuis_artifact::BuildManifestDomainBuildUnit;

pub(crate) fn summary_for_unit(
    unit: &BuildManifestDomainBuildUnit,
) -> crate::registry::NustarDomainBuildContractSummary {
    match crate::registry::load_manifest(Path::new("nustar-packages"), &unit.package_id) {
        Ok(manifest) => crate::registry::domain_build_contract_summary(&manifest),
        Err(_) => crate::registry::domain_build_contract_summary_for_domain(&unit.domain_family),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(domain: &str, package_id: &str) -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: package_id.to_owned(),
            domain_family: domain.to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: None,
            vendor: None,
            device_class: None,
            selected_lowering_target: None,
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            contract_family: format!("nustar.{domain}"),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    #[test]
    fn resolves_registered_kernel_contract_summary() {
        let summary = summary_for_unit(&unit("kernel", "official.kernel"));

        assert_eq!(summary.lowering.bridge_surface, "host-ffi.bridge.hetero");
        assert_eq!(
            summary.bridge.bridge_entry,
            "nuis.kernel.bridge.dispatch.v1"
        );
    }

    #[test]
    fn falls_back_to_domain_summary_when_package_is_missing() {
        let summary = summary_for_unit(&unit("shader", "missing.shader"));

        assert_eq!(summary.lowering.bridge_surface, "host-ffi.bridge.hetero");
        assert_eq!(
            summary.bridge.bridge_entry,
            "nuis.shader.bridge.dispatch.v1"
        );
    }
}
