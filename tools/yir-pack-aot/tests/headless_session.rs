#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::{
    fs,
    io::Read,
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};
use yir_core::{
    provider_runtime_ipc::{hash_bytes, DispatchFrame, DispatchTarget, Message},
    ProviderPhysicalCompletion,
};

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bounded(command: &mut Command, log: &Path) -> ExitStatus {
    let mut child = command
        .stdout(Stdio::null())
        .stderr(fs::File::create(log).unwrap())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "headless process exceeded deadline: {}",
                fs::read_to_string(log).unwrap()
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn packaged_headless_session_finishes_or_drains_without_a_window_backend() {
    let output = Artifacts(std::env::temp_dir().join(format!("ns-script-{}", std::process::id())));
    fs::create_dir(&output.0).unwrap();
    let source = format!(
        "application-session counter {} open update close state\n{}",
        yir_core::APPLICATION_SESSION_CONTRACT,
        include_str!("../../../crates/yir-runtime-host/tests/fixtures/application_session.yir"),
    );
    let input = output.0.join("counter.yir");
    fs::write(&input, &source).unwrap();
    let bundle = output.0.join("bundle");
    let log = output.0.join("process.log");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let status = bounded(
        Command::new(env!("CARGO_BIN_EXE_yir-pack-aot"))
            .current_dir(root)
            .arg(&input)
            .arg(&bundle)
            .arg("--headless")
            .env("CARGO_INCREMENTAL", "0")
            .env("CARGO_BUILD_JOBS", "1"),
        &log,
    );
    assert!(status.success(), "{}", fs::read_to_string(&log).unwrap());
    let binary = bundle.join("counter");
    let manifest = fs::read_to_string(bundle.join("bundle.txt")).unwrap();
    assert!(manifest.contains("cpu_host_binary_mode=embedded_yir_headless"));
    assert!(manifest.contains("runtime_bootstrap_mode=embedded_yir_session"));
    assert!(!manifest.contains("window_session_contract="));
    assert!(!manifest.contains("fallback_frame_asset="));
    assert!(!manifest.contains("llvm_ir="));
    assert!(!manifest.contains("macos_affinity_worker_thread"));
    assert!(!bundle.join("counter.ll").exists());
    assert!(!bundle.join("counter_shim.c").exists());

    for drain in [false, true] {
        let socket = output
            .0
            .join(if drain { "drain.sock" } else { "finish.sock" });
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let source = source.clone();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            && Instant::now() < deadline =>
                    {
                        thread::sleep(Duration::from_millis(5))
                    }
                    error => panic!("missing headless connection: {error:?}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            let target = DispatchTarget {
                source_yir_fnv1a64: hash_bytes(source.as_bytes()),
                module: "shader".to_owned(),
                instruction: "draw_instanced".to_owned(),
                node: "draw".to_owned(),
                resource: "gpu".to_owned(),
            };
            Message::Hello(target.clone())
                .write_to(&mut stream)
                .unwrap();
            for sequence in 0..2 {
                let Message::Dispatch {
                    sequence: actual,
                    target: actual_target,
                    arguments,
                } = Message::read_from(&mut stream).unwrap()
                else {
                    panic!("expected dispatch")
                };
                assert_eq!(actual, sequence);
                assert_eq!(actual_target, target);
                Message::Frame(DispatchFrame {
                    sequence,
                    arguments,
                    request_id: "render".to_owned(),
                    provider_family: "test:device".to_owned(),
                    element_type: "u8".to_owned(),
                    layout: "image-2d-row-major:pixel-format=rgba8".to_owned(),
                    shape: vec![1, 1],
                    row_stride_bytes: 4,
                    payload: vec![sequence as u8, 2, 3, 255],
                    completion_wire: ProviderPhysicalCompletion::new(
                        "shader.clock.frame.v1",
                        "test.clock",
                        "test.fence",
                        sequence as i64 + 1,
                    )
                    .unwrap()
                    .to_wire(),
                })
                .write_to(&mut stream)
                .unwrap();
            }
            match Message::read_from(&mut stream).unwrap() {
                Message::Drain(receipt) if drain => {
                    receipt.admit(&target, 2).unwrap();
                    Message::Drained(receipt).write_to(&mut stream).unwrap();
                }
                Message::Finish(2) if !drain => Message::Closed(2).write_to(&mut stream).unwrap(),
                unexpected => panic!("wrong terminal operation: {unexpected:?}"),
            }
            assert_eq!(stream.read(&mut [0]).unwrap(), 0);
        });
        let mut command = Command::new(&binary);
        command
            .args([
                "--application-session",
                "counter",
                "--open-args",
                "10",
                "--event-args",
                "1",
                "--event-args",
                "2",
            ])
            .env(yir_core::provider_runtime_ipc::SOCKET_ENV, socket)
            .env_remove("NUIS_YIR_PROVIDER_RESULT_STREAM");
        if drain {
            command.args(["--cancel-after-events", "--drain-provider"]);
        } else {
            command.args(["--close-args", "1"]);
        }
        let status = bounded(&mut command, &log);
        let log = fs::read_to_string(&log).unwrap();
        worker.join().unwrap();
        assert_eq!(status.code(), Some(if drain { 130 } else { 0 }), "{log}");
        for sequence in 0..2 {
            assert!(
                log.contains(&format!(
                    "application_session_frame={};rgba8_fnv1a64={};bytes=4",
                    sequence + 1,
                    hash_bytes(&[sequence as u8, 2, 3, 255])
                )),
                "{log}"
            );
        }
        assert!(!log.contains("window_session"));
        if drain {
            assert!(
                log.contains("provider_status=5;provider_failure_kind=0;completed_dispatches=2")
            );
            assert!(!log.contains("application_session_outcome="));
            assert!(!log.contains("application_session_reply=Close"));
        } else {
            assert!(log.contains("application_session_outcome=[1, 1, 0]"));
        }
    }
}
