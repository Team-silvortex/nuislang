use super::{
    final_executable_render::{optional_bool_toml, optional_usize_toml},
    reports::NsldFinalExecutableEmitReport,
};

pub(crate) fn push_final_output_emit_verify_mismatches(
    issues: &mut Vec<String>,
    expected: &NsldFinalExecutableEmitReport,
    actual_checked: Option<bool>,
    actual_present: Option<bool>,
    actual_size_bytes: Option<usize>,
    actual_hash: Option<String>,
    actual_image_header_valid: Option<bool>,
    actual_runnable_candidate: Option<bool>,
) {
    if actual_checked != Some(expected.final_output_checked) {
        issues.push(format!(
            "final_output_checked mismatch: expected {}, found {}",
            expected.final_output_checked,
            actual_checked
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if actual_present != Some(expected.final_output_present) {
        issues.push(format!(
            "final_output_present mismatch: expected {}, found {}",
            expected.final_output_present,
            actual_present
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if actual_size_bytes != expected.final_output_size_bytes {
        issues.push(format!(
            "final_output_size_bytes mismatch: expected {}, found {}",
            optional_usize_toml(expected.final_output_size_bytes),
            optional_usize_toml(actual_size_bytes)
        ));
    }
    if actual_hash != expected.final_output_hash {
        issues.push(format!(
            "final_output_hash mismatch: expected {}, found {}",
            expected
                .final_output_hash
                .clone()
                .unwrap_or_else(|| "missing".to_owned()),
            actual_hash.unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if actual_image_header_valid != expected.final_output_image_header_valid {
        issues.push(format!(
            "final_output_image_header_valid mismatch: expected {}, found {}",
            optional_bool_toml(expected.final_output_image_header_valid),
            optional_bool_toml(actual_image_header_valid)
        ));
    }
    if actual_runnable_candidate != expected.final_output_runnable_candidate {
        issues.push(format!(
            "final_output_runnable_candidate mismatch: expected {}, found {}",
            optional_bool_toml(expected.final_output_runnable_candidate),
            optional_bool_toml(actual_runnable_candidate)
        ));
    }
}
