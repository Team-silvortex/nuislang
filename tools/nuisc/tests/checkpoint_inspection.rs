use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const SOURCE: &str = r#"
mod cpu Main {
  struct Counter { count: i64 }
  fn start(seed: i64) -> Counter { return Counter { count: seed }; }
  fn step(state: Counter, input: i64) -> Counter {
    let x: i64 = state.count;
    while x < 100 {
      let x: i64 = x + 1;
      if x > input && x < 50 { break; }
    }
    return Counter { count: x };
  }
  fn stop(state: Counter) -> Counter { return state; }
  fn main() { print(999); }
}
"#;

struct Project(PathBuf);

impl Project {
    fn new(mode: Option<&str>, source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let project = Self(std::env::temp_dir().join(format!(
            "nuis-inspection-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&project.0).unwrap();
        fs::write(project.0.join("main.ns"), source).unwrap();
        let mut manifest = String::from(
            "name = \"inspection\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\napplication_sessions = [\"counter open=start event=step close=stop state=state\"]\n",
        );
        if let Some(mode) = mode {
            manifest.push_str(&format!("packaging_mode = \"{mode}\"\n"));
        }
        fs::write(project.0.join("nuis.toml"), manifest).unwrap();
        project
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(input: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args(args)
        .arg(input)
        .env_remove("NUIS_TEST_QUIET_SUCCESS_LOGS")
        .output()
        .unwrap()
}

fn success(output: &Output) -> String {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn manifest_selected_headless_check_dump_and_inspection_skip_llvm() {
    let project = Project::new(Some("headless-aot-bundle"), SOURCE);
    for input in [&project.0, &project.0.join("nuis.toml")] {
        let check = success(&run(input, &["check"]));
        assert!(
            check.contains("compiler_checkpoint: verified-yir"),
            "{check}"
        );
        assert!(check.contains("llvm_emit: not_requested"));
        assert!(!check.contains("llvm_ir_bytes:"));
        for command in ["dump-ast", "dump-nir", "dump-yir"] {
            let output = run(input, &[command]);
            let dump = success(&output);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("compiler_checkpoint: verified-yir"));
            assert!(stderr.contains("llvm_emit: not_requested"));
            assert!(!dump.contains("compiler_checkpoint:"));
            assert!(!dump.is_empty());
            if command == "dump-yir" {
                let module = yir_syntax::parse_module(&dump).unwrap();
                yir_verify::verify_module_with_registry(&module, &yir_verify::default_registry())
                    .unwrap();
                assert_eq!(module.application_sessions.len(), 1);
            }
        }
        let json = success(&run(input, &["inspect-benchmarks", "--json"]));
        assert!(json.starts_with('{') && json.trim_end().ends_with('}'));
        assert!(json.contains("\"compiler_checkpoint\":\"verified-yir\""));
        assert!(json.contains("\"llvm_emit\":\"not_requested\""));
        let bindings = success(&run(input, &["bindings"]));
        assert!(bindings.contains("compiler_checkpoint: verified-yir"));
        assert!(bindings.contains("package: official.cpu"));
    }
    let mut files = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(
        files,
        ["main.ns", "nuis.toml"],
        "inspection must not package artifacts"
    );
}

#[test]
fn headless_pipeline_report_does_not_claim_native_readiness() {
    let project = Project::new(Some("headless-aot-bundle"), SOURCE);
    let resolved = nuisc::pipeline::resolve_compile_input(&project.0).unwrap();
    let inspection = resolved.compile_for_inspection().unwrap();
    let report = inspection.report(&resolved);
    assert!(inspection.view().llvm_ir.is_none());
    assert_eq!(report.compiler_checkpoint, "verified-yir");
    assert_eq!(report.llvm_ir_bytes, None);
    assert!(!report.ready_for_aot);
    assert_eq!(report.recommended_next_step, "build_headless");
    assert_eq!(report.stage_count(), report.ok_stage_count() + 1);
    assert!(report
        .stages
        .iter()
        .any(|stage| stage.id == "llvm_emit" && stage.status == "not_requested"));
    assert!(report
        .stages
        .iter()
        .any(|stage| stage.id == "yir_verify" && stage.status == "ok"));
    // Existing native APIs retain their explicit LLVM contract, even for this project.
    let native = resolved.compile().unwrap();
    assert!(!native.llvm_ir.is_empty());
    assert_eq!(
        nuisc::render::render_yir(inspection.view().yir),
        nuisc::render::render_yir(&native.yir)
    );
}

#[test]
fn native_and_window_inspection_still_emit_llvm() {
    for mode in [
        None,
        Some("native-cpu-llvm"),
        Some("window-aot-bundle"),
        Some("nuis-self-contained-image"),
    ] {
        let project = Project::new(mode, SOURCE);
        let check = success(&run(&project.0, &["check"]));
        assert!(check.contains("compiler_checkpoint: llvm-ir"));
        assert!(check.contains("llvm_emit: emitted"));
        assert!(check.contains("llvm_ir_bytes:"));
        assert!(!check.contains("not_requested"));
        let resolved = nuisc::pipeline::resolve_compile_input(&project.0).unwrap();
        let report = resolved.compile_for_inspection().unwrap().report(&resolved);
        assert_eq!(report.compiler_checkpoint, "llvm-ir");
        assert!(report.llvm_ir_bytes.is_some_and(|bytes| bytes > 0));
        assert!(report.ready_for_aot);
        assert_eq!(report.stage_count(), report.ok_stage_count());
    }
    let project = Project::new(Some("headless-aot-bundle"), SOURCE);
    let source_check = success(&run(&project.0.join("main.ns"), &["check"]));
    assert!(source_check.contains("compiler_checkpoint: llvm-ir"));
}

#[test]
fn inspection_rejects_invalid_registration_and_unsupported_lowering_without_fallback() {
    for source in [
        SOURCE.replace("fn start(seed:", "fn missing_start(seed:"),
        SOURCE.replace("x + 1", "undefined_value + 1"),
        SOURCE.replace("let x: i64 = state.count;", "let x: i64 = true;"),
        SOURCE.replace("let x: i64 = x + 1;", "print(x); let x: i64 = x + 1;"),
        SOURCE.replace(
            "fn stop(state: Counter) -> Counter { return state; }",
            "fn stop(state: Counter) -> i64 { return state.count; }",
        ),
    ] {
        for mode in [None, Some("headless-aot-bundle")] {
            let project = Project::new(mode, &source);
            let resolved = nuisc::pipeline::resolve_compile_input(&project.0).unwrap();
            assert_eq!(
                resolved.compile().err(),
                resolved.compile_for_inspection().err()
            );
            for args in [
                vec!["check"],
                vec!["dump-ast"],
                vec!["dump-nir"],
                vec!["dump-yir"],
                vec!["inspect-benchmarks", "--json"],
                vec!["bindings"],
            ] {
                let output = run(&project.0, &args);
                assert!(!output.status.success(), "accepted {args:?} {mode:?}");
                assert!(output.stdout.is_empty(), "no partial success report");
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!stderr.is_empty());
                assert!(!stderr.contains("compiler_checkpoint:"));
            }
        }
    }
}

#[test]
fn inspection_and_build_reject_unknown_packaging_selection() {
    let project = Project::new(Some("headless-typo"), SOURCE);
    let output = run(&project.0, &["check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("unsupported packaging mode `headless-typo`"));
    let build_dir = project.0.join("output");
    let error = nuisc::run(nuisc::CommandKind::Compile {
        input: project.0.clone(),
        output_dir: build_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap_err();
    assert!(error.contains("unsupported packaging mode `headless-typo`"));
    assert!(!build_dir.exists());
}
