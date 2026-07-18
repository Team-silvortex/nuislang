use super::{
    cli::Command, main_final_executable_commands::run_final_executable_command,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_emit_final_stage_plan_report, nsld_final_executable_output_report, nsld_prepare_report,
};
use nuisc::aot::{BuildManifestContext, CompileArtifacts};
use std::{env, fs, path::Path};

#[test]
fn final_executable_output_command_persists_nsdb_handoff_record() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-output-nsdb-handoff-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let manifest = write_test_build_manifest_with_packaging_mode(&dir, "nuis-self-contained-image");
    let plan = nuisc::linker::build_link_plan_from_manifest(Path::new(&manifest)).unwrap();

    nsld_prepare_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_layout_plan_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new(&manifest), &plan).unwrap();

    let command = Command::FinalExecutableOutput {
        input: Path::new(&manifest).to_path_buf(),
        json: true,
    };
    run_final_executable_command(&command).unwrap();
    let handoff_path = dir.join("nuis.nsdb.payload-execution-handoff.toml");
    let handoff = fs::read_to_string(&handoff_path).unwrap();
    let mut output = nsld_final_executable_output_report(Path::new(&manifest), &plan);
    let summary = super::final_executable_output_nsdb_handoff::persist_final_output_nsdb_handoff(
        Path::new(&plan.output_dir),
        &output,
    );
    super::final_executable_output_nsdb_handoff::attach_final_output_nsdb_handoff_summary(
        &mut output,
        summary,
    );
    let output_json = super::json::nsld_final_executable_output_report_json(&output);
    fs::remove_dir_all(dir).unwrap();

    assert!(handoff.contains("protocol = \"nuis-nsdb-payload-execution-handoff-v1\""));
    assert!(handoff.contains("debugger_contract = \"nsdb-yir-payload-execution-trace-v1\""));
    assert!(handoff.contains("source = \"nsld-final-executable-output\""));
    assert!(handoff.contains("record_count = 1"));
    assert!(handoff.contains("ready_record_count = 1"));
    assert!(handoff.contains(
        "first_trace_id = \"payload-trace:container-loader:nuis.bootstrap.lifecycle.v1\""
    ));
    assert!(handoff.contains("first_status = \"ready\""));
    assert!(handoff.contains("first_next_action = \"handoff-payload-trace-to-nsdb\""));
    assert!(handoff.contains("[[records]]"));
    assert!(handoff.contains("execution_phase = \"container-loader-handoff\""));
    assert!(handoff.contains("entry_symbol = \"nuis.bootstrap.lifecycle.v1\""));
    assert!(handoff.contains("entry_kind = \"lifecycle-bootstrap\""));
    assert!(output.final_output_nsdb_handoff_persisted);
    assert_eq!(output.final_output_nsdb_handoff_record_count, 1);
    assert_eq!(output.final_output_nsdb_handoff_ready_record_count, 1);
    assert_eq!(
        output.final_output_nsdb_handoff_first_trace_id.as_deref(),
        Some("payload-trace:container-loader:nuis.bootstrap.lifecycle.v1")
    );
    assert_eq!(
        output.final_output_nsdb_replay_contract,
        "nsdb-payload-execution-replay-plan-v1"
    );
    assert!(output.final_output_nsdb_replay_ready);
    assert_eq!(
        output.final_output_nsdb_replay_status,
        "replay-evidence-ready"
    );
    assert_eq!(output.final_output_nsdb_replay_checkpoint_count, 1);
    assert_eq!(output.final_output_nsdb_replayable_checkpoint_count, 1);
    assert!(output
        .final_output_nsdb_replay_command
        .as_deref()
        .is_some_and(|command| command.starts_with("nsdb replay ")));
    assert_eq!(
        output.final_output_nsdb_replay_next_action,
        "replay-nsdb-payload-execution"
    );
    assert_eq!(
        output.owned_package_summary_contract,
        "nsld-owned-package-summary-v1"
    );
    assert_eq!(output.owned_package_summary_status, "replay-ready");
    assert!(output.owned_package_summary_ready);
    assert_eq!(
        output.owned_package_summary_replay_status,
        "replay-evidence-ready"
    );
    assert!(output.owned_package_summary_replay_ready);
    assert_eq!(
        output.owned_package_summary_next_action,
        "replay-nsdb-payload-execution"
    );
    assert!(output
        .final_output_nsdb_replay_next_command
        .as_deref()
        .is_some_and(|command| command.starts_with("nsdb replay ")));
    assert!(output
        .owned_package_summary_next_command
        .as_deref()
        .is_some_and(|command| command.starts_with("nsdb replay ")));
    assert_eq!(
        output.object_package_summary_contract,
        "nsld-object-package-summary-v1"
    );
    assert_eq!(output.object_package_summary_status, "replay-ready");
    assert!(output.object_package_summary_ready);
    assert_eq!(
        output.object_package_summary_replay_status,
        "replay-evidence-ready"
    );
    assert!(output.object_package_summary_replay_ready);
    assert_eq!(
        output.object_package_summary_next_action,
        "replay-nsdb-payload-execution"
    );
    assert!(output
        .object_package_summary_next_command
        .as_deref()
        .is_some_and(|command| command.starts_with("nsdb replay ")));
    assert!(output.final_output_nsdb_replay_first_blocker.is_none());
    assert!(output_json.contains(
        "\"final_output_nsdb_handoff_protocol\":\"nuis-nsdb-payload-execution-handoff-v1\""
    ));
    assert!(output_json.contains("\"final_output_nsdb_handoff_persisted\":true"));
    assert!(output_json.contains("\"final_output_nsdb_handoff_record_count\":1"));
    assert!(output_json.contains("\"final_output_nsdb_handoff_ready_record_count\":1"));
    assert!(output_json.contains(
        "\"final_output_nsdb_handoff_first_trace_id\":\"payload-trace:container-loader:nuis.bootstrap.lifecycle.v1\""
    ));
    assert!(output_json.contains("\"final_output_nsdb_handoff_error\":null"));
    assert!(output_json.contains(
        "\"final_output_nsdb_replay_contract\":\"nsdb-payload-execution-replay-plan-v1\""
    ));
    assert!(output_json.contains("\"final_output_nsdb_replay_ready\":true"));
    assert!(output_json.contains("\"final_output_nsdb_replay_status\":\"replay-evidence-ready\""));
    assert!(output_json.contains("\"final_output_nsdb_replay_command\":\"nsdb replay "));
    assert!(output_json
        .contains("\"final_output_nsdb_replay_next_action\":\"replay-nsdb-payload-execution\""));
    assert!(output_json.contains("\"final_output_nsdb_replay_next_command\":\"nsdb replay "));
    assert!(output_json
        .contains("\"owned_package_summary_contract\":\"nsld-owned-package-summary-v1\""));
    assert!(output_json.contains("\"owned_package_summary_status\":\"replay-ready\""));
    assert!(output_json.contains("\"owned_package_summary_ready\":true"));
    assert!(
        output_json.contains("\"owned_package_summary_replay_status\":\"replay-evidence-ready\"")
    );
    assert!(output_json.contains("\"owned_package_summary_replay_ready\":true"));
    assert!(output_json
        .contains("\"owned_package_summary_next_action\":\"replay-nsdb-payload-execution\""));
    assert!(output_json.contains("\"owned_package_summary_next_command\":\"nsdb replay "));
    assert!(output_json
        .contains("\"object_package_summary_contract\":\"nsld-object-package-summary-v1\""));
    assert!(output_json.contains("\"object_package_summary_status\":\"replay-ready\""));
    assert!(output_json.contains("\"object_package_summary_ready\":true"));
    assert!(
        output_json.contains("\"object_package_summary_replay_status\":\"replay-evidence-ready\"")
    );
    assert!(output_json.contains("\"object_package_summary_replay_ready\":true"));
    assert!(output_json
        .contains("\"object_package_summary_next_action\":\"replay-nsdb-payload-execution\""));
    assert!(output_json.contains("\"object_package_summary_next_command\":\"nsdb replay "));
    assert!(output_json.contains("\"final_output_nsdb_replay_checkpoint_count\":1"));
    assert!(output_json.contains("\"final_output_nsdb_replayable_checkpoint_count\":1"));
    assert!(output_json.contains("\"final_output_nsdb_replay_first_blocker\":null"));
}

