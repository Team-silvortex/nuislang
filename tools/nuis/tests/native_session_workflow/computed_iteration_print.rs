use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/computed_iteration_print_cases.rs"]
mod cases;

fn run(project: &Project, output: &Path, rows: &[cases::Case]) -> Output {
    let mut args = vec![
        "--native-session".to_owned(),
        "counter".into(),
        "--open-args".into(),
        rows[0].args(),
    ];
    for row in &rows[1..] {
        args.extend(["--event-args".into(), row.args()]);
    }
    args.extend(["--close-args".into(), String::new()]);
    project.command(
        "run-artifact",
        output,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

fn expected(rows: &[cases::Case], close: bool) -> (Vec<String>, String) {
    let mut previous = None;
    let mut states = Vec::new();
    let mut trace = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let (next, printed) = row.execute(previous).unwrap();
        previous = Some(next);
        states.push(format!(
            "{}:State{{total: {next}}}",
            if i == 0 { "open" } else { "event" }
        ));
        trace.extend(printed);
    }
    if close {
        states.push(format!("close:State{{total: {}}}", previous.unwrap()));
    }
    (
        states,
        trace.into_iter().map(|v| format!("{v}\n")).collect(),
    )
}

fn image(project: &Project, output: &Path, binary: &[u8], mode: &str) {
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert_eq!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap(),
        binary
    );
    let container = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    let healthy = || {
        let rows = cases::healthy();
        let (state, trace) = expected(&rows, true);
        let output = success(run(project, output, &rows));
        assert_eq!(states(&output), state);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), trace);
        assert!(String::from_utf8_lossy(&output.stderr).contains("native_session_completed=1"));
    };
    healthy();
    for bad in cases::failures() {
        let mut rows = cases::prefix();
        let (state, mut trace) = expected(&rows, false);
        trace.extend(
            bad.execute(Some(cases::LAST_STATE))
                .unwrap_err()
                .into_iter()
                .map(|v| format!("{v}\n")),
        );
        rows.push(bad);
        let failed = run(project, output, &rows);
        assert!(!failed.status.success());
        assert_eq!(states(&failed), state);
        // A trapping child need not flush stdout. The direct native probe
        // flushes each print and proves the exact prefix; pin any flushed bytes.
        assert!(trace.starts_with(&String::from_utf8(failed.stdout).unwrap()));
        let stderr = String::from_utf8_lossy(&failed.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"));
    }
    healthy();
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
fn computed_iteration_print_fresh_cache_and_source_free_delivery_keep_selected_checks() {
    let (project, mode) =
        project_for_source_with_i64_prints(cases::SOURCE, cases::LOOP_LIMIT, cases::ENTRY_LIMIT);
    eprintln!("computed_iteration_print_delivery fresh");
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    let fresh =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    image(&project, &output, &binary, &mode);
    eprintln!("computed_iteration_print_delivery cache");
    assert!(project.build(None).contains("compile_cache: hit"));
    let cached =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    assert_eq!(fresh.binary_blob, cached.binary_blob);
    assert!(fresh.host_objects == cached.host_objects);
    assert_eq!(fresh.envelope, cached.envelope);
    assert_eq!(fresh.lifecycle, cached.lifecycle);
    assert_eq!(fresh.packaging_mode, cached.packaging_mode);
    image(&project, &output, &binary, &mode);
    let artifact = project.0.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    let bytes = fs::read(&artifact).unwrap();
    fs::remove_dir_all(&output).unwrap();
    for name in ["main.ns", "nuis.toml"] {
        fs::remove_file(project.0.join(name)).unwrap();
    }
    fs::remove_dir_all(project.0.join(".nuis")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    for i in 0..2 {
        eprintln!("computed_iteration_print_delivery restore {i}");
        let restored = project.0.join(format!("restored-{i}"));
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        image(&project, &restored, &binary, &mode);
        assert_eq!(fs::read(&artifact).unwrap(), bytes);
    }
    for name in ["main.ns", "nuis.toml", ".nuis"] {
        assert!(!project.0.join(name).exists());
    }
}
