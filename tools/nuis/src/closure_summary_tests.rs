use super::*;

#[test]
fn final_output_closure_becomes_ready_when_replay_evidence_is_ready() {
    let mut final_output = final_output_summary(true, true, None);
    final_output.debugger_cursor_lineage_repair_rotation_generation = Some(4);
    final_output.debugger_cursor_lineage_repair_evicted_prefix_hash =
        Some("0x0123456789abcdef".to_owned());
    final_output.debugger_cursor_lineage_repair_window_hash = Some("0xfedcba9876543210".to_owned());
    final_output.nsdb_provider_completion_count = 2;
    final_output.nsdb_first_provider_family = Some("metal:apple-silicon-gpu".to_owned());
    final_output.nsdb_first_provider_output_contract =
        Some("nuis-provider-output-payload-handoff-v1".to_owned());
    final_output.nsdb_first_provider_output_evidence =
        Some("provider-output.toml:hash=0x1234".to_owned());
    final_output.nsdb_provider_completion_claim_authority_contract =
        Some("nuis-provider-completion-claim-authority-v1".to_owned());
    final_output.nsdb_provider_completion_claim_authority =
        Some("nsdb:payload-execution-handoff-writer:v1".to_owned());
    final_output.nsdb_provider_completion_claim_authority_status = "authorized".to_owned();
    final_output.nsdb_provider_completion_digest_contract =
        Some("nuis-provider-completion-digest-fnv1a64-v1".to_owned());
    final_output.nsdb_provider_completion_set_hash_claim = Some("0xset1234".to_owned());
    final_output.nsdb_provider_completion_set_hash = Some("0xset1234".to_owned());
    final_output.nsdb_provider_completion_set_hash_validation_status = "verified".to_owned();
    final_output.nsdb_provider_completions = vec![
        crate::workflow::ProviderCompletionBoundarySummary {
            trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
            provider_family: "metal:apple-silicon-gpu".to_owned(),
            output_contract: "nuis-provider-output-payload-handoff-v1".to_owned(),
            output_evidence: "provider-output.toml:hash=0x1234".to_owned(),
            record_hash: "0xrecord1234".to_owned(),
        },
        crate::workflow::ProviderCompletionBoundarySummary {
            trace_id: "hetero-trace:kernel:coreml:apple-ane".to_owned(),
            provider_family: "coreml:apple-ane".to_owned(),
            output_contract: "nuis-provider-output-payload-handoff-v1".to_owned(),
            output_evidence: "coreml-output.toml:hash=0x5678".to_owned(),
            record_hash: "0xrecord5678".to_owned(),
        },
    ];

    let summary = FrontdoorClosureSummary::from_nsld_final_output_closure(
        "workflow-link-plan",
        "inspect-final-executable-output",
        Some("nsld final-executable-output out --json"),
        "fallback reason",
        Some(&final_output),
    );
    assert_eq!(summary.status, "ready");
    assert!(summary.ready);
    assert_eq!(summary.primary_blocker, None);
    assert_eq!(summary.next_action, "run-artifact-or-replay-nsdb");
    assert_eq!(
        summary.next_command.as_deref(),
        Some("nsdb replay out --json")
    );
    assert_eq!(
        summary.object_package_summary_contract.as_deref(),
        Some("nsld-object-package-summary-v1")
    );
    assert_eq!(summary.object_package_summary_ready, Some(true));
    assert_eq!(
        summary.debugger_transcript_contract.as_deref(),
        Some("nsdb-yir-replay-transcript-v1")
    );
    assert_eq!(summary.debugger_transcript_ready, Some(true));
    let provider = summary.provider_completion.as_ref().unwrap();
    assert_eq!(
        provider.digest_contract.as_deref(),
        Some("nuis-provider-completion-digest-fnv1a64-v1")
    );
    assert_eq!(
        provider.claim_authority.as_deref(),
        Some("nsdb:payload-execution-handoff-writer:v1")
    );
    assert_eq!(provider.claim_authority_status, "authorized");
    assert_eq!(provider.count, 2);
    assert_eq!(provider.family.as_deref(), Some("metal:apple-silicon-gpu"));
    assert_eq!(
        provider.output_contract.as_deref(),
        Some("nuis-provider-output-payload-handoff-v1")
    );
    assert_eq!(
        provider.output_evidence.as_deref(),
        Some("provider-output.toml:hash=0x1234")
    );
    assert_eq!(provider.set_hash_claim.as_deref(), Some("0xset1234"));
    assert_eq!(provider.set_hash.as_deref(), Some("0xset1234"));
    assert_eq!(provider.set_hash_validation_status, "verified");
    assert_eq!(provider.records.len(), 2);
    assert_eq!(provider.records[0].record_hash, "0xrecord1234");
    assert_eq!(provider.records[1].provider_family, "coreml:apple-ane");
    let lineage = summary.debugger_cursor_lineage.as_ref().unwrap();
    assert_eq!(lineage.repair_rotation_generation, Some(4));
    assert_eq!(
        lineage.repair_evicted_prefix_hash.as_deref(),
        Some("0x0123456789abcdef")
    );
    assert_eq!(
        lineage.repair_window_hash.as_deref(),
        Some("0xfedcba9876543210")
    );
    let fields = summary.json_fields();
    assert!(fields.contains(
        &"\"closure_summary_debugger_cursor_lineage_repair_rotation_generation\":4".to_owned()
    ));
    assert!(fields.contains(&"\"closure_summary_debugger_cursor_lineage_repair_evicted_prefix_hash\":\"0x0123456789abcdef\"".to_owned()));
    assert!(fields.contains(
        &"\"closure_summary_debugger_cursor_lineage_repair_window_hash\":\"0xfedcba9876543210\""
            .to_owned()
    ));
    assert!(fields.contains(&"\"closure_summary_provider_completion_count\":2".to_owned()));
    assert!(fields.contains(
        &"\"closure_summary_first_provider_family\":\"metal:apple-silicon-gpu\"".to_owned()
    ));
    assert!(fields
        .contains(&"\"closure_summary_provider_completion_set_hash\":\"0xset1234\"".to_owned()));
    assert!(fields.contains(
        &"\"closure_summary_provider_completion_claim_authority_status\":\"authorized\"".to_owned()
    ));
    assert!(fields.contains(
        &"\"closure_summary_provider_completion_digest_contract\":\"nuis-provider-completion-digest-fnv1a64-v1\"".to_owned()
    ));
    assert!(fields.contains(
        &"\"closure_summary_provider_completion_set_hash_claim\":\"0xset1234\"".to_owned()
    ));
    assert!(fields.contains(
        &"\"closure_summary_provider_completion_set_hash_validation_status\":\"verified\""
            .to_owned()
    ));
    assert!(fields
        .iter()
        .any(|field| field.starts_with("\"closure_summary_provider_completions\":[{")));
    assert_eq!(
        summary.debugger_transcript_status.as_deref(),
        Some("transcript-ready")
    );
    assert_eq!(
        summary.debugger_transcript_next_command.as_deref(),
        Some("nsdb replay out --json")
    );
}

