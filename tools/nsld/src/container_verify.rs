use super::container;

#[derive(Clone, Copy)]
pub(crate) enum TomlFieldKind {
    Array,
    Bool,
    Isize,
    String,
    Usize,
}

pub(crate) fn container_section_entries(source: &str) -> Vec<container::NsldContainerSectionEntry> {
    toml_table_blocks(source, "section")
        .into_iter()
        .filter_map(|block| {
            Some(container::NsldContainerSectionEntry {
                order_index: toml_block_usize_value(&block, "order_index")?,
                section_id: toml_block_string_value(&block, "section_id")?,
                section_kind: toml_block_string_value(&block, "section_kind")?,
                source_path: toml_block_string_value(&block, "source_path")?,
                source_hash: toml_block_string_value(&block, "source_hash")?,
                payload_hash: toml_block_string_value(&block, "payload_hash")?,
                required: toml_block_bool_value(&block, "required")?,
                offset: toml_block_usize_value(&block, "offset")?,
                size_bytes: toml_block_usize_value(&block, "size_bytes")?,
            })
        })
        .collect()
}

pub(crate) fn loader_symbol_entries(source: &str) -> Vec<container::NsldContainerLoaderSymbol> {
    toml_table_blocks(source, "loader_symbol")
        .into_iter()
        .filter_map(|block| {
            Some(container::NsldContainerLoaderSymbol {
                symbol_id: toml_block_string_value(&block, "symbol_id")?,
                symbol_kind: toml_block_string_value(&block, "symbol_kind")?,
                symbol_name: toml_block_string_value(&block, "symbol_name")?,
                section_id: toml_block_string_value(&block, "section_id")?,
                offset: toml_block_usize_value(&block, "offset")?,
                size_bytes: toml_block_usize_value(&block, "size_bytes")?,
                payload_hash: toml_block_string_value(&block, "payload_hash")?,
            })
        })
        .collect()
}

pub(crate) fn relocation_entries(source: &str) -> Vec<container::NsldContainerRelocationEntry> {
    toml_table_blocks(source, "relocation")
        .into_iter()
        .filter_map(|block| {
            Some(container::NsldContainerRelocationEntry {
                relocation_id: toml_block_string_value(&block, "relocation_id")?,
                relocation_kind: toml_block_string_value(&block, "relocation_kind")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_offset: toml_block_usize_value(&block, "source_offset")?,
                target_symbol_id: toml_block_string_value(&block, "target_symbol_id")?,
                addend: toml_block_isize_value(&block, "addend")?,
            })
        })
        .collect()
}

pub(crate) fn external_import_entries(source: &str) -> Vec<container::NsldContainerExternalImport> {
    toml_table_blocks(source, "external_import")
        .into_iter()
        .filter_map(|block| {
            Some(container::NsldContainerExternalImport {
                import_id: toml_block_string_value(&block, "import_id")?,
                import_kind: toml_block_string_value(&block, "import_kind")?,
                import_name: toml_block_string_value(&block, "import_name")?,
                provider: toml_block_string_value(&block, "provider")?,
                required: toml_block_bool_value(&block, "required")?,
            })
        })
        .collect()
}

pub(crate) fn container_section_issues(
    expected: &[container::NsldContainerSectionEntry],
    actual: &[container::NsldContainerSectionEntry],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "container_section_table_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("container_section[{index}] missing"));
            continue;
        };
        if actual_entry.order_index != expected_entry.order_index {
            issues.push(format!(
                "container_section[{index}].order_index mismatch: expected {}, found {}",
                expected_entry.order_index, actual_entry.order_index
            ));
        }
        if actual_entry.section_id != expected_entry.section_id {
            issues.push(format!(
                "container_section[{index}].section_id mismatch: expected {}, found {}",
                expected_entry.section_id, actual_entry.section_id
            ));
        }
        if actual_entry.section_kind != expected_entry.section_kind {
            issues.push(format!(
                "container_section[{index}].section_kind mismatch: expected {}, found {}",
                expected_entry.section_kind, actual_entry.section_kind
            ));
        }
        if actual_entry.source_path != expected_entry.source_path {
            issues.push(format!(
                "container_section[{index}].source_path mismatch: expected {}, found {}",
                expected_entry.source_path, actual_entry.source_path
            ));
        }
        if actual_entry.source_hash != expected_entry.source_hash {
            issues.push(format!(
                "container_section[{index}].source_hash mismatch: expected {}, found {}",
                expected_entry.source_hash, actual_entry.source_hash
            ));
        }
        if actual_entry.payload_hash != expected_entry.payload_hash {
            issues.push(format!(
                "container_section[{index}].payload_hash mismatch: expected {}, found {}",
                expected_entry.payload_hash, actual_entry.payload_hash
            ));
        }
        if actual_entry.required != expected_entry.required {
            issues.push(format!(
                "container_section[{index}].required mismatch: expected {}, found {}",
                expected_entry.required, actual_entry.required
            ));
        }
        if actual_entry.offset != expected_entry.offset {
            issues.push(format!(
                "container_section[{index}].offset mismatch: expected {}, found {}",
                expected_entry.offset, actual_entry.offset
            ));
        }
        if actual_entry.size_bytes != expected_entry.size_bytes {
            issues.push(format!(
                "container_section[{index}].size_bytes mismatch: expected {}, found {}",
                expected_entry.size_bytes, actual_entry.size_bytes
            ));
        }
    }
    issues
}

