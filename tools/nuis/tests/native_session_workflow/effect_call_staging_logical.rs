use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/staging_logical_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 14] = [
    "enabled",
    "outer",
    "a",
    "b",
    "c",
    "da",
    "db",
    "dc",
    "dd",
    "de",
    "df",
    "first_leaf",
    "second_leaf",
    "tail_divisor",
];

fn selected(outer: bool, a: bool, b: bool, c: bool) -> (bool, bool) {
    if outer {
        ((a && b) || c, !(a || (b && c)))
    } else {
        (!((a || b) && c), a && (b || c))
    }
}

fn values(enabled: bool, outer: bool, a: bool, b: bool, c: bool) -> Vec<String> {
    let mut values = [enabled, outer, a, b, c].map(|v| v.to_string()).to_vec();
    let (first, second) = selected(outer, a, b, c);
    for (active, divisor) in [
        (enabled, 1),
        (enabled && if outer { a } else { !a }, 1),
        (enabled && if outer { !(a && b) } else { a || b }, 1),
        (enabled, 1),
        (enabled && if outer { !a } else { a }, 1),
        (enabled && if outer { !a && b } else { a && !b }, 1),
        (enabled && first, 2),
        (enabled && second, 2),
        (enabled && first, 1),
    ] {
        values.push(if active { divisor } else { 0 }.to_string());
    }
    values
}

fn result(enabled: bool, outer: bool, a: bool, b: bool, c: bool) -> i64 {
    let (first, second) = selected(outer, a, b, c);
    let mut saved = 11;
    if enabled && first {
        saved += saved / 2;
    }
    if enabled && second {
        saved += saved / 2;
    }
    saved
}

fn prints(enabled: bool, outer: bool, a: bool, b: bool, c: bool) -> String {
    let mut trace = vec![99, 89];
    if enabled {
        trace.extend([90, 91]);
        if if outer { a } else { !a } {
            trace.push(92);
        }
        if if outer { !(a && b) } else { a || b } {
            trace.push(93);
        }
        let (first, second) = selected(outer, a, b, c);
        if first {
            trace.push(70);
        }
        trace.push(94);
        if if outer { !a } else { a } {
            trace.push(95);
        }
        if if outer { !a && b } else { a && !b } {
            trace.push(96);
        }
        if second {
            trace.push(71);
        }
        if first {
            trace.push(97);
        }
    }
    trace.push(80);
    trace.iter().map(|v| format!("{v}\n")).collect()
}

fn check_tree(project: &Project, output: &Path) {
    for enabled in [false, true] {
        for outer in [false, true] {
            for a in [false, true] {
                for b in [false, true] {
                    for c in [false, true] {
                        let first = values(enabled, outer, a, b, c).join(",");
                        let second = values(!enabled, !outer, !a, !b, !c).join(",");
                        let run = success(project.command(
                            "run-artifact",
                            output,
                            &[
                                "--native-session",
                                "counter",
                                "--open-args",
                                &format!("10,{first}"),
                                "--event-args",
                                &second,
                                "--close-args",
                                "",
                            ],
                        ));
                        assert_eq!(
                            String::from_utf8(run.stdout.clone()).unwrap(),
                            format!(
                                "{}{}",
                                prints(enabled, outer, a, b, c),
                                prints(!enabled, !outer, !a, !b, !c)
                            )
                        );
                        let state = |phase, enabled, outer, a, b, c| {
                            let mut fields = vec!["seed: 10".to_owned()];
                            fields.extend(
                                FIELDS
                                    .into_iter()
                                    .zip(values(enabled, outer, a, b, c))
                                    .map(|(name, value)| format!("{name}: {value}")),
                            );
                            fields.push(format!("result: {}", result(enabled, outer, a, b, c)));
                            format!("{phase}:State{{{}}}", fields.join(", "))
                        };
                        assert_eq!(
                            states(&run),
                            [
                                state("open", enabled, outer, a, b, c),
                                state("event", !enabled, !outer, !a, !b, !c),
                                state("close", !enabled, !outer, !a, !b, !c)
                            ]
                        );
                        assert!(String::from_utf8_lossy(&run.stderr)
                            .contains("native_session_completed=1"));
                    }
                }
            }
        }
    }
}

#[test]
fn native_staging_logical_effectful_scalar_selections_build_cache_restore_and_keep_initializers_at_original_stages(
) {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_tree(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_tree(&project, &output);
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
        check_tree(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

fn trap(project: &Project, output: &Path, arguments: &[String], stage: usize) {
    let run = project.command(
        "run-artifact",
        output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            &format!("10,{}", arguments.join(",")),
            "--close-args",
            "",
        ],
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    // Fatal signals may discard earlier buffered output, but cannot publish
    // later effects or state beyond the selected original check.
    for print in &[91, 92, 93, 70, 94, 95, 96, 71, 97, 80][stage..] {
        assert!(!stdout.contains(&format!("{print}\n")), "{stdout}");
    }
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
        "{stderr}"
    );
    rejected_before_open(run);
}

#[test]
fn native_staging_logical_effectful_scalar_selections_trap_at_selected_initializers_children_and_suffixes_before_publication(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for outer in [false, true] {
        for (stage, index, a, b, c) in [
            (0, 5, false, false, false),
            (1, 6, outer, false, false),
            (2, 7, false, !outer, true),
            (3, 11, outer, outer, outer),
            (4, 8, false, false, false),
            (5, 9, !outer, false, false),
            (6, 10, !outer, outer, true),
            (7, 12, !outer, !outer, !outer),
            (8, 13, outer, outer, outer),
        ] {
            let mut args = values(true, outer, a, b, c);
            args[index] = "0".into();
            trap(&project, &output, &args, stage);
        }
        // The nested left must finish even if its result would skip a later RHS.
        let mut args = values(true, outer, outer, true, false);
        args[6] = "0".into();
        trap(&project, &output, &args, 1);
    }
}