#[test]
fn final_output_closure_blocks_on_replay_evidence_when_output_is_ready() {
    let final_output = final_output_summary(
        true,
        false,
        Some("payload-execution-replay:no-checkpoints".to_owned()),
    );

    let summary = FrontdoorClosureSummary::from_nsld_final_output_closure(
        "workflow-link-plan",
        "inspect-final-executable-output",
        Some("nsld final-executable-output out --json"),
        "fallback reason",
        Some(&final_output),
    );

    assert_eq!(summary.status, "blocked");
    assert!(!summary.ready);
    assert_eq!(
            summary.primary_blocker.as_deref(),
            Some(
                "final executable output replay evidence is blocked by `payload-execution-replay:no-checkpoints`"
            )
        );
    assert_eq!(summary.next_action, "inspect-nsdb-replay-evidence");
    assert_eq!(
        summary.next_command.as_deref(),
        Some("nsld final-executable-output out/nuis.build.manifest.toml --json")
    );
    assert_eq!(summary.object_package_summary_ready, Some(false));
    assert_eq!(
        summary.debugger_transcript_status.as_deref(),
        Some("transcript-blocked")
    );
    assert_eq!(
        summary.debugger_transcript_first_blocker.as_deref(),
        Some("payload-execution-replay:no-checkpoints")
    );
}

