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
    assemble_provider_complete_final_image(project, output_dir, provider_record_count);
    super::replay::assert_multi_checkpoint_replay_resume(output_dir);
}

fn assemble_provider_complete_final_image(
    project: &str,
    output_dir: &Path,
    provider_record_count: usize,
) {
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
                "\"loader_provider_dispatch_count\":{provider_record_count}"
            ))
            && seal_stdout.contains("\"loader_provider_dispatch_table_hash\":\"0x")
            && seal_stdout.contains("\"completed\":true")
            && seal_stdout.contains(&format!(
                "\"loader_selected_provider_bundle_count\":{provider_record_count}"
            )),
        "provider-complete final output did not complete bounded sealing\n{seal_stdout}"
    );

    let executed = nsdb::execute_provider_samples(output_dir, None)
        .expect("sealed final image authorizes provider execution");
    assert_eq!(executed.final_image_dispatch_authority_status, "verified");
    assert_eq!(executed.final_image_dispatch_count, provider_record_count);
    assert_eq!(
        executed.final_image_dispatch_matched_count,
        provider_record_count
    );
    assert!(executed.final_image_dispatch_table_hash.starts_with("0x"));
    assert_eq!(
        executed.final_image_dispatch_selected_set_hash,
        executed.selected_provider_bundle_set_hash
    );
}
