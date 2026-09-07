use super::*;
use yir_core::provider_runtime_ipc::Message;

fn reject_dispatch(stream: &mut std::os::unix::net::UnixStream) -> Result<Message, String> {
    let request = transport::read_request(stream, Duration::from_secs(5))?;
    if matches!(request, Message::Dispatch { .. }) {
        return Err("injected terminal provider failure".to_owned());
    }
    Ok(request)
}

pub(super) fn verify_compiled_provider_failure_cleanup(output: &Path, binary: &Path) {
    let previous = fs::read_dir(output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
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
    let manifest = nsdb::provider_runtime_result_stream_path(output);
    let manifest_bytes = fs::read(&manifest).unwrap();
    let log_path = output.join("window-failure.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut command = Command::new(binary);
    command
        .args(["--window-session", "window", "--window-events", ""])
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log));
    let result = crate::artifact_runtime_provider_results::run_command_with_request_reader(
        output,
        &mut command,
        Duration::from_secs(60),
        reject_dispatch,
    );
    let log = fs::read_to_string(&log_path).unwrap();
    let error = result.expect_err("provider failure cannot become successful child cleanup");
    assert!(
        error.contains("injected terminal provider failure"),
        "{error}\n{log}"
    );
    assert!(!error.contains("cleanup deadline"), "{error}\n{log}");
    assert!(!error.contains("wall-clock limit"), "{error}\n{log}");
    assert_eq!(log.matches("window_session_opened").count(), 1, "{log}");
    assert_eq!(
        log.matches("window_session_close_requested").count(),
        1,
        "{log}"
    );
    assert_eq!(
        log.matches("window_session_close_reason=1\n").count(),
        1,
        "{log}"
    );
    assert_eq!(
        log.matches("window_session_cleanup_completed=1\n").count(),
        1,
        "{log}"
    );
    assert!(!log.contains("window_session_closed\n"), "{log}");
    assert!(
        log.contains("window_session_close_failure_kind=4\n"),
        "{log}"
    );
    assert!(log.contains("window_session_failure_kind=4\n"), "{log}");
    assert!(!log.contains("window_session_presented"), "{log}");
    verify_compiled_replay_exhaustion_cleanup(output, binary);
    assert_eq!(fs::read(manifest).unwrap(), manifest_bytes);
    assert!(!previous.is_empty());
    for (path, bytes) in previous {
        assert_eq!(
            fs::read(path).unwrap(),
            bytes,
            "failure replaced old replay evidence"
        );
    }
}

fn verify_compiled_replay_exhaustion_cleanup(output: &Path, binary: &Path) {
    let log_path = output.join("window-replay-exhaustion.log");
    let log = fs::File::create(&log_path).unwrap();
    // The successful run saved two frames; the third draw must exhaust replay,
    // not reconnect a live provider or borrow a reference renderer.
    let mut child = Command::new(binary)
        .args(["--window-session", "window", "--window-events", "32,32"])
        .env_remove(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV)
        .env(
            yir_runtime_host::PROVIDER_RESULT_STREAM_ENV,
            nsdb::provider_runtime_result_stream_path(output),
        )
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("compiled replay failure did not complete bounded cleanup");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let log = fs::read_to_string(log_path).unwrap();
    assert!(!status.success(), "{status}\n{log}");
    assert_eq!(log.matches("window_session_presented").count(), 2, "{log}");
    assert_eq!(
        log.matches("window_session_close_requested").count(),
        1,
        "{log}"
    );
    assert!(log.contains("window_session_close_reason=1\n"), "{log}");
    assert!(
        log.contains("window_session_close_failure_kind=7\n"),
        "{log}"
    );
    assert!(log.contains("window_session_failure_kind=7\n"), "{log}");
    assert!(
        log.contains("window_session_cleanup_completed=1\n"),
        "{log}"
    );
    assert!(!log.contains("window_session_closed\n"), "{log}");
}
