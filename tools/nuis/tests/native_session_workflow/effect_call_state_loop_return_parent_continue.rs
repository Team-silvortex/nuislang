use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_return_parent_continue_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path, variant: cases::Variant, full: bool) {
    let mut profiles = vec![];
    for ((gain, marker), flag) in cases::CLI_WORDS.into_iter().zip([false, true]) {
        let mut call = cases::base(variant);
        call.input = cases::State {
            result: flag,
            gain,
            marker,
            ..call.input
        };
        call.select_first(true);
        profiles.push(cases::sequence(call));
    }
    profiles.push(cases::inactive_sequence(variant, 0, i64::MAX));
    profiles.push(cases::inactive_sequence(variant, -1, i64::MIN));
    profiles.push(cases::sequence(cases::late(variant)));
    profiles.push(cases::sequence(cases::child_continued(variant)));
    if !full {
        profiles.truncate(1);
    }
    for calls in profiles {
        let mut args = vec![
            "--native-session".to_owned(),
            "counter".to_owned(),
            "--open-args".to_owned(),
            calls[0].args(),
        ];
        let mut expected = vec![calls[0].next().text("open")];
        let mut printed = calls[0].trace(false);
        for call in &calls[1..] {
            args.extend(["--event-args".to_owned(), call.args()]);
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
        for (actual, call) in states(&run).iter().zip(calls.iter().chain(calls.last())) {
            let expected = call.next();
            let gain = actual
                .split(", gain: ")
                .nth(1)
                .unwrap()
                .split("f32,")
                .next()
                .unwrap();
            let marker = actual
                .split(", marker: ")
                .nth(1)
                .unwrap()
                .strip_suffix("f64}")
                .unwrap();
            assert_eq!(gain.parse::<f32>().unwrap().to_bits(), expected.gain);
            assert_eq!(marker.parse::<f64>().unwrap().to_bits(), expected.marker);
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
fn image(project: &Project, output: &Path, mode: &str, binary: &[u8], variant: cases::Variant) {
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    let container = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    healthy(project, output, variant, true);
    let prefix = cases::prefix(variant);
    for failing in cases::failures(variant).into_iter().take(4) {
        let failing = cases::after_prefix(failing);
        let run = project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &prefix[0].args(),
                "--event-args",
                &prefix[1].args(),
                "--event-args",
                &prefix[2].args(),
                "--event-args",
                &failing.args(),
                "--event-args",
                &prefix[0].args(),
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
        // Abort may discard buffered debug stdout. The native probe checks
        // flushed effect prefixes; this route checks publication and completion.
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"));
    }
    healthy(project, output, variant, false);
    assert!(fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary);
    assert!(fs::read(output.join("nuis.compiled.artifact")).unwrap() == container);
}

#[test]
fn state_loop_return_parent_continue_build_cache_and_source_free_restores_preserve_exact_payloads()
{
    for variant in cases::cli_variants() {
        eprintln!("parent_continue_delivery fresh: {variant:?}");
        let (project, mode) =
            project_for_source_with_limits(&cases::source(variant), cases::LOOP_LIMIT, 100);
        project.build(None);
        let output = project.0.join("build");
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let fresh =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        image(&project, &output, &mode, &binary, variant);
        eprintln!("parent_continue_delivery cache: {variant:?}");
        assert!(project.build(None).contains("compile_cache: hit"));
        let cached =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        assert!(fresh.binary_blob == cached.binary_blob);
        assert!(fresh.host_objects == cached.host_objects);
        assert_eq!(fresh.envelope, cached.envelope);
        assert_eq!(fresh.lifecycle, cached.lifecycle);
        assert_eq!(fresh.packaging_mode, cached.packaging_mode);
        image(&project, &output, &mode, &binary, variant);
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
        for restore in 0..2 {
            eprintln!("parent_continue_delivery restore {restore}: {variant:?}");
            success(project.command(
                "materialize-artifact",
                &artifact,
                &[restored.to_str().unwrap()],
            ));
            image(&project, &restored, &mode, &binary, variant);
            assert!(fs::read(&artifact).unwrap() == bytes);
        }
        for file in ["main.ns", "nuis.toml", ".nuis"] {
            assert!(!project.0.join(file).exists());
        }
    }
}
