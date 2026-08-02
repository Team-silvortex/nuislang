use super::*;

pub(super) fn finalize_official_hetero(
    label: &str,
    project: &str,
    output_dir: &Path,
    provider_record_count: usize,
) {
    if label != "pixelmagic_pipeline_demo" {
        return;
    }
    assemble_provider_complete_final_image(
        project,
        output_dir,
        provider_record_count,
        provider_record_count,
    );
    super::replay::assert_multi_checkpoint_replay_resume(output_dir);
}

pub(super) fn assemble_provider_complete_final_image(
    project: &str,
    output_dir: &Path,
    provider_record_count: usize,
    provider_dispatch_count: usize,
) -> String {
    let output_dir_text = output_dir.display().to_string();
    let before_seal = run_nsdb(&["replay", &output_dir_text, "--json"]);
    assert_success(&before_seal, "nsdb inspects provider-complete intermediate");
    let before_seal_stdout = String::from_utf8_lossy(&before_seal.stdout);
    assert!(
        before_seal_stdout.contains("\"debugger_transcript_ready\":false")
            && before_seal_stdout.contains(
                "\"debugger_transcript_first_blocker\":\"final-image-binding-proof:legacy-unbound\"",
            ),
        "provider-complete intermediate became replayable before final-image sealing\n{before_seal_stdout}"
    );

    let seal = run_nuis(&["build", project, &output_dir_text]);
    assert_success(
        &seal,
        "nuis rebuilds provider-complete graph for self-contained sealing",
    );
    assert_file_contains(
        &output_dir.join("nuis.build.manifest.toml"),
        "packaging_mode = \"nuis-self-contained-image\"",
        "provider-complete sealing manifest",
    );

    let seal = run_nsld(&["seal", &output_dir_text, "--json"]);
    assert_success(&seal, "nsld seals provider-complete final output");
    let seal_stdout = String::from_utf8_lossy(&seal.stdout);
    assert!(
        seal_stdout.contains("\"kind\":\"nsld_seal\"")
            && seal_stdout.contains("\"protocol\":\"nsld-provider-neutral-seal-v1\"")
            && seal_stdout.contains("\"preflight_valid\":true")
            && seal_stdout.contains("\"bounded_stage_count\":3")
            && seal_stdout.contains("\"completed_stage_count\":3")
            && seal_stdout.contains("\"prepare_valid\":true")
            && seal_stdout.contains("\"pipeline_valid\":true")
            && seal_stdout.contains("\"final_executable_emitted\":true")
            && seal_stdout.contains("\"boundary_status\":\"ready\"")
            && seal_stdout.contains("\"final_output_nsdb_handoff_persisted\":true")
            && seal_stdout.contains("\"final_image_binding_proof_status\":\"verified\"")
            && seal_stdout.contains("\"replay_status\":\"replay-evidence-ready\"")
            && seal_stdout.contains("\"loader_provider_dispatch_status\":\"verified\"")
            && seal_stdout.contains(&format!(
                "\"loader_provider_dispatch_count\":{provider_dispatch_count}"
            ))
            && seal_stdout.contains("\"loader_provider_dispatch_table_hash\":\"0x")
            && seal_stdout.contains("\"completed\":true")
            && seal_stdout.contains(&format!(
                "\"loader_selected_provider_bundle_count\":{provider_dispatch_count}"
            )),
        "provider-complete final output did not complete bounded sealing\n{seal_stdout}"
    );

    let executed = nsdb::execute_provider_samples(output_dir, None)
        .expect("sealed final image authorizes provider execution");
    assert_eq!(executed.final_image_dispatch_authority_status, "verified");
    assert_eq!(executed.final_image_dispatch_count, provider_dispatch_count);
    assert_eq!(
        executed.final_image_dispatch_matched_count,
        provider_dispatch_count
    );
    assert!(executed.final_image_dispatch_table_hash.starts_with("0x"));
    assert_eq!(
        executed.final_image_dispatch_selected_set_hash,
        executed.selected_provider_bundle_set_hash
    );
    let replay = nsdb::payload_execution_replay_summary(output_dir);
    assert_eq!(
        replay.provider_completion_dispatch_authority_status,
        "verified"
    );
    assert_eq!(
        replay.provider_completion_dispatch_table_hash.as_deref(),
        Some(executed.final_image_dispatch_table_hash.as_str())
    );
    assert_eq!(
        replay
            .provider_completion_dispatch_selected_set_hash
            .as_deref(),
        Some(executed.final_image_dispatch_selected_set_hash.as_str())
    );
    assert!(replay.provider_completion_dispatch_identity_hash.is_some());
    assert_eq!(replay.provider_completions.len(), provider_record_count);
    assert!(replay.provider_completions.iter().all(|completion| {
        completion.dispatch_authority_status == "verified"
            && completion.dispatch_id != "none"
            && completion.dispatch_runner_adapter_id != "none"
            && completion.completion_evidence_contract
                == "nuis-provider-completion-evidence-collection-v1"
            && completion.completion_evidence_status == "verified"
            && completion.completion_evidence_count > 0
            && completion
                .completion_clock_evidence
                .starts_with("nuis-provider-completion-clock-v1:")
            && completion
                .completion_tokens
                .starts_with("provider-completion:0x")
            && completion.glm_release_contract == "nuis-provider-glm-release-evidence-v1"
            && completion.glm_release_tokens.starts_with("glm-release:0x")
            && completion.glm_release_status == "released-at-graph-close"
            && completion.request_completion_contract
                == "nuis-provider-request-completion-receipt-collection-v1"
            && completion.request_completion_status == "verified"
            && completion.request_completion_count == completion.completion_evidence_count
            && completion.request_completion_root_hash.starts_with("0x")
    }));
    let request_completion_count = replay
        .provider_completions
        .iter()
        .map(|completion| completion.request_completion_count)
        .sum::<usize>();
    for completion in &replay.provider_completions {
        assert_provider_output_evidence_hash(output_dir, &completion.output_evidence);
    }
    let final_output = run_nsld(&["final-executable-output", &output_dir_text, "--json"]);
    assert_success(
        &final_output,
        "Nsld consumes provider completion and GLM evidence",
    );
    let final_output_stdout = String::from_utf8_lossy(&final_output.stdout);
    assert!(
        final_output_stdout.contains(
            "\"completion_evidence_contract\":\"nuis-provider-completion-evidence-collection-v1\""
        ) && final_output_stdout.contains("\"completion_evidence_status\":\"verified\"")
            && final_output_stdout.contains("\"completion_tokens\":\"provider-completion:0x")
            && final_output_stdout
                .contains("\"glm_release_contract\":\"nuis-provider-glm-release-evidence-v1\"")
            && final_output_stdout.contains("\"glm_release_tokens\":\"glm-release:0x")
            && final_output_stdout.contains(
                "\"request_completion_contract\":\"nuis-provider-request-completion-receipt-collection-v1\""
            )
            && final_output_stdout.contains("\"request_completions\":[{")
            && final_output_stdout.contains(&format!(
                "\"selected_set_hash\":\"{}\"",
                executed.final_image_dispatch_selected_set_hash
            )),
        "Nsld final output omitted verified provider completion evidence\n{final_output_stdout}"
    );

    let frontdoor = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(
        &frontdoor,
        "Nuis mirrors sealed final-image replay into package/debugger frontdoors",
    );
    let frontdoor_stdout = String::from_utf8_lossy(&frontdoor.stdout);
    assert!(
        frontdoor_stdout
            .contains("\"nsld_final_executable_output_nsdb_replay_ready\":true")
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_object_package_ready\":true"
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_object_package_status\":\"replay-ready\""
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_object_package_replay_vocabulary_contract\":\"nuis-final-output-replay-vocabulary-v1\""
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_debugger_transcript_ready\":true"
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_debugger_transcript_status\":\"transcript-ready\""
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_debugger_transcript_replay_vocabulary_contract\":\"nuis-final-output-replay-vocabulary-v1\""
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_object_package_provider_dispatch_identity_status\":\"verified\""
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_debugger_api_provider_dispatch_identity_status\":\"verified\""
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_provider_dispatch_identity_projection_source\":\"final_output_provider_completion_dispatch_identity_hash\""
            )
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_object_package_provider_request_completion_status\":\"verified\""
            )
            && frontdoor_stdout.contains(&format!(
                "\"nsld_final_executable_output_object_package_provider_request_completion_receipt_count\":{request_completion_count}"
            ))
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_debugger_api_provider_request_completion_status\":\"verified\""
            )
            && frontdoor_stdout.contains(&format!(
                "\"nsld_final_executable_output_debugger_api_provider_request_completion_receipt_count\":{request_completion_count}"
            ))
            && frontdoor_stdout.contains(
                "\"nsld_final_executable_output_debugger_api_provider_request_completion_collections\":[{"
            )
            && frontdoor_stdout.contains(&format!(
                "\"nsld_final_executable_output_object_package_selected_provider_bundle_count\":{provider_dispatch_count}"
            ))
            && frontdoor_stdout.contains(&format!(
                "\"closure_summary_provider_completion_count\":{provider_record_count}"
            ))
            && frontdoor_stdout.contains("\"closure_summary_object_package_ready\":true")
            && frontdoor_stdout.contains(
                "\"closure_summary_object_package_status\":\"replay-ready\""
            )
            && frontdoor_stdout.contains("\"closure_summary_debugger_transcript_ready\":true")
            && frontdoor_stdout.contains(
                "\"closure_summary_debugger_transcript_status\":\"transcript-ready\""
            )
            && frontdoor_stdout.contains(
                "\"closure_summary_provider_dispatch_identity_projection_source\":\"final_output_provider_completion_dispatch_identity_hash\""
            )
            && frontdoor_stdout.contains(
                "\"closure_summary_object_package_provider_request_completion_status\":\"verified\""
            )
            && frontdoor_stdout.contains(&format!(
                "\"closure_summary_object_package_provider_request_completion_receipt_count\":{request_completion_count}"
            ))
            && frontdoor_stdout.contains(
                "\"closure_summary_debugger_api_provider_request_completion_collections\":[{"
            ),
        "Nuis frontdoor omitted sealed final-image package/debugger replay projection\n{frontdoor_stdout}"
    );
    frontdoor_stdout.into_owned()
}

fn assert_provider_output_evidence_hash(output_dir: &Path, evidence: &str) {
    let mut parts = evidence.split(':');
    let file_name = parts.next().expect("provider output evidence file");
    let hash_claim = parts
        .find_map(|part| part.strip_prefix("hash="))
        .expect("provider output evidence hash");
    let bytes = fs::read(output_dir.join(file_name)).expect("provider output payload");
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    assert_eq!(hash_claim, format!("0x{hash:016x}"));
}
