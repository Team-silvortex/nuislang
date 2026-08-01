use super::*;
use crate::{
    provider_adapter_binding::ProviderAdapterBinding,
    provider_code_asset::CODE_ASSET_FNV1A64_DIGEST_CONTRACT,
    provider_input_binding::ProviderInputBinding,
    provider_request::{
        ProviderBufferDescriptor, ProviderKernelDescriptor, ProviderOutputBinding,
        ProviderOutputComparisonDescriptor, ProviderRequest, ProviderScalarBinding,
    },
};
use std::env;

#[test]
fn vulkan_execution_registration_still_fails_closed_at_probe_boundary() {
    assert_eq!(
        REGISTRATION.registry_contract,
        PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
    );
    assert_eq!(REGISTRATION.adapter_kind, "vulkan-spirv-real-device-runner");
    assert!(REGISTRATION.requires_worker_descriptors);
    #[cfg(target_os = "linux")]
    assert!(REGISTRATION.prepare_worker_adapter.is_some());
    #[cfg(not(target_os = "linux"))]
    assert!(REGISTRATION.prepare_worker_adapter.is_none());
}

#[test]
fn vulkan_session_plan_accepts_registered_u32_shapes() {
    let output_dir =
        env::temp_dir().join(format!("nsdb-vulkan-session-plan-{}", std::process::id()));
    let _ = fs::remove_dir_all(&output_dir);
    fs::create_dir_all(&output_dir).unwrap();
    for (operation, entry, asset_id, path, input_count) in [
        (
            "copy-u32",
            "nuis_vulkan_copy_u32",
            "shader.vulkan.copy-u32.spirv",
            "nuis.shader.vulkan.copy-u32.spv",
            1,
        ),
        (
            "add-u32",
            "nuis_vulkan_add_u32",
            "shader.vulkan.add-u32.spirv",
            "nuis.shader.vulkan.add-u32.spv",
            1,
        ),
        (
            "add-pair-u32",
            "nuis_vulkan_add_pair_u32",
            "shader.vulkan.add-pair-u32.spirv",
            "nuis.shader.vulkan.add-pair-u32.spv",
            2,
        ),
        (
            "sub-u32",
            "nuis_vulkan_sub_u32",
            "shader.vulkan.sub-u32.spirv",
            "nuis.shader.vulkan.sub-u32.spv",
            1,
        ),
        (
            "mul-u32",
            "nuis_vulkan_mul_u32",
            "shader.vulkan.mul-u32.spirv",
            "nuis.shader.vulkan.mul-u32.spv",
            1,
        ),
        (
            "xor-u32",
            "nuis_vulkan_xor_u32",
            "shader.vulkan.xor-u32.spirv",
            "nuis.shader.vulkan.xor-u32.spv",
            1,
        ),
    ] {
        let spirv = registered_spirv_fixture(asset_id);
        fs::write(output_dir.join(path), &spirv).unwrap();
        let mut request = u32_request(operation, entry, asset_id, path, &spirv);
        if input_count == 2 {
            push_aux_u32_input(&mut request, "artifact");
        }
        let plan = validate_vulkan_execution_session_plan(
            &output_dir,
            VULKAN_PROVIDER_FAMILY,
            &request,
            input_count,
        )
        .expect("validated Vulkan execution plan");

        assert_eq!(plan.contract, VULKAN_EXECUTION_SESSION_PLAN_CONTRACT);
        assert_eq!(plan.status, VULKAN_EXECUTION_SESSION_PLAN_STATUS);
        assert_eq!(plan.asset_id, asset_id);
        assert_eq!(plan.entry, entry);
        assert_eq!(plan.element_count, 4);
        assert_eq!(plan.input_byte_length, 16);
        assert_eq!(plan.descriptor_set, 0);
        assert_eq!(
            plan.input_bindings,
            (0..input_count as u32).collect::<Vec<_>>()
        );
        assert_eq!(plan.output_layouts.len(), 1);
        assert_eq!(plan.output_layouts[0].binding, input_count as u32);
        assert_eq!(plan.output_layouts[0].logical_byte_length, 16);
        assert_eq!(plan.output_layouts[0].carrier_byte_length, 16);
        assert_eq!(plan.dispatch, [4, 1, 1]);
    }
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn vulkan_session_plan_rejects_asset_or_binding_drift() {
    let output_dir =
        env::temp_dir().join(format!("nsdb-vulkan-session-drift-{}", std::process::id()));
    let _ = fs::remove_dir_all(&output_dir);
    fs::create_dir_all(&output_dir).unwrap();
    let spirv = registered_spirv_fixture("shader.vulkan.copy-u32.spirv");
    fs::write(output_dir.join("nuis.shader.vulkan.copy-u32.spv"), &spirv).unwrap();
    let mut request = u32_request(
        "copy-u32",
        "nuis_vulkan_copy_u32",
        "shader.vulkan.copy-u32.spirv",
        "nuis.shader.vulkan.copy-u32.spv",
        &spirv,
    );
    request.code_asset.as_mut().expect("code asset").format = "metal-source".to_owned();
    assert!(validate_vulkan_execution_session_plan(
        &output_dir,
        VULKAN_PROVIDER_FAMILY,
        &request,
        1,
    )
    .unwrap_err()
    .contains("registered SPIR-V ABI"));
    let mut request = u32_request(
        "copy-u32",
        "nuis_vulkan_copy_u32",
        "shader.vulkan.copy-u32.spirv",
        "nuis.shader.vulkan.copy-u32.spv",
        &spirv,
    );
    request
        .adapter_binding
        .as_mut()
        .expect("adapter binding")
        .provider_family = "cuda:nvidia-gpu".to_owned();
    assert!(validate_vulkan_execution_session_plan(
        &output_dir,
        VULKAN_PROVIDER_FAMILY,
        &request,
        1,
    )
    .unwrap_err()
    .contains("registered SPIR-V ABI"));
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn vulkan_session_plan_accepts_verified_dependency_input() {
    let output_dir = env::temp_dir().join(format!(
        "nsdb-vulkan-session-dependency-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&output_dir);
    fs::create_dir_all(&output_dir).unwrap();
    let spirv = registered_spirv_fixture("shader.vulkan.mul-u32.spirv");
    fs::write(output_dir.join("nuis.shader.vulkan.mul-u32.spv"), &spirv).unwrap();
    let mut request = u32_request(
        "mul-u32",
        "nuis_vulkan_mul_u32",
        "shader.vulkan.mul-u32.spirv",
        "nuis.shader.vulkan.mul-u32.spv",
        &spirv,
    );
    request.input_bindings[0].source = "dependency".to_owned();
    request.input_bindings[0].payload_path = "none".to_owned();
    request.input_bindings[0].producer_request_id = "shader.vulkan.chain.add-u32".to_owned();
    request.input_bindings[0].producer_output_buffer = "output.values".to_owned();
    let plan =
        validate_vulkan_execution_session_plan(&output_dir, VULKAN_PROVIDER_FAMILY, &request, 1)
            .expect("dependency-backed Vulkan plan");
    assert_eq!(plan.asset_id, "shader.vulkan.mul-u32.spirv");

    request.input_bindings[0].source = "ambient".to_owned();
    assert!(validate_vulkan_execution_session_plan(
        &output_dir,
        VULKAN_PROVIDER_FAMILY,
        &request,
        1,
    )
    .unwrap_err()
    .contains("registered SPIR-V ABI"));
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn vulkan_session_plan_accepts_registered_pair_fan_in() {
    let output_dir =
        env::temp_dir().join(format!("nsdb-vulkan-session-fan-in-{}", std::process::id()));
    let _ = fs::remove_dir_all(&output_dir);
    fs::create_dir_all(&output_dir).unwrap();
    let spirv = registered_spirv_fixture("shader.vulkan.add-pair-u32.spirv");
    fs::write(
        output_dir.join("nuis.shader.vulkan.add-pair-u32.spv"),
        &spirv,
    )
    .unwrap();
    let mut request = u32_request(
        "add-pair-u32",
        "nuis_vulkan_add_pair_u32",
        "shader.vulkan.add-pair-u32.spirv",
        "nuis.shader.vulkan.add-pair-u32.spv",
        &spirv,
    );
    use_row_major_2x2(&mut request);
    push_aux_u32_input(&mut request, "dependency");

    let plan =
        validate_vulkan_execution_session_plan(&output_dir, VULKAN_PROVIDER_FAMILY, &request, 2)
            .expect("registered pair fan-in plan");
    assert_eq!(plan.input_bindings, vec![0, 1]);
    assert_eq!(plan.output_layouts[0].binding, 2);

    request.input_bindings[1].row_stride_bytes = 4;
    assert!(validate_vulkan_execution_session_plan(
        &output_dir,
        VULKAN_PROVIDER_FAMILY,
        &request,
        2,
    )
    .unwrap_err()
    .contains("registered SPIR-V ABI"));
    request.input_bindings[1].row_stride_bytes = 8;

    request.input_bindings.pop();
    request.kernel.input_buffers.pop();
    assert!(validate_vulkan_execution_session_plan(
        &output_dir,
        VULKAN_PROVIDER_FAMILY,
        &request,
        1,
    )
    .unwrap_err()
    .contains("descriptor layout"));
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn vulkan_session_plan_accepts_independent_padded_output_layout() {
    let output_dir = env::temp_dir().join(format!(
        "nsdb-vulkan-session-padded-output-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&output_dir);
    fs::create_dir_all(&output_dir).unwrap();
    let spirv = registered_spirv_fixture("shader.vulkan.add-xor-pair-u32.spirv");
    fs::write(
        output_dir.join("nuis.shader.vulkan.add-xor-pair-u32.spv"),
        &spirv,
    )
    .unwrap();
    let mut request = u32_request(
        "add-xor-pair-u32",
        "nuis_vulkan_add_xor_pair_u32",
        "shader.vulkan.add-xor-pair-u32.spirv",
        "nuis.shader.vulkan.add-xor-pair-u32.spv",
        &spirv,
    );
    use_row_major_2x2(&mut request);
    push_aux_u32_input(&mut request, "artifact");
    let mut secondary = request.output_bindings[0].clone();
    secondary.role = "output.xor".to_owned();
    secondary.buffer = "output.xor".to_owned();
    secondary.row_stride_bytes = 12;
    secondary.byte_length = 24;
    secondary.comparison_id = "none".to_owned();
    request.output_bindings.push(secondary);

    let plan =
        validate_vulkan_execution_session_plan(&output_dir, VULKAN_PROVIDER_FAMILY, &request, 2)
            .expect("padded multi-output Vulkan plan");

    assert_eq!(plan.output_layouts.len(), 2);
    assert_eq!(plan.output_layouts[0].binding, 2);
    assert_eq!(plan.output_layouts[0].carrier_byte_length, 16);
    assert_eq!(plan.output_layouts[1].binding, 3);
    assert_eq!(plan.output_layouts[1].logical_byte_length, 16);
    assert_eq!(plan.output_layouts[1].carrier_byte_length, 24);
    assert_eq!(plan.output_layouts[1].row_byte_length, 8);
    assert_eq!(plan.output_layouts[1].row_stride_bytes, 12);
    assert_eq!(plan.output_layouts[1].row_count, 2);
    assert_eq!(
        render_vulkan_output_layout_manifest(&plan.output_layouts),
        "16:16:8:8:2,16:24:8:12:2"
    );
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn vulkan_session_plan_accepts_reduced_secondary_output_extent() {
    let output_dir = env::temp_dir().join(format!(
        "nsdb-vulkan-session-reduced-output-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&output_dir);
    fs::create_dir_all(&output_dir).unwrap();
    let spirv = registered_spirv_fixture("shader.vulkan.add-xor-pair-u32.spirv");
    fs::write(
        output_dir.join("nuis.shader.vulkan.add-xor-pair-u32.spv"),
        &spirv,
    )
    .unwrap();
    let mut request = u32_request(
        "add-xor-pair-u32",
        "nuis_vulkan_add_xor_pair_u32",
        "shader.vulkan.add-xor-pair-u32.spirv",
        "nuis.shader.vulkan.add-xor-pair-u32.spv",
        &spirv,
    );
    use_row_major_2x2(&mut request);
    push_aux_u32_input(&mut request, "artifact");
    let mut secondary = request.output_bindings[0].clone();
    secondary.role = "output.xor".to_owned();
    secondary.buffer = "output.xor".to_owned();
    secondary.shape = vec![2, 1];
    secondary.row_stride_bytes = 8;
    secondary.byte_length = 8;
    secondary.comparison_id = "none".to_owned();
    request.output_bindings.push(secondary);

    let plan =
        validate_vulkan_execution_session_plan(&output_dir, VULKAN_PROVIDER_FAMILY, &request, 2)
            .expect("reduced multi-output Vulkan plan");

    assert_eq!(plan.element_count, 4);
    assert_eq!(plan.output_layouts[0].logical_byte_length, 16);
    assert_eq!(plan.output_layouts[1].logical_byte_length, 8);
    assert_eq!(plan.output_layouts[1].carrier_byte_length, 8);
    assert_eq!(plan.output_layouts[1].row_count, 1);
    assert_eq!(
        render_vulkan_output_layout_manifest(&plan.output_layouts),
        "16:16:8:8:2,8:8:8:8:1"
    );
    fs::remove_dir_all(output_dir).unwrap();
}

fn u32_request(
    operation: &str,
    entry: &str,
    asset_id: &str,
    path: &str,
    spirv: &[u8],
) -> ProviderRequest {
    ProviderRequest {
        source: "test",
        buffer: ProviderBufferDescriptor {
            id: "input.values".to_owned(),
            element_type: "u32".to_owned(),
            layout: "tensor-contiguous".to_owned(),
            shape: vec![4],
            row_stride_bytes: 16,
            byte_length: 16,
            payload_path: "input.bin".to_owned(),
            content_hash: "0x1111111111111111".to_owned(),
        },
        kernel: ProviderKernelDescriptor {
            id: format!("shader.vulkan.{operation}"),
            operation: operation.to_owned(),
            input_buffer: "input.values".to_owned(),
            input_buffers: vec!["input.values".to_owned()],
            output_buffer: "output.values".to_owned(),
            dispatch: vec![4, 1, 1],
            scalar_bindings: vec![ProviderScalarBinding {
                name: "element_count".to_owned(),
                value_type: "u32".to_owned(),
                value: "4".to_owned(),
            }],
        },
        output_bindings: vec![ProviderOutputBinding {
            role: "output.result".to_owned(),
            buffer: "output.values".to_owned(),
            element_type: "u32".to_owned(),
            layout: "tensor-contiguous".to_owned(),
            shape: vec![4],
            row_stride_bytes: 16,
            byte_length: 16,
            comparison_id: "comparison.output.values".to_owned(),
        }],
        model_asset: None,
        code_asset: Some(ProviderCodeAssetDescriptor {
            id: asset_id.to_owned(),
            format: VULKAN_SPIRV_FORMAT.to_owned(),
            target: VULKAN_SPIRV_TARGET.to_owned(),
            entry: entry.to_owned(),
            path: path.to_owned(),
            byte_length: spirv.len(),
            digest_contract: CODE_ASSET_FNV1A64_DIGEST_CONTRACT.to_owned(),
            content_hash: fnv1a64_hex(spirv),
        }),
        output_comparison: Some(ProviderOutputComparisonDescriptor {
            id: "comparison.output.values".to_owned(),
            output_buffer: "output.values".to_owned(),
            element_type: "u32".to_owned(),
            shape: vec![4],
            expected_path: "expected.bin".to_owned(),
            expected_byte_length: 16,
            expected_content_hash: "0x2222222222222222".to_owned(),
            absolute_tolerance: "0".to_owned(),
            relative_tolerance: "0".to_owned(),
            non_finite_policy: "reject".to_owned(),
        }),
        output_comparisons: Vec::new(),
        dependencies: Vec::new(),
        input_bindings: vec![ProviderInputBinding {
            name: "input.values".to_owned(),
            source: "artifact".to_owned(),
            element_type: "u32".to_owned(),
            layout: "tensor-contiguous".to_owned(),
            shape: vec![4],
            row_stride_bytes: 16,
            byte_length: 16,
            content_hash: "0x1111111111111111".to_owned(),
            payload_path: "input.bin".to_owned(),
            producer_request_id: "none".to_owned(),
            producer_output_buffer: "none".to_owned(),
        }],
        adapter_binding: Some(ProviderAdapterBinding {
            provider_family: VULKAN_PROVIDER_FAMILY.to_owned(),
            execution_requirement: "real-device".to_owned(),
        }),
    }
}

fn push_aux_u32_input(request: &mut ProviderRequest, source: &str) {
    request.kernel.input_buffers.push("input.right".to_owned());
    let mut aux = request.input_bindings[0].clone();
    aux.name = "input.right".to_owned();
    aux.source = source.to_owned();
    if source == "dependency" {
        aux.payload_path = "none".to_owned();
        aux.producer_request_id = "shader.vulkan.chain.copy-u32".to_owned();
        aux.producer_output_buffer = "output.values".to_owned();
    }
    request.input_bindings.push(aux);
}

fn use_row_major_2x2(request: &mut ProviderRequest) {
    request.buffer.layout = "tensor-row-major".to_owned();
    request.buffer.shape = vec![2, 2];
    request.buffer.row_stride_bytes = 8;
    request.output_bindings[0].layout = "tensor-row-major".to_owned();
    request.output_bindings[0].shape = vec![2, 2];
    request.output_bindings[0].row_stride_bytes = 8;
    request
        .output_comparison
        .as_mut()
        .expect("output comparison")
        .shape = vec![2, 2];
    request.input_bindings[0].layout = "tensor-row-major".to_owned();
    request.input_bindings[0].shape = vec![2, 2];
    request.input_bindings[0].row_stride_bytes = 8;
}

fn registered_spirv_fixture(asset_id: &str) -> Vec<u8> {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader").unwrap();
    nuisc::registry::code_asset_registration_by_id(root, &manifest, asset_id)
        .unwrap()
        .expect("registered Shader SPIR-V fixture")
        .bytes
}

#[test]
fn vulkan_worker_protocol_is_request_bound() {
    let protocol = b"protocol=nuis-vulkan-spirv-provider-runner-v1\nstatus=ready\ndevice_inventory_contract=nuis-vulkan-device-inventory-v1\ndevice_inventory_count=2\ndevice_selection_contract=nuis-vulkan-device-selection-v1\ndevice_selection_status=verified\nselected_device_index=0\nselected_queue_family_index=3\ninstance_api_version=4206592\noutput_count=1\noutput_bytes=16\noutput_byte_lengths=16\noutput_hash=1\n";
    let evidence = parse_vulkan_worker_protocol(protocol).expect("selection evidence");
    assert_eq!(evidence.device_inventory_count, 2);
    assert_eq!(evidence.selected_device_index, 0);
    assert_eq!(evidence.selected_queue_family_index, 3);
    assert_eq!(evidence.output_byte_lengths, vec![16]);
    let drifted = String::from_utf8_lossy(protocol).replace("status=ready", "status=drift");
    assert!(parse_vulkan_worker_protocol(drifted.as_bytes()).is_err());
}
