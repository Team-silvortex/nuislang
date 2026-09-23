//! Real frontdoor build/cache/relocation/launch, without provider materialization.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[path = "native_session_workflow/helper_entries.rs"]
mod helper_entries;
#[path = "native_session_workflow/loop_work.rs"]
mod loop_work;

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
const AGGREGATE_ONE_SIDED_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_one_sided_loops.ns");
const AGGREGATE_COMPOUND_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_compound_loops.ns");
const AGGREGATE_NESTED_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_nested_loops.ns");
const AGGREGATE_SEQUENCES_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_sequences_loops.ns");
const AGGREGATE_TEMPORARIES_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_temporaries_loops.ns");
const AGGREGATE_CHECKED_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_checked_loops.ns");
const AGGREGATE_CALLS_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_calls_loops.ns");
const AGGREGATE_LOCAL_VALUES_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_local_values_loops.ns");
const AGGREGATE_LOOP_CALLS_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_loop_calls_loops.ns");
const BOOL_REBINDING_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/bool_rebinding_loops.ns");
const AGGREGATE_REBINDING_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_rebinding_loops.ns");
const AGGREGATE_CARRIES_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/aggregate_carries_loops.ns");
const BOOL_CARRIES_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/bool_carries_loops.ns");
const LITERAL_LOOPS_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/literal_loops.ns");
const VALUE_LOOP_EXITS_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/value_loop_exits.ns");
const TRAILING_VALUE_LOOPS_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/trailing_value_loops.ns");
const COUNTED_RETURNS_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/counted_returns.ns");
const CONTROL_COMPOSITION_SOURCE: &str =
    include_str!("../../nuisc/tests/native_application_bridge/control_composition.ns");
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

#[test]
fn native_guarded_one_sided_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_ONE_SIDED_SOURCE);
}

#[test]
fn native_guarded_compound_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_COMPOUND_SOURCE);
}

#[test]
fn native_guarded_nested_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_NESTED_SOURCE);
}

#[test]
fn native_multi_statement_aggregate_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_SEQUENCES_SOURCE);
}

#[test]
fn native_iteration_temporaries_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_TEMPORARIES_SOURCE);
}

#[test]
fn native_checked_iterations_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_CHECKED_SOURCE);
}

#[test]
fn native_iteration_calls_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_CALLS_SOURCE);
}

#[test]
fn native_iteration_flat_values_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_LOCAL_VALUES_SOURCE);
}

#[test]
fn native_iteration_loop_calls_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_LOOP_CALLS_SOURCE);
}

#[test]
fn native_iteration_bool_rebindings_build_cache_and_standalone_relocation() {
    check_workflow(BOOL_REBINDING_SOURCE);
}

#[test]
fn native_iteration_flat_rebindings_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_REBINDING_SOURCE);
}

#[test]
fn native_outer_flat_carries_build_cache_and_standalone_relocation() {
    check_workflow(AGGREGATE_CARRIES_SOURCE);
}

#[test]
fn native_outer_bool_carries_build_cache_and_standalone_relocation() {
    check_workflow(BOOL_CARRIES_SOURCE);
}

#[test]
fn native_literal_nested_loops_build_cache_and_standalone_relocation() {
    check_workflow(LITERAL_LOOPS_SOURCE);
}

#[test]
fn native_trailing_value_loops_build_cache_and_standalone_relocation() {
    check_workflow(TRAILING_VALUE_LOOPS_SOURCE);
}

#[test]
fn native_value_loop_exits_build_cache_and_standalone_relocation() {
    check_workflow(VALUE_LOOP_EXITS_SOURCE);
}

#[test]
fn native_counted_returns_build_cache_and_standalone_relocation() {
    check_workflow(COUNTED_RETURNS_SOURCE);
}

