use super::*;

#[path = "../../../nuisc/tests/native_application_bridge/return_float_owner_field_vector_oracle.rs"]
mod oracle;
use oracle::{Case, SOURCE};

#[test]
fn return_float_owner_field_vector_cli_stale_sensitive_inputs() {
    for ty in ["f32", "f64"] {
        for current in oracle::masked_cases(ty) {
            let stale = current.stale_sibling();
            assert_ne!(current.owner, stale.owner);
            assert_eq!(current.result, stale.result);
            assert_eq!(current.trace, stale.trace);
        }
        let cases = oracle::cli_cases(ty);
        for value in [-2.5, 1.5] {
            let current = cases
                .iter()
                .find(|case| {
                    case.first
                        && case.second
                        && case.small_seed == i32::MIN
                        && !case.bool_seed
                        && case.peer_seed
                        && case.float_seed == oracle::bits(ty, value)
                })
                .unwrap();
            let legacy = Case::new(
                ty,
                current.seed,
                current.small_seed,
                current.float_seed,
                current.bool_seed,
                current.peer_seed,
                current.first,
                current.second,
            );
            assert_eq!(legacy.child_ty, ty);
            assert_eq!(legacy.state_words(), current.state_words());
            assert_eq!(legacy.state_text("state"), current.state_text("state"));
            assert_eq!(legacy.open_args(), current.open_args());
            let stale = current.stale_sibling();
            assert_ne!(current.owner, stale.owner);
            assert_ne!(current.result, stale.result);
            assert_eq!(current.trace, stale.trace);
        }
    }
}

fn healthy(project: &Project, output: &Path, opened: &Case) {
    let observed = opened.observed();
    assert_eq!(opened.before_failure(14), observed[..observed.len() - 3]);
    assert_eq!(
        opened.observed().len(),
        opened.trace.len() + 1 + usize::from(opened.fields.is_some()) * 5
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

fn image(project: &Project, output: &Path, mode: &str, binary: &[u8], ty: &'static str) {
    let container_path = output.join("nuis.compiled.artifact");
    let container = fs::read(&container_path).unwrap();
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary,
        "image payload changed"
    );
    for opened in oracle::cli_cases(ty) {
        healthy(project, output, &opened);
    }
    for (index, opened) in oracle::selected(ty).iter().enumerate() {
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
fn return_float_owner_field_vector_build_cache_restore_exact_outcomes_and_event_failure_publication(
) {
    for ty in ["f32", "f64"] {
        let (project, mode) = project_for_source(&SOURCE.replace("f64", ty));
        project.build(None);
        let output = project.0.join("build");
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let fresh =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        image(&project, &output, &mode, &binary, ty);
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
        image(&project, &output, &mode, &binary, ty);
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
            image(&project, &restored, &mode, &binary, ty);
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
