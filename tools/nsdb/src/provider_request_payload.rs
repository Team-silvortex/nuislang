use crate::{
    provider_edge_transport::PROVIDER_EDGE_TRANSPORT_CONTRACT,
    provider_request::{
        provider_request_collection_from_evidence, provider_request_from_evidence, ProviderRequest,
        PROVIDER_BUFFER_DESCRIPTOR_CONTRACT, PROVIDER_KERNEL_DESCRIPTOR_CONTRACT,
        PROVIDER_MODEL_ASSET_DESCRIPTOR_CONTRACT, PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT,
        PROVIDER_REQUEST_COLLECTION_CONTRACT, PROVIDER_REQUEST_DEPENDENCY_CONTRACT,
    },
    provider_sample_artifact::fnv1a64_hex,
};

pub(crate) fn render_provider_request_evidence(input_evidence: &str) -> String {
    let Some(request) = provider_request_from_evidence(input_evidence) else {
        return String::new();
    };
    let mut out = String::new();
    if let Some(collection) = provider_request_collection_from_evidence(input_evidence) {
        push_toml_string(
            &mut out,
            "provider_request_collection_contract",
            PROVIDER_REQUEST_COLLECTION_CONTRACT,
        );
        push_toml_string(
            &mut out,
            "provider_request_count",
            &collection.requests.len().to_string(),
        );
        push_toml_string(
            &mut out,
            "provider_request_order",
            &collection
                .requests
                .iter()
                .map(|request| request.kernel.id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        push_toml_string(
            &mut out,
            "provider_request_adapter_order",
            &collection
                .requests
                .iter()
                .map(|request| {
                    request
                        .adapter_binding
                        .as_ref()
                        .map(|binding| binding.provider_family.as_str())
                        .unwrap_or("record-provider")
                })
                .collect::<Vec<_>>()
                .join(","),
        );
        let dependency_edges = collection
            .requests
            .iter()
            .flat_map(|request| {
                request.dependencies.iter().map(|dependency| {
                    format!(
                        "{}.{}->{}.{}",
                        dependency.producer_request_id,
                        dependency.producer_output_buffer,
                        request.kernel.id,
                        dependency.consumer_input_buffer
                    )
                })
            })
            .collect::<Vec<_>>();
        push_toml_string(
            &mut out,
            "provider_request_dependency_contract",
            PROVIDER_REQUEST_DEPENDENCY_CONTRACT,
        );
        push_toml_string(
            &mut out,
            "provider_request_dependency_edge_count",
            &dependency_edges.len().to_string(),
        );
        push_toml_string(
            &mut out,
            "provider_request_dependency_edges",
            &dependency_edges.join(","),
        );
        push_toml_string(
            &mut out,
            "provider_request_dependency_graph_hash",
            &fnv1a64_hex(dependency_edges.join(";").as_bytes()),
        );
        let transports = collection
            .requests
            .iter()
            .flat_map(|request| request.dependencies.iter())
            .filter_map(|dependency| dependency.transport.as_ref())
            .collect::<Vec<_>>();
        push_toml_string(
            &mut out,
            "provider_edge_transport_contract",
            PROVIDER_EDGE_TRANSPORT_CONTRACT,
        );
        push_toml_string(
            &mut out,
            "provider_edge_transport_count",
            &transports.len().to_string(),
        );
        for (name, values) in [
            (
                "ownership_tokens",
                transports
                    .iter()
                    .map(|item| item.ownership_token.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "staging_modes",
                transports
                    .iter()
                    .map(|item| item.staging_mode.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "producer_clock_evidence",
                transports
                    .iter()
                    .map(|item| item.producer_clock_evidence.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "consumer_clock_evidence",
                transports
                    .iter()
                    .map(|item| item.consumer_clock_evidence.as_str())
                    .collect::<Vec<_>>(),
            ),
        ] {
            push_toml_string(
                &mut out,
                &format!("provider_edge_transport_{name}"),
                &values.join(","),
            );
        }
    }
    push_provider_request_summary(&mut out, &request);
    out
}

fn push_provider_request_summary(out: &mut String, request: &ProviderRequest) {
    push_toml_string(out, "provider_request_source", request.source);
    push_toml_string(
        out,
        "provider_buffer_descriptor_contract",
        PROVIDER_BUFFER_DESCRIPTOR_CONTRACT,
    );
    push_toml_string(out, "provider_buffer_id", &request.buffer.id);
    push_toml_string(
        out,
        "provider_buffer_element_type",
        &request.buffer.element_type,
    );
    push_toml_string(out, "provider_buffer_layout", &request.buffer.layout);
    push_toml_string(
        out,
        "provider_kernel_descriptor_contract",
        PROVIDER_KERNEL_DESCRIPTOR_CONTRACT,
    );
    push_toml_string(out, "provider_kernel_id", &request.kernel.id);
    push_toml_string(out, "provider_kernel_operation", &request.kernel.operation);
    push_toml_string(
        out,
        "provider_kernel_input_buffer",
        &request.kernel.input_buffer,
    );
    push_toml_string(
        out,
        "provider_kernel_output_buffer",
        &request.kernel.output_buffer,
    );
    push_toml_string(
        out,
        "provider_output_binding_contract",
        crate::provider_request::PROVIDER_OUTPUT_BINDING_CONTRACT,
    );
    push_toml_string(
        out,
        "provider_output_binding_count",
        &request.output_bindings.len().to_string(),
    );
    for (index, binding) in request.output_bindings.iter().enumerate() {
        push_toml_string(
            out,
            &format!("provider_output_binding_{index}_role"),
            &binding.role,
        );
        push_toml_string(
            out,
            &format!("provider_output_binding_{index}_buffer"),
            &binding.buffer,
        );
        push_toml_string(
            out,
            &format!("provider_output_binding_{index}_element_type"),
            &binding.element_type,
        );
        push_toml_string(
            out,
            &format!("provider_output_binding_{index}_shape"),
            &binding
                .shape
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join("x"),
        );
        push_toml_string(
            out,
            &format!("provider_output_binding_{index}_byte_length"),
            &binding.byte_length.to_string(),
        );
        push_toml_string(
            out,
            &format!("provider_output_binding_{index}_comparison_id"),
            &binding.comparison_id,
        );
    }
    push_toml_string(
        out,
        "provider_input_binding_contract",
        crate::provider_input_binding::PROVIDER_INPUT_BINDING_CONTRACT,
    );
    push_toml_string(
        out,
        "provider_input_binding_count",
        &request.input_bindings.len().to_string(),
    );
    for (index, binding) in request.input_bindings.iter().enumerate() {
        let prefix = format!("provider_input_binding_{index}_");
        for (name, value) in [
            ("name", binding.name.as_str()),
            ("source", binding.source.as_str()),
            ("element_type", binding.element_type.as_str()),
            ("content_hash", binding.content_hash.as_str()),
            ("payload_path", binding.payload_path.as_str()),
            ("producer_request_id", binding.producer_request_id.as_str()),
            (
                "producer_output_buffer",
                binding.producer_output_buffer.as_str(),
            ),
        ] {
            push_toml_string(out, &format!("{prefix}{name}"), value);
        }
        push_toml_string(
            out,
            &format!("{prefix}shape"),
            &binding
                .shape
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join("x"),
        );
        push_toml_string(
            out,
            &format!("{prefix}byte_length"),
            &binding.byte_length.to_string(),
        );
    }
    if let Some(model) = &request.model_asset {
        push_toml_string(
            out,
            "provider_model_asset_descriptor_contract",
            PROVIDER_MODEL_ASSET_DESCRIPTOR_CONTRACT,
        );
        push_toml_string(out, "provider_model_asset_id", &model.id);
        push_toml_string(out, "provider_model_asset_format", &model.format);
        push_toml_string(out, "provider_model_asset_path", &model.path);
        push_toml_string(
            out,
            "provider_model_asset_byte_length",
            &model.byte_length.to_string(),
        );
        push_toml_string(
            out,
            "provider_model_asset_content_hash",
            &model.content_hash,
        );
        push_toml_string(
            out,
            "provider_model_asset_input_feature",
            &model.input_feature,
        );
        push_toml_string(
            out,
            "provider_model_asset_input_features",
            &model.input_features.join(","),
        );
        push_toml_string(
            out,
            "provider_model_asset_output_feature",
            &model.output_feature,
        );
    }
    if let Some(comparison) = &request.output_comparison {
        push_toml_string(
            out,
            "provider_output_comparison_descriptor_contract",
            PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT,
        );
        push_toml_string(
            out,
            "provider_output_comparison_output_buffer",
            &comparison.output_buffer,
        );
        push_toml_string(
            out,
            "provider_output_comparison_element_type",
            &comparison.element_type,
        );
        push_toml_string(
            out,
            "provider_output_comparison_shape",
            &comparison
                .shape
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join("x"),
        );
        push_toml_string(
            out,
            "provider_output_comparison_expected_path",
            &comparison.expected_path,
        );
        push_toml_string(
            out,
            "provider_output_comparison_expected_content_hash",
            &comparison.expected_content_hash,
        );
        push_toml_string(
            out,
            "provider_output_comparison_absolute_tolerance",
            &comparison.absolute_tolerance,
        );
        push_toml_string(
            out,
            "provider_output_comparison_relative_tolerance",
            &comparison.relative_tolerance,
        );
        push_toml_string(
            out,
            "provider_output_comparison_non_finite_policy",
            &comparison.non_finite_policy,
        );
    }
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    out.push_str("\"\n");
}
