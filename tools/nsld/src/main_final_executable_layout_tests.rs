use super::{
    main_test_support::empty_link_plan, nsld_emit_final_executable_layout_plan_report,
    nsld_final_executable_layout_plan_report, nsld_prepare_report,
    nsld_verify_final_executable_layout_plan_report,
};
use std::{env, fs, path::Path};

#[test]
fn final_executable_layout_plan_captures_nsld_owned_binary_boundary() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-layout-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_layout_plan_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.layout_hash.starts_with("0x"));
    assert_eq!(report.internal_binary_format, "nuis-hetero-unified-binary");
    assert_eq!(report.lifecycle_entry_hook, "on_process_start");
    assert_eq!(
        report.scheduler_contract,
        "deterministic-lifecycle-hook-order"
    );
    assert_eq!(
        report.scheduler_metadata_payload,
        "payload0004.scheduler-metadata"
    );
    assert_eq!(
        report.scheduler_metadata_lifecycle_hook,
        "on_scheduler_metadata_load"
    );
    assert_eq!(report.scheduler_hetero_node_count, 0);
    assert_eq!(report.scheduler_wait_event_count, 0);
    assert_eq!(report.scheduler_emit_event_count, 0);
    assert_eq!(
        report.data_segment_ordering,
        "deterministic-data-segment-order"
    );
    assert_eq!(
        report.relocation_application_strategy,
        "nsb-loader-relocation-table"
    );
    assert_eq!(
        report.relocation_application_table_source,
        "payload0000.container"
    );
    assert!(report.relocation_application_count > 0);
    assert!(report.relocation_application_table_hash.starts_with("0x"));
    assert_eq!(
        report.relocation_application_count,
        report.relocation_applications.len()
    );
    assert!(report.relocation_applications.iter().all(|record| {
        record.source_payload_id == "payload0000.container"
            && record.application_status == "planned"
            && record.image_offset >= record.source_offset
    }));
    assert_eq!(report.compatibility_domain, "cffi-native-object");
    assert_eq!(report.compatibility_lifecycle_hook, "on_cffi_native_object");
    assert_eq!(report.payload_count, report.payloads.len());
    assert!(report
        .payload_names
        .iter()
        .any(|payload| payload == "nsld-container"));
    assert!(report
        .payload_names
        .iter()
        .any(|payload| payload == "native-object-output"));
    assert!(report
        .payload_names
        .iter()
        .any(|payload| payload == "nsld-scheduler-metadata"));
    assert!(report.payloads.iter().any(|payload| {
        payload.payload_id == "payload0003.native-object"
            && payload.lifecycle_hook == "on_cffi_native_object"
            && payload.content_hash.starts_with("0x")
    }));
    assert_eq!(report.byte_alignment, 16);
    assert!(report.byte_span > 0);
    assert!(report.byte_map_hash.starts_with("0x"));
    assert_eq!(report.byte_map_entries.len(), report.payloads.len());
    assert!(report
        .byte_map_entries
        .iter()
        .all(|entry| entry.offset % entry.alignment == 0));
    assert!(report
        .byte_map_entries
        .windows(2)
        .all(|entries| { entries[0].offset + entries[0].size_bytes <= entries[1].offset }));
    assert!(report
        .notes
        .iter()
        .any(|note| note == "platform-envelope-is-compatibility-shell"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_layout_plan\""));
    assert!(report_json.contains("\"internal_binary_format\":\"nuis-hetero-unified-binary\""));
    assert!(report_json.contains("\"lifecycle_entry_hook\":\"on_process_start\""));
    assert!(
        report_json.contains("\"scheduler_metadata_payload\":\"payload0004.scheduler-metadata\"")
    );
    assert!(report_json.contains("\"scheduler_hetero_node_count\":0"));
    assert!(
        report_json.contains("\"relocation_application_strategy\":\"nsb-loader-relocation-table\"")
    );
    assert!(report_json.contains("\"relocation_application_count\":"));
    assert!(report_json.contains("\"relocation_application_table_hash\":\"0x"));
    assert!(report_json.contains("\"relocation_applications\":["));
    assert!(report_json.contains("\"application_status\":\"planned\""));
    assert!(report_json.contains("\"byte_map_hash\":\"0x"));
    assert!(report_json.contains("\"byte_map_entries\":["));
}

