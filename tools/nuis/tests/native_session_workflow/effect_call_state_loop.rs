use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_cases.rs"]
mod cases;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_effects_profile.rs"]
mod effects;

#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_statements_profile.rs"]
mod statements;

type Trace = fn(cases::Invocation, bool) -> Vec<i64>;

fn assert_marker_bits(run: &Output, expected: u64) {
    for state in states(run) {
        let marker = state
            .split(", marker: ")
            .nth(1)
            .unwrap()
            .strip_suffix("f64}")
            .unwrap();
        assert_eq!(marker.parse::<f64>().unwrap().to_bits(), expected);
    }
}
fn healthy(project: &Project, output: &Path, trace: Trace) {
    for ((marker, trips), poisoned) in cases::CLI_MARKERS
        .into_iter()
        .zip([0, 1, 3, -1])
        .zip([true, false, true, true])
    {
        let calls = cases::sequence(
            cases::Case {
                seed: 8,
                gate: false,
                flag: false,
                trips,
                marker,
            },
            poisoned,
        );
        let mut args = vec![
            "--native-session".to_owned(),
            "counter".to_owned(),
            "--open-args".to_owned(),
            calls[0].args(None),
        ];
        let mut expected = vec![calls[0].next().text("open")];
        let mut printed = trace(calls[0], false);
        for call in &calls[1..] {
            args.extend(["--event-args".to_owned(), call.args(None)]);
            expected.push(call.next().text("event"));
            printed.extend(trace(*call, false));
        }
        args.extend(["--close-args".to_owned(), String::new()]);
        expected.push(calls.last().unwrap().next().text("close"));
        assert_eq!(expected.len(), 5);
        let run = success(project.command(
            "run-artifact",
            output,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        ));
        assert_eq!(states(&run), expected);
        assert_marker_bits(&run, marker);
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
fn oversized_script(project: &Project, output: &Path) {
    let input = cases::failure_prefix()[0].args(None);
    let mut args = vec!["--native-session", "counter", "--open-args", input.as_str()];
    for _ in 0..65 {
        args.extend(["--event-args", input.as_str()]);
    }
    args.extend(["--close-args", ""]);
    let run = project.command("run-artifact", output, &args);
    assert!(String::from_utf8_lossy(&run.stderr)
        .contains("native application script exceeds its input bounds"));
    rejected_before_open(run);
}
fn image(project: &Project, output: &Path, mode: &str, binary: &[u8], trace: Trace) {
    let container_path = output.join("nuis.compiled.artifact");
    let container = fs::read(&container_path).unwrap();
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert!(mode.starts_with("native-session-policy-aot-bundle:v1."));
    assert!(mode.contains(".100."));
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    healthy(project, output, trace);
    oversized_script(project, output);
    let rows = cases::failure_prefix();
    for (bad, slot) in cases::failures()
        .into_iter()
        .filter(|(_, slot)| slot.is_none() || *slot == Some(6))
    {
        let run = project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &rows[0].args(None),
                "--event-args",
                &rows[1].args(None),
                "--event-args",
                &rows[2].args(None),
                "--event-args",
                &bad.args(slot),
                "--event-args",
                &rows[0].args(None),
                "--close-args",
                "",
            ],
        );
        assert!(!run.status.success());
        assert_eq!(
            states(&run),
            [
                rows[0].next().text("open"),
                rows[1].next().text("event"),
                rows[2].next().text("event")
            ]
        );
        assert_marker_bits(&run, rows[2].next().marker);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"));
    }
    // The flushed native word probe, not buffered fatal CLI stdout, proves prefixes.
    healthy(project, output, trace);
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    assert!(fs::read(&container_path).unwrap() == container);
}

fn deliver(sources: [String; 4], trace: Trace, loop_limit: u64) {
    for source in sources {
        let (project, mode) = project_for_source_with_limits(&source, loop_limit, 100);
        assert!(mode.starts_with(&format!(
            "native-session-policy-aot-bundle:v1.{loop_limit}.100."
        )));
        project.build(None);
        let output = project.0.join("build");
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let fresh =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        image(&project, &output, &mode, &binary, trace);
        assert!(project.build(None).contains("compile_cache: hit"));
        let cached =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        assert!(fresh.binary_blob == cached.binary_blob);
        assert!(fresh.host_objects == cached.host_objects);
        assert_eq!(fresh.packaging_mode, cached.packaging_mode);
        assert_eq!(fresh.envelope, cached.envelope);
        assert_eq!(fresh.lifecycle, cached.lifecycle);
        image(&project, &output, &mode, &binary, trace);
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
            image(&project, &restored, &mode, &binary, trace);
            assert!(
                fs::read(&artifact).unwrap() == bytes,
                "restore changed input artifact"
            );
        }
        for path in ["main.ns", "nuis.toml", ".nuis"] {
            assert!(!project.0.join(path).exists());
        }
    }
}

#[test]
fn state_loop_build_cache_and_two_source_free_restores_keep_dynamic_carry() {
    deliver(
        cases::source_variants(),
        cases::Invocation::trace,
        cases::LOOP_LIMIT,
    );
}

#[test]
fn state_loop_effects_build_cache_and_two_source_free_restores_keep_selected_prints() {
    deliver(
        effects::source_variants(),
        effects::trace,
        cases::LOOP_LIMIT,
    );
}

#[test]
fn state_loop_statements_build_cache_and_two_source_free_restores_keep_direct_prints() {
    deliver(
        statements::source_variants(),
        statements::trace,
        statements::LOOP_LIMIT,
    );
}
