use super::*;

const SOURCE: &str =
    include_str!("../../../nuisc/tests/native_application_bridge/typed_sparse_captures.ns");

fn verify_runs(project: &Project, output: &Path) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    for (input, left, divisor, flag, right, selected) in [
        ("12,0,1,99", 12, 0, 1, 99, 12),
        ("12,3,0,99", 12, 3, 0, 99, 33),
        ("-15,-3,0,99", -15, -3, 0, 99, -33),
    ] {
        let run = success(project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                input,
                "--event-args",
                "",
                "--close-args",
                "",
            ],
        ));
        assert!(run.stdout.is_empty());
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
        let states = states(&run);
        assert_eq!(states.len(), 3);
        for (phase, state) in ["open", "event", "close"].iter().zip(&states) {
            let fields = (0..64)
                .map(|i| {
                    let value = match i {
                        0 if *phase == "open" => left,
                        0 => selected,
                        1 => divisor,
                        2 => flag,
                        63 => right,
                        _ => i,
                    };
                    format!("f{i}: {value}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            assert_eq!(
                state,
                &format!("{phase}:State{{payload: Payload{{{fields}}}}}")
            );
        }
        results.push(states);
    }
    results
}

#[test]
fn native_sparse_captures_build_cache_and_restore_without_sources() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    let selection = llvm
        .lines()
        .find(|line| line.starts_with("define i64 @nuis_fn___nuis_conditional_value"))
        .unwrap();
    assert_eq!(selection.matches("i64 %").count(), 3, "{selection}");
    assert_eq!(selection.matches("i1 %").count(), 1, "{selection}");
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let expected = verify_runs(&project, &output);
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    assert_eq!(verify_runs(&project, &output), expected);
    rejected_before_open(project.command(
        "run-artifact",
        &output,
        &["--native-session", "counter", "--open-args", "12,0,1"],
    ));
    let script = ["--native-session", "counter", "--open-args", "12,0,1,99"];
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
