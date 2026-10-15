use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/return_plain_const_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path) {
    let rows = cases::cases();
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
    let rows = cases::cases();
    let opened = rows.iter().find(|c| c.result() == 1).unwrap();
    let previous = rows.iter().find(|c| c.result() == 0).unwrap();
    for (bad, slot) in cases::failures()
        .into_iter()
        .filter(|(_, slot)| [7, 9, 10, 11].contains(slot))
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
    // The flushed native probe, not buffered fatal CLI stdout, proves partial effects.
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
fn return_plain_const_build_cache_and_two_source_free_restores() {
    assert_eq!(cases::cases().len(), 16);
    assert_eq!(cases::failures().len(), 8);
    assert_eq!(
        cases::source_variants()
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    for line in cases::SOURCE
        .lines()
        .filter(|l| l.trim_start().starts_with("const "))
    {
        assert!(!line.contains("&&") && !line.contains("||"));
    }
    // Omitting the selected ancestor transformation must change the answer.
    for seed in [8, 9] {
        assert!(cases::cases()
            .into_iter()
            .any(|c| c.seed == seed && c.gate && c.first && c.result() != i64::from(c.seed == 9)));
    }
    for case in cases::cases() {
        let words = case.words();
        assert_eq!(words.len(), 12);
        assert_eq!(words[9], i64::from(case.gate && case.first));
        assert_eq!(words[10], i64::from(!case.gate && case.first));
        assert_eq!(words[11], i64::from(case.gate));
    }
    for flag in [false, true] {
        assert!(cases::cases()
            .into_iter()
            .any(|c| c.gate && !c.first && c.flag == flag && c.result() != i64::from(flag)));
    }
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
