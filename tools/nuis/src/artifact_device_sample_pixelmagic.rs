use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{fs, path::Path};

const STD_PIXEL_PAYLOAD_FILE_NAME: &str = "nuis.pixelmagic.std-preprocessed.gray8.bin";
const STD_PIXEL_INVERT_EXPECTED_FILE_NAME: &str =
    "nuis.pixelmagic.std-preprocessed.gray8-invert.expected.bin";
const STD_PIXEL_THRESHOLD_EXPECTED_FILE_NAME: &str =
    "nuis.pixelmagic.std-preprocessed.gray8-threshold.expected.bin";
const STD_PIXEL_PAYLOAD: &[u8] = &[0, 4, 9, 8];
const STD_PIXEL_INVERT_EXPECTED: &[u8] = &[15, 11, 6, 7];
const STD_PIXEL_THRESHOLD_EXPECTED: &[u8] = &[15, 15, 0, 0];

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
    let compatibility = format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.pixels;provider_buffer_element_type=u8;provider_buffer_layout=image-2d-row-major:pixel-format=gray8;provider_buffer_shape=2x2;provider_buffer_row_stride_bytes=2;provider_buffer_byte_length={};provider_buffer_payload_path={STD_PIXEL_PAYLOAD_FILE_NAME};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=pixelmagic.gray8.invert;provider_kernel_operation=invert;provider_kernel_input_buffer=input.pixels;provider_kernel_output_buffer=output.pixels;provider_kernel_dispatch=2x2x1;provider_kernel_scalar_bindings=max_value:u8:15;std-preprocessed-pgm:input_bytes=20;pixel_format=gray8;pixel_width=2;pixel_height=2;pixel_stride=2;pixel_max_value=15;pixel_operation=invert;pixel_payload_path={STD_PIXEL_PAYLOAD_FILE_NAME};pixel_payload_bytes={};pixel_payload_hash={}",
        STD_PIXEL_PAYLOAD.len(),
        fnv1a64_hex(STD_PIXEL_PAYLOAD),
        STD_PIXEL_PAYLOAD.len(),
        fnv1a64_hex(STD_PIXEL_PAYLOAD)
    );
    format!(
        "{compatibility};provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=2;{};{}",
        gray8_request(
            0,
            "pixelmagic.gray8.invert",
            "invert",
            "max_value:u8:15",
            "output.pixels.invert",
            STD_PIXEL_INVERT_EXPECTED_FILE_NAME,
            STD_PIXEL_INVERT_EXPECTED,
        ) + &gray8_artifact_input_binding(0),
        gray8_request(
            1,
            "pixelmagic.gray8.threshold",
            "threshold",
            "threshold:u8:8,max_value:u8:15",
            "output.pixels.threshold",
            STD_PIXEL_THRESHOLD_EXPECTED_FILE_NAME,
            STD_PIXEL_THRESHOLD_EXPECTED,
        ) + &gray8_dependency_input_binding(1)
    )
}

fn gray8_request(
    index: usize,
    kernel_id: &str,
    operation: &str,
    scalar_bindings: &str,
    output_buffer: &str,
    expected_path: &str,
    expected: &[u8],
) -> String {
    let prefix = format!("provider_request_{index}_");
    format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.pixels;{prefix}buffer_element_type=u8;{prefix}buffer_layout=image-2d-row-major:pixel-format=gray8;{prefix}buffer_shape=2x2;{prefix}buffer_row_stride_bytes=2;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={STD_PIXEL_PAYLOAD_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id={kernel_id};{prefix}kernel_operation={operation};{prefix}kernel_input_buffer=input.pixels;{prefix}kernel_output_buffer={output_buffer};{prefix}kernel_dispatch=2x2x1;{prefix}kernel_scalar_bindings={scalar_bindings};{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer={output_buffer};{prefix}output_comparison_element_type=u8;{prefix}output_comparison_shape=2x2;{prefix}output_comparison_expected_path={expected_path};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject",
        STD_PIXEL_PAYLOAD.len(),
        fnv1a64_hex(STD_PIXEL_PAYLOAD),
        expected.len(),
        fnv1a64_hex(expected),
    )
}

fn gray8_artifact_input_binding(index: usize) -> String {
    let prefix = format!(";provider_request_{index}_input_binding_");
    format!(
        "{prefix}contract=nuis-provider-input-binding-v1;{prefix}count=1;{prefix}0_name=input.pixels;{prefix}0_source=artifact;{prefix}0_element_type=u8;{prefix}0_shape=2x2;{prefix}0_byte_length={};{prefix}0_content_hash={};{prefix}0_payload_path={STD_PIXEL_PAYLOAD_FILE_NAME};{prefix}0_producer_request_id=none;{prefix}0_producer_output_buffer=none",
        STD_PIXEL_PAYLOAD.len(),
        fnv1a64_hex(STD_PIXEL_PAYLOAD),
    )
}

fn gray8_dependency_input_binding(index: usize) -> String {
    let prefix = format!(";provider_request_{index}_");
    format!(
        "{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id=pixelmagic.gray8.invert;{prefix}dependency_0_producer_output_buffer=output.pixels.invert;{prefix}dependency_0_consumer_input_buffer=input.pixels;{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token=glm:provider-edge:pixelmagic.gray8.invert:output.pixels.invert->pixelmagic.gray8.threshold:input.pixels;{prefix}dependency_0_transport_staging_mode=auto;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-0:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-1:dispatch-ready;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.pixels;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=u8;{prefix}input_binding_0_shape=2x2;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id=pixelmagic.gray8.invert;{prefix}input_binding_0_producer_output_buffer=output.pixels.invert",
        STD_PIXEL_INVERT_EXPECTED.len(),
        fnv1a64_hex(STD_PIXEL_INVERT_EXPECTED),
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
    .map_err(|error| format!("failed to persist PixelMagic provider payload: {error}"))?;
    fs::write(
        output_dir.join(STD_PIXEL_INVERT_EXPECTED_FILE_NAME),
        STD_PIXEL_INVERT_EXPECTED,
    )
    .map_err(|error| format!("failed to persist PixelMagic invert baseline: {error}"))?;
    fs::write(
        output_dir.join(STD_PIXEL_THRESHOLD_EXPECTED_FILE_NAME),
        STD_PIXEL_THRESHOLD_EXPECTED,
    )
    .map_err(|error| format!("failed to persist PixelMagic threshold baseline: {error}"))
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
        assert!(evidence.contains("provider_request_count=2"));
        assert!(evidence.contains("provider_request_1_kernel_id=pixelmagic.gray8.threshold"));
        assert!(evidence.contains("provider_request_1_kernel_operation=threshold"));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_producer_request_id=pixelmagic.gray8.invert"
        ));
        assert!(evidence.contains("provider_request_1_input_binding_0_source=dependency"));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_transport_consumer_clock_evidence=provider-clock:request-1:dispatch-ready"
        ));
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
        let threshold = fs::read(output_dir.join(STD_PIXEL_THRESHOLD_EXPECTED_FILE_NAME)).unwrap();
        fs::remove_dir_all(output_dir).unwrap();

        assert_eq!(payload, STD_PIXEL_PAYLOAD);
        assert_eq!(threshold, STD_PIXEL_THRESHOLD_EXPECTED);
        assert_eq!(fnv1a64_hex(&threshold), "0xfc6f93a90d12d41b");
    }
}
