use super::*;

#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/sparse_record_fixture.rs"]
mod fixture;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/control_flow_syntax_native/scoped_sparse_typed_record_carries.ns"
);

fn verify(
    project: &Project,
    output: &Path,
    branches: bool,
    width: usize,
    limits: &[u64],
    observed_child: Option<bool>,
    changed_outer_tag: bool,
) -> Vec<Vec<String>> {
    limits.iter().copied().map(|limit| {
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
                "value: 1.5f64, gain: 2.5f32, tag: 7i32, enabled: true".to_owned()
            } else {
                let index = observed_child.map_or(0, |continuing| {
                    if continuing { limit % 3 } else { (limit % 3).min(1) }
                });
                format!("value: 3.5f64, gain: 4.5f32, tag: {}i32, enabled: false", 9 + index)
            };
            let extra = (9..width).map(|i| format!(", extra{i}: {}",
                if phase == "open" || limit == 0 { 27 } else { 99 }
            )).collect::<String>();
            let tag = if changed_outer_tag && phase != "open" && limit > 0 { 17 } else { -17 };
            format!("{phase}:State{{left: Leaf{{value: {value}f64, gain: {gain}f32, tag: {tag}i32, enabled: {flag}}}, right: Leaf{{{right}}}, count: {limit}{extra}}}")
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
        &[9, 11],
        9,
    );
}

