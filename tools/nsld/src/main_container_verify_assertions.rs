use super::{
    container::{NsldContainerReport, NsldContainerVerifyReport},
    fnv1a64_hex,
    main_container_domain_assertions::{
        assert_matching_native_object_contract, assert_matching_shader_contract,
    },
    reports::NsldPrepareReport,
};

pub(crate) fn assert_matching_container_artifacts(
    preview: &NsldContainerReport,
    prepare: &NsldPrepareReport,
    payload_bytes: &[u8],
    container_source: &str,
) {
    assert_eq!(preview.loader_symbols.len(), 3);
    assert_eq!(preview.relocations.len(), 3);
    assert!(preview
        .loader_symbols
        .iter()
        .any(|symbol| symbol.symbol_kind == "native-object-output"));
    assert_eq!(payload_bytes.len(), prepare.payload_size_bytes);
    assert_eq!(fnv1a64_hex(payload_bytes), prepare.payload_hash);
    assert!(container_source.contains("offset = 0"));
    assert!(container_source.contains("size_bytes = 17"));
    assert!(container_source.contains("loader_readiness = \"host-assisted\""));
    assert!(container_source.contains("external-import:final-stage-driver:clang"));
    assert!(container_source.contains("external-import:clang-target:"));
    assert!(container_source.contains("external-import:c-world-policy:wrapped"));
    assert!(container_source.contains("loader_entry_kind = \"lifecycle-bootstrap\""));
    assert!(container_source.contains("loader_entry_symbol = \"main\""));
    assert!(container_source.contains("loader_entry_section_id = \"sec0000.compiled-artifact\""));
    assert!(container_source.contains("loader_symbol_count = 3"));
    assert!(container_source.contains("loader_symbol_table_hash = \"0x"));
    assert!(container_source.contains("[[loader_symbol]]"));
    assert!(container_source.contains("symbol_id = \"sym0000.loader-entry\""));
    assert!(container_source.contains("symbol_name = \"main\""));
    assert!(container_source.contains("lifecycle_hook = \"on_lifecycle_bootstrap\""));
    assert!(container_source.contains("section_id = \"sec0000.compiled-artifact\""));
    assert!(container_source.contains("symbol_id = \"sym0001.hetero-node.shader.official.shader\""));
    assert!(container_source.contains("symbol_kind = \"hetero-node-dispatch\""));
    assert!(container_source.contains("symbol_name = \"t0001.shader\""));
    assert!(container_source.contains("lifecycle_hook = \"on_hetero_submission_progress\""));
    assert!(container_source.contains("symbol_id = \"sym0002.native-object-output\""));
    assert!(container_source.contains("symbol_kind = \"native-object-output\""));
    assert!(container_source.contains("symbol_name = \"__nuis_native_object\""));
    assert!(container_source.contains("lifecycle_hook = \"on_cffi_native_object\""));
    assert!(container_source.contains("relocation_count = 3"));
    assert!(container_source.contains("relocation_table_hash = \"0x"));
    assert!(container_source.contains("[[relocation]]"));
    assert!(container_source.contains("relocation_id = \"rel0000.lifecycle-entry\""));
    assert!(container_source.contains("relocation_kind = \"lifecycle-entry-binding\""));
    assert!(container_source.contains("source_section_id = \"sec0000.compiled-artifact\""));
    assert!(container_source.contains("target_symbol_id = \"sym0000.loader-entry\""));
    assert!(container_source.contains("relocation_id = \"rel0001.hetero-node\""));
    assert!(container_source.contains("relocation_kind = \"hetero-node-binding\""));
    assert!(container_source
        .contains("target_symbol_id = \"sym0001.hetero-node.shader.official.shader\""));
    assert!(container_source.contains("relocation_id = \"rel0002.native-object\""));
    assert!(container_source.contains("relocation_kind = \"native-object-binding\""));
    assert!(container_source.contains("target_symbol_id = \"sym0002.native-object-output\""));
    assert!(container_source.contains("compatibility_domain_count = 1"));
    assert!(container_source.contains("compatibility_domain_table_hash = \"0x"));
    assert!(container_source.contains("[[compatibility_domain]]"));
    assert!(container_source.contains("domain_id = \"compat0000.cffi-von-neumann\""));
    assert!(container_source.contains("domain_kind = \"cffi-host-compat\""));
    assert!(container_source.contains("paradigm = \"classic-von-neumann-host\""));
    assert!(container_source.contains("lifecycle_hook = \"on_cffi_native_object\""));
    assert!(container_source.contains("abi_family = \"mach-o\""));
    assert!(container_source.contains("wrapper_policy = \"wrapped\""));
    assert!(container_source.contains("external_import_count = 3"));
    assert!(container_source.contains("external_import_table_hash = \"0x"));
    assert!(container_source.contains("[[external_import]]"));
    assert!(container_source.contains("import_kind = \"final-stage-driver\""));
    assert!(container_source.contains("import_kind = \"clang-target\""));
    assert!(container_source.contains("import_kind = \"c-world-policy\""));
    assert!(container_source.contains("payload_size_bytes = "));
    assert!(container_source.contains("payload_hash = \"0x"));
    assert!(container_source.contains("container_section_table_hash = \"0x"));
    assert!(container_source.contains("metadata_table_hash = \"0x"));
    assert!(container_source.contains("section_kind = \"shader-lowering-sidecar-input\""));
    assert!(container_source.contains("section_kind = \"native-object-output\""));
}