#[test]
fn native_control_composition_build_cache_and_standalone_relocation() {
    check_workflow(CONTROL_COMPOSITION_SOURCE);
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
    if source == CONTROL_COMPOSITION_SOURCE {
        let functions = llvm
            .lines()
            .filter(|line| line.starts_with("define ") && line.contains(" @nuis_fn_"))
            .count();
        assert!(functions <= 51, "{functions} reachable native functions");
    }
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
        AGGREGATE_ONE_SIDED_SOURCE,
        AGGREGATE_COMPOUND_SOURCE,
        AGGREGATE_NESTED_SOURCE,
        AGGREGATE_SEQUENCES_SOURCE,
        AGGREGATE_TEMPORARIES_SOURCE,
        AGGREGATE_CHECKED_SOURCE,
        AGGREGATE_CALLS_SOURCE,
        AGGREGATE_LOCAL_VALUES_SOURCE,
        AGGREGATE_LOOP_CALLS_SOURCE,
        BOOL_REBINDING_SOURCE,
        AGGREGATE_REBINDING_SOURCE,
        AGGREGATE_CARRIES_SOURCE,
        BOOL_CARRIES_SOURCE,
        LITERAL_LOOPS_SOURCE,
        VALUE_LOOP_EXITS_SOURCE,
        TRAILING_VALUE_LOOPS_SOURCE,
        COUNTED_RETURNS_SOURCE,
        CONTROL_COMPOSITION_SOURCE,
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
        AGGREGATE_ONE_SIDED_SOURCE,
        AGGREGATE_COMPOUND_SOURCE,
        AGGREGATE_NESTED_SOURCE,
        AGGREGATE_SEQUENCES_SOURCE,
        AGGREGATE_TEMPORARIES_SOURCE,
        AGGREGATE_CHECKED_SOURCE,
        AGGREGATE_CALLS_SOURCE,
        AGGREGATE_LOCAL_VALUES_SOURCE,
        AGGREGATE_LOOP_CALLS_SOURCE,
        BOOL_REBINDING_SOURCE,
        AGGREGATE_REBINDING_SOURCE,
        AGGREGATE_CARRIES_SOURCE,
        BOOL_CARRIES_SOURCE,
        LITERAL_LOOPS_SOURCE,
        VALUE_LOOP_EXITS_SOURCE,
        TRAILING_VALUE_LOOPS_SOURCE,
        COUNTED_RETURNS_SOURCE,
        CONTROL_COMPOSITION_SOURCE,
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
        AGGREGATE_ONE_SIDED_SOURCE,
        AGGREGATE_COMPOUND_SOURCE,
        AGGREGATE_NESTED_SOURCE,
        AGGREGATE_SEQUENCES_SOURCE,
        AGGREGATE_TEMPORARIES_SOURCE,
        AGGREGATE_CHECKED_SOURCE,
        AGGREGATE_CALLS_SOURCE,
        AGGREGATE_LOCAL_VALUES_SOURCE,
        AGGREGATE_LOOP_CALLS_SOURCE,
        BOOL_REBINDING_SOURCE,
        AGGREGATE_REBINDING_SOURCE,
        AGGREGATE_CARRIES_SOURCE,
        BOOL_CARRIES_SOURCE,
        LITERAL_LOOPS_SOURCE,
        VALUE_LOOP_EXITS_SOURCE,
        TRAILING_VALUE_LOOPS_SOURCE,
        COUNTED_RETURNS_SOURCE,
        CONTROL_COMPOSITION_SOURCE,
    ]
    .contains(&source)
    {
        assert!(llvm.contains("@nuis_fn_rebalance("));
    }
    if source == AGGREGATE_CARRIED_SOURCE {
        assert!(llvm.contains("loop_while_scalar_chain_body"));
    }
    if [
        AGGREGATE_CONDITIONAL_SOURCE,
        AGGREGATE_ONE_SIDED_SOURCE,
        AGGREGATE_COMPOUND_SOURCE,
    ]
    .contains(&source)
    {
        assert!(llvm.contains("loop_while_scalar_cond_chain_body"));
    }
    if source == AGGREGATE_COMPOUND_SOURCE {
        assert!(llvm.contains("carry_predicate_and_rhs"));
        assert!(llvm.contains("carry_predicate_or_rhs"));
        assert!(llvm.contains("phi i1"));
    }
    if [
        AGGREGATE_NESTED_SOURCE,
        AGGREGATE_SEQUENCES_SOURCE,
        AGGREGATE_TEMPORARIES_SOURCE,
        AGGREGATE_CHECKED_SOURCE,
        AGGREGATE_CALLS_SOURCE,
        AGGREGATE_LOCAL_VALUES_SOURCE,
        AGGREGATE_LOOP_CALLS_SOURCE,
        BOOL_REBINDING_SOURCE,
        AGGREGATE_REBINDING_SOURCE,
        AGGREGATE_CARRIES_SOURCE,
        BOOL_CARRIES_SOURCE,
        LITERAL_LOOPS_SOURCE,
        VALUE_LOOP_EXITS_SOURCE,
        TRAILING_VALUE_LOOPS_SOURCE,
        COUNTED_RETURNS_SOURCE,
        CONTROL_COMPOSITION_SOURCE,
    ]
    .contains(&source)
    {
        assert!(llvm.contains("@nuis_fn___nuis_scalar_iteration_"));
        assert!(llvm.contains("@nuis_fn___nuis_buffer_branch_"));
    }
    if [
        BOOL_REBINDING_SOURCE,
        AGGREGATE_REBINDING_SOURCE,
        AGGREGATE_CARRIES_SOURCE,
        BOOL_CARRIES_SOURCE,
        LITERAL_LOOPS_SOURCE,
        VALUE_LOOP_EXITS_SOURCE,
        TRAILING_VALUE_LOOPS_SOURCE,
        COUNTED_RETURNS_SOURCE,
        CONTROL_COMPOSITION_SOURCE,
    ]
    .contains(&source)
    {
        let yir = fs::read_to_string(output.join(format!("{stem}.yir"))).unwrap();
        assert!(yir.contains("cpu.cast_bool_to_i64"));
        assert!(yir.contains("cpu.cast_i64_to_bool"));
        assert!(llvm.contains("store i64 1048576, ptr %nuis_loop_work"));
        assert!(llvm.contains("store i64 1048576, ptr %nuis_helper_entries"));
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
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    assert!(relocated.join(format!("{stem}.ll")).is_file());
    if source == CONTROL_COMPOSITION_SOURCE {
        assert_eq!(
            fs::read_to_string(relocated.join(format!("{stem}.ll"))).unwrap(),
            llvm
        );
    }
    assert_eq!(
        states(&success(project.command(
            "run-artifact",
            &relocated,
            SCRIPT
        ))),
        expected
    );
}
