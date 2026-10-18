use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_returns_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path, leading: bool) {
    for (marker, gate) in cases::CLI_MARKERS
        .into_iter()
        .zip([false, true, false, true])
    {
        let calls = cases::sequence(cases::Invocation {
            marker,
            gate,
            ..cases::base(leading)
        });
        let mut args = vec![
            "--native-session".to_owned(),
            "counter".to_owned(),
            "--open-args".to_owned(),
            calls[0].args(None),
        ];
        let mut expected = vec![calls[0].next().text("open")];
        let mut printed = calls[0].trace(false);
        for call in &calls[1..] {
            args.extend(["--event-args".to_owned(), call.args(None)]);
            expected.push(call.next().text("event"));
            printed.extend(call.trace(false));
        }
        args.extend(["--close-args".to_owned(), String::new()]);
        expected.push(calls.last().unwrap().next().text("close"));
        let run = success(project.command(
            "run-artifact",
            output,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        ));
        assert_eq!(states(&run), expected);
        for state in states(&run) {
            let value = state
                .split(", marker: ")
                .nth(1)
                .unwrap()
                .strip_suffix("f64}")
                .unwrap();
            assert_eq!(value.parse::<f64>().unwrap().to_bits(), marker);
        }
        assert_eq!(
            String::from_utf8(run.stdout).unwrap(),
            printed
                .into_iter()
                .map(|n| format!("{n}\n"))
                .collect::<String>()
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    }
}

fn image(project: &Project, output: &Path, mode: &str, binary: &[u8], leading: bool) {
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    let container = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    healthy(project, output, leading);
    let prefix = cases::failure_prefix(leading);
    for (call, bad) in cases::failures(leading) {
        let call = cases::after_prefix(call);
        let run = project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &prefix[0].args(None),
                "--event-args",
                &prefix[1].args(None),
                "--event-args",
                &prefix[2].args(None),
                "--event-args",
                &call.args(bad),
                "--event-args",
                &prefix[0].args(None),
                "--close-args",
                "",
            ],
        );
        assert!(!run.status.success());
        assert_eq!(
            states(&run),
            [
                prefix[0].next().text("open"),
                prefix[1].next().text("event"),
                prefix[2].next().text("event")
            ]
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"));
    }
    // Native word probes prove flushed pre-trap prefixes; CLI proves no failed
    // event is published and delivered artifact bytes remain reusable.
    healthy(project, output, leading);
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    assert!(fs::read(output.join("nuis.compiled.artifact")).unwrap() == container);
}

#[test]
fn state_loop_returns_build_cache_and_two_source_free_restores_preserve_early_return() {
    for variant in cases::variants() {
        let (project, mode) =
            project_for_source_with_limits(&variant.source(), cases::LOOP_LIMIT, 100);
        project.build(None);
        let output = project.0.join("build");
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let fresh =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        image(&project, &output, &mode, &binary, variant.leading);
        assert!(project.build(None).contains("compile_cache: hit"));
        let cached =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        assert!(fresh.binary_blob == cached.binary_blob);
        assert!(fresh.host_objects == cached.host_objects);
        assert_eq!(fresh.packaging_mode, cached.packaging_mode);
        assert_eq!(fresh.envelope, cached.envelope);
        assert_eq!(fresh.lifecycle, cached.lifecycle);
        image(&project, &output, &mode, &binary, variant.leading);
        let artifact = project.0.join("nuis.compiled.artifact");
        fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
        let bytes = fs::read(&artifact).unwrap();
        fs::remove_dir_all(&output).unwrap();
        for file in ["main.ns", "nuis.toml"] {
            fs::remove_file(project.0.join(file)).unwrap();
        }
        fs::remove_dir_all(project.0.join(".nuis")).unwrap();
        success(project.command("verify-artifact", &artifact, &[]));
        let restored = project.0.join("restored");
        for _ in 0..2 {
            success(project.command(
                "materialize-artifact",
                &artifact,
                &[restored.to_str().unwrap()],
            ));
            image(&project, &restored, &mode, &binary, variant.leading);
            assert!(fs::read(&artifact).unwrap() == bytes);
        }
        for file in ["main.ns", "nuis.toml", ".nuis"] {
            assert!(!project.0.join(file).exists());
        }
    }
}
