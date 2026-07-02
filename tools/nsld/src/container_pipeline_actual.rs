use super::toml;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActualContainerFields {
    pub(crate) actual_container_layout_hash: Option<String>,
    pub(crate) actual_container_hash: Option<String>,
    pub(crate) actual_metadata_table_hash: Option<String>,
    pub(crate) actual_payload_size_bytes: Option<usize>,
    pub(crate) actual_payload_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) actual_container_section_table_hash: Option<String>,
    pub(crate) actual_loader_readiness: Option<String>,
    pub(crate) actual_loader_entry_kind: Option<String>,
    pub(crate) actual_loader_entry_symbol: Option<String>,
    pub(crate) actual_loader_entry_section_id: Option<String>,
    pub(crate) actual_loader_symbol_count: Option<usize>,
    pub(crate) actual_loader_symbol_id: Option<String>,
    pub(crate) actual_loader_symbol_kind: Option<String>,
    pub(crate) actual_loader_symbol_name: Option<String>,
    pub(crate) actual_loader_symbol_section_id: Option<String>,
    pub(crate) actual_loader_symbol_table_hash: Option<String>,
    pub(crate) actual_relocation_count: Option<usize>,
    pub(crate) actual_relocation_table_hash: Option<String>,
    pub(crate) actual_relocation_id: Option<String>,
    pub(crate) actual_relocation_kind: Option<String>,
    pub(crate) actual_relocation_source_section_id: Option<String>,
    pub(crate) actual_relocation_source_offset: Option<usize>,
    pub(crate) actual_relocation_target_symbol_id: Option<String>,
    pub(crate) actual_relocation_addend: Option<isize>,
    pub(crate) actual_external_import_count: Option<usize>,
    pub(crate) actual_external_import_table_hash: Option<String>,
    pub(crate) actual_external_import_id: Option<String>,
    pub(crate) actual_external_import_kind: Option<String>,
    pub(crate) actual_external_import_name: Option<String>,
    pub(crate) actual_external_import_provider: Option<String>,
    pub(crate) actual_external_import_required: Option<bool>,
}

pub(crate) fn actual_container_fields(
    actual: Result<&String, &String>,
    issues: &mut Vec<String>,
) -> ActualContainerFields {
    let source = match actual {
        Ok(source) => source,
        Err(error) => {
            issues.push(error.clone());
            return ActualContainerFields::default();
        }
    };

    ActualContainerFields {
        actual_container_layout_hash: toml::string_value(source, "container_layout_hash"),
        actual_container_hash: toml::string_value(source, "container_hash"),
        actual_metadata_table_hash: toml::string_value(source, "metadata_table_hash"),
        actual_payload_size_bytes: toml::usize_value(source, "payload_size_bytes"),
        actual_payload_hash: toml::string_value(source, "payload_hash"),
        actual_section_count: toml::usize_value(source, "section_count"),
        actual_container_section_table_hash: toml::string_value(
            source,
            "container_section_table_hash",
        ),
        actual_loader_readiness: toml::string_value(source, "loader_readiness"),
        actual_loader_entry_kind: toml::string_value(source, "loader_entry_kind"),
        actual_loader_entry_symbol: toml::string_value(source, "loader_entry_symbol"),
        actual_loader_entry_section_id: toml::string_value(source, "loader_entry_section_id"),
        actual_loader_symbol_count: toml::usize_value(source, "loader_symbol_count"),
        actual_loader_symbol_id: toml::first_table_string_value(
            source,
            "loader_symbol",
            "symbol_id",
        ),
        actual_loader_symbol_kind: toml::first_table_string_value(
            source,
            "loader_symbol",
            "symbol_kind",
        ),
        actual_loader_symbol_name: toml::first_table_string_value(
            source,
            "loader_symbol",
            "symbol_name",
        ),
        actual_loader_symbol_section_id: toml::first_table_string_value(
            source,
            "loader_symbol",
            "section_id",
        ),
        actual_loader_symbol_table_hash: toml::string_value(source, "loader_symbol_table_hash"),
        actual_relocation_count: toml::usize_value(source, "relocation_count"),
        actual_relocation_table_hash: toml::string_value(source, "relocation_table_hash"),
        actual_relocation_id: toml::first_table_string_value(source, "relocation", "relocation_id"),
        actual_relocation_kind: toml::first_table_string_value(
            source,
            "relocation",
            "relocation_kind",
        ),
        actual_relocation_source_section_id: toml::first_table_string_value(
            source,
            "relocation",
            "source_section_id",
        ),
        actual_relocation_source_offset: toml::first_table_usize_value(
            source,
            "relocation",
            "source_offset",
        ),
        actual_relocation_target_symbol_id: toml::first_table_string_value(
            source,
            "relocation",
            "target_symbol_id",
        ),
        actual_relocation_addend: toml::first_table_isize_value(source, "relocation", "addend"),
        actual_external_import_count: toml::usize_value(source, "external_import_count"),
        actual_external_import_table_hash: toml::string_value(source, "external_import_table_hash"),
        actual_external_import_id: toml::first_table_string_value(
            source,
            "external_import",
            "import_id",
        ),
        actual_external_import_kind: toml::first_table_string_value(
            source,
            "external_import",
            "import_kind",
        ),
        actual_external_import_name: toml::first_table_string_value(
            source,
            "external_import",
            "import_name",
        ),
        actual_external_import_provider: toml::first_table_string_value(
            source,
            "external_import",
            "provider",
        ),
        actual_external_import_required: toml::first_table_bool_value(
            source,
            "external_import",
            "required",
        ),
    }
}
