use super::*;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/sequential_effectful_scalar_regions.ns"
);

fn check_regions(project: &Project, output: &Path) {
    for outer in [false, true] {
        for inner in [false, true] {
            let arguments = |outer: bool, inner: bool| {
                format!("{outer},{inner},{0},{0}", if outer { 2 } else { 0 })
            };
            let initial = arguments(outer, inner);
            let next = arguments(!outer, !inner);
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
            let prints = |outer, inner| {
                format!(
                    "99\n97\n{}80\n",
                    if outer {
                        if inner {
                            "98\n60\n61\n62\n70\n"
                        } else {
                            "98\n60\n61\n62\n71\n"
                        }
                    } else {
                        ""
                    }
                )
            };
            assert_eq!(
                String::from_utf8(run.stdout.clone()).unwrap(),
                format!(
                    "{}{}{}",
                    prints(outer, inner),
                    prints(!outer, !inner),
                    prints(outer, inner)
                )
            );
            let state = |phase: &str, outer, inner| {
                format!(
                "{phase}:State{{seed: 10, outer: {outer}, inner: {inner}, prefix_divisor: {divisor}, later_divisor: {divisor}, result: {result}}}",
                divisor = if outer { 2 } else { 0 }, result = if outer { if inner { 32 } else { 0 } } else { 11 })
            };
            assert_eq!(
                states(&run),
                [
                    state("open", outer, inner),
                    state("event", !outer, !inner),
                    state("event", outer, inner),
                    state("close", outer, inner)
                ]
            );
            assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
        }
    }
}

#[test]
fn native_sequential_effectful_scalar_regions_build_cache_restore_and_preserve_staged_versions() {
    let (project, mode) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    check_regions(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    check_regions(&project, &output);
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
        check_regions(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_sequential_effectful_scalar_regions_trap_on_selected_prefix_and_later_stage_before_publication(
) {
    let (project, _) = project_for_source(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    for (open, prefix_trap) in [
        ("10,true,true,0,2", true),
        ("10,true,false,0,2", true),
        ("10,true,true,2,0", false),
        ("10,true,false,2,0", false),
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
            !stdout.contains("70\n") && !stdout.contains("71\n") && !stdout.contains("80\n"),
            "{stdout}"
        );
        if prefix_trap {
            assert!(!stdout.contains("62\n"), "{stdout}");
        }
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        rejected_before_open(run);
    }
}
