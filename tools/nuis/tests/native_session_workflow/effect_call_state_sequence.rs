use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_sequence_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path) {
    let mut routes = std::collections::BTreeSet::new();
    for (index, (marker, poisoned)) in cases::CLI_MARKERS
        .into_iter()
        .zip([false, true, false, true])
        .enumerate()
    {
        let open = cases::Case {
            seed: 8,
            gate: false,
            second: index >= 2,
            flag: false,
            marker,
        };
        let rows = cases::sequence(open, poisoned);
        for row in &rows {
            routes.insert((row.route(), row.second_route()));
        }
        let mut args = vec![
            "--native-session".to_owned(),
            "counter".to_owned(),
            "--open-args".to_owned(),
            rows[0].args(None),
        ];
        let mut expected = vec![rows[0].next().text("open")];
        let mut trace = rows[0].trace(false);
        for call in &rows[1..] {
            args.extend(["--event-args".to_owned(), call.args(None)]);
            expected.push(call.next().text("event"));
            trace.extend(call.trace(false));
        }
        if !poisoned {
            for pair in rows[1..].windows(2) {
                assert_eq!(pair[0].args(None), pair[1].args(None));
            }
            assert!(rows[1..].windows(2).any(
                |p| (p[0].route(), p[0].second_route()) != (p[1].route(), p[1].second_route())
            ));
        }
        args.extend(["--close-args".to_owned(), String::new()]);
        expected.push(rows.last().unwrap().next().text("close"));
        assert_eq!(expected.len(), 8);
        let run = success(project.command(
            "run-artifact",
            output,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        ));
        assert_eq!(states(&run), expected);
        assert_marker_bits(&run, marker);
        assert_eq!(
            String::from_utf8(run.stdout).unwrap(),
            trace
                .into_iter()
                .map(|n| format!("{n}\n"))
                .collect::<String>()
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    }
    assert_eq!(routes.len(), 4);
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
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    healthy(project, output);
    oversized_script(project, output);
    let rows = cases::failure_prefix();
    assert_ne!(rows[0].next().result, rows[2].next().result);
    for (bad, slot) in cases::failures()
        .into_iter()
        .filter(|(_, slot)| matches!(slot, 6 | 7 | 8))
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
                &bad.args(Some(slot)),
                "--event-args",
                &bad.args(None),
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
                rows[2].next().text("event"),
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
    // Only the flushed probe establishes fatal-process partial effect prefixes.
    healthy(project, output);
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    assert!(fs::read(&container_path).unwrap() == container);
}
#[test]
fn state_sequence_build_cache_and_two_source_free_restores_preserve_sequential_handoff() {
    assert_eq!(cases::failures().len(), 17);
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
