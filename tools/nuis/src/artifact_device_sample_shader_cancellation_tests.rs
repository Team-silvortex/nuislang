use super::*;
use yir_core::provider_runtime_ipc::Message;

fn reject_finish(stream: &mut std::os::unix::net::UnixStream) -> Result<Message, String> {
    let message = transport::read_request(stream, Duration::from_secs(5))?;
    if matches!(message, Message::Finish(_)) {
        return Err("cancelled window unexpectedly sent Finish".to_owned());
    }
    Ok(message)
}

pub(super) fn verify_compiled_window_cancellation(output: &Path, binary: &Path) {
    let manifest = nsdb::provider_runtime_result_stream_path(output);
    let previous = fs::read_dir(output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path == &manifest
                || path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("nuis.runtime.provider-result")
        })
        .map(|path| {
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    assert!(!previous.is_empty());
    for (events, frames) in [("", 1), ("32,128578", 2)] {
        let log_path = output.join("window-cancellation.log");
        let log = fs::File::create(&log_path).unwrap();
        let mut command = Command::new(binary);
        command
            .args([
                "--window-session",
                "window",
                "--window-events",
                events,
                "--window-cancel-after-events",
            ])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log));
        let result = crate::artifact_runtime_provider_results::run_command_with_request_reader(
            output,
            &mut command,
            Duration::from_secs(60),
            reject_finish,
        );
        let log = fs::read_to_string(log_path).unwrap();
        let error = result.expect_err("host cancellation cannot certify provider completion");
        assert!(
            error.contains("runtime IPC idle read failed"),
            "{error}\n{log}"
        );
        assert!(!error.contains("cleanup deadline"), "{error}\n{log}");
        assert!(!error.contains("wall-clock limit"), "{error}\n{log}");
        assert_cancelled_log(&log, frames);
        for (path, bytes) in &previous {
            assert_eq!(
                &fs::read(path).unwrap(),
                bytes,
                "cancelled run replaced {}",
                path.display()
            );
        }
    }

    // The same compiled host consumes a replay but still must exit cancelled,
    // not run close or claim a successful replay finalization.
    let args = [
        "--window-session",
        "window",
        "--window-cancel-after-events",
        "--window-events",
        "32,128578",
    ];
    let (status, log) = run_replay_bounded(output, binary, &manifest, &args);
    assert_eq!(status.code(), Some(130), "{status}\n{log}");
    assert_cancelled_log(&log, 2);
    for args in [
        vec!["--window-session", "window", "--window-cancel-after-events"],
        vec![
            "--window-session",
            "window",
            "--window-events",
            "",
            "--window-cancel-after-events",
            "--window-cancel-after-events",
        ],
        vec![
            "--window-session",
            "window",
            "--window-events",
            "",
            "--window-cancel-after-events",
            "--window-parent-session",
            "parent",
        ],
        vec![
            "--window-session",
            "window",
            "--window-cancel-after-events",
            "--window-events",
        ],
    ] {
        let (status, log) = run_replay_bounded(output, binary, &manifest, &args);
        assert_eq!(status.code(), Some(2), "{args:?}\n{status}\n{log}");
        assert!(!log.contains("window_session_opened"), "{log}");
        assert!(!log.contains("window_parent_opened"), "{log}");
    }

    let error = crate::artifact_runtime_command::handle_run_artifact_with_window(
        output.to_owned(),
        crate::cli::WindowSessionOptions {
            id: "window".to_owned(),
            events: Some(String::new()),
            parent: None,
            cancel_after_events: true,
        },
    )
    .expect_err("production cancellation stays non-success until provider drain is defined");
    assert!(error.contains("runtime IPC idle read failed"), "{error}");
    for (path, bytes) in previous {
        assert_eq!(
            fs::read(path).unwrap(),
            bytes,
            "frontdoor cancellation replaced old replay"
        );
    }
}

fn assert_cancelled_log(log: &str, frames: usize) {
    for marker in [
        "window_session_opened\n",
        "window_session_cancel_admitted\n",
        "window_session_host_retired\n",
        "window_session_cancel_cleanup_completed=0\n",
        "window_session_cancel_failure_kind=0\n",
    ] {
        assert_eq!(log.matches(marker).count(), 1, "{marker}\n{log}");
    }
    assert_eq!(
        log.matches("window_session_presented\n").count(),
        frames,
        "{log}"
    );
    assert!(
        log.find("window_session_cancel_admitted\n").unwrap()
            < log.find("window_session_host_retired\n").unwrap()
    );
    for forbidden in [
        "window_session_close_requested",
        "window_session_closed",
        "window_session_outcome_",
        "window_parent_",
        "window_session_cancel_rejected",
        "window_session_cancel_receipt_missing",
        "embedded runtime frame generation",
    ] {
        assert!(!log.contains(forbidden), "{forbidden}\n{log}");
    }
}

fn run_replay_bounded(
    output: &Path,
    binary: &Path,
    manifest: &Path,
    args: &[&str],
) -> (std::process::ExitStatus, String) {
    let log_path = output.join("window-cancel-replay.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = Command::new(binary)
        .args(args)
        .env_remove(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV)
        .env(yir_runtime_host::PROVIDER_RESULT_STREAM_ENV, manifest)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return (status, fs::read_to_string(log_path).unwrap());
        }
        if started.elapsed() >= Duration::from_secs(60) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("compiled window cancellation exceeded its deadline");
        }
        thread::sleep(Duration::from_millis(10));
    }
}
