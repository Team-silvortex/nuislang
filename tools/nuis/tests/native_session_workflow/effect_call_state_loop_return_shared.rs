use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path, variant: cases::Variant, kind: cases::SourceKind) {
    let mut opens = vec![cases::base(variant), cases::continued_then_break(variant)];
    if kind == cases::SourceKind::BranchBoundCalls {
        opens.push(cases::bound_calls::state_dependent(variant));
    }
    for open in opens {
        let calls = if kind == cases::SourceKind::BranchBoundCalls {
            cases::bound_calls::sequence(cases::sequence(open))
        } else {
            cases::sequence(open)
        };
        let mut args = vec![
            "--native-session".to_owned(),
            "counter".to_owned(),
            "--open-args".to_owned(),
            calls[0].args(),
        ];
        let mut expected = vec![kind.next(calls[0]).text("open")];
        let mut trace = kind.execute(calls[0], false, None).unwrap().1;
        for call in &calls[1..] {
            args.extend(["--event-args".to_owned(), call.args()]);
            expected.push(kind.next(*call).text("event"));
            trace.extend(kind.execute(*call, false, None).unwrap().1);
        }
        args.extend(["--close-args".to_owned(), String::new()]);
        expected.push(kind.next(*calls.last().unwrap()).text("close"));
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
    let mut failures = vec![cases::base(variant), cases::continued_then_break(variant)];
    for bad in &mut failures {
        if bad.finish == 0 {
            bad.root = 0;
        } else {
            bad.finish = 0;
        }
    }
    if matches!(
        kind,
        cases::SourceKind::HelperCalls
            | cases::SourceKind::LoopHelperCalls
            | cases::SourceKind::BranchHelperCalls
            | cases::SourceKind::BranchBoundCalls
    ) {
        for root in [-7, -8] {
            let mut bad = cases::base(variant);
            bad.root = root;
            failures.push(bad);
        }
    }
    if kind == cases::SourceKind::LoopHelperCalls {
        for callee in [false, true] {
            failures.push(cases::loop_calls::failure(variant, callee));
        }
    }
    if kind == cases::SourceKind::BranchHelperCalls {
        for callee in [false, true] {
            for odd in [false, true] {
                failures.push(cases::branch_calls::failure(variant, callee, odd));
            }
        }
    }
    if kind == cases::SourceKind::BranchBoundCalls {
        for stage in 0..4 {
            failures.push(cases::bound_calls::failure(variant, stage));
        }
    }
    for bad in failures {
        let prefix = if kind == cases::SourceKind::BranchBoundCalls {
            cases::bound_calls::failure_prefix(variant, bad.trips == i64::MIN && bad.divisor == -1)
        } else {
            cases::prefix(variant)
        };
        let bad = if kind == cases::SourceKind::BranchBoundCalls {
            cases::bound_calls::after_failure_prefix(bad, &prefix)
        } else {
            cases::after_prefix(bad)
        };
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
                kind.next(prefix[0]).text("open"),
                kind.next(prefix[1]).text("event"),
                kind.next(prefix[2]).text("event")
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

#[test]
fn shared_return_storage_helper_calls_fresh_cache_and_source_free_delivery_keeps_order_and_faults()
{
    delivery(cases::SourceKind::HelperCalls);
}

#[test]
fn shared_return_storage_loop_helper_calls_fresh_cache_and_source_free_delivery_keeps_exits_and_faults(
) {
    delivery(cases::SourceKind::LoopHelperCalls);
}

#[test]
fn shared_return_storage_branch_helper_calls_fresh_cache_and_source_free_delivery_keeps_selection()
{
    delivery(cases::SourceKind::BranchHelperCalls);
}

#[test]
fn branch_bound_calls_fresh_cache_and_source_free_delivery_keep_computed_work_and_faults() {
    delivery(cases::SourceKind::BranchBoundCalls);
}

fn delivery(kind: cases::SourceKind) {
    let variant = cases::variants()[0];
    eprintln!("shared_return_delivery fresh ({kind:?}): {variant:?}");
    let source = kind.source(variant);
    let (project, mode) =
        project_for_source_with_limits(&source, cases::LOOP_LIMIT, kind.helper_entry_limit());
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
