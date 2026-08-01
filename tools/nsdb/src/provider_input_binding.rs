use crate::provider_request::{
    ProviderBufferDescriptor, ProviderRequest, ProviderRequestDependency,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PROVIDER_INPUT_BINDING_CONTRACT: &str = "nuis-provider-input-binding-v2";
pub(crate) const LEGACY_PROVIDER_INPUT_BINDING_CONTRACT: &str = "nuis-provider-input-binding-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderInputBinding {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) element_type: String,
    pub(crate) layout: String,
    pub(crate) shape: Vec<usize>,
    pub(crate) row_stride_bytes: usize,
    pub(crate) byte_length: usize,
    pub(crate) content_hash: String,
    pub(crate) payload_path: String,
    pub(crate) producer_request_id: String,
    pub(crate) producer_output_buffer: String,
}

pub(crate) fn parse_input_bindings(
    fields: &BTreeMap<String, String>,
    prefix: &str,
    buffer: &ProviderBufferDescriptor,
    dependencies: &[ProviderRequestDependency],
) -> Option<Vec<ProviderInputBinding>> {
    let Some(contract) = field(fields, prefix, "contract") else {
        let dependency = dependencies
            .iter()
            .find(|dependency| dependency.consumer_input_buffer == buffer.id);
        return Some(vec![ProviderInputBinding {
            name: buffer.id.clone(),
            source: if dependency.is_some() {
                "dependency"
            } else {
                "artifact"
            }
            .to_owned(),
            element_type: buffer.element_type.clone(),
            layout: buffer.layout.clone(),
            shape: buffer.shape.clone(),
            row_stride_bytes: buffer.row_stride_bytes,
            byte_length: buffer.byte_length,
            content_hash: buffer.content_hash.clone(),
            payload_path: if dependency.is_some() {
                "none".to_owned()
            } else {
                buffer.payload_path.clone()
            },
            producer_request_id: dependency
                .map(|dependency| dependency.producer_request_id.clone())
                .unwrap_or_else(|| "none".to_owned()),
            producer_output_buffer: dependency
                .map(|dependency| dependency.producer_output_buffer.clone())
                .unwrap_or_else(|| "none".to_owned()),
        }]);
    };
    let uses_typed_layout = contract == PROVIDER_INPUT_BINDING_CONTRACT;
    (uses_typed_layout || contract == LEGACY_PROVIDER_INPUT_BINDING_CONTRACT).then_some(())?;
    let count = field(fields, prefix, "count")?.parse::<usize>().ok()?;
    (1..=16).contains(&count).then_some(())?;
    (0..count)
        .map(|index| {
            let item = format!("{prefix}{index}_");
            let name = field(fields, &item, "name")?.clone();
            let element_type = field(fields, &item, "element_type")?.clone();
            let shape = parse_dimensions(field(fields, &item, "shape")?)?;
            let (layout, row_stride_bytes) = if uses_typed_layout {
                (
                    field(fields, &item, "layout")?.clone(),
                    field(fields, &item, "row_stride_bytes")?.parse().ok()?,
                )
            } else if name == buffer.id {
                (buffer.layout.clone(), buffer.row_stride_bytes)
            } else {
                (
                    "tensor-contiguous".to_owned(),
                    contiguous_byte_length(&element_type, &shape)?,
                )
            };
            Some(ProviderInputBinding {
                name,
                source: field(fields, &item, "source")?.clone(),
                element_type,
                layout,
                shape,
                row_stride_bytes,
                byte_length: field(fields, &item, "byte_length")?.parse().ok()?,
                content_hash: field(fields, &item, "content_hash")?.clone(),
                payload_path: field(fields, &item, "payload_path")?.clone(),
                producer_request_id: field(fields, &item, "producer_request_id")?.clone(),
                producer_output_buffer: field(fields, &item, "producer_output_buffer")?.clone(),
            })
        })
        .collect()
}

