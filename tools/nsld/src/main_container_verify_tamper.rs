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
            "lifecycle_hook = \"on_lifecycle_bootstrap\"",
            "lifecycle_hook = \"on_manual_lifecycle\"",
        )
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
        .replace(
            "compatibility_domain_count = 1",
            "compatibility_domain_count = 2",
        )
        .replace(
            &format!(
                "compatibility_domain_table_hash = \"{}\"",
                report.expected_compatibility_domain_table_hash
            ),
            "compatibility_domain_table_hash = \"0x0000000000000000\"",
        )
        .replace(
            "domain_id = \"compat0000.cffi-von-neumann\"",
            "domain_id = \"compat9999.manual\"",
        )
        .replace(
            "domain_kind = \"cffi-host-compat\"",
            "domain_kind = \"manual-compat\"",
        )
        .replace(
            "paradigm = \"classic-von-neumann-host\"",
            "paradigm = \"manual-host\"",
        )
        .replace(
            "lifecycle_hook = \"on_cffi_native_object\"",
            "lifecycle_hook = \"on_manual_compat\"",
        )
        .replace("abi_family = \"mach-o\"", "abi_family = \"manual-object\"")
        .replace(
            "wrapper_policy = \"wrapped\"",
            "wrapper_policy = \"manual-wrapper\"",
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
