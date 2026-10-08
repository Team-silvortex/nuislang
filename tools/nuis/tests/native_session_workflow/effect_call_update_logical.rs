use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/update_logical_effectful_scalar_selections.ns"
);
const FIELDS: [&str; 10] = [
    "enabled",
    "first",
    "second",
    "reply",
    "da",
    "db",
    "dc",
    "dd",
    "first_leaf",
    "second_leaf",
];

fn versions(seed: bool, first: bool, second: bool, reply: bool) -> (bool, bool) {
    let prefix = seed && reply;
    let first = if first { !prefix } else { prefix };
    let middle = first || reply;
    (first, if second { middle } else { !middle })
}

fn values(seed: bool, enabled: bool, first: bool, second: bool, reply: bool) -> Vec<String> {
    let (after_first, after_second) = versions(seed, first, second, reply);
    let mut values = [enabled, first, second, reply]
        .map(|v| v.to_string())
        .to_vec();
    for active in [
        enabled && seed,
        enabled && !after_first,
        enabled,
        enabled && !after_second,
        enabled && first,
        enabled && !second,
    ] {
        values.push(if active { "1" } else { "0" }.into());
    }
    values
}

fn result(seed: bool, enabled: bool, first: bool, second: bool, reply: bool) -> bool {
    if enabled {
        versions(seed, first, second, reply).1 || reply
    } else {
        seed
    }
}

fn prints(seed: bool, enabled: bool, first: bool, second: bool, reply: bool) -> String {
    let mut trace = vec![99, 89];
    if enabled {
        if seed {
            trace.push(91);
        }
        trace.push(90);
        if first {
            trace.push(70);
        }
        let (after_first, after_second) = versions(seed, first, second, reply);
        if !after_first {
            trace.push(92);
        }
        trace.push(95);
        if !second {
            trace.push(71);
        }
        trace.push(93);
        if !after_second {
            trace.push(94);
        }
    }
    trace.push(80);
    trace.iter().map(|v| format!("{v}\n")).collect()
}

fn check_updates(project: &Project, output: &Path) {
    for seed in [false, true] {
        for enabled in [false, true] {
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
                                &format!(
                                    "{seed},{}",
                                    values(seed, enabled, first, second, reply).join(",")
                                ),
                                "--event-args",
                                &values(seed, !enabled, !first, !second, !reply).join(","),
                                "--close-args",
                                "",
                            ],
                        ));
                        assert_eq!(
                            String::from_utf8(run.stdout.clone()).unwrap(),
                            format!(
                                "{}{}",
                                prints(seed, enabled, first, second, reply),
                                prints(seed, !enabled, !first, !second, !reply)
                            )
                        );
                        let state = |phase, enabled, first, second, reply| {
                            let mut fields = vec![format!("seed: {seed}")];
                            fields.extend(
                                FIELDS
                                    .into_iter()
                                    .zip(values(seed, enabled, first, second, reply))
                                    .map(|(name, value)| format!("{name}: {value}")),
                            );
                            fields.push(format!(
                                "result: {}",
                                result(seed, enabled, first, second, reply)
                            ));
                            format!("{phase}:State{{{}}}", fields.join(", "))
                        };
                        assert_eq!(
                            states(&run),
                            [
                                state("open", enabled, first, second, reply),
                                state("event", !enabled, !first, !second, !reply),
                                state("close", !enabled, !first, !second, !reply)
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
fn native_update_logical_effectful_scalar_selections_build_cache_restore_and_keep_preceding_bool_versions(
) {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_updates(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_updates(&project, &output);
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
        check_updates(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_update_logical_effectful_scalar_selections_trap_before_later_effects_or_bool_publication()
{
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for (stage, index, seed, first, second, reply) in [
        (0, 4, true, false, true, false),
        (2, 8, false, true, true, false),
        (3, 5, false, false, true, false),
        (5, 9, false, false, false, false),
        (6, 6, false, false, true, false),
        (7, 7, false, false, true, false),
        (6, 6, false, true, true, false),
    ] {
        let mut args = values(seed, true, first, second, reply);
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
        // Fatal signals can discard buffered output, never publish later stages.
        for print in &[91, 90, 70, 92, 95, 71, 93, 94, 80][stage..] {
            assert!(!stdout.contains(&format!("{print}\n")), "{stdout}");
        }
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        rejected_before_open(run);
    }
}
