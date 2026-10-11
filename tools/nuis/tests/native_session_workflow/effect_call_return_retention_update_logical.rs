use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/return_retention_update_logical_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 15] = [
    "first", "second", "reply", "outer", "init", "cond", "before", "rhs", "mid", "replace",
    "unused", "no", "after", "terminal", "answer",
];

// Independent imperative replay of the original source, not its lowered helpers.
fn oracle(seed: bool, first: bool, second: bool, reply: bool) -> (bool, Vec<i32>) {
    let mut local = seed;
    let mut trace = vec![99, 90, 87, 88];
    let result;
    if first {
        if second {
            if reply && !local {
                trace.push(89);
                local = local || first;
            }
            trace.push(77);
            let observed = local;
            local = observed && first;
            trace.push(72);
            if local {
                local = local && reply;
            }
            trace.push(83);
            if !local {
                trace.push(71);
                local = local || reply;
            }
        } else if local {
            trace.push(86);
            local = local && reply;
        }
        trace.push(92);
        if local {
            trace.push(93);
        }
        result = if local {
            trace.push(97);
            local && reply
        } else {
            false
        };
    } else {
        if second {
            if local {
                trace.push(86);
                local = local && reply;
            }
            trace.extend([83, 72]);
            if !reply && !local {
                trace.push(89);
                local = local || first;
            }
            trace.push(92);
            if local {
                trace.push(93);
            }
        }
        result = if local {
            true
        } else {
            trace.push(97);
            local || seed
        };
    }
    trace.push(79);
    (result, trace)
}

const STAGES: [i32; 12] = [90, 87, 88, 77, 89, 72, 71, 83, 86, 92, 93, 97];

fn values(seed: bool, first: bool, second: bool, reply: bool) -> Vec<String> {
    let trace = oracle(seed, first, second, reply).1;
    let mut values = [first, second, reply].map(|v| v.to_string()).to_vec();
    values.extend(STAGES.map(|stage| if trace.contains(&stage) { "1" } else { "0" }.into()));
    values
}

fn result(seed: bool, first: bool, second: bool, reply: bool) -> bool {
    oracle(seed, first, second, reply).0
}

fn trace(seed: bool, first: bool, second: bool, reply: bool) -> Vec<i32> {
    oracle(seed, first, second, reply).1
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
fn native_return_retention_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation(
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
fn native_return_retention_update_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for (offset, stage) in STAGES.into_iter().enumerate() {
        let (seed, first, second, reply) = (0..16)
            .map(|bits| (bits & 8 != 0, bits & 4 != 0, bits & 2 != 0, bits & 1 != 0))
            .find(|&(seed, first, second, reply)| {
                trace(seed, first, second, reply).contains(&stage)
            })
            .unwrap();
        let index = offset + 3;
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
