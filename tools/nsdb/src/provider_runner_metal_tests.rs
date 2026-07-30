use super::parse_metal_runner_output;

#[cfg(target_os = "macos")]
use super::{
    execute_f32_argmax_input, execute_f32_bias_input, execute_gray8_invert,
    execute_gray8_threshold, execute_u32_canonical_input, execute_u32_copy_input,
};
#[cfg(target_os = "macos")]
use crate::provider_carrier_input::ProviderCarrierInput;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(target_os = "macos")]
fn shader_asset(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../nustar-packages/assets/shader")
        .join(name)
}

#[cfg(target_os = "macos")]
fn registered_shader_asset(asset_id: &str) -> (Vec<u8>, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(&root, "shader").unwrap();
    let asset = nuisc::registry::code_asset_registration_by_id(&root, &manifest, asset_id)
        .unwrap()
        .expect("registered shader asset");
    (asset.bytes, asset.entry)
}

#[test]
fn parses_ready_metal_runner_output() {
    let execution = parse_metal_runner_output(
        "protocol=nuis-metal-gray8-provider-runner-v1\nstatus=ready\ndevice=Apple M2\noutput_bytes=4\noutput_hex=0f0b0607\n",
    )
    .unwrap();

    assert_eq!(execution.contract, "nuis-metal-gray8-provider-runner-v1");
    assert_eq!(execution.status, "metal-command-buffer-completed");
    assert_eq!(execution.device, "Apple M2");
    assert_eq!(execution.output_payload.as_bytes(), [15, 11, 6, 7]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_gray8_invert_on_the_system_metal_device() {
    let input = std::env::temp_dir().join(format!(
        "nuis-metal-gray8-input-{}-{}.bin",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&input, [0, 4, 9, 8]).unwrap();
    let execution = execute_gray8_invert(&input, 15).expect("system Metal provider execution");
    let _ = std::fs::remove_file(input);

    assert_eq!(execution.contract, "nuis-metal-gray8-provider-runner-v1");
    assert_eq!(execution.status, "metal-command-buffer-completed");
    assert!(!execution.device.is_empty());
    assert_eq!(execution.output_payload.as_bytes(), [15, 11, 6, 7]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_gray8_threshold_on_the_system_metal_device() {
    let input = std::env::temp_dir().join(format!(
        "nuis-metal-gray8-threshold-input-{}-{}.bin",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&input, [0, 4, 9, 8]).unwrap();
    let execution =
        execute_gray8_threshold(&input, 8, 15).expect("system Metal threshold execution");
    let _ = std::fs::remove_file(input);

    assert_eq!(
        execution.contract,
        "nuis-metal-gray8-threshold-provider-runner-v1"
    );
    assert_eq!(execution.output_payload.as_bytes(), [0, 0, 15, 15]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_f32_bias_from_opaque_carrier_bytes() {
    let input = ProviderCarrierInput::OpaqueBytes {
        handle: "memory:metal-test".to_owned(),
        bytes: [10.0f32, 16.0, 22.0, 28.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect(),
    };
    let execution = execute_f32_bias_input(
        &input,
        1.0,
        &shader_asset("witsage_vector_bias.metal"),
        "nuis_witsage_vector_bias_f32",
    )
    .expect("opaque Metal input");
    let values = execution
        .output_payload
        .as_bytes()
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(values, [11.0, 17.0, 23.0, 29.0]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_f32_argmax_from_opaque_carrier_bytes() {
    let input = ProviderCarrierInput::OpaqueBytes {
        handle: "memory:metal-argmax-test".to_owned(),
        bytes: [0.0f32, 16.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect(),
    };
    let execution = execute_f32_argmax_input(
        &input,
        &shader_asset("witsage_argmax.metal"),
        "nuis_witsage_argmax_f32",
    )
    .expect("opaque Metal argmax input");
    assert_eq!(
        u32::from_le_bytes(execution.output_payload.as_bytes().try_into().unwrap()),
        1
    );
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_copy_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.copy-u32.msl", "copy-u32");
    assert_eq!(execution.contract, "nuis-metal-u32-copy-provider-runner-v1");
    assert_eq!(u32_values(&execution.output_payload), [1, 8, 13, 21]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_add_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.add-u32.msl", "add-u32");
    assert_eq!(
        execution.contract,
        "nuis-metal-u32-canonical-provider-runner-v1"
    );
    assert_eq!(u32_values(&execution.output_payload), [2, 16, 26, 42]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_sub_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.sub-u32.msl", "sub-u32");
    assert_eq!(
        execution.contract,
        "nuis-metal-u32-canonical-provider-runner-v1"
    );
    assert_eq!(u32_values(&execution.output_payload), [0, 0, 0, 0]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_mul_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.mul-u32.msl", "mul-u32");
    assert_eq!(
        execution.contract,
        "nuis-metal-u32-canonical-provider-runner-v1"
    );
    assert_eq!(u32_values(&execution.output_payload), [1, 64, 169, 441]);
}

#[cfg(target_os = "macos")]
fn execute_registered_u32_msl(asset_id: &str, operation: &str) -> super::MetalProviderExecution {
    let (metal_source, entry) = registered_shader_asset(asset_id);
    let source_path = std::env::temp_dir().join(format!(
        "nuis-metal-u32-{operation}-source-{}-{}.metal",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&source_path, metal_source).unwrap();
    let input = ProviderCarrierInput::OpaqueBytes {
        handle: format!("memory:metal-u32-{operation}-test"),
        bytes: [1u32, 8, 13, 21]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect(),
    };
    let execution = if operation == "copy-u32" {
        execute_u32_copy_input(&input, &source_path, &entry)
    } else {
        execute_u32_canonical_input(&input, &source_path, &entry, operation)
    }
    .expect("generated Metal u32 operation");
    let _ = std::fs::remove_file(source_path);
    execution
}

#[cfg(target_os = "macos")]
fn u32_values(
    payload: &crate::provider_output_carrier_registry::ProviderOutputPayload,
) -> Vec<u32> {
    payload
        .as_bytes()
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect()
}