#[test]
fn final_executable_layout_plan_emit_and_verify_round_trip() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-layout-plan-emit-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify = nsld_verify_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    let emit_json = super::json::nsld_final_executable_layout_plan_emit_report_json(&emit);
    let verify_json = super::json::nsld_final_executable_layout_plan_verify_report_json(&verify);
    let source = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(emit.layout_hash.starts_with("0x"));
    assert_eq!(
        verify.actual_layout_hash.as_deref(),
        Some(emit.layout_hash.as_str())
    );
    assert!(verify.valid, "{:?}", verify.issues);
    assert!(source.contains("schema = \"nuis-nsld-final-executable-layout-plan-v1\""));
    assert!(source.contains("platform_envelope_family = \"mach-o\""));
    assert!(source.contains("payloads = ["));
    assert!(source.contains("byte_alignment = 16"));
    assert!(source.contains("relocation_application_strategy = \"nsb-loader-relocation-table\""));
    assert!(source.contains("relocation_application_table_hash = \"0x"));
    assert!(source.contains("[[relocation_application]]"));
    assert!(source.contains("application_status = \"planned\""));
    assert!(source.contains("byte_map_hash = \"0x"));
    assert!(source.contains("[[byte_map_entry]]"));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_layout_plan_emit\""));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_layout_plan_verify\""));
    assert!(verify_json.contains("\"valid\":true"));
}

#[test]
fn verify_final_executable_layout_plan_reports_protocol_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-layout-plan-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let byte_span_line = source
        .lines()
        .find(|line| line.starts_with("byte_span = "))
        .unwrap()
        .to_owned();
    let payloads_line = source
        .lines()
        .find(|line| line.starts_with("payloads = "))
        .unwrap()
        .to_owned();
    let relocation_application_count_line = source
        .lines()
        .find(|line| line.starts_with("relocation_application_count = "))
        .unwrap()
        .to_owned();
    let relocation_application_table_hash_line = source
        .lines()
        .find(|line| line.starts_with("relocation_application_table_hash = "))
        .unwrap()
        .to_owned();
    let tampered_payloads_line = payloads_line.replacen('"', "\"tampered-", 1);
    let damaged = source
        .replacen("[[payload]]", "[[payload_tampered]]", 1)
        .replacen("[[byte_map_entry]]", "[[byte_map_entry_tampered]]", 1)
        .replace(
            "lifecycle_entry_hook = \"on_process_start\"",
            "lifecycle_entry_hook = \"drift\"",
        )
        .replace(
            "platform_envelope_family = \"mach-o\"",
            "platform_envelope_family = \"elf\"",
        )
        .replace(&byte_span_line, "byte_span = 0")
        .replace(&payloads_line, &tampered_payloads_line)
        .replace("payload_count = 5", "payload_count = 0")
        .replace(
            "scheduler_hetero_node_count = 0",
            "scheduler_hetero_node_count = 9",
        )
        .replace(
            "relocation_application_strategy = \"nsb-loader-relocation-table\"",
            "relocation_application_strategy = \"drift\"",
        )
        .replace(
            &relocation_application_count_line,
            "relocation_application_count = 0",
        )
        .replace(
            &relocation_application_table_hash_line,
            "relocation_application_table_hash = \"0x0000000000000000\"",
        );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify = nsld_verify_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_layout_plan_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_lifecycle_entry_hook.as_deref(), Some("drift"));
    assert_eq!(
        verify.actual_platform_envelope_family.as_deref(),
        Some("elf")
    );
    assert_eq!(verify.actual_payload_count, Some(0));
    assert_eq!(verify.actual_scheduler_hetero_node_count, Some(9));
    assert_eq!(
        verify.actual_relocation_application_strategy.as_deref(),
        Some("drift")
    );
    assert_eq!(verify.actual_relocation_application_count, Some(0));
    assert_eq!(
        verify.actual_relocation_application_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert!(verify
        .actual_payloads
        .iter()
        .any(|payload| payload.starts_with("tampered-")));
    assert_eq!(
        verify.actual_payload_entry_count + 1,
        verify.expected_payload_entry_count
    );
    assert_eq!(
        verify.actual_byte_map_entry_count + 1,
        verify.expected_byte_map_entry_count
    );
    assert_eq!(verify.actual_byte_span, Some(0));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final-executable-layout-plan-content-mismatch"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("byte_span mismatch: expected ")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("payloads mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("payload_entry_count mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("byte_map_entry_count mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue
            == "lifecycle_entry_hook mismatch: expected on_process_start, found drift"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "scheduler_hetero_node_count mismatch: expected 0, found 9"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue
            == "relocation_application_strategy mismatch: expected nsb-loader-relocation-table, found drift"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_application_count mismatch: expected ")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_application_table_hash mismatch: expected 0x")));
    assert!(verify_json.contains("\"actual_lifecycle_entry_hook\":\"drift\""));
    assert!(verify_json.contains("\"actual_scheduler_hetero_node_count\":9"));
    assert!(verify_json.contains("\"actual_relocation_application_strategy\":\"drift\""));
    assert!(verify_json.contains("\"actual_relocation_application_count\":0"));
    assert!(verify_json.contains("\"actual_platform_envelope_family\":\"elf\""));
    assert!(verify_json.contains("tampered-"));
    assert!(verify_json.contains("\"actual_payload_entry_count\":"));
    assert!(verify_json.contains("\"actual_byte_map_entry_count\":"));
}
