use std::path::Path;

use super::*;

fn component_identity() -> &'static str {
    "1111111111111111111111111111111111111111111111111111111111111111"
}

fn rejected_report() -> CompilerDiagnosticReport {
    let diagnostics = [
        CompilerDiagnosticInput {
            module: "cpu.Scanner",
            code: "NBS004",
            path: "items[2]",
            message: "effect is outside the bootstrap subset",
        },
        CompilerDiagnosticInput {
            module: "cpu.Scanner",
            code: "NBS001",
            path: "items[0]",
            message: "domain is not admitted",
        },
    ];
    build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: "nuisc-stage0-reference",
        component_record_sha256: component_identity(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        accepted: false,
        semantic_pipeline: "skipped",
        semantic_error: None,
        diagnostics: &diagnostics,
    })
    .expect("build rejected report")
}

#[test]
fn diagnostic_report_normalizes_and_round_trips() {
    let report = rejected_report();
    assert_eq!(report.diagnostic_count, 2);
    assert_eq!(report.diagnostics[0].code, "NBS001");
    assert_eq!(report.diagnostics[1].code, "NBS004");
    let source = render_compiler_diagnostic_report(&report);
    let parsed = parse_compiler_diagnostic_report_from_source(
        &source,
        Path::new("nuis.compiler-diagnostics.toml"),
    )
    .expect("parse diagnostic report");
    assert_eq!(parsed, report);
    assert_eq!(render_compiler_diagnostic_report(&parsed), source);
}

#[test]
fn accepted_report_must_be_clean_and_checked() {
    let report = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: "nuisc-stage0-reference",
        component_record_sha256: component_identity(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        accepted: true,
        semantic_pipeline: "checked",
        semantic_error: None,
        diagnostics: &[],
    })
    .expect("build accepted report");
    assert!(report.accepted);
    assert_eq!(report.diagnostic_count, 0);

    let diagnostic = [CompilerDiagnosticInput {
        module: "cpu.Scanner",
        code: "NBS001",
        path: "module",
        message: "rejected",
    }];
    let error = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: "nuisc-stage0-reference",
        component_record_sha256: component_identity(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        accepted: true,
        semantic_pipeline: "checked",
        semantic_error: None,
        diagnostics: &diagnostic,
    })
    .expect_err("accepted report with diagnostics must fail");
    assert!(error.to_string().contains("diagnostic-free"));
}

#[test]
fn diagnostic_report_rejects_identity_tampering() {
    let source = render_compiler_diagnostic_report(&rejected_report());
    let tampered = source.replacen("NBS001", "NBS002", 1);
    let error = parse_compiler_diagnostic_report_from_source(
        &tampered,
        Path::new("nuis.compiler-diagnostics.toml"),
    )
    .expect_err("tampered diagnostic must fail");
    assert!(error.to_string().contains("record identity mismatch"));
}

#[test]
fn diagnostic_report_binds_component_and_producer() {
    let report = rejected_report();
    assert_eq!(report.component_record_sha256, component_identity());
    assert_eq!(report.producer_id, "nuisc-stage0-reference");
    assert_eq!(report.report_sha256.len(), 64);
}

#[test]
fn diagnostic_modules_accept_utf8_but_paths_reject_host_locations() {
    let diagnostic = [CompilerDiagnosticInput {
        module: "cpu.扫描器",
        code: "NBS001",
        path: "items/0",
        message: "类型不在自举子集内",
    }];
    build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: "nuisc-stage0-reference",
        component_record_sha256: component_identity(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        accepted: false,
        semantic_pipeline: "skipped",
        semantic_error: None,
        diagnostics: &diagnostic,
    })
    .expect("UTF-8 diagnostic module and message are valid");

    let absolute = [CompilerDiagnosticInput {
        path: "/host/source.ns",
        ..diagnostic[0]
    }];
    let error = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: "nuisc-stage0-reference",
        component_record_sha256: component_identity(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        accepted: false,
        semantic_pipeline: "skipped",
        semantic_error: None,
        diagnostics: &absolute,
    })
    .expect_err("absolute diagnostic path must fail");
    assert!(error.to_string().contains("portable logical path"));

    let drive = [CompilerDiagnosticInput {
        path: "C:/host/source.ns",
        ..diagnostic[0]
    }];
    let error = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: "nuisc-stage0-reference",
        component_record_sha256: component_identity(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        accepted: false,
        semantic_pipeline: "skipped",
        semantic_error: None,
        diagnostics: &drive,
    })
    .expect_err("drive-prefixed diagnostic path must fail");
    assert!(error.to_string().contains("portable logical path"));
}
