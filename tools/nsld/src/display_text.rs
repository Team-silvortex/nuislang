pub(crate) fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

pub(crate) fn optional_string_text(value: Option<&str>) -> String {
    value.unwrap_or("missing").to_owned()
}

pub(crate) fn optional_bool_text(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "absent".to_owned())
}
