use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/repeated_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 16] = [
    "enabled",
    "outer",
    "first",
    "second",
    "left_prefix",
    "right_prefix",
    "left_first",
    "right_first",
    "left_middle",
    "right_middle",
    "left_predicate",
    "right_predicate",
    "left_second",
    "right_second",
    "left_suffix",
    "right_suffix",
];

fn values(enabled: bool, outer: bool, first: bool, second: bool) -> Vec<String> {
    let mut values = [enabled, outer, first, second]
        .map(|v| v.to_string())
        .to_vec();
    for stage in 0..6 {
        for branch in [true, false] {
            let selected = enabled
                && outer == branch
                && match stage {
                    1 => first == branch,
                    4 => second == branch,
                    _ => true,
                };
            values.push(if selected { "2" } else { "0" }.into());
        }
    }
    values
}

fn result(enabled: bool, outer: bool, first: bool, second: bool) -> i64 {
    if !enabled {
        return 11;
    }
    let mut saved = if outer { 16 } else { 6 };
    if outer == first {
        saved += saved / 2;
    }
    let staged = saved;
    saved += staged;
    if outer == second {
        saved += staged / 2;
    }
    saved + (saved + staged) / 2
}

fn check_repeated(project: &Project, output: &Path) {
    for enabled in [false, true] {
        for outer in [false, true] {
            for first in [false, true] {
                for second in [false, true] {
                    let initial = values(enabled, outer, first, second).join(",");
                    let next = values(!enabled, !outer, !first, !second).join(",");
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
                            "--event-args",
                            &initial,
                            "--close-args",
                            "",
                        ],
                    ));
                    let prints = |enabled: bool, outer: bool, first: bool, second: bool| {
                        let mut lines = vec![99, 94];
                        if enabled {
                            lines.extend([97, 61, 98]);
                            if outer == first {
                                lines.push(70);
                            }
                            lines.extend([62, 95]);
                            if outer == second {
                                lines.push(71);
                            }
                            lines.extend([63, 64]);
                        }
                        lines.push(80);
                        lines.iter().map(|v| format!("{v}\n")).collect::<String>()
                    };
                    assert_eq!(
                        String::from_utf8(run.stdout.clone()).unwrap(),
                        format!(
                            "{}{}{}",
                            prints(enabled, outer, first, second),
                            prints(!enabled, !outer, !first, !second),
                            prints(enabled, outer, first, second)
                        )
                    );
                    let state = |phase, enabled, outer, first, second| {
                        let mut record = vec!["seed: 10".to_owned()];
                        record.extend(
                            FIELDS
                                .into_iter()
                                .zip(values(enabled, outer, first, second))
                                .map(|(name, value)| format!("{name}: {value}")),
                        );
                        record.push(format!("result: {}", result(enabled, outer, first, second)));
                        format!("{phase}:State{{{}}}", record.join(", "))
                    };
                    assert_eq!(
                        states(&run),
                        [
                            state("open", enabled, outer, first, second),
                            state("event", !enabled, !outer, !first, !second),
                            state("event", enabled, outer, first, second),
                            state("close", enabled, outer, first, second),
                        ]
                    );
                    assert!(
                        String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1")
                    );
                }
            }
        }
    }
}

#[test]
fn native_repeated_effectful_scalar_selections_build_cache_restore_and_read_current_sibling_versions(
) {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_repeated(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_repeated(&project, &output);
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
        check_repeated(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_repeated_effectful_scalar_selections_trap_at_each_selected_stage_before_publication() {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for outer in [false, true] {
        for stage in 0..6 {
            let first = outer;
            let second = outer;
            let mut arguments = values(true, outer, first, second);
            arguments[4 + 2 * stage + usize::from(!outer)] = "0".into();
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
            // Fatal stdio may lose already completed prints; check absence of
            // later effects/publication, with full traces on successful runs.
            assert!(
                !stdout.contains("64\n") && !stdout.contains("80\n"),
                "{stdout}"
            );
            if stage < 5 {
                assert!(!stdout.contains("63\n"), "{stdout}");
            }
            if stage < 4 {
                assert!(!stdout.contains("71\n"), "{stdout}");
            }
            if stage < 3 {
                assert!(!stdout.contains("95\n"), "{stdout}");
            }
            if stage < 2 {
                assert!(!stdout.contains("62\n"), "{stdout}");
            }
            if stage < 1 {
                assert!(
                    !stdout.contains("98\n") && !stdout.contains("70\n"),
                    "{stdout}"
                );
            }
            if stage == 1 {
                assert!(!stdout.contains("70\n"), "{stdout}");
            }
            if stage == 4 {
                assert!(!stdout.contains("71\n"), "{stdout}");
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
