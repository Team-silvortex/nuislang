use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/ordered_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 19] = [
    "enabled",
    "outer",
    "first",
    "second",
    "third",
    "left_prefix",
    "right_prefix",
    "left_first",
    "right_first",
    "left_predicate",
    "right_predicate",
    "left_second",
    "right_second",
    "left_middle",
    "right_middle",
    "left_third",
    "right_third",
    "left_suffix",
    "right_suffix",
];

fn values(enabled: bool, outer: bool, first: bool, second: bool, third: bool) -> Vec<String> {
    let mut values = [enabled, outer, first, second, third]
        .map(|v| v.to_string())
        .to_vec();
    for stage in 0..7 {
        for branch in [true, false] {
            let selected = enabled
                && outer == branch
                && match stage {
                    1 => first == branch,
                    3 => second == branch,
                    5 => third == branch,
                    6 => !branch,
                    _ => true,
                };
            values.push(
                if !selected {
                    "0"
                } else if stage == 2 {
                    "1"
                } else {
                    "2"
                }
                .into(),
            );
        }
    }
    values
}

fn result(enabled: bool, outer: bool, first: bool, second: bool, third: bool) -> i64 {
    if !enabled {
        return 11;
    }
    let mut saved = if outer { 16 } else { 6 };
    if outer == first {
        saved += saved / 2;
    }
    if outer == second {
        saved += saved / 2;
    }
    let staged = saved;
    saved += staged;
    if outer == third {
        saved += staged / 2;
    }
    if !outer {
        saved /= 2;
    }
    saved
}

fn check_ordered(project: &Project, output: &Path) {
    for enabled in [false, true] {
        for outer in [false, true] {
            for first in [false, true] {
                for second in [false, true] {
                    for third in [false, true] {
                        let initial = values(enabled, outer, first, second, third).join(",");
                        let next = values(!enabled, !outer, !first, !second, !third).join(",");
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
                            |enabled: bool, outer: bool, first: bool, second: bool, third: bool| {
                                let mut lines = vec![99, 94];
                                if enabled {
                                    lines.extend([97, 61, 98]);
                                    if outer == first {
                                        lines.push(70);
                                    }
                                    lines.push(95);
                                    if outer == second {
                                        lines.push(71);
                                    }
                                    lines.extend([62, 96]);
                                    if outer == third {
                                        lines.push(72);
                                    }
                                    if !outer {
                                        lines.push(63);
                                    }
                                }
                                lines.push(80);
                                lines.iter().map(|v| format!("{v}\n")).collect::<String>()
                            };
                        assert_eq!(
                            String::from_utf8(run.stdout.clone()).unwrap(),
                            format!(
                                "{}{}",
                                prints(enabled, outer, first, second, third),
                                prints(!enabled, !outer, !first, !second, !third)
                            )
                        );
                        let state = |phase, enabled, outer, first, second, third| {
                            let mut record = vec!["seed: 10".to_owned()];
                            record.extend(
                                FIELDS
                                    .into_iter()
                                    .zip(values(enabled, outer, first, second, third))
                                    .map(|(name, value)| format!("{name}: {value}")),
                            );
                            record.push(format!(
                                "result: {}",
                                result(enabled, outer, first, second, third)
                            ));
                            format!("{phase}:State{{{}}}", record.join(", "))
                        };
                        assert_eq!(
                            states(&run),
                            [
                                state("open", enabled, outer, first, second, third),
                                state("event", !enabled, !outer, !first, !second, !third),
                                state("close", !enabled, !outer, !first, !second, !third),
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
fn native_ordered_effectful_scalar_selections_build_cache_restore_and_keep_adjacent_and_final_child_versions(
) {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_ordered(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_ordered(&project, &output);
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
        check_ordered(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_ordered_effectful_scalar_selections_trap_at_selected_adjacent_middle_final_and_suffix_stages(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for outer in [false, true] {
        for stage in 0..7 {
            if outer && stage == 6 {
                continue;
            }
            let mut arguments = values(true, outer, outer, outer, outer);
            arguments[5 + 2 * stage + usize::from(!outer)] = "0".into();
            let run = project.command(
                "run-artifact",
                &output,
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
            // A fatal signal need not flush completed host prints. Successful
            // runs check complete traces; traps forbid later work/publication.
            assert!(!stdout.contains("80\n"), "{stdout}");
            for (boundary, print) in [(1, 98), (2, 95), (4, 62), (5, 96), (6, 63)] {
                if stage < boundary {
                    assert!(!stdout.contains(&format!("{print}\n")), "{stdout}");
                }
            }
            for (boundary, print) in [(1, 70), (3, 71), (5, 72)] {
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
    }
}