#[test]
fn final_executable_output_replay_blocks_pending_hetero_closure() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-output-nsdb-handoff-closure-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let manifest = write_test_build_manifest_with_packaging_mode(&dir, "nuis-self-contained-image");
    let plan = nuisc::linker::build_link_plan_from_manifest(Path::new(&manifest)).unwrap();

    nsld_prepare_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_layout_plan_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new(&manifest), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new(&manifest), &plan).unwrap();

    let mut output = nsld_final_executable_output_report(Path::new(&manifest), &plan);
    let summary = super::final_executable_output_nsdb_handoff::persist_final_output_nsdb_handoff(
        Path::new(&plan.output_dir),
        &output,
    );
    let handoff_path = dir.join("nuis.nsdb.payload-execution-handoff.toml");
    let handoff = fs::read_to_string(&handoff_path).unwrap();
    fs::write(
        &handoff_path,
        handoff.replace(
            "record_count = 1\n",
            "record_count = 1\nhetero_execution_closure_protocol = \"nuis-hetero-execution-closure-v1\"\nhetero_execution_closure_status = \"host-runner-pending\"\nhetero_execution_closure_ready = \"false\"\nhetero_execution_closure_first_blocker = \"host-runner-backend-artifact-payload:not-observed\"\nhetero_execution_closure_next_action = \"run-host-runner-payload-probe\"\n",
        ),
    )
    .unwrap();

    super::final_executable_output_nsdb_handoff::attach_final_output_nsdb_handoff_summary(
        &mut output,
        summary,
    );
    let output_json = super::json::nsld_final_executable_output_report_json(&output);
    fs::remove_dir_all(dir).unwrap();

    assert!(!output.final_output_nsdb_replay_ready);
    assert_eq!(output.final_output_nsdb_replay_status, "blocked");
    assert_eq!(
        output.final_output_nsdb_replay_first_blocker.as_deref(),
        Some("hetero-execution-closure:host-runner-backend-artifact-payload:not-observed")
    );
    assert_eq!(
        output.final_output_nsdb_replay_next_action,
        "resolve-final-output-nsdb-replay"
    );
    assert_eq!(output.owned_package_summary_status, "replay-blocked");
    assert!(!output.owned_package_summary_ready);
    assert_eq!(output.object_package_summary_status, "replay-blocked");
    assert!(!output.object_package_summary_ready);
    assert_eq!(output.object_package_summary_replay_status, "blocked");
    assert!(!output.object_package_summary_replay_ready);
    assert_eq!(
        output.object_package_summary_next_action,
        "resolve-final-output-nsdb-replay"
    );
    assert!(output_json.contains("\"final_output_nsdb_replay_ready\":false"));
    assert!(output_json.contains("\"final_output_nsdb_replay_status\":\"blocked\""));
    assert!(output_json.contains(
        "\"final_output_nsdb_replay_first_blocker\":\"hetero-execution-closure:host-runner-backend-artifact-payload:not-observed\""
    ));
    assert!(output_json.contains("\"object_package_summary_status\":\"replay-blocked\""));
    assert!(output_json.contains("\"object_package_summary_ready\":false"));
    assert!(output_json.contains("\"object_package_summary_replay_status\":\"blocked\""));
    assert!(output_json.contains("\"object_package_summary_replay_ready\":false"));
}

fn write_test_build_manifest_with_packaging_mode(dir: &Path, packaging_mode: &str) -> String {
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(&bin, "bin").unwrap();

    let written = CompileArtifacts {
        ast_path: ast.display().to_string(),
        nir_path: nir.display().to_string(),
        yir_path: yir.display().to_string(),
        llvm_ir_path: ll.display().to_string(),
        binary_path: bin.display().to_string(),
        packaging_mode: packaging_mode.to_owned(),
    };
    nuisc::aot::write_build_manifest(
        dir,
        &written,
        &BuildManifestContext {
            input_path: dir.join("demo.ns").display().to_string(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache: None,
            project: None,
            doc_index: None,
            cpu_target: nuisc::aot::host_cpu_build_target(),
        },
    )
    .unwrap()
}
