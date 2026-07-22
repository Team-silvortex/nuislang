use crate::{
    provider_request::{
        ProviderOutputComparisonDescriptor, PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT,
    },
    provider_sample_payload::fnv1a64_hex,
};
use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderOutputComparisonResult {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) compared_elements: usize,
    pub(crate) mismatch_count: usize,
    pub(crate) max_absolute_error: String,
    pub(crate) max_relative_error: String,
    pub(crate) non_finite_count: usize,
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
    if descriptor.element_type != "f32" || actual.len() % 4 != 0 {
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
        status: "comparison-passed",
        compared_elements: actual.len() / 4,
        mismatch_count,
        max_absolute_error: format_number(max_absolute_error),
        max_relative_error: format_number(max_relative_error),
        non_finite_count,
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
    use std::time::SystemTime;

    fn descriptor(expected: &[u8]) -> ProviderOutputComparisonDescriptor {
        ProviderOutputComparisonDescriptor {
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
            "nuis-provider-output-comparison-{}-{nonce}",
            std::process::id()
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
