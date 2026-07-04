use super::{
    container_verify::{self, TomlFieldKind},
    reports::{NsldObjectRelocationSeedDiagnostic, NsldObjectSectionDiagnostic},
};

pub(crate) fn object_section_table_field_issues(source: &str) -> Vec<String> {
    let mut issues = container_verify::table_field_issues(
        source,
        "object_section",
        "object_section",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("source_section_id", TomlFieldKind::String),
            ("source_section_kind", TomlFieldKind::String),
            ("object_section_name", TomlFieldKind::String),
            ("object_section_role", TomlFieldKind::String),
            ("source_path", TomlFieldKind::String),
            ("source_hash", TomlFieldKind::String),
            ("source_size_bytes", TomlFieldKind::Usize),
            ("payload_offset_seed", TomlFieldKind::Usize),
            ("file_offset_seed", TomlFieldKind::Usize),
            ("file_size_seed", TomlFieldKind::Usize),
            ("alignment", TomlFieldKind::Usize),
            ("required", TomlFieldKind::Bool),
        ],
    );
    issues.extend(container_verify::table_field_issues(
        &format!("[[object_plan_header]]\n{source}"),
        "object_plan_header",
        "object_plan_header",
        &[
            ("writer_target_id", TomlFieldKind::String),
            ("writer_backend_kind", TomlFieldKind::String),
            ("writer_status", TomlFieldKind::String),
            ("object_family", TomlFieldKind::String),
            ("unsupported_features", TomlFieldKind::Array),
        ],
    ));
    issues
}

pub(crate) fn relocation_seed_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "object_relocation_seed",
        "object_relocation_seed",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("relocation_seed_id", TomlFieldKind::String),
            ("relocation_seed_kind", TomlFieldKind::String),
            ("source_section_id", TomlFieldKind::String),
            ("source_offset_seed", TomlFieldKind::Usize),
            ("target_symbol", TomlFieldKind::String),
            ("addend", TomlFieldKind::Isize),
            ("native_relocation_ready", TomlFieldKind::Bool),
        ],
    )
}

pub(crate) fn object_section_entries(source: &str) -> Vec<NsldObjectSectionDiagnostic> {
    toml_table_blocks(source, "object_section")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectSectionDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_section_kind: toml_block_string_value(&block, "source_section_kind")?,
                object_section_name: toml_block_string_value(&block, "object_section_name")?,
                object_section_role: toml_block_string_value(&block, "object_section_role")?,
                source_path: toml_block_string_value(&block, "source_path")?,
                source_hash: toml_block_string_value(&block, "source_hash")?,
                source_size_bytes: toml_block_usize_value(&block, "source_size_bytes")?,
                payload_offset_seed: toml_block_usize_value(&block, "payload_offset_seed")?,
                file_offset_seed: toml_block_usize_value(&block, "file_offset_seed")?,
                file_size_seed: toml_block_usize_value(&block, "file_size_seed")?,
                alignment: toml_block_usize_value(&block, "alignment")?,
                required: toml_block_bool_value(&block, "required")?,
            })
        })
        .collect()
}

pub(crate) fn relocation_seed_entries(source: &str) -> Vec<NsldObjectRelocationSeedDiagnostic> {
    toml_table_blocks(source, "object_relocation_seed")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectRelocationSeedDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                relocation_seed_id: toml_block_string_value(&block, "relocation_seed_id")?,
                relocation_seed_kind: toml_block_string_value(&block, "relocation_seed_kind")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_offset_seed: toml_block_usize_value(&block, "source_offset_seed")?,
                target_symbol: toml_block_string_value(&block, "target_symbol")?,
                addend: toml_block_isize_value(&block, "addend")?,
                native_relocation_ready: toml_block_bool_value(&block, "native_relocation_ready")?,
            })
        })
        .collect()
}