pub(crate) fn assert_matching_container_verify_report(report: &NsldContainerVerifyReport) {
    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.actual_section_count, Some(6));
    assert_eq!(
        report.actual_container_section_table_hash.as_deref(),
        Some(report.expected_container_section_table_hash.as_str())
    );
    assert_eq!(
        report.actual_metadata_table_hash.as_deref(),
        Some(report.expected_metadata_table_hash.as_str())
    );
    assert_eq!(report.expected_loader_readiness, "host-assisted");
    assert_eq!(
        report.actual_loader_readiness.as_deref(),
        Some("host-assisted")
    );
    assert_eq!(report.expected_loader_entry_kind, "lifecycle-bootstrap");
    assert_eq!(
        report.actual_loader_entry_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(report.expected_loader_entry_symbol, "main");
    assert_eq!(report.actual_loader_entry_symbol.as_deref(), Some("main"));
    assert_eq!(
        report.expected_loader_entry_section_id,
        "sec0000.compiled-artifact"
    );
    assert_eq!(
        report.actual_loader_entry_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(report.expected_external_import_count, 3);
    assert_eq!(report.actual_external_import_count, Some(3));
    assert_eq!(report.expected_loader_symbol_count, 3);
    assert_eq!(report.actual_loader_symbol_count, Some(3));
    assert_eq!(
        report.actual_loader_symbol_table_hash.as_deref(),
        Some(report.expected_loader_symbol_table_hash.as_str())
    );
    assert_eq!(report.expected_loader_symbol_id, "sym0000.loader-entry");
    assert_eq!(
        report.actual_loader_symbol_id.as_deref(),
        Some("sym0000.loader-entry")
    );
    assert_eq!(report.expected_loader_symbol_kind, "lifecycle-bootstrap");
    assert_eq!(
        report.actual_loader_symbol_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(report.expected_loader_symbol_name, "main");
    assert_eq!(report.actual_loader_symbol_name.as_deref(), Some("main"));
    assert_eq!(
        report.expected_loader_symbol_section_id,
        "sec0000.compiled-artifact"
    );
    assert_eq!(
        report.actual_loader_symbol_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(report.expected_relocation_count, 3);
    assert_eq!(report.actual_relocation_count, Some(3));
    assert_eq!(
        report.actual_relocation_table_hash.as_deref(),
        Some(report.expected_relocation_table_hash.as_str())
    );
    assert_eq!(report.expected_relocation_id, "rel0000.lifecycle-entry");
    assert_eq!(
        report.actual_relocation_id.as_deref(),
        Some("rel0000.lifecycle-entry")
    );
    assert_eq!(report.expected_relocation_kind, "lifecycle-entry-binding");
    assert_eq!(
        report.actual_relocation_kind.as_deref(),
        Some("lifecycle-entry-binding")
    );
    assert_eq!(
        report.expected_relocation_source_section_id,
        "sec0000.compiled-artifact"
    );
    assert_eq!(
        report.actual_relocation_source_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(report.expected_relocation_source_offset, 0);
    assert_eq!(report.actual_relocation_source_offset, Some(0));
    assert_eq!(
        report.expected_relocation_target_symbol_id,
        "sym0000.loader-entry"
    );
    assert_eq!(
        report.actual_relocation_target_symbol_id.as_deref(),
        Some("sym0000.loader-entry")
    );
    assert_eq!(report.expected_relocation_addend, 0);
    assert_eq!(report.actual_relocation_addend, Some(0));
    assert_matching_compat_domain(report);
    assert_matching_external_import(report);
    assert_matching_shader_contract(report);
    assert_matching_native_object_contract(report);
}

pub(crate) fn assert_matching_container_verify_json(report_json: &str) {
    assert!(report_json.contains("\"expected_shader_section_present\":true"));
    assert!(report_json.contains("\"actual_shader_section_present\":true"));
    assert!(report_json.contains(
        "\"expected_shader_loader_symbol_id\":\"sym0001.hetero-node.shader.official.shader\""
    ));
    assert!(report_json.contains("\"actual_shader_relocation_id\":\"rel0001.hetero-node\""));
    assert!(report_json.contains("\"expected_native_object_section_present\":true"));
    assert!(report_json.contains("\"actual_native_object_section_present\":true"));
    assert!(report_json.contains("\"expected_compatibility_domain_count\":1"));
    assert!(report_json.contains("\"actual_compatibility_domain_count\":1"));
    assert!(report_json
        .contains("\"expected_compatibility_domain_paradigm\":\"classic-von-neumann-host\""));
    assert!(report_json
        .contains("\"actual_compatibility_domain_lifecycle_hook\":\"on_cffi_native_object\""));
    assert!(report_json
        .contains("\"expected_compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x"));
    assert!(report_json
        .contains("\"actual_compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x"));
    assert!(report_json
        .contains("\"expected_native_object_loader_symbol_id\":\"sym0002.native-object-output\""));
    assert!(
        report_json.contains("\"actual_native_object_relocation_id\":\"rel0002.native-object\"")
    );
}

pub(crate) fn assert_tampered_container_report(
    tampered_report: &NsldContainerVerifyReport,
    tampered_json: &str,
) {
    assert!(!tampered_report.valid);
    assert!(tampered_json.contains("\"container_section_issues\":["));
    assert!(tampered_json.contains("\"loader_symbol_issues\":["));
    assert!(tampered_json.contains("\"relocation_issues\":["));
    assert!(tampered_json.contains("\"compatibility_domain_issues\":["));
    assert!(tampered_json.contains("\"external_import_issues\":["));
    assert_eq!(
        tampered_report.actual_loader_readiness.as_deref(),
        Some("self-contained")
    );
    assert_eq!(
        tampered_report
            .actual_container_section_table_hash
            .as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_metadata_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert!(tampered_report
        .container_section_issues
        .iter()
        .any(|issue| issue.starts_with("container_section[1].section_id mismatch")));
    assert_tampered_loader_symbol(tampered_report);
    assert_tampered_relocation(tampered_report);
    assert_tampered_compat_domain(tampered_report);
    assert_tampered_external_import(tampered_report);
    for prefix in EXPECTED_TAMPERED_ISSUE_PREFIXES {
        assert!(
            tampered_report
                .issues
                .iter()
                .any(|issue| issue.starts_with(prefix)),
            "missing tampered issue prefix: {prefix}"
        );
    }
}

pub(crate) fn assert_corrupted_payload_report(corrupted_report: &NsldContainerVerifyReport) {
    assert!(!corrupted_report.valid);
    assert!(corrupted_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("payload_file_hash mismatch")));
    assert!(corrupted_report
        .section_range_issues
        .iter()
        .any(|issue| issue.starts_with("section_payload_hash mismatch")));
}

fn assert_matching_compat_domain(report: &NsldContainerVerifyReport) {
    assert_eq!(report.expected_compatibility_domain_count, 1);
    assert_eq!(report.actual_compatibility_domain_count, Some(1));
    assert_eq!(
        report.actual_compatibility_domain_table_hash.as_deref(),
        Some(report.expected_compatibility_domain_table_hash.as_str())
    );
    assert_eq!(
        report.expected_compatibility_domain_id,
        "compat0000.cffi-von-neumann"
    );
    assert_eq!(
        report.actual_compatibility_domain_id.as_deref(),
        Some("compat0000.cffi-von-neumann")
    );
    assert_eq!(
        report.expected_compatibility_domain_kind,
        "cffi-host-compat"
    );
    assert_eq!(
        report.actual_compatibility_domain_kind.as_deref(),
        Some("cffi-host-compat")
    );
    assert_eq!(
        report.expected_compatibility_domain_paradigm,
        "classic-von-neumann-host"
    );
    assert_eq!(
        report.actual_compatibility_domain_paradigm.as_deref(),
        Some("classic-von-neumann-host")
    );
    assert_eq!(
        report.expected_compatibility_domain_lifecycle_hook,
        "on_cffi_native_object"
    );
    assert_eq!(
        report.actual_compatibility_domain_lifecycle_hook.as_deref(),
        Some("on_cffi_native_object")
    );
    assert_eq!(report.expected_compatibility_domain_abi_family, "mach-o");
    assert_eq!(
        report.actual_compatibility_domain_abi_family.as_deref(),
        Some("mach-o")
    );
    assert_eq!(
        report.expected_compatibility_domain_wrapper_policy,
        "wrapped"
    );
    assert_eq!(
        report.actual_compatibility_domain_wrapper_policy.as_deref(),
        Some("wrapped")
    );
    assert!(report.expected_compatibility_domain_required);
    assert_eq!(report.actual_compatibility_domain_required, Some(true));
}

fn assert_matching_external_import(report: &NsldContainerVerifyReport) {
    assert_eq!(
        report.actual_external_import_table_hash.as_deref(),
        Some(report.expected_external_import_table_hash.as_str())
    );
    assert_eq!(
        report.expected_external_import_id,
        "imp0000.final-stage-driver"
    );
    assert_eq!(
        report.actual_external_import_id.as_deref(),
        Some("imp0000.final-stage-driver")
    );
    assert_eq!(report.expected_external_import_kind, "final-stage-driver");
    assert_eq!(
        report.actual_external_import_kind.as_deref(),
        Some("final-stage-driver")
    );
    assert_eq!(report.expected_external_import_name, "clang");
    assert_eq!(report.actual_external_import_name.as_deref(), Some("clang"));
    assert_eq!(report.expected_external_import_provider, "host-toolchain");
    assert_eq!(
        report.actual_external_import_provider.as_deref(),
        Some("host-toolchain")
    );
    assert!(report.expected_external_import_required);
    assert_eq!(report.actual_external_import_required, Some(true));
}

fn assert_tampered_loader_symbol(tampered_report: &NsldContainerVerifyReport) {
    assert_eq!(
        tampered_report.actual_loader_entry_kind.as_deref(),
        Some("manual-entry")
    );
    assert_eq!(
        tampered_report.actual_loader_entry_symbol.as_deref(),
        Some("alt")
    );
    assert_eq!(
        tampered_report.actual_loader_entry_section_id.as_deref(),
        Some("sec9999.missing")
    );
    assert_eq!(tampered_report.actual_external_import_count, Some(0));
    assert_eq!(tampered_report.actual_loader_symbol_count, Some(0));
    assert_eq!(
        tampered_report.actual_loader_symbol_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_id.as_deref(),
        Some("sym9999.manual")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_kind.as_deref(),
        Some("manual-symbol")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_name.as_deref(),
        Some("alt")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_section_id.as_deref(),
        Some("sec9999.missing")
    );
    assert!(tampered_report
        .loader_symbol_issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol[1].symbol_name mismatch")));
}

fn assert_tampered_relocation(tampered_report: &NsldContainerVerifyReport) {
    assert_eq!(tampered_report.actual_relocation_count, Some(4));
    assert_eq!(
        tampered_report.actual_relocation_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_relocation_id.as_deref(),
        Some("rel9999.manual")
    );
    assert_eq!(
        tampered_report.actual_relocation_kind.as_deref(),
        Some("manual-relocation")
    );
    assert_eq!(
        tampered_report
            .actual_relocation_source_section_id
            .as_deref(),
        Some("sec9999.missing")
    );
    assert_eq!(tampered_report.actual_relocation_source_offset, Some(7));
    assert_eq!(
        tampered_report
            .actual_relocation_target_symbol_id
            .as_deref(),
        Some("sym9999.manual")
    );
    assert_eq!(tampered_report.actual_relocation_addend, Some(-4));
    assert!(tampered_report
        .relocation_issues
        .iter()
        .any(|issue| issue.starts_with("relocation[1].target_symbol_id mismatch")));
}

fn assert_tampered_compat_domain(tampered_report: &NsldContainerVerifyReport) {
    assert_eq!(tampered_report.actual_compatibility_domain_count, Some(2));
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_table_hash
            .as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_compatibility_domain_id.as_deref(),
        Some("compat9999.manual")
    );
    assert_eq!(
        tampered_report.actual_compatibility_domain_kind.as_deref(),
        Some("manual-compat")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_paradigm
            .as_deref(),
        Some("manual-host")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_lifecycle_hook
            .as_deref(),
        Some("on_manual_compat")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_abi_family
            .as_deref(),
        Some("manual-object")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_wrapper_policy
            .as_deref(),
        Some("manual-wrapper")
    );
    assert_eq!(
        tampered_report.actual_compatibility_domain_required,
        Some(false)
    );
    assert!(tampered_report
        .compatibility_domain_issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain[0].domain_id mismatch")));
    assert!(tampered_report
        .compatibility_domain_issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain[0].paradigm mismatch")));
}

fn assert_tampered_external_import(tampered_report: &NsldContainerVerifyReport) {
    assert_eq!(
        tampered_report.actual_external_import_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_external_import_id.as_deref(),
        Some("imp9999.manual")
    );
    assert_eq!(
        tampered_report.actual_external_import_kind.as_deref(),
        Some("manual-driver")
    );
    assert!(tampered_report
        .external_import_issues
        .iter()
        .any(|issue| issue.starts_with("external_import[1].import_kind mismatch")));
    assert_eq!(
        tampered_report.actual_external_import_name.as_deref(),
        Some("manual-clang")
    );
    assert_eq!(
        tampered_report.actual_external_import_provider.as_deref(),
        Some("manual-provider")
    );
    assert_eq!(tampered_report.actual_external_import_required, Some(false));
}

const EXPECTED_TAMPERED_ISSUE_PREFIXES: &[&str] = &[
    "loader_readiness mismatch",
    "container_section_table_hash mismatch",
    "metadata_table_hash mismatch",
    "loader_entry_kind mismatch",
    "loader_entry_symbol mismatch",
    "loader_entry_section_id mismatch",
    "loader_symbol_count mismatch",
    "loader_symbol_table_hash mismatch",
    "loader_symbol_id mismatch",
    "loader_symbol_kind mismatch",
    "loader_symbol_name mismatch",
    "loader_symbol_section_id mismatch",
    "relocation_count mismatch",
    "relocation_table_hash mismatch",
    "relocation_id mismatch",
    "relocation_kind mismatch",
    "relocation_source_section_id mismatch",
    "relocation_source_offset mismatch",
    "relocation_target_symbol_id mismatch",
    "relocation_addend mismatch",
    "compatibility_domain_count mismatch",
    "compatibility_domain_table_hash mismatch",
    "compatibility_domain_id mismatch",
    "compatibility_domain_kind mismatch",
    "compatibility_domain_paradigm mismatch",
    "compatibility_domain_lifecycle_hook mismatch",
    "compatibility_domain_abi_family mismatch",
    "compatibility_domain_wrapper_policy mismatch",
    "compatibility_domain_required mismatch",
    "external_import_count mismatch",
    "external_import_table_hash mismatch",
    "external_import_id mismatch",
    "external_import_kind mismatch",
    "external_import_name mismatch",
    "external_import_provider mismatch",
    "external_import_required mismatch",
];
