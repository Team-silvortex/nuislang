use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/computed_logical_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 11] = [
    "enabled",
    "outer",
    "first",
    "second",
    "reply",
    "first_left",
    "second_left",
    "first_rhs",
    "second_rhs",
    "first_leaf",
    "second_leaf",
];

fn values(enabled: bool, outer: bool, first: bool, second: bool, reply: bool) -> Vec<String> {
    let mut values = [enabled, outer, first, second, reply]
        .map(|v| v.to_string())
        .to_vec();
    for (selected, divisor) in [
        (enabled, 1),
        (enabled, 1),
        (enabled && first == outer, 1),
        (enabled && second != outer, 1),
        (enabled && first == outer && reply, 2),
        (enabled && second != outer && reply, 2),
    ] {
        values.push(if selected { divisor } else { 0 }.to_string());
    }
    values
}

fn result(enabled: bool, outer: bool, first: bool, second: bool, reply: bool) -> i64 {
    let mut saved = 11;
    if enabled && first == outer && reply {
        saved += saved / 2;
    }
    if enabled && second != outer && reply {
        saved += saved / 2;
    }
    saved
}

fn check_computed(project: &Project, output: &Path) {
    for enabled in [false, true] {
        for outer in [false, true] {
            for first in [false, true] {
                for second in [false, true] {
                    for reply in [false, true] {
                        let initial = values(enabled, outer, first, second, reply).join(",");
                        let next = values(!enabled, !outer, !first, !second, !reply).join(",");
                        let run = success(project.command(
                            "run-artifact",
                            output,
                            &[
                                "--native-session",
                                "counter",
                                "--open-args",
                                &format!("10,{initial}"),
                                "--event-args",
                                &next,
                                "--close-args",
                                "",
                            ],
                        ));
                        let prints =
                            |enabled: bool, outer: bool, first: bool, second: bool, reply: bool| {
                                let mut lines = vec![99, 94];
                                if enabled {
                                    lines.extend([97, 91]);
                                    if first == outer {
                                        lines.push(95);
                                        if reply {
                                            lines.push(70);
                                        }
                                    }
                                    lines.push(92);
                                    if second != outer {
                                        lines.push(96);
                                        if reply {
                                            lines.push(71);
                                        }
                                    }
                                }
                                lines.push(80);
                                lines.iter().map(|v| format!("{v}\n")).collect::<String>()
                            };
                        assert_eq!(
                            String::from_utf8(run.stdout.clone()).unwrap(),
                            format!(
                                "{}{}",
                                prints(enabled, outer, first, second, reply),
                                prints(!enabled, !outer, !first, !second, !reply)
                            )
                        );
                        let state = |phase, enabled, outer, first, second, reply| {
                            let mut record = vec!["seed: 10".to_owned()];
                            record.extend(
                                FIELDS
                                    .into_iter()
                                    .zip(values(enabled, outer, first, second, reply))
                                    .map(|(name, value)| format!("{name}: {value}")),
                            );
                            record.push(format!(
                                "result: {}",
                                result(enabled, outer, first, second, reply)
                            ));
                            format!("{phase}:State{{{}}}", record.join(", "))
                        };
                        assert_eq!(
                            states(&run),
                            [
                                state("open", enabled, outer, first, second, reply),
                                state("event", !enabled, !outer, !first, !second, !reply),
                                state("close", !enabled, !outer, !first, !second, !reply)
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
fn native_computed_logical_effectful_scalar_selections_build_cache_restore_and_evaluate_left_once()
{
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_computed(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_computed(&project, &output);
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
        check_computed(&project, &restored);
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
    // Fatal signals may discard earlier buffered prints; later effects and
    // publication must nevertheless remain absent after any selected trap.
    for (boundary, print) in [
        (0, 91),
        (1, 95),
        (2, 70),
        (3, 92),
        (4, 96),
        (5, 71),
        (6, 80),
    ] {
        if stage <= boundary {
            assert!(!stdout.contains(&format!("{print}\n")), "{stdout}");
        }
    }
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
        "{stderr}"
    );
    rejected_before_open(run);
}

#[test]
fn native_computed_logical_effectful_scalar_selections_trap_at_left_rhs_and_leaf_including_skipped_rhs(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for outer in [false, true] {
        for (stage, index) in [5, 7, 9, 6, 8, 10].into_iter().enumerate() {
            let mut arguments = values(true, outer, outer, !outer, true);
            arguments[index] = "0".into();
            trap(&project, &output, &arguments, stage);
        }
        // A left operand is required even when its result would skip the RHS.
        for (stage, index, first, second) in [(0, 5, !outer, !outer), (3, 6, outer, outer)] {
            let mut arguments = values(true, outer, first, second, true);
            arguments[index] = "0".into();
            trap(&project, &output, &arguments, stage);
        }
    }
}
