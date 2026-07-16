use super::{
    apply_suggested_project_imports, artifact_doctor_command_for_output_dir,
    artifact_workflow_brief, benchmark_run_report_json, build_workflow_frontdoor_surface,
    collect_language_benchmark_run_report, default_build_output_dir, find_abi_block_span,
    handle_build, handle_check, handle_materialize_artifact, handle_release_check,
    handle_run_artifact, handle_test, handle_unpack_artifact_support,
    nsld_drive_command_set_for_output_dir, project_abi_checks_json,
    project_compile_workflow_source_profile, project_domain_registry_checks_json,
    project_workflow_json_fields, recommend_project_workflow_step,
    release_check_nsld_drive_command_for_output_dir,
    release_check_nsld_drive_dry_run_command_for_output_dir,
    release_check_nsld_drive_dry_run_json_command_for_output_dir,
    release_check_nsld_drive_json_command_for_output_dir,
    release_check_nsld_drive_until_clean_command_for_output_dir,
    release_check_nsld_drive_until_clean_json_command_for_output_dir, render_artifact_doctor_json,
    render_build_report_json, render_project_doctor_json, render_project_imports_apply_json,
    render_project_imports_json, render_project_status_json, render_run_artifact_json,
    render_scheduler_view_json, render_workflow_json, resolve_run_artifact_binary_path,
    resolve_runner_clock_domain, run_artifact_command_for_output_dir, run_build_output_self_check,
    run_language_benchmarks_for_source_file, run_language_tests_for_source_file,
    scheduler_view_domain_record, scheduler_view_domain_record_json,
    single_source_workflow_source_profile, upsert_abi_block, wait_for_test_child,
    PublicSurfaceModuleRecord, RawTestOutcome, WorkflowRecommendation,
};
use crate::galaxy;
use crate::json_surface::{
    galaxy_lock_json_fields, project_check_summary_json_fields, public_surface_summary_json_fields,
    workflow_contract_json_fields,
};
use crate::release_check_command::render_release_check_summary_json;
use crate::surface_render;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Once,
    time::{SystemTime, UNIX_EPOCH},
};

fn enable_test_quiet_success_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        env::set_var("NUIS_TEST_QUIET_SUCCESS_LOGS", "1");
    });
}