pub(crate) fn loader_symbol_issues(
    expected: &[container::NsldContainerLoaderSymbol],
    actual: &[container::NsldContainerLoaderSymbol],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "loader_symbol_table_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("loader_symbol[{index}] missing"));
            continue;
        };
        if actual_entry.symbol_id != expected_entry.symbol_id {
            issues.push(format!(
                "loader_symbol[{index}].symbol_id mismatch: expected {}, found {}",
                expected_entry.symbol_id, actual_entry.symbol_id
            ));
        }
        if actual_entry.symbol_kind != expected_entry.symbol_kind {
            issues.push(format!(
                "loader_symbol[{index}].symbol_kind mismatch: expected {}, found {}",
                expected_entry.symbol_kind, actual_entry.symbol_kind
            ));
        }
        if actual_entry.symbol_name != expected_entry.symbol_name {
            issues.push(format!(
                "loader_symbol[{index}].symbol_name mismatch: expected {}, found {}",
                expected_entry.symbol_name, actual_entry.symbol_name
            ));
        }
        if actual_entry.section_id != expected_entry.section_id {
            issues.push(format!(
                "loader_symbol[{index}].section_id mismatch: expected {}, found {}",
                expected_entry.section_id, actual_entry.section_id
            ));
        }
        if actual_entry.offset != expected_entry.offset {
            issues.push(format!(
                "loader_symbol[{index}].offset mismatch: expected {}, found {}",
                expected_entry.offset, actual_entry.offset
            ));
        }
        if actual_entry.size_bytes != expected_entry.size_bytes {
            issues.push(format!(
                "loader_symbol[{index}].size_bytes mismatch: expected {}, found {}",
                expected_entry.size_bytes, actual_entry.size_bytes
            ));
        }
        if actual_entry.payload_hash != expected_entry.payload_hash {
            issues.push(format!(
                "loader_symbol[{index}].payload_hash mismatch: expected {}, found {}",
                expected_entry.payload_hash, actual_entry.payload_hash
            ));
        }
    }
    issues
}

pub(crate) fn relocation_issues(
    expected: &[container::NsldContainerRelocationEntry],
    actual: &[container::NsldContainerRelocationEntry],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "relocation_table_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("relocation[{index}] missing"));
            continue;
        };
        if actual_entry.relocation_id != expected_entry.relocation_id {
            issues.push(format!(
                "relocation[{index}].relocation_id mismatch: expected {}, found {}",
                expected_entry.relocation_id, actual_entry.relocation_id
            ));
        }
        if actual_entry.relocation_kind != expected_entry.relocation_kind {
            issues.push(format!(
                "relocation[{index}].relocation_kind mismatch: expected {}, found {}",
                expected_entry.relocation_kind, actual_entry.relocation_kind
            ));
        }
        if actual_entry.source_section_id != expected_entry.source_section_id {
            issues.push(format!(
                "relocation[{index}].source_section_id mismatch: expected {}, found {}",
                expected_entry.source_section_id, actual_entry.source_section_id
            ));
        }
        if actual_entry.source_offset != expected_entry.source_offset {
            issues.push(format!(
                "relocation[{index}].source_offset mismatch: expected {}, found {}",
                expected_entry.source_offset, actual_entry.source_offset
            ));
        }
        if actual_entry.target_symbol_id != expected_entry.target_symbol_id {
            issues.push(format!(
                "relocation[{index}].target_symbol_id mismatch: expected {}, found {}",
                expected_entry.target_symbol_id, actual_entry.target_symbol_id
            ));
        }
        if actual_entry.addend != expected_entry.addend {
            issues.push(format!(
                "relocation[{index}].addend mismatch: expected {}, found {}",
                expected_entry.addend, actual_entry.addend
            ));
        }
    }
    issues
}

