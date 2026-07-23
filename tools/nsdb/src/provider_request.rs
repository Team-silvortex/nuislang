use crate::provider_adapter_binding::{parse_adapter_binding, ProviderAdapterBinding};
use crate::provider_edge_transport::{
    parse_edge_transport, validate_dependency_transport, ProviderEdgeTransportDescriptor,
};
use crate::provider_input_binding::{
    parse_input_bindings, validate_dependency_binding, validate_input_bindings,
    ProviderInputBinding,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PROVIDER_BUFFER_DESCRIPTOR_CONTRACT: &str = "nuis-provider-buffer-descriptor-v1";
pub(crate) const PROVIDER_KERNEL_DESCRIPTOR_CONTRACT: &str = "nuis-provider-kernel-descriptor-v1";
pub(crate) const PROVIDER_MODEL_ASSET_DESCRIPTOR_CONTRACT: &str =
    "nuis-provider-model-asset-descriptor-v1";
pub(crate) const PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT: &str =
    "nuis-provider-output-comparison-descriptor-v1";
pub(crate) const PROVIDER_OUTPUT_BINDING_CONTRACT: &str = "nuis-provider-output-binding-v1";
pub(crate) const PROVIDER_REQUEST_DEPENDENCY_CONTRACT: &str = "nuis-provider-request-dependency-v1";
pub(crate) const PROVIDER_REQUEST_COLLECTION_CONTRACT: &str = "nuis-provider-request-collection-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderBufferDescriptor {
    pub(crate) id: String,
    pub(crate) element_type: String,
    pub(crate) layout: String,
    pub(crate) shape: Vec<usize>,
    pub(crate) row_stride_bytes: usize,
    pub(crate) byte_length: usize,
    pub(crate) payload_path: String,
    pub(crate) content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderScalarBinding {
    pub(crate) name: String,
    pub(crate) value_type: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderKernelDescriptor {
    pub(crate) id: String,
    pub(crate) operation: String,
    pub(crate) input_buffer: String,
    pub(crate) input_buffers: Vec<String>,
    pub(crate) output_buffer: String,
    pub(crate) dispatch: Vec<usize>,
    pub(crate) scalar_bindings: Vec<ProviderScalarBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderModelAssetDescriptor {
    pub(crate) id: String,
    pub(crate) format: String,
    pub(crate) path: String,
    pub(crate) byte_length: usize,
    pub(crate) content_hash: String,
    pub(crate) input_feature: String,
    pub(crate) input_features: Vec<String>,
    pub(crate) output_feature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderOutputComparisonDescriptor {
    pub(crate) output_buffer: String,
    pub(crate) element_type: String,
    pub(crate) shape: Vec<usize>,
    pub(crate) expected_path: String,
    pub(crate) expected_byte_length: usize,
    pub(crate) expected_content_hash: String,
    pub(crate) absolute_tolerance: String,
    pub(crate) relative_tolerance: String,
    pub(crate) non_finite_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ProviderRequestDependency {
    pub(crate) producer_request_id: String,
    pub(crate) producer_output_buffer: String,
    pub(crate) consumer_input_buffer: String,
    pub(crate) transport: Option<ProviderEdgeTransportDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderOutputBinding {
    pub(crate) role: String,
    pub(crate) buffer: String,
    pub(crate) element_type: String,
    pub(crate) shape: Vec<usize>,
    pub(crate) byte_length: usize,
    pub(crate) comparison_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRequest {
    pub(crate) source: &'static str,
    pub(crate) buffer: ProviderBufferDescriptor,
    pub(crate) kernel: ProviderKernelDescriptor,
    pub(crate) output_bindings: Vec<ProviderOutputBinding>,
    pub(crate) model_asset: Option<ProviderModelAssetDescriptor>,
    pub(crate) output_comparison: Option<ProviderOutputComparisonDescriptor>,
    pub(crate) dependencies: Vec<ProviderRequestDependency>,
    pub(crate) input_bindings: Vec<ProviderInputBinding>,
    pub(crate) adapter_binding: Option<ProviderAdapterBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRequestCollection {
    pub(crate) source: &'static str,
    pub(crate) requests: Vec<ProviderRequest>,
}

impl ProviderRequest {
    pub(crate) fn scalar_u8(&self, name: &str) -> Option<u8> {
        self.kernel
            .scalar_bindings
            .iter()
            .find(|binding| binding.name == name && binding.value_type == "u8")?
            .value
            .parse()
            .ok()
    }

    pub(crate) fn scalar_f32(&self, name: &str) -> Option<f32> {
        self.kernel
            .scalar_bindings
            .iter()
            .find(|binding| binding.name == name && binding.value_type == "f32")?
            .value
            .parse()
            .ok()
    }
}

pub(crate) fn provider_request_from_evidence(input_evidence: &str) -> Option<ProviderRequest> {
    provider_request_collection_from_evidence(input_evidence)?
        .requests
        .into_iter()
        .next()
}

pub(crate) fn provider_request_collection_from_evidence(
    input_evidence: &str,
) -> Option<ProviderRequestCollection> {
    parse_registered_collection(input_evidence).or_else(|| {
        parse_registered_request(input_evidence)
            .or_else(|| parse_legacy_pixelmagic_request(input_evidence))
            .map(|request| ProviderRequestCollection {
                source: "single-request-compatibility",
                requests: vec![request],
            })
    })
}

fn parse_registered_collection(input_evidence: &str) -> Option<ProviderRequestCollection> {
    let fields = evidence_fields(input_evidence);
    (fields.get("provider_request_collection_contract")? == PROVIDER_REQUEST_COLLECTION_CONTRACT)
        .then_some(())?;
    let count = fields
        .get("provider_request_count")?
        .parse::<usize>()
        .ok()?;
    (1..=64).contains(&count).then_some(())?;
    let requests = (0..count)
        .map(|index| {
            build_request(
                "registered-collection",
                &fields,
                &format!("provider_request_{index}_buffer_"),
                &format!("provider_request_{index}_kernel_"),
                &format!("provider_request_{index}_output_binding_"),
                &format!("provider_request_{index}_model_asset_"),
                &format!("provider_request_{index}_output_comparison_"),
                &format!("provider_request_{index}_dependency_"),
                &format!("provider_request_{index}_input_binding_"),
                &format!("provider_request_{index}_adapter_binding_"),
            )
        })
        .collect::<Option<Vec<_>>>()?;
    validate_collection_dependencies(&requests).then_some(ProviderRequestCollection {
        source: "registered-collection",
        requests,
    })
}

fn parse_registered_request(input_evidence: &str) -> Option<ProviderRequest> {
    let fields = evidence_fields(input_evidence);
    (fields.get("provider_buffer_descriptor_contract")?.as_str()
        == PROVIDER_BUFFER_DESCRIPTOR_CONTRACT)
        .then_some(())?;
    (fields.get("provider_kernel_descriptor_contract")?.as_str()
        == PROVIDER_KERNEL_DESCRIPTOR_CONTRACT)
        .then_some(())?;
    build_request(
        "registered-descriptors",
        &fields,
        "provider_buffer_",
        "provider_kernel_",
        "provider_output_binding_",
        "provider_model_asset_",
        "provider_output_comparison_",
        "provider_dependency_",
        "provider_input_binding_",
        "provider_adapter_binding_",
    )
}

fn parse_legacy_pixelmagic_request(input_evidence: &str) -> Option<ProviderRequest> {
    let fields = evidence_fields(input_evidence);
    let width = fields.get("pixel_width")?.parse().ok()?;
    let height = fields.get("pixel_height")?.parse().ok()?;
    let stride = fields.get("pixel_stride")?.parse().ok()?;
    let byte_length = fields.get("pixel_payload_bytes")?.parse().ok()?;
    let max_value = fields.get("pixel_max_value")?.clone();
    validate_request(ProviderRequest {
        source: "legacy-pixelmagic-evidence",
        buffer: ProviderBufferDescriptor {
            id: "input.pixels".to_owned(),
            element_type: "u8".to_owned(),
            layout: format!(
                "image-2d-row-major:pixel-format={}",
                fields.get("pixel_format")?
            ),
            shape: vec![width, height],
            row_stride_bytes: stride,
            byte_length,
            payload_path: fields.get("pixel_payload_path")?.clone(),
            content_hash: fields.get("pixel_payload_hash")?.clone(),
        },
        kernel: ProviderKernelDescriptor {
            id: "pixelmagic.gray8.invert".to_owned(),
            operation: fields.get("pixel_operation")?.clone(),
            input_buffer: "input.pixels".to_owned(),
            input_buffers: vec!["input.pixels".to_owned()],
            output_buffer: "output.pixels".to_owned(),
            dispatch: vec![width, height, 1],
            scalar_bindings: vec![ProviderScalarBinding {
                name: "max_value".to_owned(),
                value_type: "u8".to_owned(),
                value: max_value,
            }],
        },
        output_bindings: vec![ProviderOutputBinding {
            role: "output.result".to_owned(),
            buffer: "output.pixels".to_owned(),
            element_type: "u8".to_owned(),
            shape: vec![width, height],
            byte_length,
            comparison_id: "none".to_owned(),
        }],
        model_asset: None,
        output_comparison: None,
        dependencies: Vec::new(),
        input_bindings: vec![ProviderInputBinding {
            name: "input.pixels".to_owned(),
            source: "artifact".to_owned(),
            element_type: "u8".to_owned(),
            shape: vec![width, height],
            byte_length,
            content_hash: fields.get("pixel_payload_hash")?.clone(),
            payload_path: fields.get("pixel_payload_path")?.clone(),
            producer_request_id: "none".to_owned(),
            producer_output_buffer: "none".to_owned(),
        }],
        adapter_binding: None,
    })
}

fn build_request(
    source: &'static str,
    fields: &BTreeMap<String, String>,
    buffer_prefix: &str,
    kernel_prefix: &str,
    output_binding_prefix: &str,
    model_prefix: &str,
    comparison_prefix: &str,
    dependency_prefix: &str,
    input_binding_prefix: &str,
    adapter_binding_prefix: &str,
) -> Option<ProviderRequest> {
    let buffer = ProviderBufferDescriptor {
        id: field(fields, buffer_prefix, "id")?.clone(),
        element_type: field(fields, buffer_prefix, "element_type")?.clone(),
        layout: field(fields, buffer_prefix, "layout")?.clone(),
        shape: parse_dimensions(field(fields, buffer_prefix, "shape")?)?,
        row_stride_bytes: field(fields, buffer_prefix, "row_stride_bytes")?
            .parse()
            .ok()?,
        byte_length: field(fields, buffer_prefix, "byte_length")?.parse().ok()?,
        payload_path: field(fields, buffer_prefix, "payload_path")?.clone(),
        content_hash: field(fields, buffer_prefix, "content_hash")?.clone(),
    };
    let input_buffer = field(fields, kernel_prefix, "input_buffer")?.clone();
    let kernel = ProviderKernelDescriptor {
        id: field(fields, kernel_prefix, "id")?.clone(),
        operation: field(fields, kernel_prefix, "operation")?.clone(),
        input_buffer: input_buffer.clone(),
        input_buffers: match field(fields, kernel_prefix, "input_buffers") {
            Some(value) => parse_nonempty_list(value)?,
            None => vec![input_buffer],
        },
        output_buffer: field(fields, kernel_prefix, "output_buffer")?.clone(),
        dispatch: parse_dimensions(field(fields, kernel_prefix, "dispatch")?)?,
        scalar_bindings: match field(fields, kernel_prefix, "scalar_bindings") {
            Some(value) => parse_scalar_bindings(value)?,
            None => Vec::new(),
        },
    };
    let model_asset = parse_model_asset(fields, model_prefix)?;
    let output_comparison = parse_output_comparison(fields, comparison_prefix)?;
    let output_bindings = parse_output_bindings(
        fields,
        output_binding_prefix,
        &kernel.output_buffer,
        &buffer,
        output_comparison.as_ref(),
    )?;
    let dependencies = parse_dependencies(fields, dependency_prefix)?;
    let input_bindings =
        parse_input_bindings(fields, input_binding_prefix, &buffer, &dependencies)?;
    let adapter_binding = parse_adapter_binding(fields, adapter_binding_prefix)?;
    validate_request(ProviderRequest {
        source,
        buffer,
        kernel,
        output_bindings,
        model_asset,
        output_comparison,
        dependencies,
        input_bindings,
        adapter_binding,
    })
}

fn parse_output_bindings(
    fields: &BTreeMap<String, String>,
    prefix: &str,
    compatibility_output_buffer: &str,
    buffer: &ProviderBufferDescriptor,
    comparison: Option<&ProviderOutputComparisonDescriptor>,
) -> Option<Vec<ProviderOutputBinding>> {
    let compatibility_element_type = comparison
        .map(|value| value.element_type.as_str())
        .unwrap_or(&buffer.element_type);
    let compatibility_shape = comparison
        .map(|value| value.shape.as_slice())
        .unwrap_or(&buffer.shape);
    let compatibility_byte_length = comparison
        .map(|value| value.expected_byte_length)
        .unwrap_or(buffer.byte_length);
    let compatibility_comparison_id = comparison
        .map(|value| format!("comparison.{}", value.output_buffer))
        .unwrap_or_else(|| "none".to_owned());
    let Some(contract) = field(fields, prefix, "contract") else {
        return Some(vec![ProviderOutputBinding {
            role: "output.result".to_owned(),
            buffer: compatibility_output_buffer.to_owned(),
            element_type: compatibility_element_type.to_owned(),
            shape: compatibility_shape.to_vec(),
            byte_length: compatibility_byte_length,
            comparison_id: compatibility_comparison_id,
        }]);
    };
    (contract == PROVIDER_OUTPUT_BINDING_CONTRACT).then_some(())?;
    let count = field(fields, prefix, "count")?.parse::<usize>().ok()?;
    (1..=8).contains(&count).then_some(())?;
    (0..count)
        .map(|index| {
            let item_prefix = format!("{prefix}{index}_");
            Some(ProviderOutputBinding {
                role: field(fields, &item_prefix, "role")?.clone(),
                buffer: field(fields, &item_prefix, "buffer")?.clone(),
                element_type: field(fields, &item_prefix, "element_type")
                    .map(String::as_str)
                    .unwrap_or(compatibility_element_type)
                    .to_owned(),
                shape: field(fields, &item_prefix, "shape")
                    .map(|value| parse_dimensions(value))
                    .unwrap_or_else(|| Some(compatibility_shape.to_vec()))?,
                byte_length: field(fields, &item_prefix, "byte_length")
                    .map(|value| value.parse().ok())
                    .unwrap_or(Some(compatibility_byte_length))?,
                comparison_id: field(fields, &item_prefix, "comparison_id")
                    .cloned()
                    .unwrap_or_else(|| {
                        if index == 0 {
                            compatibility_comparison_id.clone()
                        } else {
                            "none".to_owned()
                        }
                    }),
            })
        })
        .collect()
}

fn parse_model_asset(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<Option<ProviderModelAssetDescriptor>> {
    let Some(contract) = field(fields, prefix, "descriptor_contract") else {
        return Some(None);
    };
    (contract == PROVIDER_MODEL_ASSET_DESCRIPTOR_CONTRACT).then_some(())?;
    let input_feature = field(fields, prefix, "input_feature")?.clone();
    Some(Some(ProviderModelAssetDescriptor {
        id: field(fields, prefix, "id")?.clone(),
        format: field(fields, prefix, "format")?.clone(),
        path: field(fields, prefix, "path")?.clone(),
        byte_length: field(fields, prefix, "byte_length")?.parse().ok()?,
        content_hash: field(fields, prefix, "content_hash")?.clone(),
        input_feature: input_feature.clone(),
        input_features: match field(fields, prefix, "input_features") {
            Some(value) => parse_nonempty_list(value)?,
            None => vec![input_feature],
        },
        output_feature: field(fields, prefix, "output_feature")?.clone(),
    }))
}

fn parse_output_comparison(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<Option<ProviderOutputComparisonDescriptor>> {
    let Some(contract) = field(fields, prefix, "descriptor_contract") else {
        return Some(None);
    };
    (contract == PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT).then_some(())?;
    Some(Some(ProviderOutputComparisonDescriptor {
        output_buffer: field(fields, prefix, "output_buffer")?.clone(),
        element_type: field(fields, prefix, "element_type")?.clone(),
        shape: parse_dimensions(field(fields, prefix, "shape")?)?,
        expected_path: field(fields, prefix, "expected_path")?.clone(),
        expected_byte_length: field(fields, prefix, "expected_byte_length")?
            .parse()
            .ok()?,
        expected_content_hash: field(fields, prefix, "expected_content_hash")?.clone(),
        absolute_tolerance: field(fields, prefix, "absolute_tolerance")?.clone(),
        relative_tolerance: field(fields, prefix, "relative_tolerance")?.clone(),
        non_finite_policy: field(fields, prefix, "non_finite_policy")?.clone(),
    }))
}

fn parse_dependencies(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<Vec<ProviderRequestDependency>> {
    let Some(contract) = field(fields, prefix, "contract") else {
        return field(fields, prefix, "count").is_none().then(Vec::new);
    };
    (contract == PROVIDER_REQUEST_DEPENDENCY_CONTRACT).then_some(())?;
    let count = field(fields, prefix, "count")?.parse::<usize>().ok()?;
    (count <= 64).then_some(())?;
    (0..count)
        .map(|index| {
            let edge_prefix = format!("{prefix}{index}_");
            Some(ProviderRequestDependency {
                producer_request_id: field(fields, &edge_prefix, "producer_request_id")?.clone(),
                producer_output_buffer: field(fields, &edge_prefix, "producer_output_buffer")?
                    .clone(),
                consumer_input_buffer: field(fields, &edge_prefix, "consumer_input_buffer")?
                    .clone(),
                transport: parse_edge_transport(fields, &edge_prefix)?,
            })
        })
        .collect()
}

fn validate_collection_dependencies(requests: &[ProviderRequest]) -> bool {
    let positions = requests
        .iter()
        .enumerate()
        .map(|(index, request)| (request.kernel.id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    if positions.len() != requests.len() || !dependency_graph_is_acyclic(requests, &positions) {
        return false;
    }
    requests
        .iter()
        .enumerate()
        .all(|(consumer_index, request)| {
            let unique = request.dependencies.iter().collect::<BTreeSet<_>>();
            unique.len() == request.dependencies.len()
                && request.dependencies.iter().all(|dependency| {
                    positions
                        .get(dependency.producer_request_id.as_str())
                        .is_some_and(|producer_index| {
                            *producer_index < consumer_index
                                && requests[*producer_index]
                                    .output_bindings
                                    .iter()
                                    .any(|binding| {
                                        binding.buffer == dependency.producer_output_buffer
                                    })
                                && request
                                    .kernel
                                    .input_buffers
                                    .contains(&dependency.consumer_input_buffer)
                                && validate_dependency_binding(
                                    &requests[*producer_index],
                                    request,
                                    dependency,
                                )
                                && validate_dependency_transport(
                                    &requests[*producer_index],
                                    *producer_index,
                                    request,
                                    consumer_index,
                                    dependency,
                                )
                        })
                })
        })
}

fn dependency_graph_is_acyclic(
    requests: &[ProviderRequest],
    positions: &BTreeMap<&str, usize>,
) -> bool {
    fn visit(
        index: usize,
        requests: &[ProviderRequest],
        positions: &BTreeMap<&str, usize>,
        states: &mut [u8],
    ) -> bool {
        if states[index] == 1 {
            return false;
        }
        if states[index] == 2 {
            return true;
        }
        states[index] = 1;
        for dependency in &requests[index].dependencies {
            let Some(producer) = positions.get(dependency.producer_request_id.as_str()) else {
                return false;
            };
            if !visit(*producer, requests, positions, states) {
                return false;
            }
        }
        states[index] = 2;
        true
    }

    let mut states = vec![0u8; requests.len()];
    (0..requests.len()).all(|index| visit(index, requests, positions, &mut states))
}

fn validate_request(request: ProviderRequest) -> Option<ProviderRequest> {
    let element_width = match request.buffer.element_type.as_str() {
        "u8" => 1,
        "f32" => 4,
        _ => return None,
    };
    let element_count = request
        .buffer
        .shape
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))?;
    let expected_bytes = if request.buffer.layout.starts_with("image-2d-row-major") {
        let [width, height] = request.buffer.shape.as_slice() else {
            return None;
        };
        if request.buffer.row_stride_bytes < width.checked_mul(element_width)? {
            return None;
        }
        request.buffer.row_stride_bytes.checked_mul(*height)?
    } else if request.buffer.layout == "tensor-contiguous" {
        element_count.checked_mul(element_width)?
    } else {
        return None;
    };
    let model_asset_valid = request.model_asset.as_ref().is_none_or(|asset| {
        asset.format == "coreml-specification"
            && !asset.id.is_empty()
            && !asset.path.is_empty()
            && asset.byte_length > 0
            && asset.content_hash.starts_with("0x")
            && asset.input_feature == request.kernel.input_buffer
            && asset.input_features == request.kernel.input_buffers
            && asset.output_feature == request.kernel.output_buffer
    });
    let comparison_valid = request.output_comparison.as_ref().is_none_or(|comparison| {
        let tolerance_valid = |value: &str| {
            value
                .parse::<f64>()
                .is_ok_and(|value| value.is_finite() && value >= 0.0)
        };
        let comparison_elements = comparison
            .shape
            .iter()
            .try_fold(1usize, |count, dimension| count.checked_mul(*dimension));
        comparison.output_buffer == request.kernel.output_buffer
            && comparison.element_type == "f32"
            && comparison.shape.iter().all(|dimension| *dimension > 0)
            && comparison_elements
                .and_then(|count| count.checked_mul(4))
                .is_some_and(|bytes| bytes == comparison.expected_byte_length)
            && !comparison.expected_path.is_empty()
            && comparison.expected_content_hash.starts_with("0x")
            && tolerance_valid(&comparison.absolute_tolerance)
            && tolerance_valid(&comparison.relative_tolerance)
            && matches!(comparison.non_finite_policy.as_str(), "reject" | "equal")
    });
    let output_bindings_valid = {
        let mut roles = BTreeSet::new();
        let mut buffers = BTreeSet::new();
        !request.output_bindings.is_empty()
            && request.output_bindings.len() <= 8
            && request.output_bindings[0].buffer == request.kernel.output_buffer
            && request.output_bindings.iter().all(|binding| {
                is_output_role(&binding.role)
                    && !binding.buffer.is_empty()
                    && output_binding_semantics_are_valid(binding)
                    && roles.insert(binding.role.as_str())
                    && buffers.insert(binding.buffer.as_str())
            })
    };
    (request.buffer.id == request.kernel.input_buffer
        && !request.kernel.output_buffer.is_empty()
        && request.buffer.shape.iter().all(|dimension| *dimension > 0)
        && request.buffer.byte_length == expected_bytes
        && request.buffer.content_hash.starts_with("0x")
        && request
            .kernel
            .dispatch
            .iter()
            .all(|dimension| *dimension > 0)
        && request.kernel.scalar_bindings.iter().all(|binding| {
            !binding.name.is_empty() && !binding.value_type.is_empty() && !binding.value.is_empty()
        })
        && model_asset_valid
        && comparison_valid
        && output_bindings_valid
        && validate_input_bindings(&request))
    .then_some(request)
}

fn output_binding_semantics_are_valid(binding: &ProviderOutputBinding) -> bool {
    let element_width = match binding.element_type.as_str() {
        "u8" => 1usize,
        "f32" | "u32" | "i32" => 4usize,
        "u64" | "i64" | "f64" => 8usize,
        _ => return false,
    };
    let expected_bytes = binding
        .shape
        .iter()
        .try_fold(element_width, |bytes, dimension| {
            bytes.checked_mul(*dimension)
        });
    !binding.shape.is_empty()
        && binding.shape.iter().all(|dimension| *dimension > 0)
        && expected_bytes == Some(binding.byte_length)
        && (binding.comparison_id == "none" || binding.comparison_id.starts_with("comparison."))
}

fn is_output_role(value: &str) -> bool {
    value.starts_with("output.")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn evidence_fields(input_evidence: &str) -> BTreeMap<String, String> {
    input_evidence
        .split(';')
        .filter_map(|field| field.split_once('='))
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn field<'a>(fields: &'a BTreeMap<String, String>, prefix: &str, name: &str) -> Option<&'a String> {
    fields.get(&format!("{prefix}{name}"))
}

fn parse_dimensions(value: &str) -> Option<Vec<usize>> {
    let dimensions = value
        .split('x')
        .map(str::parse)
        .collect::<Result<Vec<usize>, _>>()
        .ok()?;
    (!dimensions.is_empty()).then_some(dimensions)
}

fn parse_scalar_bindings(value: &str) -> Option<Vec<ProviderScalarBinding>> {
    value
        .split(',')
        .map(|binding| {
            let mut parts = binding.split(':');
            let parsed = ProviderScalarBinding {
                name: parts.next()?.to_owned(),
                value_type: parts.next()?.to_owned(),
                value: parts.next()?.to_owned(),
            };
            parts.next().is_none().then_some(parsed)
        })
        .collect()
}

fn parse_nonempty_list(value: &str) -> Option<Vec<String>> {
    let values = value.split(',').map(str::to_owned).collect::<Vec<_>>();
    (!values.is_empty() && values.iter().all(|value| !value.is_empty())).then_some(values)
}

#[cfg(test)]
#[path = "provider_request_tests.rs"]
mod tests;
