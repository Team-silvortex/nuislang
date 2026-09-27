use super::*;

#[path = "../../../nuisc/tests/native_application_bridge/scoped_record_fixture.rs"]
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
            let mut words = (0..64).map(|i| seed + i).collect::<Vec<_>>();
            let expected = ["open", "event", "event", "close"]
                .into_iter()
                .map(|phase| {
                    if phase == "event" {
                        let trips = words[1].max(0);
                        for word in &mut words {
                            *word += trips;
                        }
                        words[0] += trips * (trips + 1) / 2;
                    }
                    let fields = words
                        .iter()
                        .enumerate()
                        .map(|(i, value)| format!("f{i}: {value}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{phase}:State{{{fields}}}")
                })
                .collect::<Vec<_>>();
            assert_eq!(states(&run), expected);
            expected
        })
        .collect()
}

#[test]
fn native_scoped_record_inputs_cache_and_restore_the_full_mapping_without_sources() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(&fixture::source(64, false));
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    assert!(llvm.lines().any(|line| line.starts_with("define ")
        && line.contains("@nuis_fn___nuis_scalar_iteration_")
        && line.split_once('(').unwrap().1.contains("[64 x i64] %arg")));
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    let expected = verify_runs(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
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
    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(output).unwrap();
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
