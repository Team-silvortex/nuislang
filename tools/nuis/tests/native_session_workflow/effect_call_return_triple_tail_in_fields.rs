use super::*;
#[path = "../../../nuisc/tests/native_application_bridge/return_triple_tail_in_fields_oracle.rs"]
mod oracle;
use oracle::{Case, SOURCE};

#[test]
fn return_triple_tail_in_fields_cli_dead_suffix_and_source_arm_gate_proofs_stay_zero() {
    assert_eq!(oracle::cases().len(), 80);
    assert_eq!(oracle::cli_cases().len(), 16);
    assert_eq!(oracle::selected().len(), 24);
    let cases = oracle::cli_cases();
    let exited = cases.iter().filter(|c| c.exited).collect::<Vec<_>>();
    for c in &exited {
        for index in [5, 7, 8, 10, 14, 15, 16, 17, 18, 20, 21, 22, 23] {
            assert_eq!(c.words()[index + 2], 0);
        }
    }
    assert!(exited.iter().any(|c| c.words()[21] == 0));
    assert!(cases.iter().any(|c| c.first
        && !c.second
        && c.bool_seed != c.peer_seed
        && !c.exited
        && c.words()[11] == 0));
    let witness = cases
        .iter()
        .find(|c| c.first && !c.second && c.bool_seed && !c.peer_seed)
        .unwrap();
    assert_ne!(witness.before_tail.unwrap().3, witness.fields.unwrap().3);
    assert_eq!(witness.fields.unwrap().5, 0);
    assert_eq!(witness.before_tail.unwrap().4, 1);
    assert_eq!(witness.fields.unwrap().4, 0);
    assert_eq!(witness.before_tail.unwrap().1, i64::from(i32::MIN));
    assert_eq!(witness.fields.unwrap().1, i64::from(i32::MAX));
    assert_eq!(witness.owner, witness.seed);
    assert!(witness.result);
    assert!(witness.trace.contains(&61));
    for profile in [false, true] {
        assert_eq!(oracle::native_cases(profile).len(), 80);
        assert_eq!(oracle::native_selected(profile).len(), 24);
    }
}
fn healthy(project: &Project, output: &Path, opened: &Case) {
    let observed = opened.observed();
    if opened.trace.contains(&97) {
        assert_eq!(opened.before_failure(19), observed[..observed.len() - 3]);
    }
    assert_eq!(
        opened.observed().len(),
        opened.trace.len()
            + usize::from(opened.trace.contains(&97))
            + usize::from(opened.fields.is_some()) * 6
    );
    let previous = opened.previous();
    let run = success(project.command(
        "run-artifact",
        output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            &opened.open_args(),
            "--event-args",
            &previous.values().join(","),
            "--close-args",
            "",
        ],
    ));
    assert_eq!(
        states(&run),
        [
            opened.state_text("open"),
            previous.state_text("event"),
            previous.state_text("close")
        ]
    );
    let expected = opened
        .trace
        .iter()
        .chain(&previous.trace)
        .map(|v| format!("{v}\n"))
        .collect::<String>();
    assert_eq!(String::from_utf8(run.stdout).unwrap(), expected);
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
    for opened in oracle::cli_cases() {
        healthy(project, output, &opened);
    }
    for (index, opened) in oracle::selected().iter().enumerate() {
        let previous = opened.previous();
        assert_ne!(opened.state_words(), previous.state_words());
        let run = project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &opened.open_args(),
                "--event-args",
                &previous.values().join(","),
                "--event-args",
                &opened.bad_values(index).join(","),
                "--event-args",
                &opened.values().join(","),
                "--close-args",
                "",
            ],
        );
        assert!(!run.status.success());
        assert_eq!(
            states(&run),
            [opened.state_text("open"), previous.state_text("event")]
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"), "{stderr}");
        // Fatal stdout may be buffered. The separately flushed native probe
        // proves partial order; the CLI receipt proves publication boundaries.
        healthy(project, output, opened);
    }
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
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
fn return_triple_tail_in_fields_build_cache_restore_exact_outcomes_and_event_failure_publication() {
    for source in [SOURCE.to_owned(), oracle::mirror_source()] {
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
        let artifact_bytes = fs::read(&artifact).unwrap();
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
                fs::read(&artifact).unwrap() == artifact_bytes,
                "restore changed input container"
            );
        }
        for path in ["main.ns", "nuis.toml", ".nuis"] {
            assert!(!project.0.join(path).exists());
        }
    }
}
