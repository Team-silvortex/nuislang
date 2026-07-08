pub(crate) fn json_escape_local(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

pub(crate) fn json_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape_local(value))
}

pub(crate) fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\":\"{}\"", name, json_escape_local(value)),
        None => format!("\"{}\":null", name),
    }
}

pub(crate) fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

pub(crate) fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{}\":{}", name, value)
}

pub(crate) fn json_i64_field(name: &str, value: i64) -> String {
    format!("\"{}\":{}", name, value)
}

pub(crate) fn json_u128_field(name: &str, value: u128) -> String {
    format!("\"{}\":{}", name, value)
}

pub(crate) fn json_optional_u128_field(name: &str, value: Option<u128>) -> String {
    match value {
        Some(value) => json_u128_field(name, value),
        None => format!("\"{}\":null", name),
    }
}

pub(crate) fn json_optional_i64_field(name: &str, value: Option<i64>) -> String {
    match value {
        Some(value) => format!("\"{}\":{}", name, value),
        None => format!("\"{}\":null", name),
    }
}

pub(crate) fn json_string_array_field(name: &str, values: &[String]) -> String {
    let mut out = String::new();
    out.push('"');
    out.push_str(name);
    out.push_str("\":[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape_local(value));
        out.push('"');
    }
    out.push(']');
    out
}
