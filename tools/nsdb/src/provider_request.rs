use std::collections::BTreeMap;

pub(crate) const PROVIDER_BUFFER_DESCRIPTOR_CONTRACT: &str = "nuis-provider-buffer-descriptor-v1";
pub(crate) const PROVIDER_KERNEL_DESCRIPTOR_CONTRACT: &str = "nuis-provider-kernel-descriptor-v1";
pub(crate) const PROVIDER_MODEL_ASSET_DESCRIPTOR_CONTRACT: &str =
    "nuis-provider-model-asset-descriptor-v1";
pub(crate) const PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT: &str =
    "nuis-provider-output-comparison-descriptor-v1";
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRequest {
    pub(crate) source: &'static str,
    pub(crate) buffer: ProviderBufferDescriptor,
    pub(crate) kernel: ProviderKernelDescriptor,
    pub(crate) model_asset: Option<ProviderModelAssetDescriptor>,
    pub(crate) output_comparison: Option<ProviderOutputComparisonDescriptor>,
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
            )
        })
        .collect::<Option<Vec<_>>>()?;
    let mut identities = requests
        .iter()
        .map(|request| request.kernel.id.as_str())
        .collect::<Vec<_>>();
    identities.sort_unstable();
    identities.dedup();
    (identities.len() == requests.len()).then_some(ProviderRequestCollection {
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
    })
}

fn build_request(
    source: &'static str,
    fields: &BTreeMap<String, String>,
    buffer_prefix: &str,
    kernel_prefix: &str,
    model_prefix: &str,
    comparison_prefix: &str,
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
    let kernel = ProviderKernelDescriptor {
        id: field(fields, kernel_prefix, "id")?.clone(),
        operation: field(fields, kernel_prefix, "operation")?.clone(),
        input_buffer: field(fields, kernel_prefix, "input_buffer")?.clone(),
        output_buffer: field(fields, kernel_prefix, "output_buffer")?.clone(),
        dispatch: parse_dimensions(field(fields, kernel_prefix, "dispatch")?)?,
        scalar_bindings: match field(fields, kernel_prefix, "scalar_bindings") {
            Some(value) => parse_scalar_bindings(value)?,
            None => Vec::new(),
        },
    };
    let model_asset = parse_model_asset(fields, model_prefix)?;
    let output_comparison = parse_output_comparison(fields, comparison_prefix)?;
    validate_request(ProviderRequest {
        source,
        buffer,
        kernel,
        model_asset,
        output_comparison,
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
    Some(Some(ProviderModelAssetDescriptor {
        id: field(fields, prefix, "id")?.clone(),
        format: field(fields, prefix, "format")?.clone(),
        path: field(fields, prefix, "path")?.clone(),
        byte_length: field(fields, prefix, "byte_length")?.parse().ok()?,
        content_hash: field(fields, prefix, "content_hash")?.clone(),
        input_feature: field(fields, prefix, "input_feature")?.clone(),
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
        && comparison_valid)
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
}
