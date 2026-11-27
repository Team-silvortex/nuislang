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
    assert_eq!((resource.status, resource.progress), ("active", 95));
    assert!(!scalar.bootstrap_critical && !resource.bootstrap_critical);
    assert!(resource
        .expected_artifact
        .contains("does not prove native frame integration"));
    assert!(resource.evidence.contains("80 events"));
    assert!(resource.evidence.contains("120 alternating events"));
    assert!(resource.evidence.contains("checked Result<State, i64>"));
    assert!(resource.evidence.contains("cleanup_bytes_v1"));
    assert!(resource.evidence.contains("transfer_bytes_v1"));
    assert!(resource.evidence.contains("three nested conditions"));
    assert!(resource.evidence.contains("depth 8 and 32 functions"));
    assert!(resource.evidence.contains("tail_call_owned_struct_return"));
    assert!(resource.evidence.contains("one shared root continuation"));
    assert!(resource.evidence.contains("nonempty else joins"));
    assert!(resource.evidence.contains("six paired-arm paths"));
    assert!(resource
        .evidence
        .contains("explicit i64 let/const scalar shadowing"));
    assert!(resource
        .evidence
        .contains("before NIR merges declarations and assignments"));
    assert!(resource.blocker.contains("rebinding captured values"));
    assert!(resource
        .next_step
        .contains("readback local between a guarded store and its checked return"));
    assert!(resource
        .next_step
        .contains("constructor order, owner availability, selected-only reads"));
    assert!(resource.next_step.contains("terminal cleanup"));
    assert!(resource.evidence.contains("inferred and aliased i64"));
    assert!(resource.evidence.contains("before optimization"));
    assert!(resource.evidence.contains("call_owned_struct"));
    assert!(resource
        .blocker
        .contains("inlined conditional scalar returns now admit ready same-kind i64/bool select"));
    assert!(resource
        .evidence
        .contains("Logical bool arguments now normalize"));
    assert!(resource
        .next_action
        .contains("direct i64 local initialization"));
    assert!(resource
        .evidence
        .contains("Comparison/XOR wrappers now reuse ordered scalar snapshots"));
    assert!(resource
        .evidence
        .contains("XOR and comparisons evaluate both operands"));
    assert!(resource
        .evidence
        .contains("two ready same-kind i64/bool arms"));
    assert!(resource
        .evidence
        .contains("invalid unselected arms under constant predicates"));
    assert!(resource
        .evidence
        .contains("Resource-read arguments now compose"));
    assert!(resource
        .evidence
        .contains("Single-edge resource-read initializers"));
    assert!(resource
        .evidence
        .contains("Nested resource short-circuit trees"));
    assert!(resource
        .evidence
        .contains("exact i64/bool value parameters"));
    assert!(resource.evidence.contains("implicit bool-to-i64 coercion"));
    assert!(resource.evidence.contains("64-argument bound"));
    assert!(resource.evidence.contains("Immutable bool snapshots"));
    assert!(resource.evidence.contains("33-input guard bound"));
    assert!(resource
        .evidence
        .contains("matching canonical i64/bool shadow kinds"));
    assert!(resource
        .evidence
        .contains("Bounded call_bool and call_i64 helpers"));
    assert!(resource
        .evidence
        .contains("Canonical boolean negation now crosses pure resource branch prefixes directly"));
    assert!(resource
        .blocker
        .contains("projected readback local after the write remains outside"));
    assert!(resource
        .evidence
        .contains("Direct inline bool conditions now requeue one hygienic exact-bool snapshot"));
    assert!(resource
        .evidence
        .contains("Direct i64 local initializers now use the same ordered scalar snapshots"));
    assert!(resource.evidence.contains(
        "Returned mixed-State i64 fields now normalize through the shared layout contract"
    ));
    assert!(resource
        .next_action
        .contains("preserve the projected readback local after a guarded store"));
    assert!(resource
        .evidence
        .contains("index completion before value entry"));
    assert!(resource.evidence.contains("rejoin at one write"));
    assert!(resource
        .evidence
        .contains("dependent updates produce 47 and 29"));
    assert!(resource.evidence.contains("moved versus copied pixels"));
    assert!(resource.evidence.contains(
        "Direct resource conditions now share the same exact local-record scalar projection proof"
    ));
    assert!(resource.evidence.contains(
        "Direct-condition delivery now asserts exactly one declaration and negated branch replacement"
    ));
    assert!(resource
        .evidence
        .contains("Standalone bool/i64 declarations now reuse the same complete expression proof"));
    assert!(resource
        .evidence
        .contains("Local-record scalar operands now compose inside bounded record constructors"));
    assert!(resource
        .evidence
        .contains("Previously bound record fields now survive earlier lazy fields"));
    assert!(resource
        .evidence
        .contains("Previously bound record fields now use a block-local availability table"));
    assert!(resource
        .evidence
        .contains("Record-valued helper fields now evaluate once into a local value"));
    assert!(resource
        .evidence
        .contains("Nested record literal locals now stage every i64/bool leaf in source order"));
    assert!(resource
        .evidence
        .contains("Helper-returned value-record locals now stage scalar arguments"));
    assert!(resource
        .evidence
        .contains("Borrowed buffer_len now shares exact ref Buffer admission"));
    assert!(resource
        .evidence
        .contains("Direct checked Err i64 expressions now requeue one intact typed snapshot"));
    assert!(resource.evidence.contains("even with no captured Bytes"));
    assert!(resource
        .next_action
        .contains("positive(load_at(pixels, 0))"));
    assert!(resource
        .next_action
        .contains("choose(mode < 3 && load_at(pixels, 0) > 0).value"));
    assert!(resource
        .evidence
        .contains("allocation-free private LLVM value returns"));
    assert!(resource.evidence.contains("1..=64 i64/bool leaves"));
    assert!(resource.evidence.contains("unselected siblings"));
    assert!(resource.evidence.contains("same-kind equality/inequality"));
    assert!(resource.evidence.contains("bool ordering remain rejected"));
    assert!(resource
        .evidence
        .contains("1..=16 i64 fields plus one owned Bytes"));
    assert!(resource
        .evidence
        .contains("successfully closed RGBA8 State"));
    assert_eq!(
        (
            find("native-cpu-frame-dispatch").status,
            find("native-cpu-frame-dispatch").progress
        ),
        ("early", 10)
    );
    assert!(resource.blocker.contains("provider/device completion"));
    assert!(resource
        .evidence
        .contains("binary-only launch with empty PATH"));
    assert!(resource.evidence.contains("source-free NSB"));
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
