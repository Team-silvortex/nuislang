use super::container::NsldContainerVerifyReport;

pub(crate) fn tampered_container_source(
    container_source: &str,
    report: &NsldContainerVerifyReport,
) -> String {
    container_source
        .replace(
            "loader_readiness = \"host-assisted\"",
            "loader_readiness = \"self-contained\"",
        )
        .replace(
            &format!(
                "metadata_table_hash = \"{}\"",
                report.expected_metadata_table_hash
            ),
            "metadata_table_hash = \"0x0000000000000000\"",
        )
        .replace(
            &format!(
                "container_section_table_hash = \"{}\"",
                report.expected_container_section_table_hash
            ),
            "container_section_table_hash = \"0x0000000000000000\"",
        )
        .replace(
            "section_id = \"sec0001.nsld-link-input-table\"",
            "section_id = \"sec9999.manual-section\"",
        )
        .replace(
            "loader_entry_kind = \"lifecycle-bootstrap\"",
            "loader_entry_kind = \"manual-entry\"",
        )
        .replace(
            "loader_entry_symbol = \"main\"",
            "loader_entry_symbol = \"alt\"",
        )
        .replace(
            "loader_entry_section_id = \"sec0000.compiled-artifact\"",
            "loader_entry_section_id = \"sec9999.missing\"",
        )
        .replace("loader_symbol_count = 3", "loader_symbol_count = 0")
        .replace(
            &format!(
                "loader_symbol_table_hash = \"{}\"",
                report.expected_loader_symbol_table_hash
            ),
            "loader_symbol_table_hash = \"0x0000000000000000\"",
        )
        .replace(
            "symbol_id = \"sym0000.loader-entry\"",
            "symbol_id = \"sym9999.manual\"",
        )
        .replace(
            "symbol_kind = \"lifecycle-bootstrap\"",
            "symbol_kind = \"manual-symbol\"",
        )
        .replace("symbol_name = \"main\"", "symbol_name = \"alt\"")
        .replace(
            "symbol_name = \"t0001.shader\"",
            "symbol_name = \"t9999.shader.manual\"",
        )
        .replace(
            "section_id = \"sec0000.compiled-artifact\"",
            "section_id = \"sec9999.missing\"",
        )
        .replace("relocation_count = 3", "relocation_count = 4")
        .replace(
            "relocation_id = \"rel0000.lifecycle-entry\"",
            "relocation_id = \"rel9999.manual\"",
        )
        .replace(
            "relocation_kind = \"lifecycle-entry-binding\"",
            "relocation_kind = \"manual-relocation\"",
        )
        .replace(
            "source_section_id = \"sec0000.compiled-artifact\"",
            "source_section_id = \"sec9999.missing\"",
        )
        .replace("source_offset = 0", "source_offset = 7")
        .replace(
            "target_symbol_id = \"sym0000.loader-entry\"",
            "target_symbol_id = \"sym9999.manual\"",
        )
        .replace(
            "target_symbol_id = \"sym0001.hetero-node.shader.official.shader\"",
            "target_symbol_id = \"sym9999.hetero.manual\"",
        )
        .replace("addend = 0", "addend = -4")
        .replace(
            &format!(
                "relocation_table_hash = \"{}\"",
                report.expected_relocation_table_hash
            ),
            "relocation_table_hash = \"0x0000000000000000\"",
        )
        .replace("external_import_count = 3", "external_import_count = 0")
        .replace(
            &format!(
                "external_import_table_hash = \"{}\"",
                report.expected_external_import_table_hash
            ),
            "external_import_table_hash = \"0x0000000000000000\"",
        )
        .replace(
            "import_id = \"imp0000.final-stage-driver\"",
            "import_id = \"imp9999.manual\"",
        )
        .replace(
            "import_kind = \"final-stage-driver\"",
            "import_kind = \"manual-driver\"",
        )
        .replace(
            "import_kind = \"clang-target\"",
            "import_kind = \"manual-clang-target\"",
        )
        .replace("import_name = \"clang\"", "import_name = \"manual-clang\"")
        .replace(
            "provider = \"host-toolchain\"",
            "provider = \"manual-provider\"",
        )
        .replace("required = true", "required = false")
}
