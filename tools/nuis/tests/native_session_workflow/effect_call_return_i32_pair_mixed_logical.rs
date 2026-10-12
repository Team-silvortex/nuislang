use super::*;

const SOURCE: &str = include_str!("../../../nuisc/tests/native_application_bridge/return_i32_pair_mixed_logical_effectful_scalar_selections.ns");
const FIELDS: [&str; 11] = [
    "first",
    "second",
    "outer",
    "init",
    "child_init",
    "cond",
    "child_rhs",
    "parent_rhs",
    "after",
    "unused",
    "answer",
];
const STAGES: [i32; 9] = [90, 87, 91, 92, 93, 95, 96, 83, 97];
const SEEDS: [i64; 5] = [i32::MIN as i64, -1, 0, 1, i32::MAX as i64];

#[path = "../../../nuisc/tests/native_application_bridge/return_i32_pair_oracle.rs"]
mod pair_oracle;
use pair_oracle::oracle;
fn values(seed: i64, first: bool, second: bool) -> Vec<String> {
    let trace = oracle(seed, first, second).1;
    let mut values = vec![first.to_string(), second.to_string()];
    values.extend(STAGES.map(|stage| if trace.contains(&stage) { "1" } else { "0" }.into()));
    values
}

fn prints(seed: i64, first: bool, second: bool) -> String {
    oracle(seed, first, second)
        .1
        .iter()
        .map(|v| format!("{v}\n"))
        .collect()
}

fn check_returns(project: &Project, output: &Path) {
    for seed in SEEDS {
        for first in [false, true] {
            for second in [false, true] {
                let run = success(project.command(
                    "run-artifact",
                    output,
                    &[
                        "--native-session",
                        "counter",
                        "--open-args",
                        &format!("{seed},{}", values(seed, first, second).join(",")),
                        "--event-args",
                        &values(seed, !first, !second).join(","),
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
                let state = |phase, first, second| {
                    let mut fields = vec![format!("seed: {seed}")];
                    fields.extend(
                        FIELDS
                            .into_iter()
                            .zip(values(seed, first, second))
                            .map(|(name, value)| format!("{name}: {value}")),
                    );
                    fields.push(format!("result: {}", oracle(seed, first, second).0));
                    format!("{phase}:State{{{}}}", fields.join(", "))
                };
                assert_eq!(
                    states(&run),
                    [
                        state("open", first, second),
                        state("event", !first, !second),
                        state("close", !first, !second)
                    ]
                );
                assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
            }
        }
    }
}

#[test]
fn native_return_i32_pair_mixed_logical_effectful_scalar_selections_build_cache_restore_and_keep_current_retained_outcomes_and_caller_continuation(
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
fn native_return_i32_pair_mixed_logical_effectful_scalar_selections_trap_at_selected_updates_unused_work_and_returns_before_publication(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for (offset, stage) in STAGES.into_iter().enumerate() {
        let (seed, first, second) = SEEDS
            .into_iter()
            .flat_map(|seed| {
                [false, true].into_iter().flat_map(move |first| {
                    [false, true]
                        .into_iter()
                        .map(move |second| (seed, first, second))
                })
            })
            .find(|&(seed, first, second)| oracle(seed, first, second).1.contains(&stage))
            .unwrap();
        let index = offset + 2;
        let mut args = values(seed, first, second);
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
        let expected = oracle(seed, first, second).1;
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
