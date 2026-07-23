use super::{
    CompletedProviderOutput, CompletedProviderOutputs, PROVIDER_GRAPH_OUTPUT_OWNERSHIP_CONTRACT,
};
use crate::{
    provider_edge_transport::ProviderEdgeTransportDescriptor,
    provider_input_binding::ProviderInputBinding,
    provider_output_carrier_registry::ProviderOutputPayload,
    provider_prepared_input::PreparedProviderInput, provider_sample_payload::fnv1a64_hex,
};
use std::path::Path;

#[cfg(unix)]
use super::completed_additional_worker_outputs;
#[cfg(unix)]
use crate::{
    provider_request::provider_request_from_evidence, provider_worker_lease::ProviderWorkerOutput,
};

fn completed(role: &str, buffer: &str, payload: &[u8]) -> CompletedProviderOutput {
    CompletedProviderOutput {
        role: role.to_owned(),
        buffer: buffer.to_owned(),
        payload: ProviderOutputPayload::owned(payload.to_vec()),
        transferable: None,
    }
}

fn dependency_binding(name: &str, buffer: &str, payload: &[u8]) -> ProviderInputBinding {
    ProviderInputBinding {
        name: name.to_owned(),
        source: "dependency".to_owned(),
        element_type: "u8".to_owned(),
        shape: vec![payload.len()],
        byte_length: payload.len(),
        content_hash: fnv1a64_hex(payload),
        payload_path: "none".to_owned(),
        producer_request_id: "producer.fan-out".to_owned(),
        producer_output_buffer: buffer.to_owned(),
    }
}

fn transport(buffer: &str, input: &str) -> ProviderEdgeTransportDescriptor {
    ProviderEdgeTransportDescriptor {
        ownership_token: format!("glm:provider-edge:producer.fan-out:{buffer}->consumer:{input}"),
        staging_mode: "auto".to_owned(),
        producer_clock_evidence: "provider-clock:request-0:completed".to_owned(),
        consumer_clock_evidence: "provider-clock:request-1:dispatch-ready".to_owned(),
    }
}

#[test]
fn graph_routes_distinct_outputs_to_consumers_and_releases_all_at_close() {
    let primary = [3u8, 5, 7, 11];
    let audit = [13u8, 17, 19, 23];
    let mut completed_outputs = CompletedProviderOutputs::new();
    completed_outputs
        .insert(
            "producer.fan-out",
            completed("output.primary", "buffer.primary", &primary),
        )
        .expect("primary output");
    completed_outputs
        .insert(
            "producer.fan-out",
            completed("output.audit", "buffer.audit", &audit),
        )
        .expect("audit output");

    let primary_binding = dependency_binding("input.primary", "buffer.primary", &primary);
    let audit_binding = dependency_binding("input.audit", "buffer.audit", &audit);
    let primary_transport = transport("buffer.primary", "input.primary");
    let audit_transport = transport("buffer.audit", "input.audit");
    let primary_input = PreparedProviderInput::new(
        Path::new("."),
        &primary_binding,
        Some(&primary_transport),
        &completed_outputs,
        false,
    )
    .expect("primary consumer input");
    let audit_input = PreparedProviderInput::new(
        Path::new("."),
        &audit_binding,
        Some(&audit_transport),
        &completed_outputs,
        false,
    )
    .expect("audit consumer input");

    assert_eq!(primary_input.input().bytes(), Some(primary.as_slice()));
    assert_eq!(audit_input.input().bytes(), Some(audit.as_slice()));
    assert_eq!(
        primary_input
            .finish()
            .expect("finish primary")
            .expect("primary receipt")
            .consume_payload_hash,
        fnv1a64_hex(&primary)
    );
    assert_eq!(
        audit_input
            .finish()
            .expect("finish audit")
            .expect("audit receipt")
            .consume_payload_hash,
        fnv1a64_hex(&audit)
    );

    let close = completed_outputs.close();
    assert_eq!(close.contract, PROVIDER_GRAPH_OUTPUT_OWNERSHIP_CONTRACT);
    assert_eq!(close.released_output_count, 2);
    assert!(close.released_output_roles.contains("output.primary"));
    assert!(close.released_output_roles.contains("output.audit"));
}

#[cfg(unix)]
#[test]
fn worker_additional_output_becomes_registered_graph_output() {
    let evidence = "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.bytes;provider_buffer_element_type=u8;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=4;provider_buffer_row_stride_bytes=4;provider_buffer_byte_length=4;provider_buffer_payload_path=input.bin;provider_buffer_content_hash=0x1234;provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=producer.fan-out;provider_kernel_operation=fan-out;provider_kernel_input_buffer=input.bytes;provider_kernel_output_buffer=buffer.primary;provider_kernel_dispatch=4x1x1;provider_output_binding_contract=nuis-provider-output-binding-v1;provider_output_binding_count=2;provider_output_binding_0_role=output.primary;provider_output_binding_0_buffer=buffer.primary;provider_output_binding_0_element_type=u8;provider_output_binding_0_shape=4;provider_output_binding_0_byte_length=4;provider_output_binding_0_comparison_id=none;provider_output_binding_1_role=output.audit;provider_output_binding_1_buffer=buffer.audit;provider_output_binding_1_element_type=u64;provider_output_binding_1_shape=3;provider_output_binding_1_byte_length=24;provider_output_binding_1_comparison_id=none";
    let request = provider_request_from_evidence(evidence).expect("multi-output request");
    let payload = [29u64, 31, 37]
        .into_iter()
        .flat_map(u64::to_le_bytes)
        .collect::<Vec<_>>();
    let outputs = completed_additional_worker_outputs(
        &request,
        vec![ProviderWorkerOutput {
            role: "output.audit".to_owned(),
            byte_length: payload.len(),
            payload_hash: fnv1a64_hex(&payload),
            payload: payload.clone(),
            result: None,
        }],
    )
    .expect("bound graph output");

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].role, "output.audit");
    assert_eq!(outputs[0].buffer, "buffer.audit");
    assert_eq!(request.output_bindings[1].element_type, "u64");
    assert_eq!(request.output_bindings[1].shape, [3]);
    assert_eq!(request.output_bindings[1].byte_length, 24);
    assert_eq!(outputs[0].payload.as_bytes(), payload);
}
