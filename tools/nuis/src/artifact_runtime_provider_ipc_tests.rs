use super::*;
use std::{
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
};

#[test]
fn accepted_connection_waits_for_delayed_dispatch_even_if_nonblocking_was_inherited() {
    let (mut stream, mut peer) = UnixStream::pair().unwrap();
    stream.set_nonblocking(true).unwrap();
    transport::configure_connection(&stream, transport::IO_TIMEOUT).unwrap();
    let sender = thread::spawn(move || {
        thread::sleep(Duration::from_millis(30));
        peer.write_all(b"dispatch")
    });
    let mut received = [0; 8];
    let result = stream.read_exact(&mut received);
    sender.join().unwrap().unwrap();
    result.unwrap();
    assert_eq!(&received, b"dispatch");
}

#[test]
fn unconnected_server_rejects_success_and_removes_private_socket() {
    let mut server = RuntimeProviderServer::start(Path::new(".")).unwrap();
    let directory = server.directory.clone();
    assert_eq!(
        fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert!(directory.join("dispatch").exists());
    assert!(server
        .finish()
        .unwrap_err()
        .contains("did not complete a provider lifecycle"));
    drop(server);
    assert!(!directory.exists());
}

#[test]
fn bounded_child_timeout_kills_and_reaps_without_waiting_for_an_ipc_connection() {
    let mut command = Command::new("sleep");
    command.arg("10");
    let started = Instant::now();
    let result = run_command_with_timeout(
        Path::new("."),
        &mut command,
        Some(Duration::from_millis(30)),
    );
    assert!(result.unwrap_err().contains("explicit wall-clock limit"));
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn supervisor_shutdown_wakes_an_idle_provider_without_acknowledging_success() {
    let (mut stream, _peer) = UnixStream::pair().unwrap();
    transport::configure_connection(&stream, transport::IO_TIMEOUT).unwrap();
    let observer = stream.try_clone().unwrap();
    let active = Arc::new(Mutex::new(Some(stream.try_clone().unwrap())));
    let mut server = RuntimeProviderServer {
        directory: private_socket_directory().unwrap(),
        stop: Arc::new(AtomicBool::new(false)),
        active,
        thread: Some(thread::spawn(move || {
            transport::read_request(&mut stream, transport::IO_TIMEOUT).map(|_| 1)
        })),
    };
    let started = Instant::now();
    while observer.read_timeout().unwrap().is_some() {
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "reader did not enter idle waiting"
        );
        thread::sleep(Duration::from_millis(1));
    }
    let error = server.finish().unwrap_err();
    assert!(error.contains("idle read failed"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn provider_failure_allows_bounded_child_cleanup_instead_of_immediate_kill() {
    let mut child = Command::new("sh")
        .args(["-c", "sleep 0.1; printf cleanup-observed"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let status = supervise_child(
        &mut child,
        Some(Duration::from_secs(3)),
        Duration::from_secs(1),
        || true,
    )
    .unwrap();
    assert!(status.success());
    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    assert_eq!(output, "cleanup-observed");
    assert!(child.try_wait().unwrap().is_some());
}

#[test]
fn failure_grace_is_latched_and_kills_a_child_that_will_not_close() {
    let mut child = Command::new("sleep").arg("10").spawn().unwrap();
    let mut polls = 0;
    let started = Instant::now();
    let error = supervise_child(
        &mut child,
        Some(Duration::from_secs(3)),
        Duration::from_millis(50),
        || {
            polls += 1;
            polls == 1
        },
    )
    .unwrap_err();
    assert!(error.contains("failure cleanup deadline"), "{error}");
    assert!(polls > 1);
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(child.try_wait().unwrap().is_some());
}

#[test]
fn explicit_wall_clock_limit_is_not_extended_by_failure_cleanup() {
    let mut child = Command::new("sleep").arg("10").spawn().unwrap();
    let error = supervise_child(
        &mut child,
        Some(Duration::from_millis(30)),
        Duration::from_secs(5),
        || true,
    )
    .unwrap_err();
    assert!(error.contains("explicit wall-clock limit"), "{error}");
    assert!(child.try_wait().unwrap().is_some());
}
