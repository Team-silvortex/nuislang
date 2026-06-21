use std::{collections::BTreeMap, path::Path};

use crate::ArtifactError;

pub(crate) fn parse_required_toml_string(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<String, ArtifactError> {
    parse_optional_toml_string(source, key)
        .ok_or_else(|| ArtifactError::new(format!("`{}` is missing required key `{key}`", path.display())))
}

pub(crate) fn parse_required_toml_usize(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_toml_usize(source, key)
        .ok_or_else(|| ArtifactError::new(format!("`{}` is missing required key `{key}`", path.display())))
}

pub(crate) fn parse_required_toml_bool(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<bool, ArtifactError> {
    parse_optional_toml_bool(source, key)
        .ok_or_else(|| ArtifactError::new(format!("`{}` is missing required key `{key}`", path.display())))
}

pub(crate) fn parse_required_toml_string_array(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<Vec<String>, ArtifactError> {
    parse_optional_toml_string_array(source, key)
        .ok_or_else(|| ArtifactError::new(format!("`{}` is missing required key `{key}`", path.display())))
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
                items.push(item[1..item.len() - 1].to_owned());
            }
            return Some(items);
        }
    }
    None
}

pub(crate) fn parse_optional_map_string(values: &BTreeMap<String, String>, key: &str) -> Option<String> {
    let value = values.get(key)?;
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        unescape_toml_basic_string(&value[1..value.len() - 1])
    } else {
        None
    }
}

pub(crate) fn parse_optional_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
    block_name: &str,
) -> Result<Option<usize>, ArtifactError> {
    let Some(value) = values.get(key) else {
        return Ok(None);
    };
    value.parse::<usize>().map(Some).map_err(|_| {
        ArtifactError::new(format!(
            "`{}` {block_name} key `{key}` must be an unsigned integer",
            manifest_path.display()
        ))
    })
}

pub(crate) fn parse_required_map_string_in_block(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
    block_name: &str,
) -> Result<String, ArtifactError> {
    let value = values.get(key).ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` {block_name} block is missing required key `{key}`",
            manifest_path.display()
        ))
    })?;
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return unescape_toml_basic_string(&value[1..value.len() - 1]).ok_or_else(|| {
            ArtifactError::new(format!(
                "`{}` {block_name} key `{key}` contains an unsupported escape sequence",
                manifest_path.display()
            ))
        });
    }
    Err(ArtifactError::new(format!(
        "`{}` {block_name} key `{key}` must be a quoted string",
        manifest_path.display()
    )))
}

pub(crate) fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub(crate) fn render_string_array(values: &[String]) -> String {
    let quoted = values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml_string(value)))
        .collect::<Vec<_>>();
    format!("[{}]", quoted.join(", "))
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
