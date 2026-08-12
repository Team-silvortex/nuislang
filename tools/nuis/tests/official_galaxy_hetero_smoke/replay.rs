use super::*;

pub(super) fn assert_multi_frame_replay_resume(output_dir: &Path) {
    let output_dir_text = output_dir.display().to_string();
    let unavailable = run_nuis(&["debug-resume", "--json", &output_dir_text]);
    assert!(
        !unavailable.status.success()
            && String::from_utf8_lossy(&unavailable.stderr).contains("cursor-unavailable"),
        "Nuis debug-resume must reject an unavailable cursor before dispatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&unavailable.stdout),
        String::from_utf8_lossy(&unavailable.stderr)
    );
    let replay = run_nsdb(&["replay", &output_dir_text, "--json"]);
    assert_success(&replay, "nsdb replay official multi-checkpoint artifact");
    let replay_stdout = String::from_utf8_lossy(&replay.stdout);
    assert!(
        replay_stdout.contains("\"debugger_transcript_ready\":true")
            && replay_stdout.contains(
                "\"debugger_transcript_identity_contract\":\"nsdb-yir-replay-identity-v1\""
            )
            && replay_stdout
                .contains("\"debugger_transcript_final_image_binding_proof_hash\":\"fnv1a64:",)
            && replay_stdout.contains(
                "\"debugger_transcript_provider_dispatch_authority_status\":\"verified\"",
            )
            && replay_stdout
                .contains("\"debugger_transcript_provider_dispatch_identity_hash\":\"0x",),
        "provider-complete final image must expose a replay identity\n{replay_stdout}"
    );
    let frame_ids = json_string_values(&replay_stdout, "frame_id");
    assert!(
        frame_ids.len() >= 3,
        "official hetero replay should expose at least three YIR frames\n{replay_stdout}"
    );
    let first = &frame_ids[0];
    let second = &frame_ids[1];
    let third = &frame_ids[2];
    assert!(first.starts_with("frame:payload:"));
    assert!(second.starts_with("frame:request:"));
    assert!(third.starts_with("frame:request:"));
    let cursor_path = output_dir.join("nuis.nsdb.replay-cursor.toml");
    let cursor_path_text = cursor_path.display().to_string();

    let stopped = run_nsdb(&[
        "replay",
        &output_dir_text,
        "--break-at",
        first,
        "--save-cursor",
        &cursor_path_text,
        "--json",
    ]);
    assert_success(&stopped, "nsdb persist first hetero replay cursor");
    assert_file_contains(
        &cursor_path,
        "protocol = \"nsdb-yir-replay-cursor-record-v2\"",
        "persisted replay cursor protocol",
    );
    assert_file_contains(
        &cursor_path,
        "identity_contract = \"nsdb-yir-replay-identity-v1\"",
        "persisted replay cursor identity contract",
    );
    assert_file_contains(
        &cursor_path,
        "final_image_binding_proof_hash = \"fnv1a64:",
        "persisted replay cursor final image identity",
    );
    assert_file_contains(
        &cursor_path,
        "provider_dispatch_identity_hash = \"0x",
        "persisted replay cursor provider dispatch identity",
    );
    assert_file_contains(
        &cursor_path,
        &format!("after_frame_id = \"{first}\""),
        "persisted replay cursor stopped frame",
    );
    assert_file_contains(
        &cursor_path,
        &format!("next_frame_id = \"{second}\""),
        "persisted replay cursor next frame",
    );

    let report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&report, "nuis mirror persisted debugger cursor");
    let report_stdout = String::from_utf8_lossy(&report.stdout);
    assert!(
        report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_handoff_contract\":\"nuis-debugger-cursor-handoff-v1\""
        ) && report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_ready\":true"
        ) && report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_status\":\"cursor-resume-ready\""
        ) && report_stdout.contains(
            "\"closure_summary_debugger_cursor_handoff_contract\":\"nuis-debugger-cursor-handoff-v1\""
        ) && report_stdout.contains("\"closure_summary_debugger_cursor_ready\":true")
            && report_stdout.contains(
                "\"closure_summary_debugger_cursor_status\":\"cursor-resume-ready\""
            )
            && report_stdout.contains(
                "\"nsld_final_executable_output_debugger_cursor_next_command\":\"nuis debug-resume "
            )
            && report_stdout.contains(
                "\"closure_summary_debugger_cursor_next_command\":\"nuis debug-resume "
            )
            && report_stdout.contains("--json")
            && report_stdout.contains("nuis.nsdb.replay-cursor.toml"),
        "Nuis frontdoors should mirror the persisted debugger cursor without Nsdb type coupling\n{report_stdout}"
    );

    let resumed = run_nuis(&[
        "debug-resume",
        "--json",
        "--break-at",
        second,
        "--save-cursor",
        &cursor_path_text,
        &output_dir_text,
    ]);
    assert_success(&resumed, "nuis first-class heterogeneous debug resume");
    let resumed_stdout = String::from_utf8_lossy(&resumed.stdout);
    assert!(
        resumed_stdout.contains("\"debugger_transcript_resume_input_status\":\"cursor-accepted\"")
            && resumed_stdout.contains("\"debugger_transcript_control_status\":\"breakpoint-hit\"")
            && resumed_stdout.contains(&format!(
                "\"debugger_transcript_selected_frame_id\":\"{second}\""
            ))
            && resumed_stdout.contains("\"debugger_transcript_replayed_checkpoint_count\":0")
            && resumed_stdout.contains("\"debugger_transcript_replayed_frame_count\":1"),
        "Nuis debug-resume should validate, resume, and stop at the selected heterogeneous frame\n{resumed_stdout}"
    );
    assert_file_contains(
        &cursor_path,
        &format!("after_frame_id = \"{second}\""),
        "replaced replay cursor stopped frame",
    );
    assert_file_contains(
        &cursor_path,
        &format!("next_frame_id = \"{third}\""),
        "replaced replay cursor next frame",
    );
    let lineage_path = output_dir.join("nuis.nsdb.replay-cursor.lineage.toml");
    assert_file_contains(
        &lineage_path,
        "protocol = \"nsdb-yir-replay-cursor-lineage-v2\"",
        "replay cursor lineage protocol",
    );
    assert_file_contains(
        &lineage_path,
        "identity_contract = \"nsdb-yir-replay-identity-v1\"",
        "replay cursor lineage identity contract",
    );
    assert_file_contains(
        &lineage_path,
        "final_image_binding_proof_hash = \"fnv1a64:",
        "replay cursor lineage final image identity",
    );
    assert_file_contains(
        &lineage_path,
        "provider_dispatch_identity_hash = \"0x",
        "replay cursor lineage provider dispatch identity",
    );
    assert_file_contains(
        &lineage_path,
        "entry_count = 2",
        "replay cursor lineage entry count",
    );
    assert_file_contains(
        &lineage_path,
        "sequence = 0",
        "initial replay cursor lineage sequence",
    );
    assert_file_contains(
        &lineage_path,
        "sequence = 1",
        "replacement replay cursor lineage sequence",
    );
    assert_file_contains(
        &lineage_path,
        "current_hash = \"0x",
        "replay cursor lineage content hash",
    );
    let lineage_source = fs::read_to_string(&lineage_path).expect("read replay cursor lineage");
    let latest_hash = lineage_source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("current_hash = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .last()
        .expect("replay cursor lineage latest hash");
    let provider_dispatch_identity_hash = lineage_source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("provider_dispatch_identity_hash = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .last()
        .expect("replay cursor lineage provider dispatch identity hash");
    let lineage_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&lineage_report, "nuis mirror debugger cursor lineage");
    let lineage_report_stdout = String::from_utf8_lossy(&lineage_report.stdout);
    assert!(
        lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_contract\":\"nuis-debugger-cursor-lineage-mirror-v1\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_source_protocol\":\"nsdb-yir-replay-cursor-lineage-v2\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_ready\":true"
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_status\":\"lineage-ready\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_entry_count\":2"
        ) && lineage_report_stdout.contains(&format!(
            "\"nsld_final_executable_output_debugger_cursor_lineage_latest_hash\":\"{latest_hash}\""
        )) && lineage_report_stdout.contains(
            &format!(
                "\"nsld_final_executable_output_debugger_cursor_lineage_provider_dispatch_identity_hash\":\"{provider_dispatch_identity_hash}\""
            )
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_object_package_provider_dispatch_identity_capability_contract\":\"nuis-validated-provider-dispatch-identity-capability-v1\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_object_package_provider_dispatch_identity_status\":\"verified\""
        ) && lineage_report_stdout.contains(
            &format!(
                "\"nsld_final_executable_output_object_package_provider_dispatch_identity_hash\":\"{provider_dispatch_identity_hash}\""
            )
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_api_provider_dispatch_identity_capability_contract\":\"nuis-validated-provider-dispatch-identity-capability-v1\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_api_provider_dispatch_identity_status\":\"verified\""
        ) && lineage_report_stdout.contains(
            &format!(
                "\"nsld_final_executable_output_debugger_api_provider_dispatch_identity_hash\":\"{provider_dispatch_identity_hash}\""
            )
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_provider_dispatch_identity_projection_source\":\"debugger_cursor_lineage_provider_dispatch_identity_hash\""
        ) && lineage_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_ready\":true"
        ) && lineage_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_entry_count\":2"
        ) && lineage_report_stdout.contains(&format!(
            "\"closure_summary_debugger_cursor_lineage_latest_hash\":\"{latest_hash}\""
        )) && lineage_report_stdout.contains(
            &format!(
                "\"closure_summary_debugger_cursor_lineage_provider_dispatch_identity_hash\":\"{provider_dispatch_identity_hash}\""
            )
        ) && lineage_report_stdout.contains(
            "\"closure_summary_object_package_provider_dispatch_identity_capability_contract\":\"nuis-validated-provider-dispatch-identity-capability-v1\""
        ) && lineage_report_stdout.contains(
            &format!(
                "\"closure_summary_object_package_provider_dispatch_identity_hash\":\"{provider_dispatch_identity_hash}\""
            )
        ) && lineage_report_stdout.contains(
            "\"closure_summary_debugger_api_provider_dispatch_identity_capability_contract\":\"nuis-validated-provider-dispatch-identity-capability-v1\""
        ) && lineage_report_stdout.contains(
            &format!(
                "\"closure_summary_debugger_api_provider_dispatch_identity_hash\":\"{provider_dispatch_identity_hash}\""
            )
        ) && lineage_report_stdout.contains(
            "\"closure_summary_provider_dispatch_identity_projection_source\":\"debugger_cursor_lineage_provider_dispatch_identity_hash\""
        ),
        "Nuis final-output, object-package, debugger API, and closure summaries should share the hash-checked debugger cursor lineage identity\n{lineage_report_stdout}"
    );
    fs::write(
        &lineage_path,
        lineage_source.replacen(latest_hash, "0x0000000000000000", 1),
    )
    .expect("damage replay cursor lineage latest hash");
    let invalid_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(
        &invalid_report,
        "nuis diagnose invalid debugger cursor lineage",
    );
    let invalid_report_stdout = String::from_utf8_lossy(&invalid_report.stdout);
    assert!(
        invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_status\":\"lineage-invalid\""
        ) && invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_first_blocker\":\"lineage-latest-hash-mismatch\""
        ) && invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_next_action\":\"repair-cursor-lineage\""
        ) && invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_next_command\":\"nuis debug-lineage-repair "
        ) && invalid_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_first_blocker\":\"lineage-latest-hash-mismatch\""
        ),
        "Nuis should expose an actionable stable blocker for stale cursor lineage\n{invalid_report_stdout}"
    );
    let repaired = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(&repaired, "nuis repair debugger cursor lineage");
    let repaired_stdout = String::from_utf8_lossy(&repaired.stdout);
    assert!(
        repaired_stdout.contains("\"contract\":\"nsdb-yir-replay-cursor-lineage-repair-v2\"")
            && repaired_stdout.contains("\"status\":\"lineage-rebuilt\"")
            && repaired_stdout.contains("\"mutated\":true")
            && repaired_stdout.contains("\"archived_path\":\"")
            && repaired_stdout.contains("\"entry_count\":1")
            && repaired_stdout.contains("\"latest_hash\":\"0x"),
        "Nsdb should archive and rebuild invalid cursor lineage\n{repaired_stdout}"
    );
    let already_ready = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(&already_ready, "nuis keep healthy cursor lineage unchanged");
    let already_ready_stdout = String::from_utf8_lossy(&already_ready.stdout);
    assert!(
        already_ready_stdout.contains("\"status\":\"already-ready\"")
            && already_ready_stdout.contains("\"mutated\":false"),
        "Nuis should preserve Nsdb's idempotent healthy-lineage result\n{already_ready_stdout}"
    );
    let repaired_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(
        &repaired_report,
        "nuis mirror repaired debugger cursor lineage",
    );
    let repaired_report_stdout = String::from_utf8_lossy(&repaired_report.stdout);
    assert!(
        repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_status\":\"lineage-ready\""
        ) && repaired_report_stdout
            .contains("\"nsld_final_executable_output_debugger_cursor_lineage_entry_count\":1")
            && repaired_report_stdout.contains(
                "\"nsld_final_executable_output_debugger_cursor_lineage_first_blocker\":null"
            ),
        "Nuis should report repaired cursor lineage as ready\n{repaired_report_stdout}"
    );
    assert!(
        repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_contract\":\"nuis-debugger-cursor-lineage-repair-mirror-v1\""
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_entry_count\":1"
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_rotation_generation\":0"
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_mutated\":true"
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_path\":\""
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_hash\":\"0x"
        ) && repaired_report_stdout.contains(&format!(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_rebuilt_hash\":\"{latest_hash}\""
        )) && repaired_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ) && repaired_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_repair_entry_count\":1"
        ) && repaired_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_repair_rotation_generation\":0"
        ) && repaired_report_stdout.contains(&format!(
            "\"closure_summary_debugger_cursor_lineage_repair_latest_rebuilt_hash\":\"{latest_hash}\""
        )),
        "Nuis should preserve cursor-lineage repair audit evidence beyond command stdout\n{repaired_report_stdout}"
    );
    let repaired_lineage_source = fs::read_to_string(&lineage_path)
        .expect("read repaired cursor lineage before journal recovery smoke");
    let repair_journal_path = output_dir.join("nuis.nsdb.replay-cursor.lineage-repairs.toml");
    assert_file_contains(
        &repair_journal_path,
        "protocol = \"nsdb-yir-replay-cursor-lineage-repair-journal-v6\"",
        "replay cursor lineage repair journal protocol",
    );
    assert_file_contains(
        &repair_journal_path,
        "provider_dispatch_identity_hash = \"0x",
        "repair journal provider dispatch identity ancestry",
    );
    fs::write(
        &lineage_path,
        repaired_lineage_source.replacen(latest_hash, "0x0000000000000000", 1),
    )
    .expect("damage cursor lineage before journal recovery smoke");
    fs::write(&repair_journal_path, "protocol = \"damaged-journal\"\n")
        .expect("damage cursor lineage repair journal");
    let recovered = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(&recovered, "nuis recover damaged lineage repair journal");
    let recovered_stdout = String::from_utf8_lossy(&recovered.stdout);
    assert!(
        recovered_stdout.contains("\"status\":\"lineage-rebuilt\"")
            && recovered_stdout.contains("\"archived_repair_journal_path\":\""),
        "Nuis should archive the damaged repair journal before rebuilding lineage\n{recovered_stdout}"
    );
    let recovered_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&recovered_report, "nuis mirror recovered repair journal");
    let recovered_report_stdout = String::from_utf8_lossy(&recovered_report.stdout);
    assert!(
        recovered_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ) && recovered_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ),
        "Nuis should mirror the recovered cursor-lineage repair journal\n{recovered_report_stdout}"
    );
    let healthy_lineage =
        fs::read(&lineage_path).expect("read healthy lineage before journal-only recovery smoke");
    fs::write(&repair_journal_path, "protocol = \"journal-only-damage\"\n")
        .expect("damage only cursor lineage repair journal");
    let invalid_history = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&invalid_history, "nuis diagnose invalid repair history");
    let invalid_history_stdout = String::from_utf8_lossy(&invalid_history.stdout);
    assert!(
        invalid_history_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_first_blocker\":\"repair-history-contract-invalid\"")
            && invalid_history_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_next_action\":\"repair-cursor-lineage-history\"")
            && invalid_history_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_next_command\":\"nuis debug-lineage-repair ")
            && invalid_history_stdout.contains("\"closure_summary_debugger_cursor_lineage_repair_first_blocker\":\"repair-history-contract-invalid\""),
        "Nuis should expose an actionable invalid repair-history diagnosis\n{invalid_history_stdout}"
    );
    let journal_only = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(
        &journal_only,
        "nuis recover journal without lineage rebuild",
    );
    let journal_only_stdout = String::from_utf8_lossy(&journal_only.stdout);
    assert!(
        journal_only_stdout.contains("\"contract\":\"nsdb-yir-replay-cursor-lineage-repair-v2\"")
            && journal_only_stdout.contains("\"status\":\"repair-history-recovered\"")
            && journal_only_stdout.contains("\"mutated\":true")
            && journal_only_stdout.contains("\"lineage_mutated\":false")
            && journal_only_stdout.contains("\"repair_journal_mutated\":true")
            && journal_only_stdout.contains("\"archived_repair_journal_path\":\""),
        "Nuis should report journal-only recovery with separate mutation scopes\n{journal_only_stdout}"
    );
    assert_eq!(
        fs::read(&lineage_path).expect("read lineage after journal-only recovery"),
        healthy_lineage,
        "journal-only recovery must preserve authoritative lineage bytes"
    );
    let journal_only_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&journal_only_report, "nuis mirror journal-only recovery");
    let journal_only_report_stdout = String::from_utf8_lossy(&journal_only_report.stdout);
    assert!(
        journal_only_report_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_event_status\":\"repair-history-recovered\"")
            && journal_only_report_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_lineage_mutated\":false")
            && journal_only_report_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_journal_mutated\":true")
            && journal_only_report_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_journal_path\":\"")
            && journal_only_report_stdout.contains("\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_journal_hash\":\"0x")
            && journal_only_report_stdout.contains("\"closure_summary_debugger_cursor_lineage_repair_latest_event_status\":\"repair-history-recovered\"")
            && journal_only_report_stdout.contains("\"closure_summary_debugger_cursor_lineage_repair_latest_lineage_mutated\":false")
            && journal_only_report_stdout.contains("\"closure_summary_debugger_cursor_lineage_repair_latest_journal_mutated\":true"),
        "Nuis should return journal-only recovery to repair-history-ready"
    );

    let resumed_again = run_nuis(&[
        "debug-resume",
        "--json",
        "--break-at",
        third,
        &output_dir_text,
    ]);
    assert_success(&resumed_again, "nuis chained heterogeneous debug resume");
    let resumed_again_stdout = String::from_utf8_lossy(&resumed_again.stdout);
    assert!(
        resumed_again_stdout
            .contains("\"debugger_transcript_resume_input_status\":\"cursor-accepted\"")
            && resumed_again_stdout
                .contains("\"debugger_transcript_control_status\":\"breakpoint-hit\"")
            && resumed_again_stdout.contains(&format!(
                "\"debugger_transcript_selected_frame_id\":\"{third}\""
            ))
            && resumed_again_stdout
                .contains("\"debugger_transcript_replayed_checkpoint_count\":0")
            && resumed_again_stdout.contains("\"debugger_transcript_replayed_frame_count\":1"),
        "Nuis debug-resume should consume the replaced cursor and stop at the third heterogeneous frame\n{resumed_again_stdout}"
    );
}
