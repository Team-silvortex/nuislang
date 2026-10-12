use super::*;

#[path = "../../../nuisc/tests/native_application_bridge/private_event_cases.rs"]
mod cases;
use cases::{Case, Family};

fn healthy(project: &Project, output: &Path, opened: &Case, previous: &Case) {
    let run = success(project.command(
        "run-artifact",
        output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            &opened.open_args(),
            "--event-args",
            &previous.values.join(","),
            "--close-args",
            "",
        ],
    ));
    assert_eq!(
        states(&run),
        [
            opened.state_text("open"),
            previous.state_text("event"),
            previous.state_text("close"),
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

fn image(project: &Project, output: &Path, family: Family, mode: &str, binary: &[u8]) {
    let container_path = output.join("nuis.compiled.artifact");
    let container = fs::read(&container_path).unwrap();
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary,
        "executable payload changed at {}",
        output.display()
    );
    for (index, opened) in family.selected().iter().enumerate() {
        let previous = opened.previous();
        assert_ne!(opened.state_words(), previous.state_words());
        assert_eq!(opened.open_words().len(), opened.values.len() + 1);
        let bad = opened.bad_values(index);
        let run = project.command(
            "run-artifact",
            output,
            &[
                "--native-session",
                "counter",
                "--open-args",
                &opened.open_args(),
                "--event-args",
                &previous.values.join(","),
                "--event-args",
                &bad.join(","),
                "--event-args",
                &opened.values.join(","),
                "--close-args",
                "",
            ],
        );
        assert!(!run.status.success(), "must reject without fallback");
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
        // A fatal trap may lose buffered stdout. Exact partial execution order
        // is checked by the separately flushed native probe, not by absence here.
        healthy(project, output, opened, &previous);
    }
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary,
        "execution changed executable payload at {}",
        output.display()
    );
    assert!(
        fs::read(&container_path).unwrap() == container,
        "execution changed its container"
    );
}

fn check(family: Family) {
    let (project, mode) = project_for_source(&family.source());
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    let fresh =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    image(&project, &output, family, &mode, &binary);
    assert!(project.build(None).contains("compile_cache: hit"));
    let cached =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    // The build manifest legitimately changes miss to hit. Compare executable
    // payloads and contracts, not the entire diagnostic-bearing container.
    assert!(
        fresh.binary_blob == cached.binary_blob,
        "cache changed executable bytes"
    );
    assert_eq!(fresh.packaging_mode, cached.packaging_mode);
    assert_eq!(fresh.envelope, cached.envelope);
    assert_eq!(fresh.lifecycle, cached.lifecycle);
    assert!(
        fresh.host_objects == cached.host_objects,
        "cache changed native object bytes"
    );
    let artifact_bytes = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    image(&project, &output, family, &mode, &binary);
    let artifact = project.0.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(&output).unwrap();
    for file in ["main.ns", "nuis.toml"] {
        fs::remove_file(project.0.join(file)).unwrap();
    }
    fs::remove_dir_all(project.0.join(".nuis")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    for _ in 0..3 {
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        image(&project, &restored, family, &mode, &binary);
        assert!(
            fs::read(&artifact).unwrap() == artifact_bytes,
            "restoration changed input container"
        );
    }
    for path in ["main.ns", "nuis.toml", ".nuis"] {
        assert!(!project.0.join(path).exists());
    }
}

#[test]
fn private_event_failure_bool_preserves_last_publication_cache_and_source_free_identity() {
    check(Family::Bool);
}
#[test]
fn private_event_failure_i32_preserves_last_publication_cache_and_source_free_identity() {
    check(Family::I32);
}
#[test]
fn private_event_failure_i64_preserves_last_publication_cache_and_source_free_identity() {
    check(Family::I64);
}
#[test]
fn private_event_failure_f32_preserves_last_publication_cache_and_source_free_identity() {
    check(Family::F32);
}
#[test]
fn private_event_failure_f64_preserves_last_publication_cache_and_source_free_identity() {
    check(Family::F64);
}
