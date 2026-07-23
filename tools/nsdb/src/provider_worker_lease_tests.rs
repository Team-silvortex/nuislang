use super::{
    attach_adapter_control, render_adapter_control, validate_adapter_launch,
    validate_dispatch_payload_size, ProviderWorkerAdapterLaunch, ProviderWorkerLeaseManager,
    MAX_PROVIDER_WORKER_DISPATCH_PAYLOAD_BYTES,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[test]
fn dispatch_payload_is_provider_and_request_bound() {
    let source = include_str!("provider_worker_lease.rs");
    assert!(source.contains("provider={provider_family}"));
    assert!(source.contains("request.kernel.id"));
    assert!(source.contains("operation.operation_token"));
    assert!(source.contains("capsule.capsule_token"));
    assert!(source.contains("invoker.invoker_token"));
    assert!(source.contains("output_roles"));
    assert_eq!(
        crate::provider_worker_image::PROVIDER_WORKER_IMAGE_RESOLVER_CONTRACT,
        "nuis-provider-worker-image-resolver-v1"
    );
}

#[test]
fn adapter_launch_rejects_unbound_or_frame_unsafe_identity() {
    let invalid_hash = ProviderWorkerAdapterLaunch {
        executable_path: Path::new("adapter"),
        executable_hash: "not-a-hash",
        runner_contract: "runner.v1",
        cache_contract: "cache.v1",
        cache_identity: "adapter:0x0123456789abcdef",
        cache_status: "compiled",
        arguments: &["descriptor-path:0".to_owned()],
        output_byte_length: 4,
    };
    assert!(validate_adapter_launch(Some(&invalid_hash), 1).is_err());

    let invalid_literal = ProviderWorkerAdapterLaunch {
        executable_path: Path::new("adapter"),
        executable_hash: "0x0123456789abcdef",
        runner_contract: "runner.v1",
        cache_contract: "cache.v1",
        cache_identity: "adapter:0x0123456789abcdef",
        cache_status: "compiled",
        arguments: &["literal:15\nnext".to_owned()],
        output_byte_length: 4,
    };
    assert!(validate_adapter_launch(Some(&invalid_literal), 1).is_err());

    let invalid_descriptor = ProviderWorkerAdapterLaunch {
        executable_path: Path::new("adapter"),
        executable_hash: "0x0123456789abcdef",
        runner_contract: "runner.v1",
        cache_contract: "cache.v1",
        cache_identity: "adapter:0x0123456789abcdef",
        cache_status: "compiled",
        arguments: &["descriptor-carrier:1:0:4096:42".to_owned()],
        output_byte_length: 4,
    };
    assert!(validate_adapter_launch(Some(&invalid_descriptor), 1).is_err());

    let ordered_arguments = ProviderWorkerAdapterLaunch {
        executable_path: Path::new("adapter"),
        executable_hash: "0x0123456789abcdef",
        runner_contract: "runner.v1",
        cache_contract: "cache.v1",
        cache_identity: "adapter:0x0123456789abcdef",
        cache_status: "compiled",
        arguments: &[
            "verified-path:0x0123456789abcdef:model.mlmodel".to_owned(),
            "literal:--multi".to_owned(),
            "descriptor-carrier:0:0:4096:42".to_owned(),
        ],
        output_byte_length: 4,
    };
    assert!(validate_adapter_launch(Some(&ordered_arguments), 1).is_ok());
    let control = render_adapter_control(&ordered_arguments);
    assert!(control.starts_with(
        "nuis-provider-worker-adapter-control-v1\tnuis-provider-worker-process-adapter-v4\t"
    ));
    assert_eq!(
        control.split('\t').skip(7).collect::<Vec<_>>(),
        ordered_arguments.arguments
    );
    assert!(!control.contains("adapter_argument_"));

    let wide_arguments = [
        format!("literal:{}", "a".repeat(1000)),
        format!("literal:{}", "b".repeat(1000)),
    ];
    let wide = ProviderWorkerAdapterLaunch {
        arguments: &wide_arguments,
        ..ordered_arguments
    };
    let spilled =
        attach_adapter_control("base=ready\n".to_owned(), &wide).expect("spill wide control");
    assert!(spilled
        .payload
        .contains("adapter_control_ref=nuis-provider-worker-adapter-control-carrier-v1"));
    assert!(!spilled.payload.contains("literal:"));
    assert_eq!(
        spilled.spilled_control.as_deref(),
        Some(render_adapter_control(&wide).as_bytes())
    );

    let oversized_argument = format!("literal:{}", "x".repeat(2048));
    let oversized = ProviderWorkerAdapterLaunch {
        arguments: &[oversized_argument],
        ..ordered_arguments
    };
    assert!(validate_adapter_launch(Some(&oversized), 1).is_err());
    assert!(validate_dispatch_payload_size(
        &"x".repeat(MAX_PROVIDER_WORKER_DISPATCH_PAYLOAD_BYTES + 1)
    )
    .expect_err("oversized provider control payload")
    .contains("dispatch payload is too large"));
}

#[test]
fn lease_preserves_registered_primary_and_additional_outputs() {
    let paths = LeaseFanOutPaths::new();
    fs::create_dir_all(&paths.root).expect("root");
    let input = [17u8, 29, 31, 43];
    fs::write(paths.root.join("input.bin"), input).expect("input");
    let hash = crate::provider_sample_artifact::fnv1a64_hex(&input);
    let evidence = format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.pixels;provider_buffer_element_type=u8;provider_buffer_layout=image-2d-row-major:pixel-format=gray8;provider_buffer_shape=2x2;provider_buffer_row_stride_bytes=2;provider_buffer_byte_length=4;provider_buffer_payload_path=input.bin;provider_buffer_content_hash={hash};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=provider.fan-out;provider_kernel_operation=fan-out;provider_kernel_input_buffer=input.pixels;provider_kernel_output_buffer=output.primary;provider_kernel_dispatch=2x2x1;provider_output_binding_contract=nuis-provider-output-binding-v1;provider_output_binding_count=2;provider_output_binding_0_role=output.primary;provider_output_binding_0_buffer=output.primary;provider_output_binding_1_role=output.audit;provider_output_binding_1_buffer=output.audit"
    );
    let request =
        crate::provider_request::provider_request_from_evidence(&evidence).expect("request");
    let prepared = crate::provider_prepared_input::PreparedProviderInput::new(
        &paths.root,
        &request.input_bindings[0],
        None,
        &BTreeMap::new(),
        true,
    )
    .expect("prepared input");
    let mut manager = ProviderWorkerLeaseManager::new(&paths.root);
    let receipt = manager
        .dispatch(
            "generic.device.host-simulated",
            "data:host",
            "lease:fan-out",
            0,
            &request,
            &[prepared],
            None,
        )
        .expect("worker fan-out");
    assert_eq!(receipt.worker_output_descriptor_count, 2);
    assert_eq!(receipt.worker_output_payload.len(), 24);
    assert_eq!(receipt.additional_worker_outputs.len(), 1);
    assert_eq!(receipt.additional_worker_outputs[0].role, "output.audit");
    assert_eq!(
        receipt.additional_worker_outputs[0].retention_status(),
        "verified-payload"
    );
    manager.close().expect("close");
}

struct LeaseFanOutPaths {
    root: PathBuf,
}

impl LeaseFanOutPaths {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self {
            root: std::env::temp_dir().join(format!(
                "nuis-provider-lease-fan-out-{}-{nonce}",
                std::process::id()
            )),
        }
    }
}

impl Drop for LeaseFanOutPaths {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
