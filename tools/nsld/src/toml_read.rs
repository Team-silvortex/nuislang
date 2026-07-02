pub(crate) fn string_value(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        if found_key.trim() != key {
            return None;
        }
        decode_string_value(value.trim())
    })
}

pub(crate) fn usize_value(source: &str, key: &str) -> Option<usize> {
    source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key)
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    })
}

pub(crate) fn first_table_string_value(source: &str, table: &str, key: &str) -> Option<String> {
    first_table_value(source, table, key).and_then(decode_string_value)
}

pub(crate) fn first_table_bool_value(source: &str, table: &str, key: &str) -> Option<bool> {
    first_table_value(source, table, key).and_then(|value| value.parse::<bool>().ok())
}

pub(crate) fn first_table_usize_value(source: &str, table: &str, key: &str) -> Option<usize> {
    first_table_value(source, table, key).and_then(|value| value.parse::<usize>().ok())
}

pub(crate) fn first_table_isize_value(source: &str, table: &str, key: &str) -> Option<isize> {
    first_table_value(source, table, key).and_then(|value| value.parse::<isize>().ok())
}

pub(crate) fn string_array_value(source: &str, key: &str) -> Vec<String> {
    let Some(value) = source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim().to_owned())
    }) else {
        return Vec::new();
    };
    let Some(body) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    body.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            entry
                .strip_prefix('"')
                .and_then(|entry| entry.strip_suffix('"'))
                .map(str::to_owned)
        })
        .collect()
}

fn first_table_value<'a>(source: &'a str, table: &str, key: &str) -> Option<&'a str> {
    let header = format!("[[{table}]]");
    let mut in_target_table = false;

    for raw in source.lines() {
        let line = raw.trim();
        if line.starts_with("[[") && line.ends_with("]]") {
            if in_target_table {
                return None;
            }
            in_target_table = line == header;
            continue;
        }
        if !in_target_table {
            continue;
        }
        if let Some((found_key, value)) = line.split_once('=') {
            if found_key.trim() == key {
                return Some(value.trim());
            }
        }
    }

    None
}

fn decode_string_value(value: &str) -> Option<String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| {
            value
                .replace("\\n", "\n")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
        })
}