fn repo_root() -> PathBuf {
    enable_test_quiet_success_logs();
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn checked_in_path(relative_from_manifest_dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(relative_from_manifest_dir)
        .canonicalize()
        .unwrap_or_else(|error| panic!("checked-in path `{relative_from_manifest_dir}`: {error}"))
}

fn write_prepared_nsld_chain_placeholders(output_dir: &Path) {
    for file_name in [
        "nuis.nsld.link-inputs.toml",
        "nuis.nsld.link-units.toml",
        "nuis.nsld.link-bundle.toml",
        "nuis.nsld.assemble-plan.toml",
        "nuis.nsld.section-manifest.toml",
        "nuis.nsld.object-plan.toml",
        "nuis.nsld.object-writer-input.toml",
        "nuis.nsld.object-byte-layout.toml",
        "nuis.nsld.object-file-layout.toml",
        "nuis.nsld.object-image-dry-run.toml",
        "nuis.nsld.object.blocked.toml",
        "nuis.nsld.object-writer-dry-run.toml",
        "nuis.nsld.container-plan.toml",
        "nuis.nsld.container",
        "nuis.nsld.closure.toml",
        "nuis.nsld.final-stage-plan.toml",
    ] {
        fs::write(output_dir.join(file_name), "prepared-stage-placeholder\n")
            .expect("write prepared nsld placeholder");
    }
}

fn write_ready_nsld_final_tail_placeholders(output_dir: &Path) {
    for file_name in [
        "nuis.nsld.final-executable-writer-input.toml",
        "nuis.nsld.final-executable-host-invoke-plan.toml",
        "nuis.nsld.final-executable-layout.toml",
        "nuis.nsld.final-executable-image-dry-run.toml",
        "nuis.nsld.final-executable-image-dry-run.bin",
        "nuis.nsld.final-executable.blocked.toml",
        "nuis.nsld.final-executable-launcher.toml",
        "nuis.nsld.final-executable-launcher-dry-run.toml",
    ] {
        fs::write(output_dir.join(file_name), "final-tail-stage-placeholder\n")
            .expect("write final tail nsld placeholder");
    }
    fs::write(
        output_dir.join("nuis.nsld.final-executable-pipeline.toml"),
        r#"
valid = true
final_executable_emitted = true
launcher_manifest_ready = true
launcher_dry_run_ready = true
would_enter_lifecycle_hook = true
execution_handoff_contract = "nsld-final-output-handoff-v1"
execution_handoff_ready = true
execution_handoff_status = "entrypoint-materializer-required"
execution_handoff_target = "entrypoint-materializer"
execution_handoff_evidence_status = "image-header-and-hash-ready"
execution_handoff_decision_code = "handoff-entrypoint-materializer"
entrypoint_materialization_kind = "host-shell-entrypoint-plan"
entrypoint_materialization_path = "nuis.host-entrypoint.sh"
entrypoint_materialization_ready = true
entrypoint_materialization_first_blocker = ""
entrypoint_materialization_present = true
entrypoint_materialization_hash = "0xabcd"
entrypoint_materialization_runner_command = "nuis-host-runner --manifest 'manifest.toml' --nsb 'nuis-app.nsb' --output-dir 'out' --scheduler-entry 'nuis.scheduler.loop.v1' --lifecycle-hook 'on_process_start'"
scheduler_metadata_payload_id = "payload0004.scheduler-metadata"
scheduler_metadata_present = true
scheduler_metadata_hash = "0x1234"
required_stage_path_count = 10
required_stage_path_present_count = 10
missing_required_stage_paths = []
blocker_count = 0
blockers = []
"#
        .trim_start(),
    )
    .expect("write ready final executable pipeline placeholder");
    fs::write(
        output_dir.join("nuis.host-entrypoint.sh"),
        "#!/bin/sh\nset -eu\nNUIS_HOST_ENTRYPOINT_STUB_PROTOCOL='nuis-nsld-host-entrypoint-v1'\nexport NUIS_HOST_ENTRYPOINT_STUB_PROTOCOL\nexec nuis-host-runner --manifest 'manifest.toml' --nsb 'nuis-app.nsb' --output-dir 'out' --scheduler-entry 'nuis.scheduler.loop.v1' --lifecycle-hook 'on_process_start'\n",
    )
    .expect("write host entrypoint placeholder");
}

fn write_nsdb_payload_handoff_placeholder(output_dir: &Path) {
    fs::write(
        output_dir.join("nuis.nsdb.payload-execution-handoff.toml"),
        r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
source = "run-artifact-launch-evidence"
record_count = 1
ready_record_count = 1
first_trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
first_status = "ready"
first_next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
status = "ready"
execution_phase = "container-loader-handoff"
target = "container-loader"
entry_symbol = "nuis.bootstrap.lifecycle.v1"
entry_kind = "lifecycle-bootstrap"
entry_section_id = "sec0000.compiled-artifact"
first_blocker = ""
next_action = "handoff-payload-trace-to-nsdb"
"#
        .trim_start(),
    )
    .expect("write nsdb payload handoff placeholder");
}

fn load_stdlib_source_modules(root: &Path, module_dir: &str) -> Vec<String> {
    let module_path = root.join("stdlib").join(module_dir).join("module.toml");
    let source = fs::read_to_string(&module_path)
        .unwrap_or_else(|error| panic!("{}: {error}", module_path.display()));
    let mut inside = false;
    let mut modules = Vec::new();
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if !inside {
            if line.starts_with("source_modules") && line.contains('[') {
                inside = true;
            }
            continue;
        }
        if line.starts_with(']') {
            break;
        }
        let entry = line.trim_end_matches(',').trim();
        if entry.is_empty() {
            continue;
        }
        let entry = entry.trim_matches('"');
        if !entry.is_empty() {
            modules.push(format!("stdlib/{module_dir}/{entry}"));
        }
    }
    assert!(
        !modules.is_empty(),
        "{} did not declare any source_modules",
        module_path.display()
    );
    modules
}

#[path = "main_tests/abi_surface.rs"]
mod abi_surface;
#[path = "main_tests/artifact_runtime.rs"]
mod artifact_runtime;
#[path = "main_tests/artifact_runtime_run_artifact.rs"]
mod artifact_runtime_run_artifact;
#[path = "main_tests/language_runner.rs"]
mod language_runner;
#[path = "main_tests/project_health.rs"]
mod project_health;
#[path = "main_tests/project_imports_and_scheduler.rs"]
mod project_imports_and_scheduler;
fn temp_dir(label: &str) -> PathBuf {
    enable_test_quiet_success_logs();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = env::temp_dir().join(format!("nuis_{label}_{}_{}", std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_temp_project_fixture(name: &str, manifest: &str, entry_source: &str) -> PathBuf {
    let root = temp_dir(name);
    fs::write(root.join("nuis.toml"), manifest).expect("write manifest");
    fs::write(root.join("main.ns"), entry_source).expect("write entry");
    root
}

fn assert_checked_in_tooling_project_runs(project_root: &str, output_label: &str) {
    let project_root = checked_in_path(project_root);
    let output_dir = temp_dir(output_label);

    handle_build(project_root, output_dir.clone(), false, None, None, None).expect("build passes");
    handle_run_artifact(output_dir.join("nuis.build.manifest.toml"), false)
        .expect("checked-in tooling project run-artifact passes");
}

fn empty_galaxy_doctor(project_root: &Path) -> galaxy::GalaxyDoctorReport {
    galaxy::GalaxyDoctorReport {
        project_root: project_root.to_path_buf(),
        project_plan_summary: "<none>".to_owned(),
        deps_root: project_root.join(".nuis").join("deps"),
        local_registry_root: project_root.join(".nuis").join("registry"),
        lock_path: project_root.join("nuis.galaxy.lock"),
        lock_status: "missing".to_owned(),
        lock_error: None,
        dependencies: vec![],
    }
}

#[path = "main_tests/stdlib_and_artifact.rs"]
mod stdlib_and_artifact;
#[path = "main_tests/workflow_surface.rs"]
mod workflow_surface;
