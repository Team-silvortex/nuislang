use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/return_typed_exit_prefix_cases.rs"]
mod cases;

#[test]
fn return_typed_exit_prefix_cli_audits_lazy_proofs_and_current_value_witnesses() {
    assert_eq!(cases::cases().len(), 160);
    assert_eq!(cases::cli_cases().len(), 16);
    assert_eq!(cases::delivery_cases().len(), 22);
    assert_eq!(cases::failures().len(), 12);
    for case in cases::cases() {
        let words = case.words();
        assert_eq!(words.len(), 21);
        assert_eq!(words[11], i64::from(case.take && case.pick));
        assert_eq!(words[12], i64::from(case.take && !case.pick));
        for slot in 14..=18 {
            assert_eq!(words[slot], i64::from(case.take && case.inner));
        }
        assert_eq!(words[20], i64::from(!case.take));
    }
    for field in 0..6 {
        assert!(cases::witnesses().iter().any(|c| {
            let current = c.fields();
            let mut stale = current;
            stale[field] = c.before()[field];
            cases::answer(current) != cases::answer(stale)
        }));
    }
}

fn healthy(project: &Project, output: &Path) {
    let rows = cases::delivery_cases();
    let mut args = vec![
        "--native-session".to_owned(),
        "counter".to_owned(),
        "--open-args".to_owned(),
        rows[0].args(None),
    ];
    for case in rows.iter().skip(1) {
        args.extend(["--event-args".to_owned(), case.args(None)]);
    }
    args.extend(["--close-args".to_owned(), String::new()]);
    let run = success(project.command(
        "run-artifact",
        output,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    ));
    let mut expected = vec![rows[0].state("open")];
    expected.extend(rows.iter().skip(1).map(|c| c.state("event")));
    expected.push(rows.last().unwrap().state("close"));
    assert_eq!(states(&run), expected);
    let stdout = rows
        .iter()
        .flat_map(|c| c.trace(false))
        .map(|n| format!("{n}\n"))
        .collect::<String>();
    assert_eq!(String::from_utf8(run.stdout).unwrap(), stdout);
    assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
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
    let rows = cases::cli_cases();
    let opened = rows.iter().find(|c| c.result() == 1).unwrap();
    let previous = rows.iter().find(|c| c.result() == 0).unwrap();
    for (bad, slot, _) in cases::failures()
        .into_iter()
        .filter(|(_, _, stage)| [20, 30, 28].contains(stage))
    {
        let run = project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &opened.args(None),
                "--event-args",
                &previous.args(None),
                "--event-args",
                &bad.args(Some(slot)),
                "--event-args",
                &opened.args(None),
                "--close-args",
                "",
            ],
        );
        assert!(!run.status.success());
        assert_eq!(
            states(&run),
            [opened.state("open"), previous.state("event")]
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"));
    }
    // Fatal stdout can be buffered; the separate flushed probe proves partial effects.
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
fn return_typed_exit_prefix_build_cache_and_two_source_free_restores() {
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
