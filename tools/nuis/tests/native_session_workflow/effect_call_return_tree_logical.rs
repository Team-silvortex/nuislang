use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/return_tree_logical_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 14] = [
    "first", "second", "reply", "outer", "ycond", "ncond", "a", "ar", "b", "br", "c", "cr", "d",
    "dr",
];

fn values(seed: bool, first: bool, second: bool, reply: bool) -> Vec<String> {
    let mut values = [first, second, reply].map(|v| v.to_string()).to_vec();
    for active in [
        true,
        first && second,
        !first && !second,
        first && second,
        first && second && seed,
        first && !second,
        first && !second && !seed,
        !first && second,
        !first && second && seed,
        !first && !second,
        !first && !second && !seed,
    ] {
        values.push(if active { "1" } else { "0" }.into());
    }
    values
}

fn result(seed: bool, second: bool, reply: bool) -> bool {
    if second {
        seed && reply
    } else {
        seed || reply
    }
}

fn trace(seed: bool, first: bool, second: bool) -> Vec<i32> {
    let mut trace = vec![99, 90];
    if first && second {
        trace.push(91);
    }
    if !first && !second {
        trace.push(92);
    }
    trace.push(match (first, second) {
        (true, true) => 93,
        (true, false) => 94,
        (false, true) => 95,
        (false, false) => 96,
    });
    if (second && seed) || (!second && !seed) {
        trace.push(97);
    }
    trace.push(79);
    trace
}

fn prints(seed: bool, first: bool, second: bool) -> String {
    trace(seed, first, second)
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
                            prints(seed, first, second),
                            prints(seed, !first, !second)
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
                        fields.push(format!("result: {}", result(seed, second, reply)));
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
fn native_return_tree_logical_effectful_scalar_selections_build_cache_restore_and_keep_complete_leaf_returns_and_caller_continuation(
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
fn native_return_tree_logical_effectful_scalar_selections_trap_at_selected_ancestor_conditions_prefixes_and_leaf_rhs_before_publication(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for (index, stage, seed, first, second, reply) in [
        (3, 90, false, true, true, false),
        (4, 91, false, true, true, false),
        (5, 92, true, false, false, true),
        (6, 93, false, true, true, false),
        (8, 94, true, true, false, true),
        (10, 95, false, false, true, false),
        (12, 96, true, false, false, true),
        (7, 97, true, true, true, false),
        (9, 97, false, true, false, false),
        (11, 97, true, false, true, false),
        (13, 97, false, false, false, false),
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
        let expected = trace(seed, first, second);
        let position = expected.iter().position(|print| *print == stage).unwrap();
        // A trapping argument cannot publish its call, later leaf work or caller continuation.
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
