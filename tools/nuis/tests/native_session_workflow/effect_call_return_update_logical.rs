use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/return_update_logical_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 15] = [
    "first",
    "second",
    "reply",
    "outer",
    "init",
    "cond",
    "yes",
    "no",
    "retained_cond",
    "retained_rhs",
    "unused",
    "leaf",
    "leaf_cond",
    "leaf_rhs",
    "answer",
];

fn current(seed: bool, first: bool, second: bool, reply: bool) -> (bool, bool) {
    let mut local = seed;
    if first {
        local = if second {
            local || reply
        } else {
            local && reply
        };
        let before_retained = local;
        if !local {
            local = local || reply;
        }
        (before_retained, local)
    } else {
        if second {
            local = local && reply;
        }
        (local, local)
    }
}

fn values(seed: bool, first: bool, second: bool, reply: bool) -> Vec<String> {
    let (before_retained, local) = current(seed, first, second, reply);
    let mut values = [first, second, reply].map(|v| v.to_string()).to_vec();
    for active in [
        true,
        true,
        true,
        first && second && !seed,
        ((first && !second) || (!first && second)) && seed,
        first,
        first && !before_retained,
        first,
        first && local,
        first && local,
        first && local && second,
        !(first && local && second && !reply),
    ] {
        values.push(if active { "1" } else { "0" }.into());
    }
    values
}

fn result(seed: bool, first: bool, second: bool, reply: bool) -> bool {
    let (_, local) = current(seed, first, second, reply);
    if first && local {
        let mut selected = local;
        if second {
            selected = selected && reply;
        }
        selected && reply
    } else if local {
        local && reply
    } else {
        local || seed
    }
}

fn trace(seed: bool, first: bool, second: bool, reply: bool) -> Vec<i32> {
    let (before_retained, local) = current(seed, first, second, reply);
    let mut trace = vec![99, 90, 87, 88];
    if first && second && !seed {
        trace.push(89);
    }
    if ((first && !second) || (!first && second)) && seed {
        trace.push(86);
    }
    if first {
        trace.push(85);
        if !before_retained {
            trace.push(84);
        }
        trace.push(83);
        if local {
            trace.extend([82, 81]);
            if second {
                trace.push(78);
            }
        }
    }
    if !(first && local && second && !reply) {
        trace.push(97);
    }
    trace.push(79);
    trace
}

fn prints(seed: bool, first: bool, second: bool, reply: bool) -> String {
    trace(seed, first, second, reply)
        .iter()
        .map(|v| format!("{v}\n"))
        .collect()
}

fn check_returns(project: &Project, output: &Path) {
    for seed in [false, true] {
        for first in [false, true] {
            for second in [false, true] {
                for reply in [false, true] {
                    let run = success(project.command(
                        "run-artifact",
                        output,
                        &[
                            "--native-session",
                            "counter",
                            "--open-args",
                            &format!("{seed},{}", values(seed, first, second, reply).join(",")),
                            "--event-args",
                            &values(seed, !first, !second, !reply).join(","),
                            "--close-args",
                            "",
                        ],
                    ));
                    assert_eq!(
                        String::from_utf8(run.stdout.clone()).unwrap(),
                        format!(
                            "{}{}",
                            prints(seed, first, second, reply),
                            prints(seed, !first, !second, !reply)
                        )
                    );
                    let state = |phase, first, second, reply| {
                        let mut fields = vec![format!("seed: {seed}")];
                        fields.extend(
                            FIELDS
                                .into_iter()
                                .zip(values(seed, first, second, reply))
                                .map(|(name, value)| format!("{name}: {value}")),
                        );
                        fields.push(format!("result: {}", result(seed, first, second, reply)));
                        format!("{phase}:State{{{}}}", fields.join(", "))
                    };
                    assert_eq!(
                        states(&run),
                        [
                            state("open", first, second, reply),
                            state("event", !first, !second, !reply),
                            state("close", !first, !second, !reply)
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
fn native_return_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation(
) {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_returns(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_returns(&project, &output);
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
        check_returns(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_return_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for (index, stage, seed, first, second, reply) in [
        (3, 90, false, true, true, true),
        (4, 87, false, false, false, true),
        (5, 88, false, true, true, true),
        (6, 89, false, true, true, true),
        (7, 86, true, false, true, false),
        (8, 85, true, true, true, false),
        (9, 84, false, true, false, true),
        (10, 83, false, true, false, true),
        (11, 82, false, true, false, true),
        (12, 81, false, true, false, true),
        (13, 78, true, true, true, false),
        (14, 97, false, true, false, true),
    ] {
        let mut args = values(seed, first, second, reply);
        assert_eq!(args[index], "1");
        args[index] = "0".into();
        let run = project.command(
            "run-artifact",
            &output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &format!("{seed},{}", args.join(",")),
                "--close-args",
                "",
            ],
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        let expected = trace(seed, first, second, reply);
        let position = expected.iter().position(|print| *print == stage).unwrap();
        for print in &expected[position..] {
            assert!(!stdout.contains(&format!("{print}\n")), "{stdout}");
        }
        assert!(!stdout.contains("80\n"), "{stdout}");
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        rejected_before_open(run);
    }
}
