use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/continued_effectful_scalar_selections.ns"
);

fn values(enabled: bool, outer: bool, child: bool) -> Vec<String> {
    let left = enabled && outer;
    let right = enabled && !outer;
    let divisor = |active| if active { 2 } else { 0 };
    vec![
        enabled.to_string(),
        outer.to_string(),
        child.to_string(),
        divisor(left).to_string(),
        divisor(right).to_string(),
        divisor(left).to_string(),
        divisor(right).to_string(),
        divisor(left && child).to_string(),
        divisor(right && !child).to_string(),
        i64::from(left).to_string(),
        i64::from(right).to_string(),
        divisor(left).to_string(),
        divisor(right).to_string(),
    ]
}

fn check_continued(project: &Project, output: &Path) {
    for enabled in [false, true] {
        for outer in [false, true] {
            for child in [false, true] {
                let initial = values(enabled, outer, child).join(",");
                let next = values(!enabled, !outer, !child).join(",");
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
                let prints = |enabled: bool, outer: bool, child: bool| {
                    let mut lines = vec![99, 94];
                    if enabled {
                        lines.extend([97, 60, 61, 62, if outer { 98 } else { 95 }]);
                        if outer == child {
                            lines.push(70);
                        }
                        lines.extend([65, 63, 64]);
                    }
                    lines.push(80);
                    lines.iter().map(|v| format!("{v}\n")).collect::<String>()
                };
                assert_eq!(
                    String::from_utf8(run.stdout.clone()).unwrap(),
                    format!(
                        "{}{}{}",
                        prints(enabled, outer, child),
                        prints(!enabled, !outer, !child),
                        prints(enabled, outer, child)
                    )
                );
                let state = |phase, enabled: bool, outer: bool, child: bool| {
                    let result = if !enabled {
                        11
                    } else if outer {
                        if child {
                            64
                        } else {
                            40
                        }
                    } else if child {
                        15
                    } else {
                        24
                    };
                    let mut record = vec!["seed: 10".to_owned()];
                    record.extend(
                        [
                            "enabled",
                            "outer",
                            "child",
                            "left_prefix",
                            "right_prefix",
                            "left_condition",
                            "right_condition",
                            "left_leaf",
                            "right_leaf",
                            "left_argument",
                            "right_argument",
                            "left_suffix",
                            "right_suffix",
                        ]
                        .into_iter()
                        .zip(values(enabled, outer, child))
                        .map(|(name, value)| format!("{name}: {value}")),
                    );
                    record.push(format!("result: {result}"));
                    format!("{phase}:State{{{}}}", record.join(", "))
                };
                assert_eq!(
                    states(&run),
                    [
                        state("open", enabled, outer, child),
                        state("event", !enabled, !outer, !child),
                        state("event", enabled, outer, child),
                        state("close", enabled, outer, child)
                    ]
                );
                assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
            }
        }
    }
}

#[test]
fn native_continued_effectful_scalar_selections_build_cache_restore_and_keep_child_to_suffix_versions(
) {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_continued(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_continued(&project, &output);
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
        check_continued(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_continued_effectful_scalar_selections_trap_before_suffix_or_publication_at_each_selected_stage(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for outer in [false, true] {
        for stage in 0..5 {
            let child = if stage == 3 { !outer } else { outer };
            let mut arguments = values(true, outer, child);
            arguments[3 + 2 * stage + usize::from(!outer)] = "0".into();
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
            // Fatal host stdio can buffer completed prints. Successful runs
            // check the full trace; traps require no later output/publication.
            assert!(
                !stdout.contains("64\n") && !stdout.contains("80\n"),
                "{stdout}"
            );
            if stage < 3 {
                assert!(
                    !stdout.contains("65\n") && !stdout.contains("70\n"),
                    "{stdout}"
                );
            }
            if stage < 4 {
                assert!(!stdout.contains("63\n"), "{stdout}");
            }
            if stage == 0 {
                assert!(
                    !stdout.contains("62\n")
                        && !stdout.contains("98\n")
                        && !stdout.contains("95\n"),
                    "{stdout}"
                );
            }
            if stage == 3 {
                assert!(!stdout.contains("70\n"), "{stdout}");
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
