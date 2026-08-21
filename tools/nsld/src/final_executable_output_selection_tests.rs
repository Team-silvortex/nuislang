use super::*;
use crate::main_test_support::empty_link_plan;
use std::{env, fs};

#[test]
fn selection_registry_has_one_compatibility_default_and_explicit_private_policy() {
    let validation = final_output_selection_registry_validation();

    assert!(validation.valid, "{:?}", validation.issues);
    assert_eq!(validation.registration_count, 2);
    assert_eq!(
        validation.default_policy_id,
        Some(COMPATIBILITY_OUTPUT_POLICY)
    );
    assert_eq!(validation.registry_hash.len(), 64);
}

#[test]
fn default_selection_preserves_existing_output_without_apply() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-output-selection-default-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = dir.join("demo.bin");
    fs::write(&output, b"compatibility-output").unwrap();
    let mut plan = empty_link_plan();
    plan.final_stage.output_path = output.display().to_string();

    let report = evaluate_final_output_selection(&plan, None, false).unwrap();
    let after = fs::read(&output).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(report.policy_id, COMPATIBILITY_OUTPUT_POLICY);
    assert!(report.default_policy);
    assert!(!report.explicit_request);
    assert!(!report.apply_requested);
    assert_eq!(report.status, "compatibility-output-preserved");
    assert!(report.selection_ready);
    assert!(!report.installation_attempted);
    assert!(report.selected);
    assert!(report.selected_output_identity_matches);
    assert_eq!(report.selected_output_name, "demo.bin");
    assert_eq!(report.selected_output_span_bytes, Some(20));
    assert_eq!(
        report.selected_output_sha256.as_deref().map(str::len),
        Some(64)
    );
    assert_eq!(report.selection_ledger_sha256.len(), 64);
    assert_eq!(after, b"compatibility-output");
}

#[test]
fn apply_requires_an_explicit_apply_capable_policy() {
    let plan = empty_link_plan();

    let implicit = evaluate_final_output_selection(&plan, None, true).unwrap_err();
    let compatibility =
        evaluate_final_output_selection(&plan, Some(COMPATIBILITY_OUTPUT_POLICY), true)
            .unwrap_err();
    let unknown =
        evaluate_final_output_selection(&plan, Some("unknown-policy"), false).unwrap_err();

    assert!(implicit.contains("requires an explicit policy"));
    assert!(compatibility.contains("does not support apply"));
    assert!(unknown.contains("unknown final-output selection policy"));
}
