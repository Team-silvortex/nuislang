pub(crate) fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{name}\":{value}")
}

pub(crate) fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

pub(crate) fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", json_escape(value))
}

pub(crate) fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{name}\":{value}")
}

pub(crate) fn json_isize_field(name: &str, value: isize) -> String {
    format!("\"{name}\":{value}")
}

pub(crate) fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

pub(crate) fn json_optional_isize_field(name: &str, value: Option<isize>) -> String {
    match value {
        Some(value) => json_isize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

pub(crate) fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => json_string_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

pub(crate) fn json_string_array_field(name: &str, values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{name}\":[{body}]")
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
