use super::*;

const SOURCE: &str =
    include_str!("../../../nuisc/tests/native_application_bridge/typed_sparse_captures.ns");
#[path = "../../../nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs"]
mod aliases;

fn verify_runs(project: &Project, output: &Path, offset: i64) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    for (input, left, divisor, flag, right, selected) in [
        ("12,0,1,99", 12, 0, 1, 99, 12),
        ("-12,0,1,99", -12, 0, 1, 99, -12),
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
                        0 => selected + offset,
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

fn verify_selected_failures(project: &Project, output: &Path) {
    for input in ["12,0,0,99", "12,-1,0,-9223372036854775808"] {
        let run = project.command(
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
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(!run.status.success(), "{input}: {stderr}");
        assert!(
            stderr.contains("native session process failed:"),
            "{input}: {stderr}"
        );
        assert!(
            stderr.contains("no fallback was attempted"),
            "{input}: {stderr}"
        );
        assert!(
            !stderr.contains("native_session_completed=1"),
            "{input}: {stderr}"
        );
        assert!(
            states(&run).iter().all(|state| state.starts_with("open:")),
            "{input}: {stderr}"
        );
        assert!(run.stdout.is_empty());
    }
}

#[test]
fn native_sparse_captures_build_cache_and_restore_without_sources() {
    check_sparse_workflow(SOURCE, None, 0);
}

#[test]
fn native_sparse_captures_aliases_build_cache_and_restore_without_sources() {
    check_sparse_workflow(&aliases::source(), Some(&[1, 2]), 0);
}

#[test]
fn native_sparse_captures_snapshots_build_cache_and_restore_without_sources() {
    check_sparse_workflow(&aliases::rebound_source(), Some(&[1, 2]), 30);
}

#[test]
fn native_sparse_captures_scoped_aliases_build_cache_and_restore_without_sources() {
    check_sparse_workflow(&aliases::scoped_source(), Some(&[1, 1, 1, 2]), 0);
}

#[test]
fn native_sparse_captures_scoped_snapshots_build_cache_and_restore_without_sources() {
    check_sparse_workflow(&aliases::scoped_rebound_source(), Some(&[1, 1, 1, 2]), 30);
}

#[test]
fn native_sparse_captures_loop_aliases_build_cache_and_restore_without_sources() {
    check_sparse_workflow(&aliases::loop_source(), Some(&[1, 2]), 0);
}

#[test]
fn native_sparse_captures_wide_iteration_inputs_build_cache_and_restore_without_sources() {
    check_sparse_workflow(&aliases::wide_loop_source(), Some(&[1, 2]), 0);
}

#[test]
fn native_sparse_captures_terminal_snapshots_build_cache_and_restore_without_sources() {
    check_sparse_workflow(&aliases::terminal_snapshot_source(), Some(&[1, 2]), 30);
}

fn check_sparse_workflow(source: &str, branch_sizes: Option<&[usize]>, offset: i64) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(source);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    if let Some(branch_sizes) = branch_sizes {
        let mut sizes = llvm
            .lines()
            .filter(|line| line.starts_with("define i64 @nuis_fn___nuis_scalar_branch"))
            .map(|line| {
                assert_eq!(line.matches("i1 %").count(), 1, "{line}");
                line.matches("i64 %").count()
            })
            .collect::<Vec<_>>();
        sizes.sort();
        assert_eq!(sizes, branch_sizes);
    } else {
        let selection = llvm
            .lines()
            .find(|line| line.starts_with("define i64 @nuis_fn___nuis_conditional_value"))
            .unwrap();
        assert_eq!(selection.matches("i64 %").count(), 3, "{selection}");
        assert_eq!(selection.matches("i1 %").count(), 1, "{selection}");
    }
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let expected = verify_runs(&project, &output, offset);
    verify_selected_failures(&project, &output);
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    assert_eq!(verify_runs(&project, &output, offset), expected);
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
    assert_eq!(verify_runs(&project, &restored, offset), expected);
    verify_selected_failures(&project, &restored);
}
