use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_encoding::fnv1a64_hex;

pub(crate) const SIGNATURE_WHITELIST_POLICY: &str = "signature-whitelist-required";

pub(crate) fn bridge(unit: &BuildManifestDomainBuildUnit) -> String {
    format!("cffi.{}.dispatch.v1", symbol_component(&unit.domain_family))
}

pub(crate) fn symbol(unit: &BuildManifestDomainBuildUnit) -> String {
    format!(
        "nuis_{}_{}_dispatch_v1",
        symbol_component(&unit.domain_family),
        symbol_component(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    )
}

pub(crate) fn signature() -> &'static str {
    "fn(payload: ptr, payload_len: usize, bridge_state: ptr) -> i64"
}

pub(crate) fn signature_hash(unit: &BuildManifestDomainBuildUnit) -> String {
    let material = format!("{}|{}|{}", bridge(unit), symbol(unit), signature());
    fnv1a64_hex(material.as_bytes())
}

fn symbol_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_coreml_dispatch_contract_is_stable() {
        let unit = BuildManifestDomainBuildUnit {
            package_id: "official.kernel".to_owned(),
            domain_family: "kernel".to_owned(),
            contract_family: "nustar.kernel".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            abi: Some("kernel.apple_ane.coreml.v1".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("coreml".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("apple-ane".to_owned()),
            selected_lowering_target: Some("coreml.apple-ane".to_owned()),
            artifact_stub_path: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            artifact_stub_inline: None,
        };

        assert_eq!(bridge(&unit), "cffi.kernel.dispatch.v1");
        assert_eq!(symbol(&unit), "nuis_kernel_coreml_apple_ane_dispatch_v1");
        assert_eq!(signature_hash(&unit), "0x80b1c5fd4c31798e");
    }
}
