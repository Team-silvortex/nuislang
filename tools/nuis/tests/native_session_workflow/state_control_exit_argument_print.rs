use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_control_exit_argument_print_cases.rs"]
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

fn expected(rows: &[cases::Case], close: bool) -> (Vec<String>, String, cases::State) {
    let mut previous = None;
    let mut states = Vec::new();
    let mut trace = Vec::new();
    let state_text = |phase: &str, state: cases::State| {
        format!(
            "{phase}:State{{total: {}{}, control: {}}}",
            rows[0].kind().decimal(state[0]),
            rows[0].kind().ty(),
            state[1]
        )
    };
    for (i, row) in rows.iter().enumerate() {
        let (next, printed) = row.execute(previous, false).unwrap();
        previous = Some(next);
        states.push(state_text(if i == 0 { "open" } else { "event" }, next));
        trace.extend(printed);
    }
    let previous = previous.unwrap();
    if close {
        states.push(state_text("close", previous));
    }
    (
        states,
        trace.into_iter().map(|word| format!("{word}\n")).collect(),
        previous,
    )
}

fn image(project: &Project, output: &Path, binary: &[u8], mode: &str, kind: cases::Kind) {
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert_eq!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap(),
        binary
    );
    let container = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    let healthy = || {
        for rows in cases::cli_sequences(kind) {
            let (state, trace, _) = expected(&rows, true);
            let run = success(run(project, output, &rows));
            assert_eq!(states(&run), state);
            assert_eq!(String::from_utf8(run.stdout).unwrap(), trace);
            assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
        }
    };
    healthy();
    for bad in cases::failures(kind) {
        let mut rows = cases::prefix(kind);
        let (state, mut trace, previous) = expected(&rows, false);
        trace.extend(
            bad.execute(Some(previous), false)
                .unwrap_err()
                .into_iter()
                .map(|word| format!("{word}\n")),
        );
        rows.push(bad);
        let failed = run(project, output, &rows);
        assert!(!failed.status.success());
        assert_eq!(states(&failed), state);
        // Direct native probes flush every print; a trapping CLI child may buffer output.
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
fn state_control_exit_early_break_delivery_counts_completed_not_reserved_updates() {
    for kind in cases::KINDS {
        let (project, mode) = project_for_source_with_i64_prints(
            &cases::SOURCE.replace("f64", kind.ty()),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        let rows = cases::early_sequences(kind).remove(0);
        let (state, trace, _) = expected(&rows, true);
        let output = project.0.join("build");
        project.build(None);
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let inspect = |output: &Path| {
            let report =
                nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml"))
                    .unwrap();
            assert_eq!(report.packaging_mode, mode);
            assert_eq!(
                fs::read(output.join(&report.artifact_binary_name)).unwrap(),
                binary
            );
            let result = success(run(&project, output, &rows));
            assert_eq!(states(&result), state);
            assert_eq!(String::from_utf8(result.stdout).unwrap(), trace);
            assert!(String::from_utf8_lossy(&result.stderr).contains("native_session_completed=1"));
        };
        inspect(&output);
        assert!(project.build(None).contains("compile_cache: hit"));
        inspect(&output);
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
            let restored = project.0.join(format!("early-restored-{i}"));
            success(project.command(
                "materialize-artifact",
                &artifact,
                &[restored.to_str().unwrap()],
            ));
            inspect(&restored);
            assert_eq!(fs::read(&artifact).unwrap(), bytes);
        }
        for name in ["main.ns", "nuis.toml", ".nuis"] {
            assert!(!project.0.join(name).exists());
        }
    }
}

#[test]
fn state_control_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words() {
    for kind in cases::KINDS {
        let (project, mode) = project_for_source_with_i64_prints(
            &cases::SOURCE.replace("f64", kind.ty()),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        eprintln!("state_control_exit {kind:?} fresh");
        project.build(None);
        let output = project.0.join("build");
        let report =
            nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
        let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
        let artifact_path = output.join("nuis.compiled.artifact");
        let fresh = nuisc::aot::parse_nuis_compiled_artifact(&artifact_path).unwrap();
        image(&project, &output, &binary, &mode, kind);
        eprintln!("state_control_exit {kind:?} cache");
        assert!(project.build(None).contains("compile_cache: hit"));
        let cached = nuisc::aot::parse_nuis_compiled_artifact(&artifact_path).unwrap();
        assert_eq!(fresh.binary_blob, cached.binary_blob);
        assert!(fresh.host_objects == cached.host_objects);
        assert_eq!(fresh.envelope, cached.envelope);
        assert_eq!(fresh.lifecycle, cached.lifecycle);
        assert_eq!(fresh.packaging_mode, cached.packaging_mode);
        image(&project, &output, &binary, &mode, kind);
        let artifact = project.0.join("nuis.compiled.artifact");
        fs::copy(&artifact_path, &artifact).unwrap();
        let bytes = fs::read(&artifact).unwrap();
        fs::remove_dir_all(&output).unwrap();
        for name in ["main.ns", "nuis.toml"] {
            fs::remove_file(project.0.join(name)).unwrap();
        }
        fs::remove_dir_all(project.0.join(".nuis")).unwrap();
        success(project.command("verify-artifact", &artifact, &[]));
        for i in 0..2 {
            eprintln!("state_control_exit {kind:?} restore {i}");
            let restored = project.0.join(format!("restored-{i}"));
            success(project.command(
                "materialize-artifact",
                &artifact,
                &[restored.to_str().unwrap()],
            ));
            image(&project, &restored, &binary, &mode, kind);
            assert_eq!(fs::read(&artifact).unwrap(), bytes);
        }
        for name in ["main.ns", "nuis.toml", ".nuis"] {
            assert!(!project.0.join(name).exists());
        }
    }
}
