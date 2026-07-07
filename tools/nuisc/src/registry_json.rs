use std::fmt::Write as _;

pub(crate) fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => write!(out, "\\u{:04x}", ch as u32).unwrap(),
            ch => out.push(ch),
        }
    }
    out
}

pub(crate) fn json_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape(value))
}

pub(crate) fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\":\"{}\"", name, json_escape(value)),
        None => format!("\"{}\":null", name),
    }
}

pub(crate) fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

pub(crate) fn json_string_array_field(name: &str, values: &[String]) -> String {
    let mut out = String::with_capacity(name.len() + values.len() * 16 + 4);
    write!(out, "\"{}\":[", name).unwrap();
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write!(out, "\"{}\"", json_escape(value)).unwrap();
    }
    out.push(']');
    out
}

pub(crate) fn json_object_field(name: &str, fields: &[String]) -> String {
    format!("\"{}\":{{{}}}", name, fields.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_string_array_field_escapes_values_without_intermediate_join() {
        let values = vec![
            "plain".to_owned(),
            "quote\"slash\\line\n".to_owned(),
            "tab\tcarriage\r".to_owned(),
        ];

        assert_eq!(
            json_string_array_field("items", &values),
            "\"items\":[\"plain\",\"quote\\\"slash\\\\line\\n\",\"tab\\tcarriage\\r\"]"
        );
    }
}
