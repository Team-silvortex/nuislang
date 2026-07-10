use super::{
    container_verify::{self, TomlFieldKind},
    object_plan_verify::{
        object_section_table_mismatch_issues, relocation_seed_table_mismatch_issues,
        toml_block_bool_value, toml_block_isize_value, toml_block_string_value,
        toml_block_usize_value, toml_table_blocks,
    },
    reports::{NsldObjectRelocationSeedDiagnostic, NsldObjectSectionDiagnostic},
};

pub(crate) fn push_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &str,
    actual: Option<&str>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual.unwrap_or("missing")
        ));
    }
}

pub(crate) fn push_usize_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: usize,
    actual: Option<usize>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

pub(crate) fn writer_input_section_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "writer_section",
        "writer_section",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("source_section_id", TomlFieldKind::String),
            ("object_section_name", TomlFieldKind::String),
            ("object_section_role", TomlFieldKind::String),
            ("source_path", TomlFieldKind::String),
            ("source_hash", TomlFieldKind::String),
            ("source_size_bytes", TomlFieldKind::Usize),
            ("file_offset_seed", TomlFieldKind::Usize),
            ("file_size_seed", TomlFieldKind::Usize),
            ("alignment", TomlFieldKind::Usize),
            ("required", TomlFieldKind::Bool),
        ],
    )
}

pub(crate) fn writer_input_relocation_seed_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "writer_relocation_seed",
        "writer_relocation_seed",
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

pub(crate) fn writer_section_entries(source: &str) -> Vec<NsldObjectSectionDiagnostic> {
    toml_table_blocks(source, "writer_section")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectSectionDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_section_kind: String::new(),
                object_section_name: toml_block_string_value(&block, "object_section_name")?,
                object_section_role: toml_block_string_value(&block, "object_section_role")?,
                source_path: toml_block_string_value(&block, "source_path")?,
                source_hash: toml_block_string_value(&block, "source_hash")?,
                source_size_bytes: toml_block_usize_value(&block, "source_size_bytes")?,
                payload_offset_seed: 0,
                file_offset_seed: toml_block_usize_value(&block, "file_offset_seed")?,
                file_size_seed: toml_block_usize_value(&block, "file_size_seed")?,
                alignment: toml_block_usize_value(&block, "alignment")?,
                required: toml_block_bool_value(&block, "required")?,
            })
        })
        .collect()
}

pub(crate) fn writer_relocation_seed_entries(
    source: &str,
) -> Vec<NsldObjectRelocationSeedDiagnostic> {
    toml_table_blocks(source, "writer_relocation_seed")
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

pub(crate) fn writer_section_table_mismatch_issues(
    expected: &[NsldObjectSectionDiagnostic],
    actual: &[NsldObjectSectionDiagnostic],
) -> Vec<String> {
    object_section_table_mismatch_issues(expected, actual)
        .into_iter()
        .filter(|issue| {
            !issue.contains(".source_section_kind mismatch")
                && !issue.contains(".payload_offset_seed mismatch")
        })
        .map(|issue| {
            issue
                .replace("object_section_entry_count", "writer_section_entry_count")
                .replace("object_section[", "writer_section[")
        })
        .collect()
}

pub(crate) fn writer_relocation_seed_table_mismatch_issues(
    expected: &[NsldObjectRelocationSeedDiagnostic],
    actual: &[NsldObjectRelocationSeedDiagnostic],
) -> Vec<String> {
    relocation_seed_table_mismatch_issues(expected, actual)
        .into_iter()
        .map(|issue| {
            issue
                .replace(
                    "object_relocation_seed_entry_count",
                    "writer_relocation_seed_entry_count",
                )
                .replace("object_relocation_seed[", "writer_relocation_seed[")
        })
        .collect()
}
