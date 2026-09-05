use super::*;

fn request() -> ProviderRequest {
    let mut request = crate::provider_request::provider_request_from_evidence(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.pixels;provider_buffer_element_type=u8;provider_buffer_layout=image-2d-row-major:pixel-format=gray8;provider_buffer_shape=2x2;provider_buffer_row_stride_bytes=2;provider_buffer_byte_length=4;provider_buffer_payload_path=pixels.bin;provider_buffer_content_hash=0x1234;provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=render;provider_kernel_operation=invert;provider_kernel_input_buffer=input.pixels;provider_kernel_output_buffer=output.pixels;provider_kernel_dispatch=2x2x1;provider_kernel_scalar_bindings=max_value:u8:15"
    ).unwrap();
    request.kernel.operation = "render-rgba8".to_owned();
    request.kernel.scalar_bindings.clear();
    request.output_bindings[0].layout = "image-2d-row-major:pixel-format=rgba8".to_owned();
    request.output_bindings[0].row_stride_bytes = 8;
    request.output_bindings[0].byte_length = 16;
    request
}

#[test]
fn runtime_draw_binding_changes_only_admitted_scalars() {
    let original = request();
    let mut request = original.clone();
    let arguments = ShaderDrawArguments {
        width: 2,
        height: 2,
        vertex_count: 3,
        instance_count: 2,
    }
    .to_dispatch();
    assert_eq!(
        prepare_runtime_arguments(&mut request, Some(&arguments)).unwrap(),
        arguments
    );
    assert_eq!(render_draw_counts(&request).unwrap(), (3, 2));
    request.kernel.scalar_bindings = original.kernel.scalar_bindings.clone();
    assert_eq!(
        request, original,
        "runtime input must not replace any code or carrier authority"
    );
}

#[test]
fn invalid_runtime_draw_binding_does_not_mutate_admitted_request() {
    for (width, vertices, instances) in [(3, 3, 2), (2, 0, 1), (2, 5, 1), (2, 4, 257)] {
        let mut request = request();
        let original = request.clone();
        let arguments = ShaderDrawArguments {
            width,
            height: 2,
            vertex_count: vertices,
            instance_count: instances,
        }
        .to_dispatch();
        assert!(prepare_runtime_arguments(&mut request, Some(&arguments)).is_err());
        assert_eq!(request, original);
    }
    let mut request = request();
    let mut arguments = ShaderDrawArguments {
        width: 2,
        height: 2,
        vertex_count: 4,
        instance_count: 1,
    }
    .to_dispatch();
    arguments.scalars.insert("resource".to_owned(), 1);
    assert!(prepare_runtime_arguments(&mut request, Some(&arguments)).is_err());
}

#[test]
fn offline_defaults_and_runtime_counts_share_validation() {
    let mut request = request();
    let arguments = prepare_runtime_arguments(&mut request, None).unwrap();
    assert_eq!(arguments.scalars["vertex_count"], 4);
    assert_eq!(arguments.scalars["instance_count"], 1);
    request
        .kernel
        .scalar_bindings
        .push(request.kernel.scalar_bindings[0].clone());
    assert!(render_draw_counts(&request)
        .unwrap_err()
        .contains("duplicate"));
}

