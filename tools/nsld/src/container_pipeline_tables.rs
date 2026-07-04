use super::{container, container_verify};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ContainerTableIssueSets {
    pub(crate) container_section_issues: Vec<String>,
    pub(crate) loader_symbol_issues: Vec<String>,
    pub(crate) relocation_issues: Vec<String>,
    pub(crate) compatibility_domain_issues: Vec<String>,
    pub(crate) external_import_issues: Vec<String>,
}

pub(crate) fn container_table_issue_sets(
    source: &str,
    expected_report: &container::NsldContainerReport,
) -> ContainerTableIssueSets {
    let mut container_section_issues = container_verify::container_section_issues(
        &expected_report.sections,
        &container_verify::container_section_entries(source),
    );
    container_section_issues.extend(container_verify::table_field_issues(
        source,
        "section",
        "container_section",
        &[
            ("order_index", container_verify::TomlFieldKind::Usize),
            ("section_id", container_verify::TomlFieldKind::String),
            ("section_kind", container_verify::TomlFieldKind::String),
            ("source_path", container_verify::TomlFieldKind::String),
            ("source_hash", container_verify::TomlFieldKind::String),
            ("payload_hash", container_verify::TomlFieldKind::String),
            ("required", container_verify::TomlFieldKind::Bool),
            ("offset", container_verify::TomlFieldKind::Usize),
            ("size_bytes", container_verify::TomlFieldKind::Usize),
        ],
    ));

    let mut loader_symbol_issues = container_verify::loader_symbol_issues(
        &expected_report.loader_symbols,
        &container_verify::loader_symbol_entries(source),
    );
    loader_symbol_issues.extend(container_verify::table_field_issues(
        source,
        "loader_symbol",
        "loader_symbol",
        &[
            ("symbol_id", container_verify::TomlFieldKind::String),
            ("symbol_kind", container_verify::TomlFieldKind::String),
            ("symbol_name", container_verify::TomlFieldKind::String),
            ("lifecycle_hook", container_verify::TomlFieldKind::String),
            ("section_id", container_verify::TomlFieldKind::String),
            ("offset", container_verify::TomlFieldKind::Usize),
            ("size_bytes", container_verify::TomlFieldKind::Usize),
            ("payload_hash", container_verify::TomlFieldKind::String),
        ],
    ));

    let mut relocation_issues = container_verify::relocation_issues(
        &expected_report.relocations,
        &container_verify::relocation_entries(source),
    );
    relocation_issues.extend(container_verify::table_field_issues(
        source,
        "relocation",
        "relocation",
        &[
            ("relocation_id", container_verify::TomlFieldKind::String),
            ("relocation_kind", container_verify::TomlFieldKind::String),
            ("source_section_id", container_verify::TomlFieldKind::String),
            ("source_offset", container_verify::TomlFieldKind::Usize),
            ("target_symbol_id", container_verify::TomlFieldKind::String),
            ("addend", container_verify::TomlFieldKind::Isize),
        ],
    ));

    let mut compatibility_domain_issues = container_verify::compatibility_domain_issues(
        &expected_report.compatibility_domains,
        &container_verify::compatibility_domain_entries(source),
    );
    compatibility_domain_issues.extend(container_verify::table_field_issues(
        source,
        "compatibility_domain",
        "compatibility_domain",
        &[
            ("domain_id", container_verify::TomlFieldKind::String),
            ("domain_kind", container_verify::TomlFieldKind::String),
            ("paradigm", container_verify::TomlFieldKind::String),
            ("lifecycle_hook", container_verify::TomlFieldKind::String),
            ("abi_family", container_verify::TomlFieldKind::String),
            ("wrapper_policy", container_verify::TomlFieldKind::String),
            ("required", container_verify::TomlFieldKind::Bool),
        ],
    ));

    let mut external_import_issues = container_verify::external_import_issues(
        &expected_report.external_imports,
        &container_verify::external_import_entries(source),
    );
    external_import_issues.extend(container_verify::table_field_issues(
        source,
        "external_import",
        "external_import",
        &[
            ("import_id", container_verify::TomlFieldKind::String),
            ("import_kind", container_verify::TomlFieldKind::String),
            ("import_name", container_verify::TomlFieldKind::String),
            ("provider", container_verify::TomlFieldKind::String),
            ("required", container_verify::TomlFieldKind::Bool),
        ],
    ));

    ContainerTableIssueSets {
        container_section_issues,
        loader_symbol_issues,
        relocation_issues,
        compatibility_domain_issues,
        external_import_issues,
    }
}
