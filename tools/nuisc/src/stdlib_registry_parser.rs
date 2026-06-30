use std::path::Path;

use super::{StdlibIndexModule, StdlibLibraryImportPolicy};

pub(in crate::stdlib_registry) fn parse_stdlib_index_modules(
    source: &str,
    path: &Path,
) -> Result<Vec<StdlibIndexModule>, String> {
    let blocks = split_table_array_blocks(source, "[[module]]");
    let mut modules = Vec::new();
    for block in blocks {
        modules.push(StdlibIndexModule {
            name: parse_required_string(&block, "name", path)?,
            kind: parse_required_string(&block, "kind", path)?,
            path: parse_required_string(&block, "path", path)?,
            package_id: parse_required_string(&block, "package_id", path)?,
            depends_on: parse_optional_string_array(&block, "depends_on").unwrap_or_default(),
            summary: parse_required_string(&block, "summary", path)?,
        });
    }
    Ok(modules)
}

fn split_table_array_blocks(source: &str, marker: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut active = false;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == marker {
            if active && !current.is_empty() {
                blocks.push(current.join("\n"));
                current.clear();
            }
            active = true;
            continue;
        }
        if active {
            current.push(raw_line.to_owned());
        }
    }
    if active && !current.is_empty() {
        blocks.push(current.join("\n"));
    }
    blocks
}

pub(in crate::stdlib_registry) fn parse_required_string(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<String, String> {
    parse_optional_string(source, key).ok_or_else(|| {
        format!(
            "stdlib manifest `{}` is missing required field `{}`",
            path.display(),
            key
        )
    })
}

pub(in crate::stdlib_registry) fn parse_library_import_policy(
    source: &str,
    path: &Path,
) -> Result<StdlibLibraryImportPolicy, String> {
    let Some(value) = parse_optional_string(source, "library_import_policy") else {
        return Ok(StdlibLibraryImportPolicy::ProjectAuto);
    };
    match value.as_str() {
        "project-auto" => Ok(StdlibLibraryImportPolicy::ProjectAuto),
        "manual-only" => Ok(StdlibLibraryImportPolicy::ManualOnly),
        other => Err(format!(
            "stdlib manifest `{}` declares unsupported library_import_policy `{other}`; expected `project-auto` or `manual-only`",
            path.display()
        )),
    }
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

pub(in crate::stdlib_registry) fn parse_optional_string_array(
    source: &str,
    key: &str,
) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    let mut lines = source.lines();
    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let mut collected = rest.trim().to_owned();
            if !collected.contains(']') {
                for next_line in lines.by_ref() {
                    collected.push(' ');
                    collected.push_str(next_line.trim());
                    if next_line.contains(']') {
                        break;
                    }
                }
            }
            let body = collected.trim();
            let body = body.strip_prefix('[')?.strip_suffix(']')?;
            let mut values = Vec::new();
            for item in split_quoted_array_items(body)? {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                values.push(parse_quoted(item)?);
            }
            return Some(values);
        }
    }
    None
}

fn split_quoted_array_items(inner: &str) -> Option<Vec<&str>> {
    let mut items = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0;
    for (index, ch) in inner.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ',' if !in_string => {
                items.push(&inner[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if in_string || escaped {
        return None;
    }
    items.push(&inner[start..]);
    Some(items)
}

fn parse_quoted(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}
