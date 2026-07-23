use crate::{
    provider_request::{
        ProviderOutputComparisonDescriptor, PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT,
    },
    provider_sample_payload::fnv1a64_hex,
    provider_sample_payload::PixelMagicNativeOutputSummary,
};
use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderOutputComparisonResult {
    pub(crate) contract: &'static str,
    pub(crate) comparison_id: String,
    pub(crate) output_buffer: String,
    pub(crate) status: &'static str,
    pub(crate) compared_elements: usize,
    pub(crate) mismatch_count: usize,
    pub(crate) max_absolute_error: String,
    pub(crate) max_relative_error: String,
    pub(crate) non_finite_count: usize,
}

pub(crate) const PROVIDER_OUTPUT_COMPARISON_COLLECTION_RESULT_CONTRACT: &str =
    "nuis-provider-output-comparison-collection-result-v1";

pub(crate) fn bind_output_comparison_collection(
    summary: &mut PixelMagicNativeOutputSummary,
    results: &[ProviderOutputComparisonResult],
    compatibility_output_buffer: &str,
) {
    summary.comparison_collection_contract =
        PROVIDER_OUTPUT_COMPARISON_COLLECTION_RESULT_CONTRACT.to_owned();
    summary.comparison_collection_count = results.len().to_string();
    summary.comparison_collection_ids =
        comparison_manifest(results, |result| result.comparison_id.clone());
    summary.comparison_collection_output_buffers =
        comparison_manifest(results, |result| result.output_buffer.clone());
    summary.comparison_collection_statuses =
        comparison_manifest(results, |result| result.status.to_owned());
    summary.comparison_collection_element_counts =
        comparison_manifest(results, |result| result.compared_elements.to_string());
    summary.comparison_collection_mismatch_counts =
        comparison_manifest(results, |result| result.mismatch_count.to_string());
    if let Some(primary) = results
        .iter()
        .find(|result| result.output_buffer == compatibility_output_buffer)
    {
        summary.comparison_contract = primary.contract.to_owned();
        summary.comparison_status = primary.status.to_owned();
        summary.comparison_element_count = primary.compared_elements.to_string();
        summary.comparison_mismatch_count = primary.mismatch_count.to_string();
        summary.comparison_max_absolute_error = primary.max_absolute_error.clone();
        summary.comparison_max_relative_error = primary.max_relative_error.clone();
        summary.comparison_non_finite_count = primary.non_finite_count.to_string();
    }
}

fn comparison_manifest(
    results: &[ProviderOutputComparisonResult],
    value: impl Fn(&ProviderOutputComparisonResult) -> String,
) -> String {
    if results.is_empty() {
        "none".to_owned()
    } else {
        results.iter().map(value).collect::<Vec<_>>().join(",")
    }
}

pub(crate) fn compare_provider_output(
    output_dir: &Path,
    descriptor: &ProviderOutputComparisonDescriptor,
    actual: &[u8],
) -> Result<ProviderOutputComparisonResult, String> {
    let expected_path = resolve_expected_path(output_dir, &descriptor.expected_path)?;
    let expected = fs::read(&expected_path).map_err(|error| {
        format!(
            "failed to read expected provider output `{}`: {error}",
            expected_path.display()
        )
    })?;
    if expected.len() != descriptor.expected_byte_length
        || fnv1a64_hex(&expected) != descriptor.expected_content_hash
    {
        return Err("expected provider output size/hash evidence mismatch".to_owned());
    }
    if actual.len() != descriptor.expected_byte_length {
        return Err(format!(
            "provider output byte length mismatch: expected {}, got {}",
            descriptor.expected_byte_length,
            actual.len()
        ));
    }
    if descriptor.element_type != "f32" {
        return compare_exact_provider_output(descriptor, actual, &expected);
    }
    if actual.len() % 4 != 0 {
        return Err("provider output comparison requires complete f32 elements".to_owned());
    }
    let absolute_tolerance = parse_tolerance("absolute", &descriptor.absolute_tolerance)?;
    let relative_tolerance = parse_tolerance("relative", &descriptor.relative_tolerance)?;
    let mut mismatch_count = 0usize;
    let mut non_finite_count = 0usize;
    let mut max_absolute_error = 0.0f64;
    let mut max_relative_error = 0.0f64;
    for (index, (actual, expected)) in actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .enumerate()
    {
        let actual = f32::from_le_bytes(actual.try_into().expect("four-byte chunk"));
        let expected = f32::from_le_bytes(expected.try_into().expect("four-byte chunk"));
        if !actual.is_finite() || !expected.is_finite() {
            non_finite_count += 1;
            if descriptor.non_finite_policy == "reject" {
                return Err(format!(
                    "provider output contains a non-finite f32 element at index {index}"
                ));
            }
            let equal = (actual.is_nan() && expected.is_nan())
                || (actual.is_infinite()
                    && expected.is_infinite()
                    && actual.is_sign_positive() == expected.is_sign_positive());
            if !equal {
                mismatch_count += 1;
            }
            continue;
        }
        let absolute_error = f64::from((actual - expected).abs());
        let relative_error = if expected == 0.0 {
            absolute_error
        } else {
            absolute_error / f64::from(expected.abs())
        };
        max_absolute_error = max_absolute_error.max(absolute_error);
        max_relative_error = max_relative_error.max(relative_error);
        let allowed = absolute_tolerance + relative_tolerance * f64::from(expected.abs());
        if absolute_error > allowed {
            mismatch_count += 1;
        }
    }
    if mismatch_count > 0 {
        return Err(format!(
            "provider output comparison failed: {mismatch_count} mismatched elements; max absolute error {}; max relative error {}",
            format_number(max_absolute_error),
            format_number(max_relative_error)
        ));
    }
    Ok(ProviderOutputComparisonResult {
        contract: PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT,
        comparison_id: descriptor.id.clone(),
        output_buffer: descriptor.output_buffer.clone(),
        status: "comparison-passed",
        compared_elements: actual.len() / 4,
        mismatch_count,
        max_absolute_error: format_number(max_absolute_error),
        max_relative_error: format_number(max_relative_error),
        non_finite_count,
    })
}

