pub(crate) fn optional_bool_text(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

pub(crate) fn push_usize_mismatch(
    issues: &mut Vec<String>,
    key: &str,
    expected: usize,
    actual: Option<usize>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{key} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

pub(crate) fn push_bool_mismatch(
    issues: &mut Vec<String>,
    key: &str,
    expected: bool,
    actual: Option<bool>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{key} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

pub(crate) fn push_optional_string_mismatch(
    issues: &mut Vec<String>,
    key: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if actual != expected {
        issues.push(format!(
            "{key} mismatch: expected {}, found {}",
            expected.unwrap_or("missing"),
            actual.unwrap_or("missing")
        ));
    }
}
