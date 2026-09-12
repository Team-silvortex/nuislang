use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Project(PathBuf);

impl Project {
    fn new(mode: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let project = Self(std::env::temp_dir().join(format!(
            "nuis-checkpoint-workflow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&project.0).unwrap();
        fs::write(project.0.join("nuis.toml"), format!(
            "name = \"checkpoint_workflow\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\npackaging_mode = \"{mode}\"\napplication_sessions = [\"counter open=start event=step close=stop state=state\"]\n"
        )).unwrap();
        fs::write(
            project.0.join("main.ns"),
            r#"
mod cpu Main {
  struct Counter { count: i64 }
  fn start(seed: i64) -> Counter { return Counter { count: seed }; }
  fn step(state: Counter, input: i64) -> Counter {
    return Counter { count: state.count + input };
  }
  fn stop(state: Counter) -> Counter { return state; }
  fn main() { print(999); }
}
"#,
        )
        .unwrap();
        project
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_nuis"))
            .args(args)
            .arg(&self.0)
            .env_remove("NUIS_TEST_QUIET_SUCCESS_LOGS")
            .output()
            .unwrap()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn headless_frontdoor_check_dump_and_workflow_report_the_selected_checkpoint() {
    let project = Project::new("headless-aot-bundle");
    let check = success(project.run(&["check"]));
    assert!(check.contains("compiler_checkpoint: verified-yir"));
    assert!(check.contains("llvm_emit: not_requested"));
    let dump = success(project.run(&["dump-yir"]));
    let module = yir_syntax::parse_module(&dump).unwrap();
    assert_eq!(module.application_sessions.len(), 1);
    let workflow = success(project.run(&["workflow", "--json"]));
    for field in [
        "\"compile_pipeline_available\":true",
        "\"compile_pipeline_checkpoint\":\"verified-yir\"",
        "\"compile_pipeline_llvm_ir_bytes\":null",
        "\"id\":\"llvm_emit\",\"status\":\"not_requested\"",
        "\"compile_pipeline_ready_for_aot\":false",
        "\"compile_pipeline_recommended_next_step\":\"build_headless\"",
        "\"artifact_ready_to_run\":false",
    ] {
        assert!(workflow.contains(field), "missing {field}: {workflow}");
    }
}

#[test]
fn native_frontdoor_workflow_keeps_native_stage_evidence() {
    let project = Project::new("native-cpu-llvm");
    let workflow = success(project.run(&["workflow", "--json"]));
    assert!(workflow.contains("\"compile_pipeline_checkpoint\":\"llvm-ir\""));
    assert!(workflow.contains("\"id\":\"llvm_emit\",\"status\":\"ok\""));
    assert!(workflow.contains("\"compile_pipeline_ready_for_aot\":true"));
    assert!(!workflow.contains("\"compile_pipeline_llvm_ir_bytes\":null"));
    assert!(workflow.contains("\"artifact_ready_to_run\":false"));
}

#[test]
fn native_session_inspection_emits_only_the_selected_callback_checkpoint() {
    let project = Project::new("native-session-aot-bundle:counter");
    let check = success(project.run(&["check"]));
    assert!(check.contains("compiler_checkpoint: llvm-ir"), "{check}");
    assert!(check.contains("llvm_emit: emitted"), "{check}");
    let path = project.0.join("nuis.toml");
    let source = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        source.replace(
            "native-session-aot-bundle:counter",
            "native-session-aot-bundle:missing",
        ),
    )
    .unwrap();
    assert!(!project.run(&["check"]).status.success());
}

#[test]
fn workflow_does_not_turn_invalid_headless_yir_into_success_evidence() {
    let project = Project::new("headless-aot-bundle");
    let path = project.0.join("main.ns");
    let source = fs::read_to_string(&path).unwrap();
    fs::write(&path, source.replace("fn start(", "fn missing_start(")).unwrap();
    let check = project.run(&["check"]);
    assert!(!check.status.success());
    assert!(check.stdout.is_empty());
    let workflow = success(project.run(&["workflow", "--json"]));
    assert!(workflow.contains("\"compile_pipeline_available\":false"));
    assert!(workflow.contains("\"compile_pipeline_error\":"));
    assert!(!workflow.contains("\"compile_pipeline_checkpoint\":"));
    assert!(workflow.contains("\"artifact_ready_to_run\":false"));
}