#[test]
fn native_full_width_nested_returns_restore_complete_state_without_sources() {
    let source = full_width_source(include_str!(
        "../../../nuisc/tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    ));
    check_captures(&source, true, &[5, 11], 64);
}

#[test]
fn native_changed_outer_tags_restore_preheader_snapshots_without_source() {
    let source = full_width_source(include_str!(
        "../../../nuisc/tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    ))
    .replace(
        "tag: selected.tag, enabled: !enabled",
        "tag: i32_from_i64(17), enabled: !enabled",
    );
    check_captures_with_tags(&source, true, &[4, 11], 64, &[0, 3], None, true);
}

#[test]
fn native_joined_preheaders_restore_full_width_snapshots_without_source() {
    check_captures_with_tags(
        &fixture::joined_return_source(64),
        true,
        &[4, 11],
        64,
        &[0, 3, 4],
        None,
        true,
    );
}

#[test]
fn native_parent_entries_restore_full_width_invariants_without_source() {
    check_captures_at_limits(
        &fixture::parent_return_source(64),
        true,
        &[6, 12],
        64,
        &[0, 3, 4],
        None,
    );
}

#[test]
fn native_post_loop_snapshots_restore_full_width_invariants_without_source() {
    for (exit, captures) in [
        ("", [5, 6, 12]),
        ("break;", [6, 6, 12]),
        ("continue;", [5, 6, 12]),
    ] {
        check_captures_at_limits(
            &fixture::post_loop_return_source(64, exit),
            true,
            &captures,
            64,
            &[0, 3, 4],
            None,
        );
    }
}

#[test]
fn native_nested_child_exits_restore_full_width_state_without_sources() {
    check_captures_at_limits(&child_source(), true, &[2, 5, 11], 64, &[0, 2, 3, 4], None);
}

#[test]
fn native_observed_child_exit_indices_restore_full_width_state_without_sources() {
    for continuing in [false, true] {
        let mut source = child_source().replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
        if continuing {
            source = source.replace("if k == 1 { break; }", "if k == 1 { continue; }");
        }
        let captures = if continuing { [1, 5, 11] } else { [3, 5, 11] };
        check_captures_at_limits(
            &source,
            true,
            &captures,
            64,
            &[0, 2, 3, 4],
            Some(continuing),
        );
    }
}

#[test]
fn native_partial_child_snapshots_restore_full_width_state_without_sources() {
    for continuing in [false, true] {
        let mut source = child_source()
            .replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)")
            .replace(
                "if k == 1 { break; }",
                "let snapshot = carry; let checked_snapshot = 10 / snapshot.count; if k == 1 { break; }",
            );
        if continuing {
            source = source.replace("if k == 1 { break; }", "if k == 1 { continue; }");
        }
        let captures = if continuing { [2, 6, 11] } else { [4, 6, 11] };
        check_captures_at_limits(
            &source,
            true,
            &captures,
            64,
            &[0, 2, 3, 4],
            Some(continuing),
        );
    }
}

#[test]
fn native_opaque_child_snapshots_restore_readonly_record_inputs_without_source() {
    let source = child_source()
        .replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)")
        .replace("if k == 1 { break; }", "let snapshot = relay(carry); let checked_snapshot = 10 / snapshot.count; if k == 1 { break; }");
    check_captures_at_limits(&source, true, &[6, 6, 11], 64, &[0, 2, 3, 4], Some(false));
}

#[test]
fn native_opaque_continue_snapshots_restore_readonly_record_inputs_without_source() {
    let source = child_source()
        .replace("if k == 1 { break; }", "let snapshot = relay(carry); let observed = snapshot.right.tag; if k == 1 { continue; }")
        .replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
    check_captures_at_limits(&source, true, &[4, 6, 11], 64, &[0, 2, 3, 4], Some(true));
}

#[test]
fn native_checked_child_snapshots_restore_computed_records_without_source() {
    let extra = (9..64)
        .map(|i| format!(", extra{i}: carry.extra{i}"))
        .collect::<String>();
    for continuing in [false, true] {
        let exit = if continuing { "continue" } else { "break" };
        let source = child_source()
            .replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)")
            .replace("if k == 1 { break; }", &format!(
                "let snapshot = State {{ left: carry.left, right: carry.right, count: carry.count + 1{extra} }}; let checked_snapshot = 10 / (snapshot.count - carry.count); if k == 1 {{ {exit}; }}"
            ));
        let captures = if continuing { [2, 6, 11] } else { [4, 6, 11] };
        check_captures_at_limits(
            &source,
            true,
            &captures,
            64,
            &[0, 2, 3, 4],
            Some(continuing),
        );
    }
}

fn child_source() -> String {
    full_width_source(include_str!(
        "../../../nuisc/tests/control_flow_syntax_native/scoped_return_child_exits.ns"
    ))
}

fn full_width_source(source: &str) -> String {
    let extra = |value: &str| {
        (9..64)
            .map(|i| format!(", extra{i}: {value}"))
            .collect::<String>()
    };
    source
        .replace("count: i64", &format!("count: i64{}", extra("i64")))
        .replacen("count: limit", &format!("count: limit{}", extra("27")), 1)
        .replace("count: limit\n", &format!("count: limit{}\n", extra("99")))
}

fn check(source: &str, branches: bool) {
    check_captures(source, branches, &[6], 9);
}

fn check_captures(source: &str, branches: bool, captures: &[usize], width: usize) {
    check_captures_at_limits(source, branches, captures, width, &[0, 3], None);
}

fn check_captures_at_limits(
    source: &str,
    branches: bool,
    captures: &[usize],
    width: usize,
    limits: &[u64],
    observed_child: Option<bool>,
) {
    check_captures_with_tags(
        source,
        branches,
        captures,
        width,
        limits,
        observed_child,
        false,
    );
}

#[allow(clippy::too_many_arguments)]
fn check_captures_with_tags(
    source: &str,
    branches: bool,
    captures: &[usize],
    width: usize,
    limits: &[u64],
    observed_child: Option<bool>,
    changed_outer_tag: bool,
) {
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
    let expected = verify(
        &project,
        &output,
        branches,
        width,
        limits,
        observed_child,
        changed_outer_tag,
    );
    assert!(project.build(None).contains("compile_cache: hit"));
    assert_eq!(
        verify(
            &project,
            &output,
            branches,
            width,
            limits,
            observed_child,
            changed_outer_tag
        ),
        expected
    );
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
    assert_eq!(
        verify(
            &project,
            &restored,
            branches,
            width,
            limits,
            observed_child,
            changed_outer_tag
        ),
        expected
    );
}
