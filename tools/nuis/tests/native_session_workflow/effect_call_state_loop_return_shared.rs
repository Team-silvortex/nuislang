use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path, variant: cases::Variant, kind: cases::SourceKind) {
    for open in [cases::base(variant), cases::continued_then_break(variant)] {
        let calls = cases::sequence(open);
        let mut args = vec![
            "--native-session".to_owned(),
            "counter".to_owned(),
            "--open-args".to_owned(),
            calls[0].args(),
        ];
        let mut expected = vec![calls[0].next().text("open")];
        let mut trace = kind.trace(calls[0].trace(false));
        for call in &calls[1..] {
            args.extend(["--event-args".to_owned(), call.args()]);
            expected.push(call.next().text("event"));
            trace.extend(kind.trace(call.trace(false)));
        }
        args.extend(["--close-args".to_owned(), String::new()]);
        expected.push(calls.last().unwrap().next().text("close"));
        let run = success(project.command(
            "run-artifact",
            output,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        ));
        assert_eq!(states(&run), expected);
        assert_eq!(
            String::from_utf8(run.stdout).unwrap(),
            trace
                .into_iter()
                .map(|v| format!("{v}\n"))
                .collect::<String>()
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    }
}

fn image(
    project: &Project,
    output: &Path,
    binary: &[u8],
    mode: &str,
    variant: cases::Variant,
    kind: cases::SourceKind,
) {
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    let container = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    assert_eq!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap(),
        binary
    );
    healthy(project, output, variant, kind);
    for mut bad in [cases::base(variant), cases::continued_then_break(variant)] {
        if bad.finish == 0 {
            bad.root = 0;
        } else {
            bad.finish = 0;
        }
        let bad = cases::after_prefix(bad);
        let prefix = cases::prefix(variant);
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
                &bad.args(),
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
    healthy(project, output, variant, kind);
    assert_eq!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap(),
        binary
    );
    assert_eq!(
        fs::read(output.join("nuis.compiled.artifact")).unwrap(),
        container
    );
}

#[test]
fn shared_return_storage_fresh_cache_and_source_free_delivery_keep_payloads_and_faults() {
    delivery(cases::SourceKind::Plain);
}

#[test]
fn shared_return_storage_prefix_fresh_cache_and_source_free_delivery_keeps_ordered_prints() {
    delivery(cases::SourceKind::Prefix);
}

#[test]
fn shared_return_storage_candidates_fresh_cache_and_source_free_delivery_keeps_state_and_faults() {
    delivery(cases::SourceKind::Candidates);
}

#[test]
fn shared_return_storage_reassignment_fresh_cache_and_source_free_delivery_keeps_writes_and_faults()
{
    delivery(cases::SourceKind::Reassigned);
}

fn delivery(kind: cases::SourceKind) {
    let variant = cases::variants()[0];
    eprintln!("shared_return_delivery fresh ({kind:?}): {variant:?}");
    let source = kind.source(variant);
    let (project, mode) = project_for_source_with_limits(&source, cases::LOOP_LIMIT, 100);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    let fresh =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    image(&project, &output, &binary, &mode, variant, kind);
    eprintln!("shared_return_delivery cache: {variant:?}");
    assert!(project.build(None).contains("compile_cache: hit"));
    let cached =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    assert_eq!(fresh.binary_blob, cached.binary_blob);
    assert!(fresh.host_objects == cached.host_objects);
    assert_eq!(fresh.envelope, cached.envelope);
    assert_eq!(fresh.lifecycle, cached.lifecycle);
    assert_eq!(fresh.packaging_mode, cached.packaging_mode);
    image(&project, &output, &binary, &mode, variant, kind);
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
        eprintln!("shared_return_delivery restore {restore}: {variant:?}");
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        image(&project, &restored, &binary, &mode, variant, kind);
        assert_eq!(fs::read(&artifact).unwrap(), bytes);
    }
    for file in ["main.ns", "nuis.toml", ".nuis"] {
        assert!(!project.0.join(file).exists());
    }
}
