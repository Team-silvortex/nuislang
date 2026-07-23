use crate::provider_request::{
    ProviderOutputBinding, ProviderOutputComparisonDescriptor,
    PROVIDER_OUTPUT_COMPARISON_COLLECTION_CONTRACT, PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn parse_output_comparisons(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<Vec<ProviderOutputComparisonDescriptor>> {
    if let Some(contract) = value(fields, prefix, "collection_contract") {
        (contract == PROVIDER_OUTPUT_COMPARISON_COLLECTION_CONTRACT).then_some(())?;
        let count = value(fields, prefix, "collection_count")?
            .parse::<usize>()
            .ok()?;
        (1..=8).contains(&count).then_some(())?;
        return (0..count)
            .map(|index| {
                parse_descriptor(fields, &format!("{prefix}item_{index}_"), None).flatten()
            })
            .collect();
    }
    parse_descriptor(fields, prefix, None).map(|descriptor| descriptor.into_iter().collect())
}

fn parse_descriptor(
    fields: &BTreeMap<String, String>,
    prefix: &str,
    default_id: Option<String>,
) -> Option<Option<ProviderOutputComparisonDescriptor>> {
    let Some(contract) = value(fields, prefix, "descriptor_contract") else {
        return Some(None);
    };
    (contract == PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT).then_some(())?;
    let output_buffer = value(fields, prefix, "output_buffer")?.clone();
    Some(Some(ProviderOutputComparisonDescriptor {
        id: value(fields, prefix, "id")
            .cloned()
            .or(default_id)
            .unwrap_or_else(|| format!("comparison.{output_buffer}")),
        output_buffer,
        element_type: value(fields, prefix, "element_type")?.clone(),
        shape: parse_dimensions(value(fields, prefix, "shape")?)?,
        expected_path: value(fields, prefix, "expected_path")?.clone(),
        expected_byte_length: value(fields, prefix, "expected_byte_length")?
            .parse()
            .ok()?,
        expected_content_hash: value(fields, prefix, "expected_content_hash")?.clone(),
        absolute_tolerance: value(fields, prefix, "absolute_tolerance")?.clone(),
        relative_tolerance: value(fields, prefix, "relative_tolerance")?.clone(),
        non_finite_policy: value(fields, prefix, "non_finite_policy")?.clone(),
    }))
}

pub(crate) fn validate_output_comparisons(
    comparisons: &[ProviderOutputComparisonDescriptor],
    bindings: &[ProviderOutputBinding],
) -> bool {
    let mut ids = BTreeSet::new();
    let mut buffers = BTreeSet::new();
    comparisons.iter().all(|comparison| {
        let Some(binding) = bindings
            .iter()
            .find(|binding| binding.buffer == comparison.output_buffer)
        else {
            return false;
        };
        ids.insert(comparison.id.as_str())
            && buffers.insert(comparison.output_buffer.as_str())
            && comparison.id.starts_with("comparison.")
            && comparison.element_type == binding.element_type
            && comparison.shape == binding.shape
            && comparison.expected_byte_length == binding.byte_length
            && comparison_shape_is_valid(comparison)
            && !comparison.expected_path.is_empty()
            && comparison.expected_content_hash.starts_with("0x")
            && valid_tolerance(&comparison.absolute_tolerance)
            && valid_tolerance(&comparison.relative_tolerance)
            && matches!(comparison.non_finite_policy.as_str(), "reject" | "equal")
            && binding.comparison_id == comparison.id
    }) && bindings.iter().all(|binding| {
        binding.comparison_id == "none"
            || comparisons
                .iter()
                .any(|comparison| comparison.id == binding.comparison_id)
    })
}

fn comparison_shape_is_valid(comparison: &ProviderOutputComparisonDescriptor) -> bool {
    let width = match comparison.element_type.as_str() {
        "u8" => 1usize,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" => 8,
        _ => return false,
    };
    comparison
        .shape
        .iter()
        .try_fold(width, |bytes, dimension| bytes.checked_mul(*dimension))
        == Some(comparison.expected_byte_length)
        && comparison.shape.iter().all(|dimension| *dimension > 0)
}

fn valid_tolerance(value: &str) -> bool {
    value
        .parse::<f64>()
        .is_ok_and(|value| value.is_finite() && value >= 0.0)
}

fn value<'a>(fields: &'a BTreeMap<String, String>, prefix: &str, name: &str) -> Option<&'a String> {
    fields.get(&format!("{prefix}{name}"))
}

fn parse_dimensions(value: &str) -> Option<Vec<usize>> {
    let dimensions = value
        .split('x')
        .map(str::parse)
        .collect::<Result<Vec<usize>, _>>()
        .ok()?;
    (!dimensions.is_empty() && dimensions.iter().all(|value| *value > 0)).then_some(dimensions)
}