pub(crate) fn compare_provider_output_collection(
    output_dir: &Path,
    descriptors: &[ProviderOutputComparisonDescriptor],
    outputs: &[(&str, &[u8])],
) -> Result<Vec<ProviderOutputComparisonResult>, String> {
    descriptors
        .iter()
        .map(|descriptor| {
            let actual = outputs
                .iter()
                .find_map(|(buffer, payload)| {
                    (*buffer == descriptor.output_buffer).then_some(*payload)
                })
                .ok_or_else(|| {
                    format!(
                        "provider comparison `{}` has no completed output buffer `{}`",
                        descriptor.id, descriptor.output_buffer
                    )
                })?;
            compare_provider_output(output_dir, descriptor, actual)
        })
        .collect()
}

fn compare_exact_provider_output(
    descriptor: &ProviderOutputComparisonDescriptor,
    actual: &[u8],
    expected: &[u8],
) -> Result<ProviderOutputComparisonResult, String> {
    let element_width = match descriptor.element_type.as_str() {
        "u8" => 1,
        "u32" | "i32" => 4,
        "u64" | "i64" => 8,
        other => {
            return Err(format!(
                "provider output exact comparison does not support `{other}`"
            ));
        }
    };
    let mismatch_count = actual
        .chunks_exact(element_width)
        .zip(expected.chunks_exact(element_width))
        .filter(|(actual, expected)| actual != expected)
        .count();
    if mismatch_count > 0 {
        return Err(format!(
            "provider output comparison failed: {mismatch_count} mismatched elements"
        ));
    }
    Ok(ProviderOutputComparisonResult {
        contract: PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT,
        comparison_id: descriptor.id.clone(),
        output_buffer: descriptor.output_buffer.clone(),
        status: "comparison-passed",
        compared_elements: actual.len() / element_width,
        mismatch_count,
        max_absolute_error: "0".to_owned(),
        max_relative_error: "0".to_owned(),
        non_finite_count: 0,
    })
}

fn parse_tolerance(kind: &str, value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or_else(|| format!("provider output {kind} tolerance is invalid"))
}

fn resolve_expected_path(output_dir: &Path, relative: &str) -> Result<std::path::PathBuf, String> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().count() != 1
        || !matches!(
            relative.components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err(
            "expected provider output path must be one output-relative file name".to_owned(),
        );
    }
    Ok(output_dir.join(relative))
}

fn format_number(value: f64) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::SystemTime,
    };

    static TEMP_DIR_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn descriptor(expected: &[u8]) -> ProviderOutputComparisonDescriptor {
        ProviderOutputComparisonDescriptor {
            id: "comparison.output.features".to_owned(),
            output_buffer: "output.features".to_owned(),
            element_type: "f32".to_owned(),
            shape: vec![2],
            expected_path: "expected.bin".to_owned(),
            expected_byte_length: expected.len(),
            expected_content_hash: fnv1a64_hex(expected),
            absolute_tolerance: "0.01".to_owned(),
            relative_tolerance: "0".to_owned(),
            non_finite_policy: "reject".to_owned(),
        }
    }

    fn temp_dir() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nuis-provider-output-comparison-{}-{nonce}-{}",
            std::process::id(),
            TEMP_DIR_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn accepts_f32_output_within_absolute_tolerance() {
        let output_dir = temp_dir();
        let expected = [1.0f32, 2.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(output_dir.join("expected.bin"), &expected).unwrap();
        let actual = [1.005f32, 1.995]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let result = compare_provider_output(&output_dir, &descriptor(&expected), &actual).unwrap();
        assert_eq!(result.status, "comparison-passed");
        assert_eq!(result.compared_elements, 2);
        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn rejects_f32_output_outside_tolerance() {
        let output_dir = temp_dir();
        let expected = [1.0f32, 2.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(output_dir.join("expected.bin"), &expected).unwrap();
        let actual = [1.0f32, 3.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let error = compare_provider_output(&output_dir, &descriptor(&expected), &actual)
            .expect_err("comparison must reject mismatch");
        assert!(error.contains("1 mismatched elements"));
        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn rejects_non_finite_output_under_reject_policy() {
        let output_dir = temp_dir();
        let expected = [1.0f32, 2.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(output_dir.join("expected.bin"), &expected).unwrap();
        let actual = [1.0f32, f32::NAN]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let error = compare_provider_output(&output_dir, &descriptor(&expected), &actual)
            .expect_err("comparison must reject NaN");
        assert!(error.contains("non-finite"));
        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn rejects_tampered_expected_output_asset() {
        let output_dir = temp_dir();
        let expected = [1.0f32, 2.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let comparison = descriptor(&expected);
        fs::write(output_dir.join("expected.bin"), [0u8; 8]).unwrap();
        let error = compare_provider_output(&output_dir, &comparison, &expected)
            .expect_err("comparison must reject tampered expected asset");
        assert!(error.contains("size/hash evidence mismatch"));
        let _ = fs::remove_dir_all(output_dir);
    }
}
