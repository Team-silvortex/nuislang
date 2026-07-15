pub(super) fn string_value(source: &str, key: &str) -> Option<String> {
    let raw = raw_value(source, key)?.trim();
    let quoted = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
}

pub(super) fn bool_value(source: &str, key: &str) -> Option<bool> {
    match raw_value(source, key)?.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub(super) fn usize_value(source: &str, key: &str) -> Option<usize> {
    raw_value(source, key)?.trim().parse().ok()
}

pub(super) fn string_array_value(source: &str, key: &str) -> Vec<String> {
    let Some(raw) = raw_value(source, key) else {
        return Vec::new();
    };
    let Some(body) = raw
        .trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    body.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            let quoted = entry.strip_prefix('"')?.strip_suffix('"')?;
            Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
        })
        .collect()
}

pub(super) fn string_value_from_lines(lines: &[&str], key: &str) -> Option<String> {
    let raw = raw_value_from_lines(lines, key)?.trim();
    let quoted = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
}

pub(super) fn bool_value_from_lines(lines: &[&str], key: &str) -> Option<bool> {
    match raw_value_from_lines(lines, key)?.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn raw_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    raw_value_from_lines(&source.lines().collect::<Vec<_>>(), key)
}

fn raw_value_from_lines<'a>(lines: &[&'a str], key: &str) -> Option<&'a str> {
    lines.iter().copied().find_map(|raw| {
        let (found_key, value) = raw.trim().split_once('=')?;
        (found_key.trim() == key).then_some(value.trim())
    })
}

pub(super) fn first_array_table_block<'a>(source: &'a str, table: &str) -> Option<Vec<&'a str>> {
    array_table_blocks(source, table).into_iter().next()
}

pub(super) fn array_table_blocks<'a>(source: &'a str, table: &str) -> Vec<Vec<&'a str>> {
    let header = format!("[[{table}]]");
    let mut in_table = false;
    let mut blocks = Vec::new();
    let mut block = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            if in_table {
                blocks.push(block);
                block = Vec::new();
            }
            in_table = true;
            continue;
        }
        if in_table && trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            blocks.push(block);
            block = Vec::new();
            in_table = false;
            continue;
        }
        if in_table {
            block.push(line);
        }
    }
    if in_table {
        blocks.push(block);
    }
    blocks
        .into_iter()
        .filter(|block| !block.is_empty())
        .collect()
}
