use std::{collections::BTreeMap, fmt::Write as _, path::Path};

pub(crate) fn render_string_array(values: &[String]) -> String {
    let mut out = String::with_capacity(2 + values.len() * 24);
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        write!(out, "\"{}\"", escape_toml_string(value)).unwrap();
    }
    out.push(']');
    out
}

pub(crate) fn escape_toml_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

pub(crate) fn parse_required_toml_string(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<String, String> {
    parse_optional_toml_string(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

pub(crate) fn parse_required_toml_bool(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<bool, String> {
    parse_optional_toml_bool(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

pub(crate) fn parse_required_toml_usize(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<usize, String> {
    parse_optional_toml_usize(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

pub(crate) fn parse_required_toml_string_array(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<Vec<String>, String> {
    parse_optional_toml_string_array(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

pub(crate) fn parse_optional_toml_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                return unescape_toml_basic_string(&value[1..value.len() - 1]);
            }
            return None;
        }
    }
    None
}

pub(crate) fn parse_optional_toml_bool(source: &str, key: &str) -> Option<bool> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return match rest.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
    }
    None
}

pub(crate) fn parse_optional_toml_usize(source: &str, key: &str) -> Option<usize> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest.trim().parse::<usize>().ok();
        }
    }
    None
}

pub(crate) fn parse_optional_toml_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if !value.starts_with('[') || !value.ends_with(']') {
                return None;
            }
            let inner = value[1..value.len() - 1].trim();
            if inner.is_empty() {
                return Some(Vec::new());
            }
            let mut items = Vec::new();
            for part in inner.split(',') {
                let item = part.trim();
                if !item.starts_with('"') || !item.ends_with('"') || item.len() < 2 {
                    return None;
                }
                items.push(unescape_toml_basic_string(&item[1..item.len() - 1])?);
            }
            return Some(items);
        }
    }
    None
}

pub(crate) fn parse_required_map_string(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
) -> Result<String, String> {
    parse_required_map_string_in_block(values, key, manifest_path, "artifact_hash")
}

pub(crate) fn parse_required_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
) -> Result<usize, String> {
    let value = values.get(key).ok_or_else(|| {
        format!(
            "`{}` artifact_hash block is missing required key `{key}`",
            manifest_path.display()
        )
    })?;
    value.parse::<usize>().map_err(|_| {
        format!(
            "`{}` artifact_hash key `{key}` must be an unsigned integer",
            manifest_path.display()
        )
    })
}

pub(crate) fn parse_required_map_string_in_block(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
    block_name: &str,
) -> Result<String, String> {
    let value = values.get(key).ok_or_else(|| {
        format!(
            "`{}` {block_name} block is missing required key `{key}`",
            manifest_path.display()
        )
    })?;
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return unescape_toml_basic_string(&value[1..value.len() - 1]).ok_or_else(|| {
            format!(
                "`{}` {block_name} key `{key}` contains an unsupported escape sequence",
                manifest_path.display()
            )
        });
    }
    Err(format!(
        "`{}` {block_name} key `{key}` must be a quoted string",
        manifest_path.display()
    ))
}

fn unescape_toml_basic_string(value: &str) -> Option<String> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let escaped = chars.next()?;
        match escaped {
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            _ => return None,
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_string_roundtrip_handles_basic_escapes() {
        let source = format!("value = \"{}\"\n", escape_toml_string("a\\b\"c\n"));

        assert_eq!(
            parse_optional_toml_string(&source, "value").as_deref(),
            Some("a\\b\"c\n")
        );
    }

    #[test]
    fn string_array_roundtrip_handles_empty_and_escaped_items() {
        let values = vec![
            "alpha".to_owned(),
            "b\"c".to_owned(),
            "line\nend".to_owned(),
        ];
        let source = format!("items = {}\nempty = []\n", render_string_array(&values));

        assert_eq!(
            parse_optional_toml_string_array(&source, "items"),
            Some(values)
        );
        assert_eq!(
            parse_optional_toml_string_array(&source, "empty"),
            Some(Vec::new())
        );
    }
}
