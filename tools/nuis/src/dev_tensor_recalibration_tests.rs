use super::*;

#[test]
fn dev_tensor_recalibration_drift_blocks_ready_cards_without_changing_the_frontier() {
    let clean = dev_tensor_summary_with_drift("clean");
    assert!(clean.weakest_bootstrap_task_card_ready);
    for status in ["drift", "unknown", ""] {
        let blocked = dev_tensor_summary_with_drift(status);
        assert_eq!(blocked.mainline.selected, clean.mainline.selected);
        assert_eq!(blocked.weakest_bootstrap_task_card_status, "blocked");
        assert!(!blocked.weakest_bootstrap_task_card_ready);
        assert!(blocked
            .weakest_bootstrap_task_card_priority_reason
            .contains("evidence drift"));
    }
}

#[test]
fn dev_tensor_recalibration_keeps_scopes_and_preparation_separate() {
    let find = |name| {
        DEV_TENSOR_CELLS
            .iter()
            .find(|c| c.function == name)
            .unwrap()
    };
    let session = find("persistent-application-session");
    assert_eq!((session.status, session.progress), ("active", 60));
    let scalar = find("native-scalar-record-session-profile");
    assert_eq!((scalar.status, scalar.progress), ("stable", 100));
    let resource = find("native-resource-session-callbacks");
    assert_eq!((resource.status, resource.progress), ("early", 0));
    assert!(!scalar.bootstrap_critical && !resource.bootstrap_critical);
    assert!(resource
        .expected_artifact
        .contains("baseline gates, not implementation success"));
    assert!(scalar.blocker.contains("outside its admission"));
    assert_eq!(
        dev_tensor_summary().bootstrap_critical_average_progress,
        100
    );
}

#[test]
fn dev_tensor_recalibration_preserves_all_six_complete_direct_entry_fields() {
    use sha2::{Digest, Sha256};
    let cell = DEV_TENSOR_CELLS
        .iter()
        .find(|c| c.function == "persistent-application-session")
        .unwrap();
    for (name, value, hash) in [
        (
            "evidence",
            cell.evidence,
            "94e53aa1c219085ebfdecb7c0f7b29f4eb8ae45873558e45716c0dac02a569ce",
        ),
        (
            "next_step",
            cell.next_step,
            "05562e026b6d32d567a4c1653d6498a10a1ffdeb57e51bced3ed578e3870ac50",
        ),
        (
            "blocker",
            cell.blocker,
            "01a54897cc79dcc229608eaf438b25045e302a90bbce098e9ff4c83658278820",
        ),
        (
            "next_action",
            cell.next_action,
            "b0f9f8ff95477b4cb9dcbfd70f25dad2d7d4902df49c98aeebf1ba826b8947fd",
        ),
        (
            "validation_command",
            cell.validation_command,
            "7aaa4a65798ce4be6acbc6784d3b4d8479e4beef42c52f1333bb20d39007f57e",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "9a4990f10ddff8e8a6b26a5d36c384ceba0c71401e552d5fbe30d71189ad4754",
        ),
    ] {
        let label = match name {
            "next_step" => "task",
            "next_action" => "action",
            "validation_command" => "validation",
            "expected_artifact" => "artifact",
            other => other,
        };
        let marker = format!("Previous direct-entry {label} checkpoint: SHA-256 {hash}\n");
        let (_, previous) = value
            .split_once(&marker)
            .expect("complete predecessor marker");
        let previous = if name == "validation_command" {
            let delimiter = "NUIS_TENSOR_RECALIBRATION_20261113_DIRECT_HISTORY";
            previous
                .strip_prefix(&format!(": <<'{delimiter}'\n"))
                .unwrap()
                .split_once(&format!("\n{delimiter}"))
                .unwrap()
                .0
        } else {
            previous
        };
        assert_eq!(
            format!("{:x}", Sha256::digest(previous.as_bytes())),
            hash,
            "{name}"
        );
    }
}