#[test]
fn storage_admission_binds_layout_and_requires_owned_payload_before_mutating_request() {
    let mut original = request();
    for (name, value) in [
        ("fragment_storage_slot", "3"),
        ("fragment_storage_count", "768"),
    ] {
        original.kernel.scalar_bindings.push(ProviderScalarBinding {
            name: name.to_owned(),
            value_type: "u64".to_owned(),
            value: value.to_owned(),
        });
    }
    let mut arguments = ShaderDrawArguments {
        width: 2,
        height: 2,
        vertex_count: 3,
        instance_count: 1,
    }
    .to_dispatch();
    let storage = yir_domain_shader::ShaderFragmentStorage {
        capability: yir_domain_shader::ShaderFragmentStorageCapability {
            slot: 3,
            element_count: 768,
        },
        upload: yir_core::provider_runtime_ipc::DispatchUpload::new(
            "u32",
            vec![768],
            vec![0; 3072],
        )
        .unwrap(),
    };
    storage.bind_dispatch(&mut arguments).unwrap();
    let mut admitted = original.clone();
    assert_eq!(
        prepare_runtime_arguments(&mut admitted, Some(&arguments)).unwrap(),
        arguments
    );
    assert!(super::super::storage::validate_upload(&admitted)
        .unwrap()
        .starts_with("3:768:0x"));
    assert_eq!(admitted.input_bindings, original.input_bindings);
    assert_eq!(admitted.code_asset, original.code_asset);
    assert_eq!(
        admitted.runtime_uploads["fragment.storage.3"]
            .payload()
            .unwrap()
            .len(),
        3072
    );
    for case in 0..7 {
        let mut request = original.clone();
        let mut invalid = arguments.clone();
        match case {
            0 => invalid.uploads.clear(),
            1 => invalid = arguments.descriptor(),
            2 => {
                invalid
                    .uploads
                    .get_mut("fragment.storage.3")
                    .unwrap()
                    .element_type = "f32".to_owned();
            }
            3 => {
                invalid.uploads.insert(
                    "fragment.storage.3".to_owned(),
                    yir_core::provider_runtime_ipc::DispatchUpload::new(
                        "u32",
                        vec![767],
                        vec![0; 3068],
                    )
                    .unwrap(),
                );
            }
            4 => {
                let upload = invalid.uploads.remove("fragment.storage.3").unwrap();
                invalid
                    .uploads
                    .insert("fragment.storage.4".to_owned(), upload);
            }
            5 => request.kernel.scalar_bindings.push(ProviderScalarBinding {
                name: "fragment_storage_slot".to_owned(),
                value_type: "u64".to_owned(),
                value: "3".to_owned(),
            }),
            _ => invalid
                .scalars
                .insert("height".to_owned(), 3)
                .map(|_| ())
                .unwrap(),
        }
        let before = request.clone();
        assert!(
            prepare_runtime_arguments(&mut request, Some(&invalid)).is_err(),
            "case {case}"
        );
        assert!(
            request == before,
            "invalid admission must not partially update request, case {case}"
        );
    }
    assert!(prepare_runtime_arguments(&mut original.clone(), None)
        .unwrap_err()
        .contains("requires runtime upload data"));
}

#[test]
fn uniform_upload_requires_exact_compiled_slot_and_typed_runtime_bytes() {
    let mut original = request();
    original.kernel.scalar_bindings.push(ProviderScalarBinding {
        name: "fragment_uniform_slot".to_owned(),
        value_type: "u64".to_owned(),
        value: "2".to_owned(),
    });
    let mut arguments = ShaderDrawArguments {
        width: 2,
        height: 2,
        vertex_count: 3,
        instance_count: 1,
    }
    .to_dispatch();
    let uniform = ShaderFragmentUniform {
        slot: 2,
        bytes: [0; 16],
    };
    uniform.bind_dispatch(&mut arguments).unwrap();
    let mut request = original.clone();
    assert_eq!(
        prepare_runtime_arguments(&mut request, Some(&arguments)).unwrap(),
        arguments
    );
    assert_eq!(
        uniform_upload(&request).unwrap(),
        format!("2:{}", "00".repeat(16))
    );
    for case in 0..6 {
        let mut request = original.clone();
        let mut invalid = arguments.clone();
        match case {
            0 => invalid.resources.clear(),
            1 => {
                invalid
                    .resources
                    .get_mut("fragment.uniform.2")
                    .unwrap()
                    .element_type = "u32".to_owned();
            }
            2 => {
                invalid
                    .resources
                    .get_mut("fragment.uniform.2")
                    .unwrap()
                    .shape = vec![2, 2];
            }
            3 => {
                invalid
                    .resources
                    .get_mut("fragment.uniform.2")
                    .unwrap()
                    .bytes[..4]
                    .copy_from_slice(&f32::INFINITY.to_le_bytes());
            }
            4 => {
                request.kernel.scalar_bindings.clear();
            }
            _ => {
                invalid.contract = "unknown".to_owned();
            }
        }
        let before = request.clone();
        assert!(prepare_runtime_arguments(&mut request, Some(&invalid)).is_err());
        assert_eq!(
            request, before,
            "failed admission must not mutate request authority"
        );
    }
    assert!(
        prepare_runtime_arguments(&mut original, None).is_err(),
        "no invented offline uniform defaults"
    );
}