pub(crate) fn validate_input_bindings(request: &ProviderRequest) -> bool {
    let names = request
        .input_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect::<BTreeSet<_>>();
    names.len() == request.input_bindings.len()
        && request.input_bindings.len() == request.kernel.input_buffers.len()
        && request
            .input_bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .eq(request.kernel.input_buffers.iter().map(String::as_str))
        && request.input_bindings.iter().all(valid_binding)
        && request.dependencies.iter().all(|dependency| {
            request.input_bindings.iter().any(|binding| {
                binding.name == dependency.consumer_input_buffer
                    && binding.source == "dependency"
                    && binding.producer_request_id == dependency.producer_request_id
                    && binding.producer_output_buffer == dependency.producer_output_buffer
            })
        })
        && request
            .input_bindings
            .iter()
            .filter(|binding| binding.source == "dependency")
            .count()
            == request.dependencies.len()
}

pub(crate) fn input_binding_matches_buffer(
    binding: &ProviderInputBinding,
    buffer: &ProviderBufferDescriptor,
) -> bool {
    binding.element_type == buffer.element_type
        && binding.layout == buffer.layout
        && binding.shape == buffer.shape
        && binding.row_stride_bytes == buffer.row_stride_bytes
        && binding.byte_length == buffer.byte_length
}

pub(crate) fn validate_dependency_binding(
    producer: &ProviderRequest,
    consumer: &ProviderRequest,
    dependency: &ProviderRequestDependency,
) -> bool {
    let Some(binding) = consumer
        .input_bindings
        .iter()
        .find(|binding| binding.name == dependency.consumer_input_buffer)
    else {
        return false;
    };
    let Some(output) = producer
        .output_bindings
        .iter()
        .find(|output| output.buffer == dependency.producer_output_buffer)
    else {
        return false;
    };
    binding.element_type == output.element_type
        && binding.layout == output.layout
        && binding.shape == output.shape
        && binding.row_stride_bytes == output.row_stride_bytes
        && binding.byte_length == output.byte_length
        && if output.comparison_id == "none" {
            true
        } else {
            producer
                .output_comparisons
                .iter()
                .find(|comparison| comparison.id == output.comparison_id)
                .is_some_and(|comparison| binding.content_hash == comparison.expected_content_hash)
        }
}

fn valid_binding(binding: &ProviderInputBinding) -> bool {
    let Some(width) = element_width(&binding.element_type) else {
        return false;
    };
    let Some(contiguous_bytes) = binding
        .shape
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
        .and_then(|count| count.checked_mul(width))
    else {
        return false;
    };
    let layout_valid = match binding.layout.as_str() {
        "tensor-contiguous" => {
            binding.row_stride_bytes == contiguous_bytes && binding.byte_length == contiguous_bytes
        }
        "tensor-row-major" => {
            let [width_elements, height] = binding.shape.as_slice() else {
                return false;
            };
            let Some(minimum_stride) = width_elements.checked_mul(width) else {
                return false;
            };
            binding.row_stride_bytes >= minimum_stride
                && binding.row_stride_bytes.checked_mul(*height) == Some(binding.byte_length)
        }
        layout if layout.starts_with("image-2d-row-major") => {
            let [width_elements, height] = binding.shape.as_slice() else {
                return false;
            };
            let Some(minimum_stride) = width_elements.checked_mul(width) else {
                return false;
            };
            binding.row_stride_bytes >= minimum_stride
                && binding.row_stride_bytes.checked_mul(*height) == Some(binding.byte_length)
        }
        _ => false,
    };
    !binding.name.is_empty()
        && binding.shape.iter().all(|dimension| *dimension > 0)
        && layout_valid
        && binding.content_hash.starts_with("0x")
        && match binding.source.as_str() {
            "artifact" => {
                !binding.payload_path.is_empty()
                    && binding.producer_request_id == "none"
                    && binding.producer_output_buffer == "none"
            }
            "dependency" => {
                binding.payload_path == "none"
                    && binding.producer_request_id != "none"
                    && binding.producer_output_buffer != "none"
            }
            _ => false,
        }
}

fn element_width(element_type: &str) -> Option<usize> {
    match element_type {
        "u8" => 1,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" => 8,
        _ => return None,
    }
    .into()
}

fn contiguous_byte_length(element_type: &str, shape: &[usize]) -> Option<usize> {
    shape
        .iter()
        .try_fold(element_width(element_type)?, |bytes, dimension| {
            bytes.checked_mul(*dimension)
        })
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
