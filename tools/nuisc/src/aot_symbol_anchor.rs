use nuis_artifact::BuildManifestDomainBuildUnit;

pub(crate) fn namespace(unit: &BuildManifestDomainBuildUnit) -> String {
    format!(
        "nuis::domain::{}::{}",
        component(&unit.domain_family),
        component(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    )
}

pub(crate) fn debug_anchor(unit: &BuildManifestDomainBuildUnit) -> String {
    format!(
        "nuis.debug.{}.{}",
        component(&unit.domain_family),
        component(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    )
}

pub(crate) fn linkage_anchor(unit: &BuildManifestDomainBuildUnit) -> String {
    format!(
        "nuis.link.{}.{}",
        component(&unit.domain_family),
        component(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    )
}

pub(crate) fn source_map_scope(unit: &BuildManifestDomainBuildUnit) -> String {
    format!(
        "domain:{}/package:{}/target:{}",
        unit.domain_family,
        unit.package_id,
        unit.selected_lowering_target.as_deref().unwrap_or("none")
    )
}

fn component(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kernel_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.kernel".to_owned(),
            domain_family: "kernel".to_owned(),
            abi: Some("kernel.apple_ane.coreml.v1".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("coreml".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("apple-ane".to_owned()),
            target_device: Some("apple-ane".to_owned()),
            ir_format: Some("mlmodel".to_owned()),
            dispatch_abi: Some("coreml-predict".to_owned()),
            backend_priority: Some(10),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("coreml.apple-ane".to_owned()),
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
            contract_family: "nustar.kernel".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    #[test]
    fn kernel_coreml_symbol_anchors_are_stable() {
        let unit = kernel_unit();

        assert_eq!(namespace(&unit), "nuis::domain::kernel::coreml_apple_ane");
        assert_eq!(debug_anchor(&unit), "nuis.debug.kernel.coreml_apple_ane");
        assert_eq!(linkage_anchor(&unit), "nuis.link.kernel.coreml_apple_ane");
        assert_eq!(
            source_map_scope(&unit),
            "domain:kernel/package:official.kernel/target:coreml.apple-ane"
        );
    }
}