pub(crate) fn external_import_issues(
    expected: &[container::NsldContainerExternalImport],
    actual: &[container::NsldContainerExternalImport],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "external_import_table_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("external_import[{index}] missing"));
            continue;
        };
        if actual_entry.import_id != expected_entry.import_id {
            issues.push(format!(
                "external_import[{index}].import_id mismatch: expected {}, found {}",
                expected_entry.import_id, actual_entry.import_id
            ));
        }
        if actual_entry.import_kind != expected_entry.import_kind {
            issues.push(format!(
                "external_import[{index}].import_kind mismatch: expected {}, found {}",
                expected_entry.import_kind, actual_entry.import_kind
            ));
        }
        if actual_entry.import_name != expected_entry.import_name {
            issues.push(format!(
                "external_import[{index}].import_name mismatch: expected {}, found {}",
                expected_entry.import_name, actual_entry.import_name
            ));
        }
        if actual_entry.provider != expected_entry.provider {
            issues.push(format!(
                "external_import[{index}].provider mismatch: expected {}, found {}",
                expected_entry.provider, actual_entry.provider
            ));
        }
        if actual_entry.required != expected_entry.required {
            issues.push(format!(
                "external_import[{index}].required mismatch: expected {}, found {}",
                expected_entry.required, actual_entry.required
            ));
        }
    }
    issues
}

pub(crate) fn table_field_issues(
    source: &str,
    table: &str,
    issue_prefix: &str,
    fields: &[(&str, TomlFieldKind)],
) -> Vec<String> {
    let mut issues = Vec::new();
    for (index, block) in toml_table_blocks(source, table).iter().enumerate() {
        for (field, kind) in fields {
            let Some(value) = toml_block_value(block, field) else {
                issues.push(format!("{issue_prefix}[{index}].{field} missing"));
                continue;
            };
            if !toml_field_kind_matches(value, *kind) {
                issues.push(format!("{issue_prefix}[{index}].{field} invalid"));
            }
        }
    }
    issues
}

fn toml_table_blocks<'a>(source: &'a str, table: &str) -> Vec<Vec<&'a str>> {
    let header = format!("[[{table}]]");
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut in_target_table = false;

    for raw in source.lines() {
        let line = raw.trim();
        if line.starts_with("[[") && line.ends_with("]]") {
            if in_target_table {
                blocks.push(current);
                current = Vec::new();
            }
            in_target_table = line == header;
            continue;
        }
        if in_target_table {
            current.push(line);
        }
    }
    if in_target_table {
        blocks.push(current);
    }

    blocks
}

fn toml_block_string_value(block: &[&str], key: &str) -> Option<String> {
    toml_block_value(block, key).and_then(toml_decode_string_value)
}

fn toml_block_usize_value(block: &[&str], key: &str) -> Option<usize> {
    toml_block_value(block, key).and_then(|value| value.parse::<usize>().ok())
}

fn toml_block_isize_value(block: &[&str], key: &str) -> Option<isize> {
    toml_block_value(block, key).and_then(|value| value.parse::<isize>().ok())
}

fn toml_block_bool_value(block: &[&str], key: &str) -> Option<bool> {
    toml_block_value(block, key).and_then(|value| value.parse::<bool>().ok())
}

fn toml_block_value<'a>(block: &'a [&'a str], key: &str) -> Option<&'a str> {
    block.iter().find_map(|line| {
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim())
    })
}

fn toml_field_kind_matches(value: &str, kind: TomlFieldKind) -> bool {
    match kind {
        TomlFieldKind::Array => value.starts_with('[') && value.ends_with(']'),
        TomlFieldKind::Bool => value.parse::<bool>().is_ok(),
        TomlFieldKind::Isize => value.parse::<isize>().is_ok(),
        TomlFieldKind::String => toml_decode_string_value(value).is_some(),
        TomlFieldKind::Usize => value.parse::<usize>().is_ok(),
    }
}

fn toml_decode_string_value(value: &str) -> Option<String> {
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
