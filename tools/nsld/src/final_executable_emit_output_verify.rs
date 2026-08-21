use super::{
    final_executable_render::{optional_bool_toml, optional_usize_toml},
    reports::NsldFinalExecutableEmitReport,
};

pub(crate) struct FinalOutputEmitActual<'a> {
    pub(crate) checked: Option<bool>,
    pub(crate) present: Option<bool>,
    pub(crate) size_bytes: Option<usize>,
    pub(crate) hash: Option<&'a str>,
    pub(crate) image_header_valid: Option<bool>,
    pub(crate) runnable_candidate: Option<bool>,
}

pub(crate) fn push_final_output_emit_verify_mismatches(
    issues: &mut Vec<String>,
    expected: &NsldFinalExecutableEmitReport,
    actual: FinalOutputEmitActual<'_>,
) {
    if actual.checked != Some(expected.final_output_checked) {
        issues.push(format!(
            "final_output_checked mismatch: expected {}, found {}",
            expected.final_output_checked,
            actual
                .checked
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if actual.present != Some(expected.final_output_present) {
        issues.push(format!(
            "final_output_present mismatch: expected {}, found {}",
            expected.final_output_present,
            actual
                .present
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if actual.size_bytes != expected.final_output_size_bytes {
        issues.push(format!(
            "final_output_size_bytes mismatch: expected {}, found {}",
            optional_usize_toml(expected.final_output_size_bytes),
            optional_usize_toml(actual.size_bytes)
        ));
    }
    if actual.hash != expected.final_output_hash.as_deref() {
        issues.push(format!(
            "final_output_hash mismatch: expected {}, found {}",
            expected
                .final_output_hash
                .clone()
                .unwrap_or_else(|| "missing".to_owned()),
            actual.hash.unwrap_or("missing")
        ));
    }
    if actual.image_header_valid != expected.final_output_image_header_valid {
        issues.push(format!(
            "final_output_image_header_valid mismatch: expected {}, found {}",
            optional_bool_toml(expected.final_output_image_header_valid),
            optional_bool_toml(actual.image_header_valid)
        ));
    }
    if actual.runnable_candidate != expected.final_output_runnable_candidate {
        issues.push(format!(
            "final_output_runnable_candidate mismatch: expected {}, found {}",
            optional_bool_toml(expected.final_output_runnable_candidate),
            optional_bool_toml(actual.runnable_candidate)
        ));
    }
}
