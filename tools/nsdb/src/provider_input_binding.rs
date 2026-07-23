use crate::provider_request::{
    ProviderBufferDescriptor, ProviderRequest, ProviderRequestDependency,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PROVIDER_INPUT_BINDING_CONTRACT: &str = "nuis-provider-input-binding-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderInputBinding {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) element_type: String,
    pub(crate) shape: Vec<usize>,
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
            shape: buffer.shape.clone(),
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
    (contract == PROVIDER_INPUT_BINDING_CONTRACT).then_some(())?;
    let count = field(fields, prefix, "count")?.parse::<usize>().ok()?;
    (1..=16).contains(&count).then_some(())?;
    (0..count)
        .map(|index| {
            let item = format!("{prefix}{index}_");
            Some(ProviderInputBinding {
                name: field(fields, &item, "name")?.clone(),
                source: field(fields, &item, "source")?.clone(),
                element_type: field(fields, &item, "element_type")?.clone(),
                shape: parse_dimensions(field(fields, &item, "shape")?)?,
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
        && binding.shape == output.shape
        && binding.byte_length == output.byte_length
        && if output.comparison_id == "none" {
            true
        } else {
            producer
                .output_comparison
                .as_ref()
                .is_some_and(|comparison| {
                    output.comparison_id == format!("comparison.{}", comparison.output_buffer)
                        && binding.content_hash == comparison.expected_content_hash
                })
        }
}

fn valid_binding(binding: &ProviderInputBinding) -> bool {
    let width = match binding.element_type.as_str() {
        "u8" => 1,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" => 8,
        _ => return false,
    };
    let bytes = binding
        .shape
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
        .and_then(|count| count.checked_mul(width));
    !binding.name.is_empty()
        && binding.shape.iter().all(|dimension| *dimension > 0)
        && bytes == Some(binding.byte_length)
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
