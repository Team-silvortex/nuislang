use super::{
    final_executable_image_relocation::relocation_patch_preview_table_hash,
    final_executable_verify_helpers::{non_empty_toml_string, optional_usize_value},
    reports::NsldFinalExecutableRelocationPatchPreviewRecord,
    toml,
};
use std::path::Path;

pub(crate) struct NsldFinalExecutableImageDryRunActual {
    pub(crate) source: String,
    pub(crate) layout_hash: Option<String>,
    pub(crate) byte_map_hash: Option<String>,
    pub(crate) image_header_size: Option<usize>,
    pub(crate) payload_byte_offset: Option<usize>,
    pub(crate) image_constructed: Option<bool>,
    pub(crate) image_ready: Option<bool>,
    pub(crate) image_size_bytes: Option<usize>,
    pub(crate) image_hash: Option<String>,
    pub(crate) scheduler_metadata_payload_id: Option<String>,
    pub(crate) scheduler_metadata_present: Option<bool>,
    pub(crate) scheduler_metadata_offset: Option<usize>,
    pub(crate) scheduler_metadata_hash: Option<String>,
    pub(crate) relocation_application_strategy: Option<String>,
    pub(crate) relocation_application_count: Option<usize>,
    pub(crate) relocation_application_table_hash: Option<String>,
    pub(crate) relocation_application_audit_status: Option<String>,
    pub(crate) relocation_application_audit_count: Option<usize>,
    pub(crate) relocation_application_audit_table_hash: Option<String>,
    pub(crate) relocation_application_audit_blockers: Vec<String>,
    pub(crate) relocation_patch_preview_status: Option<String>,
    pub(crate) relocation_patch_preview_count: Option<usize>,
    pub(crate) relocation_patch_preview_table_hash: Option<String>,
    pub(crate) relocation_patch_preview_records:
        Vec<NsldFinalExecutableRelocationPatchPreviewRecord>,
    pub(crate) relocation_patch_preview_entry_count: Option<usize>,
    pub(crate) relocation_patch_preview_record_table_hash: Option<String>,
    pub(crate) relocation_patch_application_status: Option<String>,
    pub(crate) relocation_patch_application_count: Option<usize>,
    pub(crate) relocation_patch_application_table_hash: Option<String>,
    pub(crate) relocation_patch_application_blockers: Vec<String>,
    pub(crate) blockers: Vec<String>,
}

pub(crate) fn read_final_executable_image_dry_run_actual(
    input_path: &Path,
) -> Result<NsldFinalExecutableImageDryRunActual, String> {
    let source = std::fs::read_to_string(input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_image_dry_run `{}`: {error}",
            input_path.display()
        )
    })?;
    let relocation_patch_preview_records = relocation_patch_preview_records_from_source(&source);
    let relocation_patch_preview_entry_count = Some(relocation_patch_preview_records.len());
    let relocation_patch_preview_record_table_hash = Some(relocation_patch_preview_table_hash(
        &relocation_patch_preview_records,
    ));
    Ok(NsldFinalExecutableImageDryRunActual {
        layout_hash: toml::string_value(&source, "layout_hash"),
        byte_map_hash: toml::string_value(&source, "byte_map_hash"),
        image_header_size: toml::usize_value(&source, "image_header_size"),
        payload_byte_offset: toml::usize_value(&source, "payload_byte_offset"),
        image_constructed: toml::bool_value(&source, "image_constructed"),
        image_ready: toml::bool_value(&source, "image_ready"),
        image_size_bytes: optional_usize_value(&source, "image_size_bytes"),
        image_hash: non_empty_toml_string(&source, "image_hash"),
        scheduler_metadata_payload_id: non_empty_toml_string(
            &source,
            "scheduler_metadata_payload_id",
        ),
        scheduler_metadata_present: toml::bool_value(&source, "scheduler_metadata_present"),
        scheduler_metadata_offset: optional_usize_value(&source, "scheduler_metadata_offset"),
        scheduler_metadata_hash: non_empty_toml_string(&source, "scheduler_metadata_hash"),
        relocation_application_strategy: non_empty_toml_string(
            &source,
            "relocation_application_strategy",
        ),
        relocation_application_count: toml::usize_value(&source, "relocation_application_count"),
        relocation_application_table_hash: non_empty_toml_string(
            &source,
            "relocation_application_table_hash",
        ),
        relocation_application_audit_status: non_empty_toml_string(
            &source,
            "relocation_application_audit_status",
        ),
        relocation_application_audit_count: toml::usize_value(
            &source,
            "relocation_application_audit_count",
        ),
        relocation_application_audit_table_hash: non_empty_toml_string(
            &source,
            "relocation_application_audit_table_hash",
        ),
        relocation_application_audit_blockers: toml::string_array_value(
            &source,
            "relocation_application_audit_blockers",
        ),
        relocation_patch_preview_status: non_empty_toml_string(
            &source,
            "relocation_patch_preview_status",
        ),
        relocation_patch_preview_count: toml::usize_value(
            &source,
            "relocation_patch_preview_count",
        ),
        relocation_patch_preview_table_hash: non_empty_toml_string(
            &source,
            "relocation_patch_preview_table_hash",
        ),
        relocation_patch_preview_records,
        relocation_patch_preview_entry_count,
        relocation_patch_preview_record_table_hash,
        relocation_patch_application_status: non_empty_toml_string(
            &source,
            "relocation_patch_application_status",
        ),
        relocation_patch_application_count: toml::usize_value(
            &source,
            "relocation_patch_application_count",
        ),
        relocation_patch_application_table_hash: non_empty_toml_string(
            &source,
            "relocation_patch_application_table_hash",
        ),
        relocation_patch_application_blockers: toml::string_array_value(
            &source,
            "relocation_patch_application_blockers",
        ),
        blockers: toml::string_array_value(&source, "blockers"),
        source,
    })
}

