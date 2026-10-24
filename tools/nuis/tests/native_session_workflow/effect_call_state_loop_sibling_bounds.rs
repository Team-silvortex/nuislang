use super::*;
#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs"]
mod cases;

fn healthy(project: &Project, output: &Path, variant: cases::Variant) {
    let mut open = cases::base(variant);
    open.selected = false;
    let mut calls = vec![open];
    for mode in 0..=9 {
        let mut call = cases::base(variant);
        call.mode = mode;
        if matches!(mode, 1 | 4 | 5) {
            call.second = 0;
        }
        calls.push(call);
        if matches!(mode, 1 | 4 | 5) {
            call.second = -1;
            calls.push(call);
        }
    }
    if variant.snapshot_returns {
        for mode in [10, 11] {
            let mut call = cases::base(variant);
            call.mode = mode;
            call.trips = 5;
            call.divisor = 2;
            calls.push(call);
        }
    }
    if variant.payload_calls {
        for mode in cases::payload_fault_modes(variant) {
            let mut call = cases::base(variant);
            call.mode = mode;
            call.selected = false;
            calls.push(call);
            call.selected = true;
            call.trips = 0;
            calls.push(call);
        }
    }
    let calls = cases::sequence(calls);
    let mut args = vec![
        "--native-session".to_owned(),
        "counter".into(),
        "--open-args".into(),
        calls[0].args(),
    ];
    let mut expected = vec![calls[0].next().text("open")];
    let mut trace = calls[0].execute(false, None).unwrap().1;
    for call in &calls[1..] {
        args.extend(["--event-args".into(), call.args()]);
        expected.push(call.next().text("event"));
        trace.extend(call.execute(false, None).unwrap().1);
    }
    args.extend(["--close-args".into(), String::new()]);
    expected.push(calls.last().unwrap().next().text("close"));
    let run = success(project.command(
        "run-artifact",
        output,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    ));
    assert_eq!(states(&run), expected);
    assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    assert_eq!(
        String::from_utf8(run.stdout).unwrap(),
        trace
            .into_iter()
            .map(|w| format!("{w}\n"))
            .collect::<String>()
    );
}

fn image(project: &Project, output: &Path, binary: &[u8], mode: &str, variant: cases::Variant) {
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert_eq!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap(),
        binary
    );
    let container = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    healthy(project, output, variant);
    for (prefix, bad) in cases::failures(variant) {
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
    healthy(project, output, variant);
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
fn sibling_bounds_fresh_cache_and_source_free_delivery_keep_snapshot_and_exit_targets() {
    delivery(cases::variants()[0]);
}

#[test]
fn preheader_bounds_fresh_cache_and_source_free_delivery_keep_snapshot_across_backedges() {
    delivery(cases::preheader_variants()[0]);
}

#[test]
fn epoch_bounds_fresh_cache_and_source_free_delivery_keep_distinct_snapshot_identities() {
    delivery(cases::epoch_variants()[0]);
}

#[test]
fn snapshot_returns_fresh_cache_and_source_free_delivery_keep_complete_returned_records() {
    delivery(cases::snapshot_return_variants()[0]);
}

#[test]
fn payload_calls_fresh_cache_and_source_free_delivery_keep_ordered_return_failures() {
    delivery(cases::payload_call_variants()[0]);
}

#[test]
fn field_payload_fresh_cache_and_source_free_delivery_preserve_complete_publication() {
    delivery(cases::field_payload_variants()[0]);
}

#[test]
fn mixed_payload_fresh_cache_and_source_free_delivery_preserve_all_return_origins() {
    delivery(cases::mixed_payload_variants()[0]);
}

#[test]
fn projected_payload_fresh_cache_and_source_free_delivery_preserve_each_call_occurrence() {
    delivery(cases::projected_payload_variants()[0]);
}

#[test]
fn unprojected_payload_fresh_cache_and_source_free_delivery_keep_discarded_field_effects() {
    delivery(cases::unprojected_payload_variants()[0]);
}

#[test]
fn conditional_payload_fresh_cache_and_source_free_delivery_preserve_lazy_fields() {
    delivery(cases::conditional_payload_variants()[0]);
}

#[test]
fn scalar_join_payload_fresh_cache_and_source_free_delivery_preserve_typed_values() {
    delivery(cases::scalar_join_variants()[0]);
}

fn delivery(variant: cases::Variant) {
    let (project, mode) = project_for_source_with_limits(
        &cases::source(variant),
        cases::LOOP_LIMIT,
        cases::ENTRY_LIMIT,
    );
    eprintln!("sibling_bounds_delivery fresh");
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    let fresh =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    image(&project, &output, &binary, &mode, variant);
    eprintln!("sibling_bounds_delivery cache");
    assert!(project.build(None).contains("compile_cache: hit"));
    let cached =
        nuisc::aot::parse_nuis_compiled_artifact(&output.join("nuis.compiled.artifact")).unwrap();
    assert_eq!(fresh.binary_blob, cached.binary_blob);
    assert!(fresh.host_objects == cached.host_objects);
    assert_eq!(fresh.envelope, cached.envelope);
    assert_eq!(fresh.lifecycle, cached.lifecycle);
    assert_eq!(fresh.packaging_mode, cached.packaging_mode);
    image(&project, &output, &binary, &mode, variant);
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
        eprintln!("sibling_bounds_delivery restore {restore}");
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        image(&project, &restored, &binary, &mode, variant);
        assert_eq!(fs::read(&artifact).unwrap(), bytes);
    }
    for file in ["main.ns", "nuis.toml", ".nuis"] {
        assert!(!project.0.join(file).exists());
    }
}
