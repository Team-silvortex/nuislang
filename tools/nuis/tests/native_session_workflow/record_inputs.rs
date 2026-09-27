use super::*;

#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/record_input_fixture.rs"]
mod fixture;

fn verify_runs(project: &Project, output: &Path) -> Vec<Vec<String>> {
    [-3_i64, 0, 5]
        .into_iter()
        .map(|seed| {
            let run = success(project.command(
                "run-artifact",
                output,
                &[
                    "--native-session",
                    "counter",
                    "--open-args",
                    &seed.to_string(),
                    "--event-args",
                    "",
                    "--event-args",
                    "",
                    "--close-args",
                    "",
                ],
            ));
            assert!(run.stdout.is_empty());
            assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
            let actual = states(&run);
            let expected = [("open", 0), ("event", 1), ("event", 2), ("close", 2)]
                .into_iter()
                .map(|(phase, steps)| {
                    let fields = (0..64)
                        .map(|i| {
                            format!(
                                "f{i}: {}",
                                seed + i + if i == 0 && seed > 0 { steps } else { 0 }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{phase}:State{{payload: Payload{{{fields}}}}}")
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);
            actual
        })
        .collect()
}

#[test]
fn native_record_inputs_build_cache_and_restore_without_sources() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(&fixture::source(true));
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    assert!(llvm.lines().any(|line| line.starts_with("define ")
        && line.contains(" @nuis_fn___nuis_")
        && line.split_once('(').unwrap().1.contains("[64 x i64] %arg")));
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let expected = verify_runs(&project, &output);
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    assert_eq!(verify_runs(&project, &output), expected);
    rejected_before_open(project.command(
        "run-artifact",
        &output,
        &["--native-session", "counter", "--open-args", ""],
    ));
    fs::write(output.join(&llvm_name), format!("{llvm}\n")).unwrap();
    rejected_before_open(project.command(
        "run-artifact",
        &output,
        &["--native-session", "counter", "--open-args", "5"],
    ));
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
