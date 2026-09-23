use super::*;

fn source() -> String {
    let fields = (1..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (1..64)
        .rev()
        .map(|i| format!("f{i}: value + {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let saved = (1..64)
        .rev()
        .map(|i| format!("f{i}: state.f{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "mod cpu Main {{
        struct State {{ active: bool, {fields} }}
        @noinline fn relay(state: State) -> State {{ return state; }}
        @noinline fn packet(state: State) -> State {{
            let next = state;
            if state.f1 > 0 {{ let next = relay(state); }}
            else {{ let next = State {{ active: !state.active, {saved} }}; }}
            return next;
        }}
        fn start(active: bool, value: i64) -> State {{
            return packet(State {{ active: active, {values} }});
        }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ print(999); return 0; }}
    }}"
    )
}

fn verify_runs(project: &Project, output: &Path) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    for (input, active, value) in [
        ("true,1", true, 1),
        ("false,1", false, 1),
        ("true,-2", false, -2),
        ("false,-2", true, -2),
    ] {
        let script = [
            "--native-session",
            "counter",
            "--open-args",
            input,
            "--event-args",
            "",
            "--close-args",
            "",
        ];
        let run = success(project.command("run-artifact", output, &script));
        assert!(run.stdout.is_empty());
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
        let states = states(&run);
        assert_eq!(states.len(), 3);
        let fields = (1..64)
            .map(|index| format!("f{index}: {}", value + index))
            .collect::<Vec<_>>()
            .join(", ");
        let expected = format!("State{{active: {active}, {fields}}}");
        for (phase, state) in ["open", "event", "close"].iter().zip(&states) {
            assert_eq!(state, &format!("{phase}:{expected}"));
        }
        results.push(states);
    }
    results
}

#[test]
fn native_64_slot_captures_build_cache_and_restore_without_sources() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(&source());
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    let selection = llvm
        .lines()
        .find(|line| line.starts_with("define [64 x i64] @nuis_fn___nuis_conditional_value"))
        .unwrap();
    // The native helper-entry budget is an extra private pointer, not a data slot.
    assert_eq!(selection.matches("i64 %").count(), 64, "{selection}");
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let expected = verify_runs(&project, &output);
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    assert_eq!(verify_runs(&project, &output), expected);
    let invalid = ["--native-session", "counter", "--open-args", "2,1"];
    rejected_before_open(project.command("run-artifact", &output, &invalid));
    let script = ["--native-session", "counter", "--open-args", "true,1"];
    fs::write(output.join(&llvm_name), format!("{llvm}\n")).unwrap();
    rejected_before_open(project.command("run-artifact", &output, &script));
    fs::write(output.join(&llvm_name), &llvm).unwrap();

    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(&output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[restored.to_str().unwrap()],
    ));
    assert_eq!(fs::read_to_string(restored.join(&llvm_name)).unwrap(), llvm);
    assert_eq!(verify_runs(&project, &restored), expected);
}
