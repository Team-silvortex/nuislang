use crate::provider_request::{
    ProviderBufferDescriptor, ProviderOutputComparisonDescriptor, ProviderRequest,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PROVIDER_OUTPUT_BINDING_CONTRACT: &str = "nuis-provider-output-binding-v2";
pub(crate) const LEGACY_PROVIDER_OUTPUT_BINDING_CONTRACT: &str = "nuis-provider-output-binding-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderOutputBinding {
    pub(crate) role: String,
    pub(crate) buffer: String,
    pub(crate) element_type: String,
    pub(crate) layout: String,
    pub(crate) shape: Vec<usize>,
    pub(crate) row_stride_bytes: usize,
    pub(crate) byte_length: usize,
    pub(crate) comparison_id: String,
}

pub(crate) fn parse_output_bindings(
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
    let (compatibility_layout, compatibility_row_stride_bytes) = inferred_layout(
        compatibility_element_type,
        compatibility_shape,
        compatibility_byte_length,
        buffer,
    )?;
    let compatibility_comparison_id = comparison
        .map(|value| format!("comparison.{}", value.output_buffer))
        .unwrap_or_else(|| "none".to_owned());
    let Some(contract) = field(fields, prefix, "contract") else {
        return Some(vec![ProviderOutputBinding {
            role: "output.result".to_owned(),
            buffer: compatibility_output_buffer.to_owned(),
            element_type: compatibility_element_type.to_owned(),
            layout: compatibility_layout,
            shape: compatibility_shape.to_vec(),
            row_stride_bytes: compatibility_row_stride_bytes,
            byte_length: compatibility_byte_length,
            comparison_id: compatibility_comparison_id,
        }]);
    };
    let uses_typed_layout = contract == PROVIDER_OUTPUT_BINDING_CONTRACT;
    (uses_typed_layout || contract == LEGACY_PROVIDER_OUTPUT_BINDING_CONTRACT).then_some(())?;
    let count = field(fields, prefix, "count")?.parse::<usize>().ok()?;
    (1..=8).contains(&count).then_some(())?;
    (0..count)
        .map(|index| {
            let item = format!("{prefix}{index}_");
            let element_type = field(fields, &item, "element_type")
                .map(String::as_str)
                .unwrap_or(compatibility_element_type)
                .to_owned();
            let shape = field(fields, &item, "shape")
                .map(|value| parse_dimensions(value))
                .unwrap_or_else(|| Some(compatibility_shape.to_vec()))?;
            let byte_length = field(fields, &item, "byte_length")
                .map(|value| value.parse().ok())
                .unwrap_or(Some(compatibility_byte_length))?;
            let (layout, row_stride_bytes) = if uses_typed_layout {
                (
                    field(fields, &item, "layout")?.clone(),
                    field(fields, &item, "row_stride_bytes")?.parse().ok()?,
                )
            } else {
                inferred_layout(&element_type, &shape, byte_length, buffer)?
            };
            Some(ProviderOutputBinding {
                role: field(fields, &item, "role")?.clone(),
                buffer: field(fields, &item, "buffer")?.clone(),
                element_type,
                layout,
                shape,
                row_stride_bytes,
                byte_length,
                comparison_id: field(fields, &item, "comparison_id")
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

pub(crate) fn validate_output_bindings(request: &ProviderRequest) -> bool {
    let mut roles = BTreeSet::new();
    let mut buffers = BTreeSet::new();
    !request.output_bindings.is_empty()
        && request.output_bindings.len() <= 8
        && request.output_bindings[0].buffer == request.kernel.output_buffer
        && request.output_bindings.iter().all(|binding| {
            is_output_role(&binding.role)
                && !binding.buffer.is_empty()
                && valid_binding(binding)
                && roles.insert(binding.role.as_str())
                && buffers.insert(binding.buffer.as_str())
        })
}

pub(crate) fn output_binding_matches_buffer(
    binding: &ProviderOutputBinding,
    buffer: &ProviderBufferDescriptor,
) -> bool {
    binding.element_type == buffer.element_type
        && binding.layout == buffer.layout
        && binding.shape == buffer.shape
        && binding.row_stride_bytes == buffer.row_stride_bytes
        && binding.byte_length == buffer.byte_length
}

fn inferred_layout(
    element_type: &str,
    shape: &[usize],
    byte_length: usize,
    buffer: &ProviderBufferDescriptor,
) -> Option<(String, usize)> {
    if element_type == buffer.element_type
        && shape == buffer.shape
        && byte_length == buffer.byte_length
    {
        Some((buffer.layout.clone(), buffer.row_stride_bytes))
    } else {
        Some((
            "tensor-contiguous".to_owned(),
            contiguous_byte_length(element_type, shape)?,
        ))
    }
}

fn valid_binding(binding: &ProviderOutputBinding) -> bool {
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
        "tensor-row-major" => valid_row_major(binding, width),
        layout if layout.starts_with("image-2d-row-major") => valid_row_major(binding, width),
        _ => false,
    };
    !binding.shape.is_empty()
        && binding.shape.iter().all(|dimension| *dimension > 0)
        && layout_valid
        && (binding.comparison_id == "none" || binding.comparison_id.starts_with("comparison."))
}

fn valid_row_major(binding: &ProviderOutputBinding, element_width: usize) -> bool {
    let [width, height] = binding.shape.as_slice() else {
        return false;
    };
    width
        .checked_mul(element_width)
        .is_some_and(|minimum_stride| binding.row_stride_bytes >= minimum_stride)
        && binding.row_stride_bytes.checked_mul(*height) == Some(binding.byte_length)
}

fn is_output_role(value: &str) -> bool {
    value.starts_with("output.")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
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
