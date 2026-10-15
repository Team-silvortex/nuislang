use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/return_stateful_fields_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path) {
    let rows = cases::cases();
    for (rows, marker) in rows.chunks(4).zip(cases::CLI_MARKERS) {
        let rows = rows
            .iter()
            .map(|c| cases::Case { marker, ..*c })
            .collect::<Vec<_>>();
        healthy_sequence(project, output, &rows);
    }
}

fn healthy_sequence(project: &Project, output: &Path, rows: &[cases::Case]) {
    let open = rows[0];
    let mut args = vec![
        "--native-session".to_owned(),
        "counter".to_owned(),
        "--open-args".to_owned(),
        open.args(None),
    ];
    let mut previous = open.initial();
    let mut expected = vec![previous.text("open")];
    let mut trace = open.trace(open.flag, previous.total, false);
    let mut seen = std::collections::BTreeMap::new();
    let mut repeated_input_changes_result = false;
    for event in rows.iter().copied().flat_map(cases::Case::events) {
        let input = event.args(None);
        args.extend(["--event-args".to_owned(), input.clone()]);
        let next = event.step(previous);
        trace.extend(event.trace(previous.result, next.total, false));
        previous = next;
        expected.push(previous.text("event"));
        if let Some(old) = seen.insert(input, previous.result) {
            repeated_input_changes_result |= old != previous.result;
        }
    }
    assert_eq!(expected.len(), 21);
    assert!(repeated_input_changes_result);
    args.extend(["--close-args".to_owned(), String::new()]);
    expected.push(previous.text("close"));
    let run = success(project.command(
        "run-artifact",
        output,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    ));
    assert_eq!(states(&run), expected);
    assert_marker_bits(&run, open.marker);
    assert_eq!(
        String::from_utf8(run.stdout).unwrap(),
        trace
            .into_iter()
            .map(|n| format!("{n}\n"))
            .collect::<String>()
    );
    assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
}

fn oversized_script(project: &Project, output: &Path) {
    let input = cases::identity().args(None);
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

fn assert_marker_bits(run: &Output, expected: u64) {
    assert!(f64::from_bits(expected).is_finite());
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

fn image(project: &Project, output: &Path, mode: &str, binary: &[u8]) {
    let container_path = output.join("nuis.compiled.artifact");
    let container = fs::read(&container_path).unwrap();
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary,
        "image payload changed"
    );
    healthy(project, output);
    oversized_script(project, output);
    let [open, first, second] = cases::failure_prefix();
    let opened = open.initial();
    let first_result = first.step(opened);
    let last = second.step(first_result);
    assert!(!opened.result && last.result);
    assert_eq!(last.total, opened.total + 2);
    for (bad, slot) in cases::failures()
        .into_iter()
        .filter(|(_, slot)| [7, 9, 10, 11, 13].contains(slot))
    {
        let run = project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &open.args(None),
                "--event-args",
                &first.args(None),
                "--event-args",
                &second.args(None),
                "--event-args",
                &bad.args(Some(slot)),
                "--event-args",
                &open.args(None),
                "--close-args",
                "",
            ],
        );
        assert!(!run.status.success());
        assert_eq!(
            states(&run),
            [
                opened.text("open"),
                first_result.text("event"),
                last.text("event")
            ]
        );
        assert_marker_bits(&run, last.marker);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"));
    }
    // Flushed native probes, not buffered fatal CLI stdout, prove partial effects.
    healthy(project, output);
    assert!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary,
        "execution changed payload"
    );
    assert!(
        fs::read(&container_path).unwrap() == container,
        "execution changed container"
    );
}

#[test]
fn return_stateful_fields_build_cache_and_two_source_free_restores_consume_latest_state() {
    assert_eq!(cases::failures().len(), 9);
    for source in cases::source_variants() {
        let (project, mode) = project_for_source(&source);
        project.build(None);
        let output = project.0.join("build");
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let fresh =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        image(&project, &output, &mode, &binary);
        assert!(project.build(None).contains("compile_cache: hit"));
        let cached =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        assert!(
            fresh.binary_blob == cached.binary_blob,
            "cache changed executable"
        );
        assert_eq!(fresh.packaging_mode, cached.packaging_mode);
        assert_eq!(fresh.envelope, cached.envelope);
        assert_eq!(fresh.lifecycle, cached.lifecycle);
        assert!(
            fresh.host_objects == cached.host_objects,
            "cache changed objects"
        );
        image(&project, &output, &mode, &binary);
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
            image(&project, &restored, &mode, &binary);
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
