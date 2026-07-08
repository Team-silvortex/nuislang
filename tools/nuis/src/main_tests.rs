use super::{
    apply_suggested_project_imports, artifact_doctor_command_for_output_dir,
    artifact_workflow_brief, benchmark_run_report_json, build_workflow_frontdoor_surface,
    collect_language_benchmark_run_report, default_build_output_dir, find_abi_block_span,
    handle_build, handle_check, handle_materialize_artifact, handle_release_check,
    handle_run_artifact, handle_test, handle_unpack_artifact_support, project_abi_checks_json,
    project_compile_workflow_source_profile, project_domain_registry_checks_json,
    project_workflow_json_fields, recommend_project_workflow_step, render_artifact_doctor_json,
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
use crate::surface_render;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, Once, OnceLock},
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

    handle_build(project_root, output_dir.clone(), false, None, None).expect("build passes");
    handle_run_artifact(output_dir.join("nuis.build.manifest.toml"), false)
        .expect("checked-in tooling project run-artifact passes");
}

fn with_repo_root_cwd<T>(f: impl FnOnce() -> T) -> T {
    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = CWD_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("lock cwd guard");
    let original = env::current_dir().expect("current dir");
    let root = repo_root();
    env::set_current_dir(&root).expect("set repo root cwd");
    let result = f();
    env::set_current_dir(original).expect("restore cwd");
    result
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
