use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{fs, path::Path};

const STD_PIXEL_PAYLOAD_FILE_NAME: &str = "nuis.pixelmagic.std-preprocessed.gray8.bin";
const STD_PIXEL_PAYLOAD: &[u8] = &[0, 4, 9, 8];

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: "nuis.pixelmagic",
        supports: |backend_family, target_device| {
            backend_family == "metal" && target_device == "apple-silicon-gpu"
        },
        enrich_evidence: pixelmagic_gray8_evidence,
        persist_payloads: persist_pixelmagic_payloads,
    }
}

fn pixelmagic_gray8_evidence(_base: &str) -> String {
    format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.pixels;provider_buffer_element_type=u8;provider_buffer_layout=image-2d-row-major:pixel-format=gray8;provider_buffer_shape=2x2;provider_buffer_row_stride_bytes=2;provider_buffer_byte_length={};provider_buffer_payload_path={STD_PIXEL_PAYLOAD_FILE_NAME};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=pixelmagic.gray8.invert;provider_kernel_operation=invert;provider_kernel_input_buffer=input.pixels;provider_kernel_output_buffer=output.pixels;provider_kernel_dispatch=2x2x1;provider_kernel_scalar_bindings=max_value:u8:15;std-preprocessed-pgm:input_bytes=20;pixel_format=gray8;pixel_width=2;pixel_height=2;pixel_stride=2;pixel_max_value=15;pixel_operation=invert;pixel_payload_path={STD_PIXEL_PAYLOAD_FILE_NAME};pixel_payload_bytes={};pixel_payload_hash={}",
        STD_PIXEL_PAYLOAD.len(),
        fnv1a64_hex(STD_PIXEL_PAYLOAD),
        STD_PIXEL_PAYLOAD.len(),
        fnv1a64_hex(STD_PIXEL_PAYLOAD)
    )
}

fn persist_pixelmagic_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if !evidence
        .iter()
        .any(|item| item.contains("provider_sample_registration_package=nuis.pixelmagic"))
    {
        return Ok(());
    }
    fs::write(
        output_dir.join(STD_PIXEL_PAYLOAD_FILE_NAME),
        STD_PIXEL_PAYLOAD,
    )
    .map_err(|error| format!("failed to persist PixelMagic provider payload: {error}"))
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn registration_owns_gray8_shape_kernel_and_payload() {
        let registration = registration();
        let evidence = (registration.enrich_evidence)("ignored");

        assert_eq!(registration.package_id, "nuis.pixelmagic");
        assert!((registration.supports)("metal", "apple-silicon-gpu"));
        assert!(evidence.contains("provider_buffer_shape=2x2"));
        assert!(evidence.contains("provider_kernel_id=pixelmagic.gray8.invert"));
        assert!(evidence.contains("pixel_payload_hash=0x2a974c7f8a4241d0"));
        assert!(include_str!("../../../stdlib/pixelmagic/module.toml")
            .contains("contract.pixelmagic.provider-sample-input-registration.v1"));
    }

    #[test]
    fn registration_persists_its_own_payload() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-pixelmagic-provider-registration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        persist_pixelmagic_payloads(
            &output_dir,
            &["provider_sample_registration_package=nuis.pixelmagic"],
        )
        .unwrap();
        let payload = fs::read(output_dir.join(STD_PIXEL_PAYLOAD_FILE_NAME)).unwrap();
        fs::remove_dir_all(output_dir).unwrap();

        assert_eq!(payload, STD_PIXEL_PAYLOAD);
    }
}
