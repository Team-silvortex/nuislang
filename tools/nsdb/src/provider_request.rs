use crate::provider_adapter_binding::{parse_adapter_binding, ProviderAdapterBinding};
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRequest {
    pub(crate) source: &'static str,
    pub(crate) buffer: ProviderBufferDescriptor,
    pub(crate) kernel: ProviderKernelDescriptor,
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
    let dependencies = parse_dependencies(fields, dependency_prefix)?;
    let input_bindings =
        parse_input_bindings(fields, input_binding_prefix, &buffer, &dependencies)?;
    let adapter_binding = parse_adapter_binding(fields, adapter_binding_prefix)?;
    validate_request(ProviderRequest {
        source,
        buffer,
        kernel,
        model_asset,
        output_comparison,
        dependencies,
        input_bindings,
        adapter_binding,
    })
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
                                && requests[*producer_index].kernel.output_buffer
                                    == dependency.producer_output_buffer
                                && request
                                    .kernel
                                    .input_buffers
                                    .contains(&dependency.consumer_input_buffer)
                                && validate_dependency_binding(
                                    &requests[*producer_index],
                                    request,
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
        && validate_input_bindings(&request))
    .then_some(request)
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
mod tests {
    use super::*;

    const REGISTERED: &str = "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.pixels;provider_buffer_element_type=u8;provider_buffer_layout=image-2d-row-major:pixel-format=gray8;provider_buffer_shape=2x2;provider_buffer_row_stride_bytes=2;provider_buffer_byte_length=4;provider_buffer_payload_path=pixels.bin;provider_buffer_content_hash=0x1234;provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=pixelmagic.gray8.invert;provider_kernel_operation=invert;provider_kernel_input_buffer=input.pixels;provider_kernel_output_buffer=output.pixels;provider_kernel_dispatch=2x2x1;provider_kernel_scalar_bindings=max_value:u8:15";

    fn indexed_request(index: usize, kernel_id: &str) -> String {
        REGISTERED
            .replace(
                "provider_buffer_",
                &format!("provider_request_{index}_buffer_"),
            )
            .replace(
                "provider_kernel_",
                &format!("provider_request_{index}_kernel_"),
            )
            .replace("pixelmagic.gray8.invert", kernel_id)
    }

    #[test]
    fn parses_registered_buffer_and_kernel_descriptors() {
        let request = provider_request_from_evidence(REGISTERED).expect("registered request");
        assert_eq!(request.source, "registered-descriptors");
        assert_eq!(request.buffer.shape, [2, 2]);
        assert_eq!(request.kernel.dispatch, [2, 2, 1]);
        assert_eq!(request.scalar_u8("max_value"), Some(15));
    }

    #[test]
    fn rejects_registered_descriptor_with_mismatched_buffer_binding() {
        let invalid = REGISTERED.replace(
            "provider_kernel_input_buffer=input.pixels",
            "provider_kernel_input_buffer=missing",
        );
        assert!(provider_request_from_evidence(&invalid).is_none());
    }

    #[test]
    fn parses_hash_bound_model_asset_descriptor() {
        let evidence = format!(
            "{REGISTERED};provider_model_asset_descriptor_contract={PROVIDER_MODEL_ASSET_DESCRIPTOR_CONTRACT};provider_model_asset_id=model;provider_model_asset_format=coreml-specification;provider_model_asset_path=model.mlmodel;provider_model_asset_byte_length=128;provider_model_asset_content_hash=0xabcd;provider_model_asset_input_feature=input.pixels;provider_model_asset_output_feature=output.pixels"
        );
        let model = provider_request_from_evidence(&evidence)
            .expect("model request")
            .model_asset
            .expect("model asset");
        assert_eq!(model.path, "model.mlmodel");
        assert_eq!(model.byte_length, 128);
    }

    #[test]
    fn accepts_model_request_without_scalar_bindings() {
        let evidence = REGISTERED
            .replace(
                ";provider_kernel_scalar_bindings=max_value:u8:15",
                ";provider_model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;provider_model_asset_id=model;provider_model_asset_format=coreml-specification;provider_model_asset_path=model.mlmodel;provider_model_asset_byte_length=128;provider_model_asset_content_hash=0xabcd;provider_model_asset_input_feature=input.pixels;provider_model_asset_output_feature=output.pixels",
            );
        let request = provider_request_from_evidence(&evidence).expect("scalar-free model request");
        assert!(request.kernel.scalar_bindings.is_empty());
        assert!(request.model_asset.is_some());
    }

    #[test]
    fn parses_hash_bound_output_comparison_descriptor() {
        let evidence = format!(
            "{REGISTERED};provider_output_comparison_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_output_buffer=output.pixels;provider_output_comparison_element_type=f32;provider_output_comparison_shape=2x2;provider_output_comparison_expected_path=expected.bin;provider_output_comparison_expected_byte_length=16;provider_output_comparison_expected_content_hash=0xabcd;provider_output_comparison_absolute_tolerance=0.001;provider_output_comparison_relative_tolerance=0.01;provider_output_comparison_non_finite_policy=reject"
        );
        let comparison = provider_request_from_evidence(&evidence)
            .expect("request with comparison")
            .output_comparison
            .expect("comparison descriptor");
        assert_eq!(comparison.shape, [2, 2]);
        assert_eq!(comparison.expected_byte_length, 16);
        assert_eq!(comparison.non_finite_policy, "reject");
    }

    #[test]
    fn rejects_output_comparison_with_mismatched_shape_bytes() {
        let evidence = format!(
            "{REGISTERED};provider_output_comparison_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_output_buffer=output.pixels;provider_output_comparison_element_type=f32;provider_output_comparison_shape=2x2;provider_output_comparison_expected_path=expected.bin;provider_output_comparison_expected_byte_length=8;provider_output_comparison_expected_content_hash=0xabcd;provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject"
        );
        assert!(provider_request_from_evidence(&evidence).is_none());
    }

    #[test]
    fn rejects_output_comparison_with_invalid_tolerance_policy() {
        let evidence = format!(
            "{REGISTERED};provider_output_comparison_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_output_buffer=output.pixels;provider_output_comparison_element_type=f32;provider_output_comparison_shape=2x2;provider_output_comparison_expected_path=expected.bin;provider_output_comparison_expected_byte_length=16;provider_output_comparison_expected_content_hash=0xabcd;provider_output_comparison_absolute_tolerance=-1;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=permit"
        );
        assert!(provider_request_from_evidence(&evidence).is_none());
    }

    #[test]
    fn parses_ordered_provider_request_collection() {
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{}",
            indexed_request(0, "first"),
            indexed_request(1, "second")
        );
        let collection = provider_request_collection_from_evidence(&evidence).expect("collection");
        assert_eq!(collection.source, "registered-collection");
        assert_eq!(collection.requests.len(), 2);
        assert_eq!(collection.requests[0].kernel.id, "first");
        assert_eq!(collection.requests[1].kernel.id, "second");
    }

    #[test]
    fn rejects_duplicate_collection_request_identity() {
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{}",
            indexed_request(0, "duplicate"),
            indexed_request(1, "duplicate")
        );
        assert!(provider_request_collection_from_evidence(&evidence).is_none());
    }

    fn dependency(index: usize, producer: &str) -> String {
        format!(
            "provider_request_{index}_dependency_contract={PROVIDER_REQUEST_DEPENDENCY_CONTRACT};provider_request_{index}_dependency_count=1;provider_request_{index}_dependency_0_producer_request_id={producer};provider_request_{index}_dependency_0_producer_output_buffer=output.pixels;provider_request_{index}_dependency_0_consumer_input_buffer=input.pixels"
        )
    }

    #[test]
    fn parses_backward_provider_request_dependency() {
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{}",
            indexed_request(0, "first"),
            indexed_request(1, "second"),
            dependency(1, "first")
        );
        let collection = provider_request_collection_from_evidence(&evidence).expect("dependency");
        assert_eq!(collection.requests[1].dependencies.len(), 1);
        assert_eq!(
            collection.requests[1].dependencies[0].producer_request_id,
            "first"
        );
    }

    #[test]
    fn rejects_missing_self_or_forward_dependency_target() {
        for producer in ["missing", "first", "second"] {
            let evidence = format!(
                "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{}",
                indexed_request(0, "first"),
                indexed_request(1, "second"),
                dependency(0, producer)
            );
            assert!(provider_request_collection_from_evidence(&evidence).is_none());
        }
    }

    #[test]
    fn rejects_duplicate_dependency_edge() {
        let duplicate = dependency(1, "first")
            .replace("dependency_count=1", "dependency_count=2")
            + ";provider_request_1_dependency_1_producer_request_id=first;provider_request_1_dependency_1_producer_output_buffer=output.pixels;provider_request_1_dependency_1_consumer_input_buffer=input.pixels";
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{duplicate}",
            indexed_request(0, "first"),
            indexed_request(1, "second")
        );
        assert!(provider_request_collection_from_evidence(&evidence).is_none());
    }

    #[test]
    fn rejects_cyclic_dependency_graph() {
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{};{}",
            indexed_request(0, "first"),
            indexed_request(1, "second"),
            dependency(0, "second"),
            dependency(1, "first")
        );
        assert!(provider_request_collection_from_evidence(&evidence).is_none());
    }

    fn fan_in_bindings(second_name: &str) -> String {
        format!(
            "provider_request_1_input_binding_contract={};provider_request_1_input_binding_count=2;provider_request_1_input_binding_0_name=input.pixels;provider_request_1_input_binding_0_source=artifact;provider_request_1_input_binding_0_element_type=u8;provider_request_1_input_binding_0_shape=2x2;provider_request_1_input_binding_0_byte_length=4;provider_request_1_input_binding_0_content_hash=0x1234;provider_request_1_input_binding_0_payload_path=pixels.bin;provider_request_1_input_binding_0_producer_request_id=none;provider_request_1_input_binding_0_producer_output_buffer=none;provider_request_1_input_binding_1_name={second_name};provider_request_1_input_binding_1_source=dependency;provider_request_1_input_binding_1_element_type=u8;provider_request_1_input_binding_1_shape=2x2;provider_request_1_input_binding_1_byte_length=4;provider_request_1_input_binding_1_content_hash=0xabcd;provider_request_1_input_binding_1_payload_path=none;provider_request_1_input_binding_1_producer_request_id=first;provider_request_1_input_binding_1_producer_output_buffer=output.pixels",
            crate::provider_input_binding::PROVIDER_INPUT_BINDING_CONTRACT
        )
    }

    #[test]
    fn parses_named_multi_input_fan_in_bindings() {
        let second = indexed_request(1, "second").replace(
            "provider_request_1_kernel_input_buffer=input.pixels",
            "provider_request_1_kernel_input_buffer=input.pixels;provider_request_1_kernel_input_buffers=input.pixels,input.aux",
        );
        let edge = dependency(1, "first").replace(
            "consumer_input_buffer=input.pixels",
            "consumer_input_buffer=input.aux",
        );
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{second};{edge};{}",
            indexed_request(0, "first"),
            fan_in_bindings("input.aux")
        );
        let collection = provider_request_collection_from_evidence(&evidence).expect("fan-in");
        assert_eq!(collection.requests[1].kernel.input_buffers.len(), 2);
        assert_eq!(collection.requests[1].input_bindings.len(), 2);
        assert_eq!(
            collection.requests[1].input_bindings[1].source,
            "dependency"
        );
    }

    #[test]
    fn rejects_duplicate_named_input_binding() {
        let second = indexed_request(1, "second").replace(
            "provider_request_1_kernel_input_buffer=input.pixels",
            "provider_request_1_kernel_input_buffer=input.pixels;provider_request_1_kernel_input_buffers=input.pixels,input.aux",
        );
        let edge = dependency(1, "first").replace(
            "consumer_input_buffer=input.pixels",
            "consumer_input_buffer=input.aux",
        );
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{second};{edge};{}",
            indexed_request(0, "first"),
            fan_in_bindings("input.pixels")
        );
        assert!(provider_request_collection_from_evidence(&evidence).is_none());
    }
}