pub(crate) fn object_section_table_mismatch_issues(
    expected: &[NsldObjectSectionDiagnostic],
    actual: &[NsldObjectSectionDiagnostic],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "object_section_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("object_section[{index}] missing"));
            continue;
        };
        push_object_section_mismatch(
            &mut issues,
            index,
            "order_index",
            expected_entry.order_index,
            actual_entry.order_index,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_section_id",
            &expected_entry.source_section_id,
            &actual_entry.source_section_id,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_section_kind",
            &expected_entry.source_section_kind,
            &actual_entry.source_section_kind,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "object_section_name",
            &expected_entry.object_section_name,
            &actual_entry.object_section_name,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "object_section_role",
            &expected_entry.object_section_role,
            &actual_entry.object_section_role,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_path",
            &expected_entry.source_path,
            &actual_entry.source_path,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_hash",
            &expected_entry.source_hash,
            &actual_entry.source_hash,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "source_size_bytes",
            expected_entry.source_size_bytes,
            actual_entry.source_size_bytes,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "payload_offset_seed",
            expected_entry.payload_offset_seed,
            actual_entry.payload_offset_seed,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "file_offset_seed",
            expected_entry.file_offset_seed,
            actual_entry.file_offset_seed,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "file_size_seed",
            expected_entry.file_size_seed,
            actual_entry.file_size_seed,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "alignment",
            expected_entry.alignment,
            actual_entry.alignment,
        );
        if actual_entry.required != expected_entry.required {
            issues.push(format!(
                "object_section[{index}].required mismatch: expected {}, found {}",
                expected_entry.required, actual_entry.required
            ));
        }
    }
    issues
}

pub(crate) fn relocation_seed_table_mismatch_issues(
    expected: &[NsldObjectRelocationSeedDiagnostic],
    actual: &[NsldObjectRelocationSeedDiagnostic],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "object_relocation_seed_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("object_relocation_seed[{index}] missing"));
            continue;
        };
        push_object_relocation_seed_mismatch(
            &mut issues,
            index,
            "order_index",
            expected_entry.order_index,
            actual_entry.order_index,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "relocation_seed_id",
            &expected_entry.relocation_seed_id,
            &actual_entry.relocation_seed_id,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "relocation_seed_kind",
            &expected_entry.relocation_seed_kind,
            &actual_entry.relocation_seed_kind,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "source_section_id",
            &expected_entry.source_section_id,
            &actual_entry.source_section_id,
        );
        push_object_relocation_seed_mismatch(
            &mut issues,
            index,
            "source_offset_seed",
            expected_entry.source_offset_seed,
            actual_entry.source_offset_seed,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "target_symbol",
            &expected_entry.target_symbol,
            &actual_entry.target_symbol,
        );
        if actual_entry.addend != expected_entry.addend {
            issues.push(format!(
                "object_relocation_seed[{index}].addend mismatch: expected {}, found {}",
                expected_entry.addend, actual_entry.addend
            ));
        }
        if actual_entry.native_relocation_ready != expected_entry.native_relocation_ready {
            issues.push(format!(
                "object_relocation_seed[{index}].native_relocation_ready mismatch: expected {}, found {}",
                expected_entry.native_relocation_ready, actual_entry.native_relocation_ready
            ));
        }
    }
    issues
}

fn push_object_section_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "object_section[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_object_section_string_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "object_section[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_object_relocation_seed_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "object_relocation_seed[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_object_relocation_seed_string_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "object_relocation_seed[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

pub(crate) fn toml_table_blocks<'a>(source: &'a str, table: &str) -> Vec<Vec<&'a str>> {
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

pub(crate) fn toml_block_string_value(block: &[&str], key: &str) -> Option<String> {
    toml_block_value(block, key).and_then(toml_decode_string_value)
}

pub(crate) fn toml_block_usize_value(block: &[&str], key: &str) -> Option<usize> {
    toml_block_value(block, key).and_then(|value| value.parse::<usize>().ok())
}

pub(crate) fn toml_block_isize_value(block: &[&str], key: &str) -> Option<isize> {
    toml_block_value(block, key).and_then(|value| value.parse::<isize>().ok())
}

pub(crate) fn toml_block_bool_value(block: &[&str], key: &str) -> Option<bool> {
    toml_block_value(block, key).and_then(|value| value.parse::<bool>().ok())
}

fn toml_block_value<'a>(block: &'a [&'a str], key: &str) -> Option<&'a str> {
    block.iter().find_map(|line| {
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim())
    })
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
