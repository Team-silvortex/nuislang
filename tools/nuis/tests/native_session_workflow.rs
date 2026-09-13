//! Real frontdoor build/cache/relocation/launch, without provider materialization.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SOURCE: &str = include_str!("../../nuisc/tests/native_application_bridge/multi_loops.ns");
const BREAK_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/break_loops.ns");
const BRANCH_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/branch_loops.ns");
const DIVISION_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/division_loops.ns");
const AGGREGATE_DIVISION_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_division_loops.ns");
const AGGREGATE_COUNTED_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_counted_loops.ns");
const AGGREGATE_CARRIED_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_carried_loops.ns");
const AGGREGATE_CONDITIONAL_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_conditional_loops.ns");
const SCRIPT: &[&str] = &[
    "--native-session",
    "counter",
    "--open-args",
    "10,-17,true,1.5,-2.25",
    "--event-args",
    "3,false",
    "--event-args",
    "100,true",
    "--event-args",
    "-2,false",
    "--close-args",
    "5",
];

struct Project(PathBuf);
impl Project {
    fn new(source: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nuis-native-workflow-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.ns"), source).unwrap();
        fs::write(path.join("nuis.toml"), "name = \"native_workflow\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\npackaging_mode = \"native-session-aot-bundle:counter\"\napplication_sessions = [\"counter open=start event=step close=stop state=state\", \"other open=start event=step close=stop state=state\"]\n").unwrap();
        Self(path)
    }

    fn command(&self, verb: &str, input: &Path, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_nuis"))
            .arg(verb)
            .arg(input)
            .args(arguments)
            .env("CARGO_BUILD_JOBS", "1")
            .env("CARGO_INCREMENTAL", "0")
            .env_remove("NUIS_TEST_QUIET_SUCCESS_LOGS")
            .output()
            .unwrap()
    }

    fn build(&self, mode: Option<&str>) -> String {
        let output = self.0.join("build");
        let mut args = vec![output.to_str().unwrap()];
        if let Some(mode) = mode {
            args.extend(["--packaging-mode", mode]);
        }
        let built = success(self.command("build", &self.0, &args));
        String::from_utf8(built.stdout).unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn states(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|line| {
            line.strip_prefix("nuis: native_session_state=")
                .map(str::to_owned)
        })
        .collect()
}

fn rejected_before_open(output: Output) {
    assert!(
        !output.status.success(),
        "must reject instead of falling back"
    );
    assert!(
        states(&output).is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("native_session_completed=1"));
}

#[test]
fn native_build_run_artifact_cache_and_standalone_relocation() {
    check_workflow(SOURCE);
}

#[test]
fn native_guarded_break_build_cache_and_standalone_relocation() {
    check_workflow(BREAK_SOURCE);
}

#[test]
fn native_multi_state_branch_build_cache_and_standalone_relocation() {
    check_workflow(BRANCH_SOURCE);
}

#[test]
fn native_checked_division_build_cache_and_standalone_relocation() {
    check_workflow(DIVISION_SOURCE);
}

#[test]
fn native_fallible_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_DIVISION_SOURCE);
}

#[test]
fn native_guarded_counted_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_COUNTED_SOURCE);
}

#[test]
fn native_guarded_carried_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_CARRIED_SOURCE);
}

#[test]
fn native_guarded_conditional_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_CONDITIONAL_SOURCE);
}

