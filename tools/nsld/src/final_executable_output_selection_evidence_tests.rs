use super::*;
use crate::{
    final_executable_output_selection::{
        evaluate_final_output_selection, COMPATIBILITY_OUTPUT_POLICY,
    },
    main_test_support::empty_link_plan,
};
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn explicit_selection_evidence_is_canonical_owner_private_and_replaceable() {
    let dir = unique_temp_dir("nsld-final-output-selection-evidence");
    fs::create_dir_all(&dir).unwrap();
    let output = dir.join("demo.bin");
    fs::write(&output, b"compatibility-output").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.final_stage.output_path = output.display().to_string();

    let implicit = evaluate_final_output_selection(&plan, None, false).unwrap();
    let error = persist_final_output_selection_evidence(&dir, &implicit).unwrap_err();
    assert!(error.contains("requires an explicit policy"));
    assert!(!final_output_selection_evidence_path(&dir).exists());

    let first =
        evaluate_final_output_selection(&plan, Some(COMPATIBILITY_OUTPUT_POLICY), false).unwrap();
    let path = persist_final_output_selection_evidence(&dir, &first).unwrap();
    let first_source = fs::read_to_string(&path).unwrap();
    assert_eq!(
        first_source,
        render_final_output_selection_evidence(&first).unwrap()
    );
    assert!(first_source.contains(FINAL_OUTPUT_SELECTION_EVIDENCE_FILE_CONTRACT));
    assert!(first_source.contains(&first.selection_ledger_sha256));
    assert!(!first_source.contains(dir.to_str().unwrap()));
    assert!(!first_source.contains("selected_output_path"));
    assert!(first_source.ends_with("}\n"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    fs::write(&output, b"replacement-compatibility-output").unwrap();
    let second =
        evaluate_final_output_selection(&plan, Some(COMPATIBILITY_OUTPUT_POLICY), false).unwrap();
    persist_final_output_selection_evidence(&dir, &second).unwrap();
    let second_source = fs::read_to_string(&path).unwrap();
    assert_ne!(first_source, second_source);
    assert!(second_source.contains(&second.selection_ledger_sha256));

    let mut drifted = second.clone();
    drifted.status.push_str("-drifted");
    let error = persist_final_output_selection_evidence(&dir, &drifted).unwrap_err();
    assert!(error.contains("ledger drift"));
    assert_eq!(fs::read_to_string(&path).unwrap(), second_source);

    fs::remove_dir_all(dir).unwrap();
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
