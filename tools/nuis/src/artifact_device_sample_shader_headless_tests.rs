use super::*;
use crate::artifact_runtime_command::{
    handle_run_artifact_with_application_script, ArtifactRunOutcome,
};
use crate::artifact_runtime_provider_results::{
    prepare_runtime_provider_results, PreparedRuntimeProviderResults, ProviderLaunchOutcome,
    ProviderLaunchPolicy,
};
use std::{path::PathBuf, process::Command, time::Duration};
use yir_runtime_host::{ApplicationScript, ApplicationScriptTermination};

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn launch(
    binary: &Path,
    output: &Path,
    prepared: &PreparedRuntimeProviderResults,
    events: usize,
    drain: bool,
) -> (std::process::ExitStatus, ProviderLaunchOutcome, String) {
    let log = output.join("headless.log");
    let mut command = Command::new(binary);
    command.args(["--application-session", "window", "--open-args", "160,120"]);
    for event in ["0,0", "1,32"].into_iter().take(events) {
        command.args(["--event-args", event]);
    }
    if drain {
        command.args(["--cancel-after-events", "--drain-provider"]);
    } else {
        command.args(["--close-args", "1,0"]);
    }
    command.stderr(fs::File::create(&log).unwrap());
    let (status, outcome) = prepared
        .run_command_with_policy(
            &mut command,
            Some(Duration::from_secs(180)),
            if drain {
                ProviderLaunchPolicy::ExplicitDrain
            } else {
                ProviderLaunchPolicy::CompletionOnly
            },
        )
        .unwrap();
    (status, outcome, fs::read_to_string(log).unwrap())
}

fn verify_frames(log: &str, count: usize) {
    for index in 0..count {
        let input = image_input(index == 1);
        let mut rgba8 = Vec::with_capacity(160 * 120 * 4);
        for y in 0..120 {
            for x in 0..160 {
                let offset = (y / 5 * 32 + x / 5) * 4;
                let pixel = &input[offset..offset + 4];
                rgba8.extend([255 - pixel[0], 255 - pixel[1], 255 - pixel[2], pixel[3]]);
            }
        }
        assert!(
            log.contains(&format!(
                "application_session_frame={};rgba8_fnv1a64={};bytes=76800",
                index + 1,
                fnv1a64_hex(&rgba8)
            )),
            "{log}"
        );
    }
    assert_eq!(log.matches("application_session_frame=").count(), count);
}

fn application_script(events: usize, drain: bool) -> ApplicationScript {
    ApplicationScript {
        id: "window".to_owned(),
        open: vec![160, 120],
        events: [vec![0, 0], vec![1, 32]].into_iter().take(events).collect(),
        termination: if drain {
            ApplicationScriptTermination::Cancel {
                drain_provider: true,
            }
        } else {
            ApplicationScriptTermination::Close(vec![1, 0])
        },
    }
}