fn check_workflow(source: &str) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(source);
    project.build(None);
    let output = project.0.join("build");
    let manifest_path = output.join("nuis.build.manifest.toml");
    let report = nuisc::aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(report.packaging_mode, "native-session-aot-bundle:counter");
    let stem = &report.artifact_binary_name;
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest.contains("native-scalar-llvm-v1"));
    assert!(manifest.contains("native_session_llvm_hex"));
    assert!(!manifest.contains("headless_input_schema"));
    let llvm = fs::read_to_string(output.join(format!("{stem}.ll"))).unwrap();
    assert!(llvm.contains("nuis_native_session_636f756e746572_open_v1"));
    assert!(!llvm.contains("define i64 @main"));
    for ty in ["i1", "i32", "i64", "float", "double"] {
        assert!(llvm.contains(&format!(" = call {ty} @nuis_fn_")));
    }
    assert!(llvm.contains("loop_while_i64_cond"));
    assert!(llvm.contains("loop_while_i64_body"));
    if source == SOURCE {
        assert!(llvm.contains(" = call i64 @nuis_fn_advance("));
    } else {
        assert!(llvm.contains(" = call i64 @nuis_fn_counted("));
        assert!(llvm.contains("loop_break_control_invalid"));
    }
    assert!(llvm.contains("native_loop_preflight"));
    if [
        DIVISION_SOURCE,
        AGGREGATE_DIVISION_SOURCE,
        AGGREGATE_COUNTED_SOURCE,
        AGGREGATE_CARRIED_SOURCE,
        AGGREGATE_CONDITIONAL_SOURCE,
    ]
    .contains(&source)
    {
        assert!(llvm.contains("sdiv i64"));
        assert!(llvm.contains("srem i64"));
        assert!(llvm.contains("integer_divisor_invalid"));
    }
    if [
        AGGREGATE_DIVISION_SOURCE,
        AGGREGATE_COUNTED_SOURCE,
        AGGREGATE_CARRIED_SOURCE,
        AGGREGATE_CONDITIONAL_SOURCE,
    ]
    .contains(&source)
    {
        assert!(llvm.contains("@nuis_fn_decompose("));
        assert!(llvm.contains("@nuis_fn___nuis_scalar_branch_"));
    }
    if [
        AGGREGATE_COUNTED_SOURCE,
        AGGREGATE_CARRIED_SOURCE,
        AGGREGATE_CONDITIONAL_SOURCE,
    ]
    .contains(&source)
    {
        assert!(llvm.contains("@nuis_fn_rebalance("));
    }
    if source == AGGREGATE_CARRIED_SOURCE {
        assert!(llvm.contains("loop_while_scalar_chain_body"));
    }
    if source == AGGREGATE_CONDITIONAL_SOURCE {
        assert!(llvm.contains("loop_while_scalar_cond_chain_body"));
    }
    let run = success(project.command("run-artifact", &output, SCRIPT));
    assert!(run.stdout.is_empty(), "unrelated main must not execute");
    let expected = states(&run);
    assert_eq!(expected.len(), 5);
    for (state, total) in expected.iter().zip([10, 13, 13, 11, 16]) {
        for field in [
            format!("total: {total}"),
            "tag: -17".to_owned(),
            "gain: 1.5".to_owned(),
            "scale: -2.25".to_owned(),
        ] {
            assert!(state.contains(&field), "{state} must preserve {field}");
        }
    }
    assert!(expected[4].contains("total: 16"), "{expected:?}");
    assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    let files = fs::read_dir(&output)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        !files
            .iter()
            .any(|file| file.contains("provider") || file.contains("receipt")),
        "{files:?}"
    );

    rejected_before_open(project.command("run-artifact", &output, &[]));
    for changed in [
        SCRIPT.join("\n").replace("3,false", "3,1"),
        SCRIPT.join("\n").replace("counter", "other"),
        SCRIPT
            .join("\n")
            .replace("10,-17,true", "10,2147483648,true"),
        SCRIPT
            .join("\n")
            .replace("--close-args\n5", "--close-args\nfalse"),
    ] {
        rejected_before_open(project.command(
            "run-artifact",
            &output,
            &changed.split('\n').collect::<Vec<_>>(),
        ));
    }
    for name in [
        "bundle.txt".to_owned(),
        format!("{stem}.yir"),
        format!("{stem}.ll"),
        format!("{stem}.source.ns"),
        stem.clone(),
        "nuis.doc-index.json".to_owned(),
        "nuis.project.galaxy.lock".to_owned(),
    ] {
        let path = output.join(name);
        let original = fs::read(&path).unwrap();
        let mut changed = original.clone();
        changed.push(b' ');
        fs::write(&path, changed).unwrap();
        rejected_before_open(project.command("run-artifact", &output, SCRIPT));
        fs::write(&path, original).unwrap();
    }
    // The same output/cache cannot substitute another registration's native object.
    project.build(Some("native-session-aot-bundle:other"));
    rejected_before_open(project.command("run-artifact", &output, SCRIPT));
    let mut other = SCRIPT.to_vec();
    other[1] = "other";
    assert_eq!(
        states(&success(project.command("run-artifact", &output, &other))),
        expected
    );
    assert_eq!(
        nuisc::aot::verify_build_manifest(&manifest_path)
            .unwrap()
            .packaging_mode,
        "native-session-aot-bundle:other"
    );
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    assert_eq!(
        states(&success(project.command("run-artifact", &output, SCRIPT))),
        expected
    );

    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(&output).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    assert!(relocated.join(format!("{stem}.ll")).is_file());
    assert_eq!(
        states(&success(project.command(
            "run-artifact",
            &relocated,
            SCRIPT
        ))),
        expected
    );
}