#[test]
fn drive_safe_next_wraps_mutating_blocked_closure() {
    let summary = FrontdoorClosureSummary::from_nsld_next_action(
        "workflow-link-plan",
        "prepare",
        Some("nsld prepare out/nuis.build.manifest.toml"),
        "prepared artifact chain is missing `link-inputs`",
    );
    let drive = crate::workflow::NsldDriveRecommendation {
        available: true,
        mode: "apply-next".to_owned(),
        command: Some("nsld drive out/nuis.build.manifest.toml --apply".to_owned()),
        mutates_artifacts: true,
        reason: "apply the current nsld artifact-chain next action".to_owned(),
    };
    let command_set = crate::workflow::NsldDriveCommandSet {
        protocol: "nsld-drive-command-set-v1".to_owned(),
        safe_next_contract: "nsld-drive-safe-next-v1".to_owned(),
        recommended_first_json_command: "nsld drive out/nuis.build.manifest.toml --json".to_owned(),
        safe_next_probe_json_command:
            "nsld drive out/nuis.build.manifest.toml --apply --until-clean --json".to_owned(),
        safe_next_action_field: "safe_next_action".to_owned(),
        safe_next_command_field: "safe_next_command".to_owned(),
        safe_next_gate_required_field: "safe_next_gate_required".to_owned(),
        safe_next_gate_action_field: "safe_next_gate_action".to_owned(),
        dry_run_command: "nsld drive out/nuis.build.manifest.toml".to_owned(),
        dry_run_json_command: "nsld drive out/nuis.build.manifest.toml --json".to_owned(),
        dry_run_mutates_artifacts: false,
        apply_next_command: "nsld drive out/nuis.build.manifest.toml --apply".to_owned(),
        apply_next_json_command: "nsld drive out/nuis.build.manifest.toml --apply --json"
            .to_owned(),
        apply_next_mutates_artifacts: true,
        apply_until_clean_command: "nsld drive out/nuis.build.manifest.toml --apply --until-clean"
            .to_owned(),
        apply_until_clean_json_command:
            "nsld drive out/nuis.build.manifest.toml --apply --until-clean --json".to_owned(),
        apply_until_clean_mutates_artifacts: true,
    };

    let summary = summary.with_nsld_drive_safe_next(Some(&drive), Some(&command_set));

    assert_eq!(summary.status, "blocked");
    assert!(!summary.ready);
    assert_eq!(summary.next_action, "nsld-drive-safe-next");
    assert_eq!(
        summary.next_command.as_deref(),
        Some("nsld drive out/nuis.build.manifest.toml --apply --until-clean --json")
    );
    assert!(summary
        .primary_blocker
        .as_deref()
        .is_some_and(|blocker| blocker.contains("nsld-drive-safe-next-v1 gate required")));
}

