use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/staged_effectful_scalar_selections.ns"
);

fn arguments(enabled: bool, outer: bool, child: bool) -> String {
    let left = enabled && outer;
    let right = enabled && !outer;
    format!(
        "{enabled},{outer},{child},{},{},{},{},{},{}",
        if left { 2 } else { 0 },
        if right { 2 } else { 0 },
        if left { 2 } else { 0 },
        if right { 2 } else { 0 },
        if left && child { 2 } else { 0 },
        if right && !child { 2 } else { 0 }
    )
}

fn check_staged(project: &Project, output: &Path) {
    for enabled in [false, true] {
        for outer in [false, true] {
            for child in [false, true] {
                let initial = arguments(enabled, outer, child);
                let next = arguments(!enabled, !outer, !child);
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
                let state = |phase: &str, enabled: bool, outer: bool, child: bool| {
                    let left = enabled && outer;
                    let right = enabled && !outer;
                    let result = if !enabled {
                        11
                    } else if outer {
                        if child {
                            32
                        } else {
                            16
                        }
                    } else if child {
                        6
                    } else {
                        12
                    };
                    format!("{phase}:State{{seed: 10, enabled: {enabled}, outer: {outer}, child: {child}, left_prefix: {}, right_prefix: {}, left_condition: {}, right_condition: {}, left_leaf: {}, right_leaf: {}, result: {result}}}",
                        if left {2}else{0},if right {2}else{0},if left {2}else{0},if right {2}else{0},if left && child {2}else{0},if right && !child {2}else{0})
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
fn native_staged_effectful_scalar_selections_build_cache_restore_and_keep_prefix_retention() {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_staged(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_staged(&project, &output);
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
        check_staged(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_staged_effectful_scalar_selections_trap_on_selected_prefix_predicate_or_child_argument_before_publication(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for (open, prefix_trap) in [
        ("10,true,true,true,0,0,2,0,2,0", true),
        ("10,true,false,false,0,0,0,2,0,2", true),
        ("10,true,true,true,2,0,0,0,2,0", false),
        ("10,true,false,false,0,2,0,0,0,2", false),
        ("10,true,true,true,2,0,2,0,0,0", false),
        ("10,true,false,false,0,2,0,2,0,0", false),
    ] {
        let run = project.command(
            "run-artifact",
            &output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                open,
                "--close-args",
                "",
            ],
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            !stdout.contains("70\n") && !stdout.contains("80\n"),
            "{stdout}"
        );
        if prefix_trap {
            assert!(
                !stdout.contains("62\n") && !stdout.contains("98\n") && !stdout.contains("95\n"),
                "{stdout}"
            );
        }
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        rejected_before_open(run);
    }
}
