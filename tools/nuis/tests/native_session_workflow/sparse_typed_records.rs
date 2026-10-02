use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/control_flow_syntax_native/scoped_sparse_typed_record_carries.ns"
);

fn verify(project: &Project, output: &Path, branches: bool, width: usize) -> Vec<Vec<String>> {
    [0, 3].into_iter().map(|limit| {
        let run = success(project.command("run-artifact", output, &[
            "--native-session", "counter", "--open-args", &format!("1.5,2.5,-17,{limit}"),
            "--event-args", "", "--event-args", "", "--close-args", "",
        ]));
        assert!(run.stdout.is_empty());
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
        let mut flag = false;
        let mut value = 1.5;
        let mut gain = 2.5;
        let expected = ["open", "event", "event", "close"].into_iter().map(|phase| {
            if phase == "event" && limit > 0 {
                flag = !flag;
                if branches { value += 2.0; gain += 1.0; }
            }
            let right = if phase == "open" || limit == 0 {
                "value: 1.5f64, gain: 2.5f32, tag: 7i32, enabled: true"
            } else {
                "value: 3.5f64, gain: 4.5f32, tag: 9i32, enabled: false"
            };
            let extra = (9..width).map(|i| format!(", extra{i}: {}",
                if phase == "open" || limit == 0 { 27 } else { 99 }
            )).collect::<String>();
            format!("{phase}:State{{left: Leaf{{value: {value}f64, gain: {gain}f32, tag: -17i32, enabled: {flag}}}, right: Leaf{{{right}}}, count: {limit}{extra}}}")
        }).collect::<Vec<_>>();
        assert_eq!(states(&run), expected);
        expected
    }).collect()
}

#[test]
fn native_sparse_typed_record_inputs_restore_complete_seeds_without_sources() {
    check(SOURCE, false);
}

#[test]
fn native_sparse_branch_snapshots_restore_current_versions_without_sources() {
    check(
        include_str!(
            "../../../nuisc/tests/control_flow_syntax_native/scoped_branch_typed_record_carries.ns"
        ),
        true,
    );
}

#[test]
fn native_materialized_join_snapshots_restore_current_versions_without_sources() {
    check(
        include_str!(
            "../../../nuisc/tests/control_flow_syntax_native/scoped_join_typed_record_carries.ns"
        ),
        true,
    );
}

#[test]
fn native_materialized_loop_snapshots_restore_nested_versions_without_sources() {
    check(
        include_str!(
            "../../../nuisc/tests/control_flow_syntax_native/scoped_loop_typed_record_carries.ns"
        ),
        true,
    );
}

#[test]
fn native_materialized_nested_returns_restore_complete_state_without_sources() {
    check_captures(
        include_str!(
            "../../../nuisc/tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
        ),
        true,
        &[12, 16],
        9,
    );
}

#[test]
fn native_full_width_nested_returns_restore_complete_state_without_sources() {
    let source = include_str!(
        "../../../nuisc/tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    );
    let extra = |value: &str| {
        (9..64)
            .map(|i| format!(", extra{i}: {value}"))
            .collect::<String>()
    };
    let source = source
        .replace("count: i64", &format!("count: i64{}", extra("i64")))
        .replacen("count: limit", &format!("count: limit{}", extra("27")), 1)
        .replace("count: limit\n", &format!("count: limit{}\n", extra("99")));
    check_captures(&source, true, &[5, 11], 64);
}

fn check(source: &str, branches: bool) {
    check_captures(source, branches, &[6], 9);
}

fn check_captures(source: &str, branches: bool, captures: &[usize], width: usize) {
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
    let mut iterations = llvm
        .lines()
        .filter(|line| {
            line.starts_with("define ") && line.contains("@nuis_fn___nuis_scalar_iteration_")
        })
        .map(|line| line.matches(" %arg").count())
        .collect::<Vec<_>>();
    if let [expected] = captures {
        assert_eq!(iterations.first(), Some(expected));
    } else {
        iterations.sort_unstable();
        assert_eq!(iterations, captures);
    }
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    let expected = verify(&project, &output, branches, width);
    assert!(project.build(None).contains("compile_cache: hit"));
    assert_eq!(verify(&project, &output, branches, width), expected);
    fs::write(output.join(&llvm_name), format!("{llvm}\n")).unwrap();
    rejected_before_open(project.command(
        "run-artifact",
        &output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            "1.5,2.5,-17,3",
        ],
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
    assert_eq!(verify(&project, &restored, branches, width), expected);
}
