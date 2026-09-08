use super::*;
use crate::artifact_runtime_provider_results::{
    PreparedRuntimeProviderResults, ProviderLaunchOutcome, ProviderLaunchPolicy,
};
use std::{collections::BTreeMap, process::Stdio, time::Instant};

fn evidence(output: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fs::read_dir(output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            let name = path.file_name().unwrap().to_str().unwrap();
            name.starts_with("nuis.nsdb")
                || name.starts_with("nuis.runtime")
                || name == "baseline.ppm"
        })
        .map(|path| {
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect()
}

pub(super) fn verify_packaged_drain(
    output: &Path,
    binary: &Path,
    prepared: &PreparedRuntimeProviderResults,
) {
    let previous = evidence(output);
    assert!(!previous.is_empty());
    for (events, frames) in [("", 1), ("32,128578", 2)] {
        let log_path = output.join("packaged-drain.log");
        let log = fs::File::create(&log_path).unwrap();
        let mut command = Command::new(binary);
        command
            .args([
                "--window-session",
                "window",
                "--window-events",
                events,
                "--window-cancel-after-events",
                "--drain-provider",
            ])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log));
        let result = prepared.run_command_with_policy(
            &mut command,
            Some(Duration::from_secs(180)),
            ProviderLaunchPolicy::ExplicitDrain,
        );
        let log = fs::read_to_string(log_path).unwrap();
        let (status, outcome) = result.unwrap_or_else(|error| panic!("{error}\n{log}"));
        assert_eq!(
            status.code(),
            Some(yir_runtime_host::APPLICATION_CANCELLED_EXIT_CODE),
            "{log}"
        );
        let ProviderLaunchOutcome::Drained(receipt) = outcome else {
            panic!("missing drain\n{log}");
        };
        assert_eq!(receipt.sequence, frames);
        assert_eq!(
            log.matches("window_session_presented\n").count(),
            frames,
            "{log}"
        );
        for marker in [
            "window_session_cancel_admitted\n",
            "window_session_host_retired\n",
            "window_session_cancel_failure_kind=0\n",
            "window_session_provider_drain_status=5\n",
            "window_session_provider_drain_failure_kind=0\n",
        ] {
            assert_eq!(log.matches(marker).count(), 1, "{marker}\n{log}");
        }
        assert!(
            log.contains(&format!(
                "window_session_provider_drain_dispatches={frames}\n"
            )),
            "{log}"
        );
        for forbidden in [
            "window_session_closed",
            "window_session_outcome",
            "window_parent_",
            "cancel_rejected",
            "cancel_receipt_missing",
            "embedded runtime frame generation",
        ] {
            assert!(!log.contains(forbidden), "{forbidden}\n{log}");
        }
        assert!(!output.join(".nuis-provider-worker-image").exists());
        assert_eq!(
            evidence(output),
            previous,
            "cancelled launch replaced success evidence"
        );
    }

    // The production frontdoor must retain the typed non-success outcome and
    // skip its normal launch/trace publication, not disguise cancellation as Ok(()).
    let outcome = crate::artifact_runtime_command::handle_run_artifact_with_window(
        output.to_owned(),
        crate::cli::WindowSessionOptions {
            id: "window".to_owned(),
            events: Some(String::new()),
            parent: None,
            cancel_after_events: true,
            drain_provider: true,
        },
    )
    .unwrap();
    assert!(
        matches!(&outcome, crate::artifact_runtime_command::ArtifactRunOutcome::Cancelled(receipt)
        if receipt.sequence == 1)
    );
    assert_eq!(outcome.exit_code(), std::process::ExitCode::from(130));
    assert_eq!(
        evidence(output),
        previous,
        "frontdoor drain published success evidence"
    );

    // Explicit drain intent must reject a normal Finish before provider output
    // publication. This deliberately uses the unmodified ordinary-close host.
    let mut command = Command::new(binary);
    command
        .args(["--window-session", "window", "--window-events", ""])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let error = prepared
        .run_command_with_policy(
            &mut command,
            Some(Duration::from_secs(180)),
            ProviderLaunchPolicy::ExplicitDrain,
        )
        .unwrap_err();
    assert!(error.contains("before publication"), "{error}");
    assert_eq!(
        evidence(output),
        previous,
        "rejected Finish replaced success evidence"
    );

    // A replay can retire its host, but has no live provider to drain.
    let log_path = output.join("packaged-drain-replay.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = Command::new(binary)
        .args([
            "--window-session",
            "window",
            "--window-events",
            "",
            "--window-cancel-after-events",
            "--drain-provider",
        ])
        .env_remove(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV)
        .env(
            yir_runtime_host::PROVIDER_RESULT_STREAM_ENV,
            &prepared.stream_path,
        )
        .stdout(Stdio::null())
        .stderr(log)
        .spawn()
        .unwrap();
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > Duration::from_secs(90) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("packaged drain replay exceeded its deadline");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let log = fs::read_to_string(log_path).unwrap();
    assert_eq!(status.code(), Some(1), "{status}\n{log}");
    assert!(
        log.contains("window_session_provider_drain_status=2\n"),
        "{log}"
    );
    assert!(!log.contains("window_session_outcome"), "{log}");
    assert_eq!(evidence(output), previous);
}