fn relocation_patch_preview_records_from_source(
    source: &str,
) -> Vec<NsldFinalExecutableRelocationPatchPreviewRecord> {
    toml_table_blocks(source, "relocation_patch_preview")
        .into_iter()
        .filter_map(|block| {
            Some(NsldFinalExecutableRelocationPatchPreviewRecord {
                order_index: toml_block_usize_value(&block, "order_index")?,
                relocation_id: toml_block_string_value(&block, "relocation_id")?,
                patch_kind: toml_block_string_value(&block, "patch_kind")?,
                patch_offset: toml_block_usize_value(&block, "patch_offset")?,
                patch_width_bytes: toml_block_usize_value(&block, "patch_width_bytes")?,
                resolved_patch_value: toml_block_optional_usize_value(
                    &block,
                    "resolved_patch_value",
                )?,
                patch_value_hash: toml_block_string_value(&block, "patch_value_hash")?,
                target_symbol_id: toml_block_string_value(&block, "target_symbol_id")?,
                target_symbol_image_offset: toml_block_optional_usize_value(
                    &block,
                    "target_symbol_image_offset",
                )?,
                preview_status: toml_block_string_value(&block, "preview_status")?,
                resolver_status: toml_block_string_value(&block, "resolver_status")?,
            })
        })
        .collect()
}

fn toml_table_blocks<'a>(source: &'a str, table: &str) -> Vec<Vec<&'a str>> {
    let header = format!("[[{table}]]");
    let mut blocks = Vec::new();
    let mut current: Option<Vec<&'a str>> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            if let Some(block) = current.take() {
                blocks.push(block);
            }
            current = (trimmed == header).then(Vec::new);
        } else if let Some(block) = current.as_mut() {
            block.push(line);
        }
    }
    if let Some(block) = current {
        blocks.push(block);
    }
    blocks
}

fn toml_block_string_value(block: &[&str], key: &str) -> Option<String> {
    let raw = toml_block_value(block, key)?.trim();
    let quoted = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn toml_block_usize_value(block: &[&str], key: &str) -> Option<usize> {
    toml_block_value(block, key)?.trim().parse().ok()
}

fn toml_block_optional_usize_value(block: &[&str], key: &str) -> Option<Option<usize>> {
    Some(match toml_block_value(block, key)?.trim() {
        "0" => None,
        raw => Some(raw.parse().ok()?),
    })
}

fn toml_block_value<'a>(block: &'a [&str], key: &str) -> Option<&'a str> {
    let prefix = format!("{key} =");
    block
        .iter()
        .map(|line| line.trim())
        .find_map(|line| line.strip_prefix(&prefix))
}