fn final_output_summary(
    ready: bool,
    nsdb_replay_ready: bool,
    nsdb_replay_first_blocker: Option<String>,
) -> crate::workflow::NsldFinalExecutableOutputBoundarySummary {
    crate::workflow::NsldFinalExecutableOutputBoundarySummary {
        ready,
        boundary_status: "ready".to_owned(),
        materialization_status: "self-contained-image-ready".to_owned(),
        execution_handoff_contract: "nsld-final-output-handoff-v1".to_owned(),
        execution_handoff_ready: ready,
        execution_handoff_status: "ready".to_owned(),
        execution_handoff_target: "container-loader".to_owned(),
        execution_handoff_evidence_status: "ready".to_owned(),
        execution_handoff_first_blocker: None,
        execution_handoff_decision_code: "handoff-container-loader".to_owned(),
        entrypoint_materialization_evidence_status: "ready".to_owned(),
        launcher_manifest_present: true,
        launcher_manifest_ready: Some(true),
        launcher_manifest_blocker_count: Some(0),
        launcher_dry_run_present: true,
        launcher_dry_run_ready: Some(true),
        launcher_dry_run_would_enter_lifecycle_hook: Some(true),
        launcher_dry_run_blocker_count: Some(0),
        payload_execution_trace_protocol: "nsdb-yir-payload-execution-trace-v1".to_owned(),
        payload_execution_trace_available: true,
        payload_execution_trace_record_count: 1,
        payload_execution_trace_ready_record_count: 1,
        device_provider_sample_manifest_available: false,
        device_provider_sample_manifest_status: "missing".to_owned(),
        device_provider_sample_manifest_record_count: 0,
        device_provider_sample_manifest_pending_record_count: 0,
        device_provider_sample_manifest_blocked_record_count: 0,
        device_provider_sample_manifest_first_provider_family: "none".to_owned(),
        device_provider_sample_manifest_first_materialization_status: "none".to_owned(),
        nsdb_replay_contract: "nsdb-payload-execution-replay-plan-v1".to_owned(),
        nsdb_replay_ready,
        nsdb_replay_status: if nsdb_replay_ready {
            "replay-evidence-ready"
        } else {
            "blocked"
        }
        .to_owned(),
        nsdb_replay_checkpoint_count: usize::from(nsdb_replay_ready),
        nsdb_replayable_checkpoint_count: usize::from(nsdb_replay_ready),
        nsdb_provider_completion_count: 0,
        nsdb_first_provider_family: None,
        nsdb_first_provider_output_contract: None,
        nsdb_first_provider_output_evidence: None,
        nsdb_provider_completion_claim_authority_contract: None,
        nsdb_provider_completion_claim_authority: None,
        nsdb_provider_completion_claim_authority_status: "not-applicable".to_owned(),
        nsdb_provider_completion_signature_contract: None,
        nsdb_provider_completion_signature_public_key_id: None,
        nsdb_provider_completion_signature_status: "not-applicable".to_owned(),
        nsdb_provider_completion_digest_contract: None,
        nsdb_provider_completion_set_hash_claim: None,
        nsdb_provider_completion_set_hash: None,
        nsdb_provider_completion_set_hash_validation_status: "not-applicable".to_owned(),
        nsdb_provider_completions: Vec::new(),
        nsdb_replay_command: nsdb_replay_ready.then(|| "nsdb replay out --json".to_owned()),
        nsdb_replay_next_action: if nsdb_replay_ready {
            "replay-nsdb-payload-execution"
        } else {
            "resolve-final-output-nsdb-replay"
        }
        .to_owned(),
        nsdb_replay_next_command: Some(
            if nsdb_replay_ready {
                "nsdb replay out --json"
            } else {
                "nsld final-executable-output out/nuis.build.manifest.toml --json"
            }
            .to_owned(),
        ),
        nsdb_replay_first_blocker,
        object_package_summary_contract: "nsld-object-package-summary-v1".to_owned(),
        object_package_summary_ready: nsdb_replay_ready,
        object_package_summary_status: if nsdb_replay_ready {
            "replay-ready"
        } else {
            "replay-blocked"
        }
        .to_owned(),
        object_package_summary_next_action: if nsdb_replay_ready {
            "consume-object-package-summary"
        } else {
            "resolve-object-package-replay-evidence"
        }
        .to_owned(),
        object_package_summary_next_command: Some(
            if nsdb_replay_ready {
                "nsdb replay out --json"
            } else {
                "nsld final-executable-output out/nuis.build.manifest.toml --json"
            }
            .to_owned(),
        ),
        debugger_transcript_contract: "nsdb-yir-replay-transcript-v1".to_owned(),
        debugger_transcript_ready: nsdb_replay_ready,
        debugger_transcript_status: if nsdb_replay_ready {
            "transcript-ready"
        } else {
            "transcript-blocked"
        }
        .to_owned(),
        debugger_transcript_next_action: if nsdb_replay_ready {
            "consume-nsdb-yir-replay-transcript"
        } else {
            "resolve-nsdb-yir-replay-transcript"
        }
        .to_owned(),
        debugger_transcript_first_blocker: if nsdb_replay_ready {
            None
        } else {
            Some("payload-execution-replay:no-checkpoints".to_owned())
        },
        debugger_cursor_handoff_contract: "nuis-debugger-cursor-handoff-v1".to_owned(),
        debugger_cursor_path: "out/nuis.nsdb.replay-cursor.toml".to_owned(),
        debugger_cursor_ready: false,
        debugger_cursor_status: "cursor-unavailable".to_owned(),
        debugger_cursor_next_command: None,
        debugger_cursor_lineage_contract: "nuis-debugger-cursor-lineage-mirror-v1".to_owned(),
        debugger_cursor_lineage_source_protocol: "nsdb-yir-replay-cursor-lineage-v1".to_owned(),
        debugger_cursor_lineage_path: "out/nuis.nsdb.replay-cursor.lineage.toml".to_owned(),
        debugger_cursor_lineage_ready: false,
        debugger_cursor_lineage_status: "lineage-unavailable".to_owned(),
        debugger_cursor_lineage_entry_count: 0,
        debugger_cursor_lineage_latest_hash: None,
        debugger_cursor_lineage_first_blocker: None,
        debugger_cursor_lineage_next_action: None,
        debugger_cursor_lineage_next_command: None,
        debugger_cursor_lineage_repair_contract: "nuis-debugger-cursor-lineage-repair-mirror-v1"
            .to_owned(),
        debugger_cursor_lineage_repair_path: "out/nuis.nsdb.replay-cursor.lineage-repairs.toml"
            .to_owned(),
        debugger_cursor_lineage_repair_status: "repair-history-unavailable".to_owned(),
        debugger_cursor_lineage_repair_entry_count: 0,
        debugger_cursor_lineage_repair_rotation_generation: None,
        debugger_cursor_lineage_repair_evicted_prefix_hash: None,
        debugger_cursor_lineage_repair_window_hash: None,
        debugger_cursor_lineage_repair_latest_mutated: None,
        debugger_cursor_lineage_repair_latest_event_status: None,
        debugger_cursor_lineage_repair_latest_lineage_mutated: None,
        debugger_cursor_lineage_repair_latest_journal_mutated: None,
        debugger_cursor_lineage_repair_latest_archived_path: None,
        debugger_cursor_lineage_repair_latest_archived_hash: None,
        debugger_cursor_lineage_repair_latest_archived_journal_path: None,
        debugger_cursor_lineage_repair_latest_archived_journal_hash: None,
        debugger_cursor_lineage_repair_latest_rebuilt_hash: None,
        debugger_cursor_lineage_repair_action:
            crate::artifact_nsdb_replay_cursor_lineage::DebuggerCursorLineageRepairAction {
                first_blocker: None,
                next_action: None,
                next_command: None,
            },
        recommended_next_action: "run-artifact".to_owned(),
        path_present: true,
        nsld_owned: Some(true),
        object_valid: true,
        object_path: "out/app.nsb".to_owned(),
        object_family: "mach-o".to_owned(),
        object_magic_status: "valid".to_owned(),
        object_magic: Some("0xcffaedfe".to_owned()),
        object_expected_size_bytes: Some(1),
        object_actual_size_bytes: Some(1),
        object_expected_hash: Some("0x1".to_owned()),
        object_actual_hash: Some("0x1".to_owned()),
        object_issues: Vec::new(),
        blockers: Vec::new(),
        first_blocker: None,
    }
}
