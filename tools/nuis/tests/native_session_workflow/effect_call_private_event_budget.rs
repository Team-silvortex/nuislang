use super::*;

#[path = "../../../nuisc/tests/native_application_bridge/private_event_budget_cases.rs"]
#[allow(dead_code)] // Exact private entry paths are checked in the flushed native probe.
mod cases;
use cases::{Case, Family};

fn stdout(cases: &[&Case]) -> String {
    cases
        .iter()
        .flat_map(|case| &case.trace)
        .map(|v| format!("{v}\n"))
        .collect()
}

fn image(project: &Project, output: &Path, family: Family, limit: u64, mode: &str, binary: &[u8]) {
    let container_path = output.join("nuis.compiled.artifact");
    let container = fs::read(&container_path).unwrap();
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary,
        "executable payload changed"
    );
    let [opened, previous, high] = cases::selected(family);
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
            &high.values.join(","),
            "--event-args",
            &high.values.join(","),
            "--close-args",
            "",
        ],
    );
    if limit == cases::limit(family) {
        success(run.clone());
        assert_eq!(
            states(&run),
            [
                opened.state_text("open"),
                previous.state_text("event"),
                high.state_text("event"),
                high.state_text("event"),
                high.state_text("close")
            ]
        );
        assert_eq!(
            String::from_utf8(run.stdout).unwrap(),
            stdout(&[&opened, &previous, &high, &high])
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    } else {
        assert!(!run.status.success(), "must trap without fallback");
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
        // Fatal stdout may be buffered. Exact work/order and unchanged callback
        // output are proved by the flushed native probe, not stdout absence here.
    }
    // Reenter the same image after either fatal or successful execution.
    let healthy = success(project.command(
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
        states(&healthy),
        [
            opened.state_text("open"),
            previous.state_text("event"),
            previous.state_text("close")
        ]
    );
    assert_eq!(
        String::from_utf8(healthy.stdout).unwrap(),
        stdout(&[&opened, &previous])
    );
    assert!(String::from_utf8_lossy(&healthy.stderr).contains("native_session_completed=1"));
    nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap() == binary,
        "execution changed executable payload"
    );
    assert!(
        fs::read(container_path).unwrap() == container,
        "execution changed container"
    );
}

fn check(family: Family) {
    for limit in [cases::limit(family) - 1, cases::limit(family)] {
        let (project, mode) = project_for_source_with_limit(&family.source(), limit);
        project.build(None);
        let output = project.0.join("build");
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let llvm =
            fs::read_to_string(output.join(format!("{}.ll", report.artifact_binary_name))).unwrap();
        assert_eq!(
            llvm.matches(&format!("store i64 {limit}, ptr %nuis_helper_entries"))
                .count(),
            3
        );
        assert!(llvm.contains("native_helper_entry_rejected"));
        assert!(!llvm.contains("native_loop_work_rejected"));
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let fresh =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        image(&project, &output, family, limit, &mode, &binary);
        assert!(project.build(None).contains("compile_cache: hit"));
        let cached =
            nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact"))
                .unwrap();
        // Manifest cache diagnostics can differ; executable/contracts must not.
        assert!(
            fresh.binary_blob == cached.binary_blob,
            "cache changed executable payload"
        );
        assert_eq!(fresh.packaging_mode, cached.packaging_mode);
        assert_eq!(fresh.envelope, cached.envelope);
        assert_eq!(fresh.lifecycle, cached.lifecycle);
        assert!(
            fresh.host_objects == cached.host_objects,
            "cache changed native object bytes"
        );
        image(&project, &output, family, limit, &mode, &binary);

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
            image(&project, &restored, family, limit, &mode, &binary);
            assert!(
                fs::read(&artifact).unwrap() == artifact_bytes,
                "restoration changed input artifact"
            );
        }
        for path in ["main.ns", "nuis.toml", ".nuis"] {
            assert!(!project.0.join(path).exists());
        }
    }
}

#[test]
fn private_event_budget_bool_cache_and_source_free_identity() {
    check(Family::Bool);
}
#[test]
fn private_event_budget_i32_cache_and_source_free_identity() {
    check(Family::I32);
}
#[test]
fn private_event_budget_i64_cache_and_source_free_identity() {
    check(Family::I64);
}
#[test]
fn private_event_budget_f32_cache_and_source_free_identity() {
    check(Family::F32);
}
#[test]
fn private_event_budget_f64_cache_and_source_free_identity() {
    check(Family::F64);
}
