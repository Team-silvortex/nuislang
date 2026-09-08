use super::*;

fn bundle() -> String {
    format!(
        "cpu_host_binary_mode=embedded_yir_headless\nruntime_bootstrap_mode=embedded_yir_session\nrender_mode=application_session_observation\nsingle_binary=true\napplication_script_contract={}\napplication_provider_drain_contract={}\n",
        yir_runtime_host::APPLICATION_SCRIPT_CONTRACT,
        yir_core::provider_runtime_ipc::SESSION_DRAIN_CONTRACT,
    )
}

#[test]
fn headless_capabilities_are_exact_and_not_window_aliases() {
    let source = bundle();
    validate_contracts(&source).unwrap();
    for line in source.lines() {
        assert!(validate_contracts(&source.replace(&format!("{line}\n"), "")).is_err());
        assert!(validate_contracts(&format!("{source}{line}\n")).is_err());
        let key = line.split_once('=').unwrap().0;
        assert!(validate_contracts(&format!("{source} {key} = unknown\n")).is_err());
        assert!(validate_contracts(&source.replace(line, &format!("{key}=unknown"))).is_err());
    }
    for extra in [
        "window_session_contract=anything",
        "fallback_frame_asset=frame.ppm",
        " window_session_contract = anything",
        " fallback_frame_asset = frame.ppm",
    ] {
        assert!(validate_contracts(&format!("{source}{extra}\n")).is_err());
    }
    assert!(validate_contracts(&source.replace("-v1", "-v0")).is_err());
}

#[test]
fn headless_source_identity_must_be_unique_and_exact() {
    for source in [
        "",
        "application_yir_fnv1a64=other\n",
        "application_yir_fnv1a64=hash\napplication_yir_fnv1a64=hash\n",
    ] {
        assert!(require_declaration(source, "application_yir_fnv1a64", "hash").is_err());
    }
    require_declaration(
        "application_yir_fnv1a64=hash\n",
        "application_yir_fnv1a64",
        "hash",
    )
    .unwrap();
}

#[test]
fn application_admission_requires_a_manifest_and_a_live_drain_receipt() {
    let doctor =
        crate::artifact_doctor::probe_artifact_doctor(Path::new("missing-headless-test-artifact"));
    let mut script = ApplicationScript {
        id: "counter".to_owned(),
        open: vec![],
        events: vec![],
        termination: ApplicationScriptTermination::Cancel {
            drain_provider: false,
        },
    };
    assert!(validate(&doctor, Path::new("missing"), &script)
        .unwrap_err()
        .contains("--drain-provider"));
    script.termination = ApplicationScriptTermination::Close(vec![]);
    assert!(validate(&doctor, Path::new("missing"), &script)
        .unwrap_err()
        .contains("verified build manifest"));
}
