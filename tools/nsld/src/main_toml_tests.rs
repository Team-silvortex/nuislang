use super::toml;
use crate::container_verify::{self, TomlFieldKind};

#[test]
fn scoped_toml_helpers_read_the_first_matching_table_only() {
    let source = r#"
[[loader_symbol]]
symbol_id = "sym0000.loader-entry"
section_id = "sec0000.compiled-artifact"

[[external_import]]
import_id = "imp0000.final-stage-driver"
required = true

[[section]]
section_id = "sec9999.section-table"

[[external_import]]
import_id = "imp0001.clang-target"
required = false
"#;

    assert_eq!(
        toml::first_table_string_value(source, "loader_symbol", "section_id").as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(
        toml::first_table_string_value(source, "external_import", "import_id").as_deref(),
        Some("imp0000.final-stage-driver")
    );
    assert_eq!(
        toml::first_table_bool_value(source, "external_import", "required"),
        Some(true)
    );
    assert_eq!(
        toml::first_table_string_value(source, "missing", "section_id"),
        None
    );
}

#[test]
fn table_field_issues_report_missing_and_invalid_fields() {
    let source = r#"
[[relocation]]
relocation_id = "rel0000.lifecycle-entry"
source_offset = "not-a-number"

[[relocation]]
relocation_id = "rel0001.hetero-node"
source_offset = 12
"#;

    let issues = container_verify::table_field_issues(
        source,
        "relocation",
        "relocation",
        &[
            ("relocation_id", TomlFieldKind::String),
            ("relocation_kind", TomlFieldKind::String),
            ("source_offset", TomlFieldKind::Usize),
        ],
    );

    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[0].relocation_kind missing"));
    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[0].source_offset invalid"));
    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[1].relocation_kind missing"));
}
