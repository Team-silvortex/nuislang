#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
use nuisc::aot::native_session::literal_print_packaging_mode;

#[path = "effect_call_regions.rs"]
mod regions;

#[path = "effect_call_staged.rs"]
mod staged;

#[path = "effect_call_continued.rs"]
mod continued;

#[path = "effect_call_repeated.rs"]
mod repeated;

const SOURCE: &str =
    include_str!("../../../nuisc/tests/native_application_bridge/effectful_selected_calls.ns");
const REBIND_SOURCE: &str =
    include_str!("../../../nuisc/tests/native_application_bridge/effectful_scalar_rebindings.ns");
const NESTED_SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/nested_effectful_scalar_selections.ns"
);

fn project() -> (Project, String) {
    project_for_source(SOURCE)
}

fn project_for_source(source: &str) -> (Project, String) {
    let project = Project::new(source);
    let manifest = project.0.join("nuis.toml");
    let text = fs::read_to_string(&manifest).unwrap().replace(
        "packaging_mode = \"native-session-aot-bundle:counter\"\n",
        "",
    );
    fs::write(&manifest, &text).unwrap();
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    // Explicit test grants do not change production source discovery or policy.
    let sites = module
        .nodes
        .iter()
        .filter(|node| matches!(node.op.instruction.as_str(), "print" | "guard_print"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let mode = literal_print_packaging_mode("counter", sites, 0, 100).unwrap();
    fs::write(manifest, format!("{text}packaging_mode = \"{mode}\"\n")).unwrap();
    (project, mode)
}

fn check_run(project: &Project, output: &Path) {
    for (gate, left, right) in [(true, 2, 0), (false, 0, 2), (true, 2, 2), (false, 2, 2)] {
        let open = format!("10,{gate},{left},{right}");
        let next = !gate;
        let (next_left, next_right) = if next { (2, 0) } else { (0, 2) };
        let event = format!("{next},{next_left},{next_right}");
        let again = format!("{gate},{left},{right}");
        let run = success(project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &open,
                "--event-args",
                &event,
                "--event-args",
                &again,
                "--close-args",
                "",
            ],
        ));
        let prints = |gate| format!("99\n98\n61\n62\n{}\n80\n", if gate { 70 } else { 71 });
        assert_eq!(
            String::from_utf8(run.stdout.clone()).unwrap(),
            format!("{}{}{}", prints(gate), prints(next), prints(gate))
        );
        let state = |phase, gate, left, right| {
            format!(
            "{phase}:State{{seed: 10, gate: {gate}, left_divisor: {left}, right_divisor: {right}, result: {}}}",
            if gate { 20 } else { 0 })
        };
        assert_eq!(
            states(&run),
            [
                state("open", gate, left, right),
                state("event", next, next_left, next_right),
                state("event", gate, left, right),
                state("close", gate, left, right)
            ]
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    }
}

#[test]
fn native_effectful_selected_calls_build_cache_restore_and_execute_only_selected_arguments() {
    let (project, mode) = project();
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_run(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_run(&project, &output);

    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    fs::remove_dir_all(&output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    fs::remove_dir_all(project.0.join(".nuis")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    for _ in 0..3 {
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        let report =
            nuisc::aot::verify_build_manifest(&restored.join("nuis.build.manifest.toml")).unwrap();
        assert_eq!(report.packaging_mode, mode);
        assert_eq!(
            fs::read(restored.join(&report.artifact_binary_name)).unwrap(),
            binary
        );
        check_run(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_effectful_selected_calls_trap_on_selected_division_before_suffix_or_publication() {
    let (project, _) = project();
    project.build(None);
    let output = project.0.join("build");
    for open in ["10,true,0,2", "10,false,2,0"] {
        let run = project.command(
            "run-artifact",
            &output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                open,
                "--close-args",
                "",
            ],
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        // Host stdio may buffer earlier irreversible prints on a fatal trap.
        assert!(
            !stdout.contains("62\n")
                && !stdout.contains("70\n")
                && !stdout.contains("71\n")
                && !stdout.contains("80\n"),
            "{stdout}"
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        rejected_before_open(run);
    }
}

fn check_rebindings(project: &Project, output: &Path) {
    for (gate, second_gate) in [(true, true), (false, false), (true, false), (false, true)] {
        let arguments = |gate, second_gate| {
            format!(
                "{gate},{second_gate},{},{}",
                if gate { 2 } else { 0 },
                if second_gate { 0 } else { 3 }
            )
        };
        let initial = arguments(gate, second_gate);
        let open = format!("10,{initial}");
        let next = arguments(!gate, !second_gate);
        let run = success(project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &open,
                "--event-args",
                &next,
                "--event-args",
                &initial,
                "--close-args",
                "",
            ],
        ));
        let prints = |gate, second_gate| {
            format!(
                "99\n98\n{}98\n{}80\n",
                if gate { "61\n62\n70\n" } else { "" },
                if second_gate { "" } else { "61\n62\n71\n" }
            )
        };
        assert_eq!(
            String::from_utf8(run.stdout.clone()).unwrap(),
            format!(
                "{}{}{}",
                prints(gate, second_gate),
                prints(!gate, !second_gate),
                prints(gate, second_gate)
            )
        );
        let state = |phase: &str, gate: bool, second_gate: bool| {
            let mut value = 11;
            if gate {
                value += (value / 2) * 2;
            }
            if !second_gate {
                value -= (value / 3) * 2;
            }
            format!("{phase}:State{{seed: 10, gate: {gate}, second_gate: {second_gate}, left_divisor: {}, right_divisor: {}, result: {value}}}",
                if gate { 2 } else { 0 }, if second_gate { 0 } else { 3 })
        };
        assert_eq!(
            states(&run),
            [
                state("open", gate, second_gate),
                state("event", !gate, !second_gate),
                state("event", gate, second_gate),
                state("close", gate, second_gate)
            ]
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    }
}

#[test]
fn native_effectful_scalar_rebindings_build_cache_restore_and_preserve_current_values() {
    let (project, mode) = project_for_source(REBIND_SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_rebindings(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_rebindings(&project, &output);
    let artifact = project.0.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    fs::remove_dir_all(&output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    fs::remove_dir_all(project.0.join(".nuis")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    for _ in 0..3 {
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        let report =
            nuisc::aot::verify_build_manifest(&restored.join("nuis.build.manifest.toml")).unwrap();
        assert_eq!(report.packaging_mode, mode);
        assert_eq!(
            fs::read(restored.join(&report.artifact_binary_name)).unwrap(),
            binary
        );
        check_rebindings(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_effectful_scalar_rebindings_trap_only_in_the_selected_update_before_publication() {
    let (project, _) = project_for_source(REBIND_SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for open in ["10,true,true,0,0", "10,false,false,0,0"] {
        let run = project.command(
            "run-artifact",
            &output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                open,
                "--close-args",
                "",
            ],
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            !stdout.contains("62\n")
                && !stdout.contains("70\n")
                && !stdout.contains("71\n")
                && !stdout.contains("80\n"),
            "{stdout}"
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        rejected_before_open(run);
    }
}

fn nested_arguments(outer: bool, inner: bool, leaf: bool) -> String {
    let selected_yes = outer && inner && leaf;
    let selected_no = (outer && inner && !leaf) || (!outer && !inner);
    format!(
        "{outer},{inner},{leaf},2,{},{},{},{},{}",
        if outer { 2 } else { 0 },
        if outer { 0 } else { 2 },
        if outer && inner { 2 } else { 0 },
        if selected_yes { 2 } else { 0 },
        if selected_no { 3 } else { 0 }
    )
}

fn check_nested_selections(project: &Project, output: &Path) {
    for outer in [false, true] {
        for inner in [false, true] {
            for leaf in [false, true] {
                let initial = nested_arguments(outer, inner, leaf);
                let open = format!("10,{initial}");
                let next = nested_arguments(!outer, !inner, !leaf);
                let run = success(project.command(
                    "run-artifact",
                    output,
                    &[
                        "--native-session",
                        "counter",
                        "--open-args",
                        &open,
                        "--event-args",
                        &next,
                        "--event-args",
                        &initial,
                        "--close-args",
                        "",
                    ],
                ));
                let prints = |outer: bool, inner: bool, leaf: bool| {
                    let mut lines = vec![99, 97, 91, if outer { 98 } else { 95 }];
                    if outer && inner {
                        lines.push(96);
                    }
                    let yes = outer && inner && leaf;
                    let no = (outer && inner && !leaf) || (!outer && !inner);
                    if yes || no {
                        lines.extend([61, 62, if yes { 70 } else { 71 }]);
                    }
                    lines.push(80);
                    lines.iter().map(|v| format!("{v}\n")).collect::<String>()
                };
                assert_eq!(
                    String::from_utf8(run.stdout.clone()).unwrap(),
                    format!(
                        "{}{}{}",
                        prints(outer, inner, leaf),
                        prints(!outer, !inner, !leaf),
                        prints(outer, inner, leaf)
                    )
                );
                let state = |phase: &str, outer: bool, inner: bool, leaf: bool| {
                    let yes = outer && inner && leaf;
                    let no = (outer && inner && !leaf) || (!outer && !inner);
                    let result = if yes {
                        21
                    } else if no {
                        5
                    } else {
                        11
                    };
                    format!("{phase}:State{{seed: 10, outer: {outer}, inner: {inner}, leaf: {leaf}, outer_divisor: 2, then_divisor: {}, else_divisor: {}, leaf_divisor: {}, left_divisor: {}, right_divisor: {}, result: {result}}}",
                        if outer { 2 } else { 0 }, if outer { 0 } else { 2 },
                        if outer && inner { 2 } else { 0 }, if yes { 2 } else { 0 }, if no { 3 } else { 0 })
                };
                assert_eq!(
                    states(&run),
                    [
                        state("open", outer, inner, leaf),
                        state("event", !outer, !inner, !leaf),
                        state("event", outer, inner, leaf),
                        state("close", outer, inner, leaf)
                    ]
                );
                assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
            }
        }
    }
}

#[test]
fn native_nested_effectful_scalar_selections_build_cache_restore_and_mask_inactive_conditions() {
    let (project, mode) = project_for_source(NESTED_SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_nested_selections(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_nested_selections(&project, &output);
    let artifact = project.0.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    fs::remove_dir_all(&output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    fs::remove_dir_all(project.0.join(".nuis")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    for _ in 0..3 {
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        let report =
            nuisc::aot::verify_build_manifest(&restored.join("nuis.build.manifest.toml")).unwrap();
        assert_eq!(report.packaging_mode, mode);
        assert_eq!(
            fs::read(restored.join(&report.artifact_binary_name)).unwrap(),
            binary
        );
        check_nested_selections(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_nested_effectful_scalar_selections_trap_on_selected_predicates_or_arguments_before_publication(
) {
    let (project, _) = project_for_source(NESTED_SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for open in [
        "10,true,true,true,0,2,0,2,2,0",
        "10,true,true,true,2,0,0,2,2,0",
        "10,false,false,true,2,0,0,0,0,3",
        "10,true,true,true,2,2,0,0,2,0",
        "10,true,true,true,2,2,0,2,0,0",
        "10,true,true,false,2,2,0,2,0,0",
    ] {
        let run = project.command(
            "run-artifact",
            &output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                open,
                "--close-args",
                "",
            ],
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            !stdout.contains("62\n")
                && !stdout.contains("70\n")
                && !stdout.contains("71\n")
                && !stdout.contains("80\n"),
            "{stdout}"
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        rejected_before_open(run);
    }
}