#[test]
fn headless_metal_session_uses_shared_lifecycle_without_appkit() {
    let output = Artifacts(temp_output_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    crate::handle_build(
        root.join("examples/projects/domains/ns_nova_image_showcase"),
        output.0.clone(),
        false,
        None,
        None,
        Some("headless-aot-bundle".to_owned()),
    )
    .unwrap();
    let report =
        nuisc::aot::verify_build_manifest(&output.0.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, "headless-aot-bundle");
    let manifest = fs::read_to_string(output.0.join("nuis.build.manifest.toml")).unwrap();
    assert!(manifest.contains("headless_compiler_checkpoint = \"verified-yir-v1\""));
    assert!(!manifest.contains("kind = \"llvm_ir\""));
    assert!(output.0.join("nuis.compiler-stage-handoff.toml").is_file());
    for name in ["ns_nova_image_showcase.ll", "ns_nova_image_showcase_shim.c"] {
        assert!(
            !output.0.join(name).exists(),
            "unused LLVM artifact: {name}"
        );
    }
    let input = root.join("examples/projects/domains/ns_nova_image_showcase");
    let project = nuisc::project::load_project(&input).unwrap();
    let plan = nuisc::project::build_project_compilation_plan(&project).unwrap();
    let default_cache =
        nuisc::cache::compute_compile_cache_key_with_plan(&input, Some(&project), Some(&plan))
            .unwrap();
    assert_ne!(
        report.compile_cache_key.as_deref(),
        Some(default_cache.key.as_str())
    );
    crate::handle_build(
        root.join("examples/projects/domains/ns_nova_image_showcase"),
        output.0.clone(),
        false,
        None,
        None,
        Some("headless-aot-bundle".to_owned()),
    )
    .unwrap();
    let restored =
        nuisc::aot::verify_build_manifest(&output.0.join("nuis.build.manifest.toml")).unwrap();
    assert!(!output.0.join("ns_nova_image_showcase.ll").exists());
    assert_eq!(restored.compile_cache_status.as_deref(), Some("hit"));
    assert_eq!(restored.compile_cache_key, report.compile_cache_key);
    assert_eq!(restored.packaging_mode, "headless-aot-bundle");
    let binary = output.0.join(&report.artifact_binary_name);
    let headless = output.0.clone();
    let doctor = crate::artifact_doctor::probe_artifact_doctor(&output.0);
    assert!(doctor.recommended_command.contains("--application-session"));
    assert!(crate::handle_run_artifact(output.0.clone(), false)
        .unwrap_err()
        .contains("--application-session"));
    for case in 0..3 {
        let mut script = application_script(2, false);
        match case {
            0 => script.id = "missing".to_owned(),
            1 => script.events.push(vec![]),
            _ => script.termination = ApplicationScriptTermination::Close(vec![]),
        }
        assert!(handle_run_artifact_with_application_script(output.0.clone(), script).is_err());
    }
    assert!(!output
        .0
        .join("nuis.runtime.provider-result-stream.toml")
        .exists());
    let bundle_path = output.0.join("bundle.txt");
    let original_bundle = fs::read(&bundle_path).unwrap();
    let mut tampered = original_bundle.clone();
    tampered.extend_from_slice(b"application_script_contract=unknown\n");
    fs::write(&bundle_path, tampered).unwrap();
    assert!(handle_run_artifact_with_application_script(
        output.0.clone(),
        application_script(2, false)
    )
    .is_err());
    fs::write(&bundle_path, &original_bundle).unwrap();
    let original_binary = fs::read(&binary).unwrap();
    let mut tampered = original_binary.clone();
    *tampered.last_mut().unwrap() ^= 1;
    fs::write(&binary, tampered).unwrap();
    assert!(handle_run_artifact_with_application_script(
        output.0.clone(),
        application_script(2, false)
    )
    .is_err());
    fs::write(&binary, original_binary).unwrap();
    let unrelated_binary = output.0.join("different-host");
    fs::copy(&binary, &unrelated_binary).unwrap();
    assert!(handle_run_artifact_with_application_script(
        unrelated_binary.clone(),
        application_script(2, false)
    )
    .unwrap_err()
    .contains("binary"));
    fs::remove_file(unrelated_binary).unwrap();
    assert!(!output.0.join(".nuis-provider-worker-image").exists());
    let prepared = prepare_runtime_provider_results(&output.0)
        .unwrap()
        .unwrap();
    let manifest = fs::read_to_string(headless.join("bundle.txt")).unwrap();
    assert!(manifest.contains(&format!(
        "application_script_contract={}",
        yir_runtime_host::APPLICATION_SCRIPT_CONTRACT
    )));
    assert!(manifest.contains("cpu_host_binary_mode=embedded_yir_headless"));
    assert!(!manifest.contains("window_session_contract="));
    assert!(!manifest.contains("fallback_frame_asset="));
    let libraries = Command::new("otool")
        .arg("-L")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(libraries.status.success());
    assert!(!String::from_utf8_lossy(&libraries.stdout).contains("AppKit"));

    let (status, outcome, log) = launch(&binary, &headless, &prepared, 2, false);
    assert!(status.success(), "{log}");
    assert!(matches!(
        outcome,
        ProviderLaunchOutcome::Finished {
            sessions: 1,
            invocations: 2
        }
    ));
    assert!(log.contains("application_session_outcome=[1, 1, 0]"));
    verify_frames(&log, 2);
    assert!(matches!(
        handle_run_artifact_with_application_script(output.0.clone(), application_script(2, false))
            .unwrap(),
        ArtifactRunOutcome::Completed
    ));

    let previous = fs::read_dir(&output.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.is_file()
                && path.file_name().unwrap().to_str().is_some_and(|name| {
                    name.starts_with("nuis.runtime") || name.starts_with("nuis.nsdb")
                })
        })
        .map(|path| {
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    assert!(previous
        .iter()
        .any(|(path, _)| path == &prepared.stream_path));
    for frames in [1, 2] {
        let (status, outcome, log) = launch(&binary, &headless, &prepared, frames, true);
        assert_eq!(status.code(), Some(130), "{log}");
        let ProviderLaunchOutcome::Drained(receipt) = outcome else {
            panic!("missing typed drain")
        };
        assert_eq!(receipt.sequence, frames);
        assert!(log.contains(&format!("application_session_host_retired=1;cleanup_completed=0;failure_kind=0;provider_status=5;provider_failure_kind=0;completed_dispatches={frames}")), "{log}");
        assert!(!log.contains("application_session_outcome="));
        assert!(!log.contains("application_session_reply=Close"));
        verify_frames(&log, frames);
        let frontdoor = handle_run_artifact_with_application_script(
            output.0.clone(),
            application_script(frames, true),
        )
        .unwrap();
        let ArtifactRunOutcome::Cancelled(frontdoor_receipt) = frontdoor else {
            panic!("frontdoor did not preserve typed cancellation")
        };
        assert_eq!(frontdoor_receipt.sequence, frames);
        assert!(!output.0.join(".nuis-provider-worker-image").exists());
        for (path, bytes) in &previous {
            assert_eq!(
                &fs::read(path).unwrap(),
                bytes,
                "drain replaced {}",
                path.display()
            );
        }
    }
    let replay = Command::new(&binary)
        .args([
            "--application-session",
            "window",
            "--open-args",
            "160,120",
            "--event-args",
            "0,0",
            "--event-args",
            "1,32",
            "--cancel-after-events",
            "--drain-provider",
        ])
        .env_remove(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV)
        .env(
            yir_runtime_host::PROVIDER_RESULT_STREAM_ENV,
            &prepared.stream_path,
        )
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&replay.stderr);
    assert_eq!(replay.status.code(), Some(1), "{log}");
    assert!(log.contains("provider_status=2;"), "{log}");
    assert!(!log.contains("application_session_outcome="));
    for (path, bytes) in previous {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}
