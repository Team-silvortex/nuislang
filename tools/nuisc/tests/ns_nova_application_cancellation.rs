#![cfg(unix)]

use std::{
    fs,
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};
use yir_core::{
    provider_runtime_ipc::{hash_bytes, DispatchTarget, Message},
    ApplicationFailureKind, Value,
};
use yir_runtime_host::{
    nuis_application_cancellation_free, nuis_application_cancellation_poll,
    nuis_window_session_cancel, nuis_window_session_close, nuis_window_session_free,
    nuis_window_session_outcome_field, ApplicationEventPump, ApplicationProviderSource,
    ApplicationPumpPhase, WindowSession,
};

struct Socket(PathBuf);

impl Socket {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        Self(std::env::temp_dir().join(format!(
            "ns-cancel-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn accept(listener: UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(10);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "application never connected");
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("{error}"),
        }
    };
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    stream
}

fn compiled_window() -> (String, DispatchTarget) {
    let module = nuisc::pipeline::compile_project(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/projects/domains/ns_nova_image_showcase"),
    )
    .unwrap()
    .yir;
    yir_runtime_host::validate_window_session(&module, "window").unwrap();
    let source = nuisc::render::render_yir(&module);
    let draw = module
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "draw_instanced")
        .unwrap();
    let target = DispatchTarget {
        source_yir_fnv1a64: hash_bytes(source.as_bytes()),
        module: draw.op.module.clone(),
        instruction: draw.op.instruction.clone(),
        node: draw.name.clone(),
        resource: draw.resource.clone(),
    };
    (source, target)
}

fn start_peer(
    socket: &Socket,
    target: DispatchTarget,
    expect_finish: bool,
) -> thread::JoinHandle<()> {
    let listener = UnixListener::bind(&socket.0).unwrap();
    listener.set_nonblocking(true).unwrap();
    thread::spawn(move || {
        let mut stream = accept(listener);
        Message::Hello(target).write_to(&mut stream).unwrap();
        if expect_finish {
            assert!(matches!(
                Message::read_from(&mut stream).unwrap(),
                Message::Finish(0)
            ));
            Message::Closed(0).write_to(&mut stream).unwrap();
        }
        // A timeout is not an EOF or acknowledgement. Neither path may
        // dispatch a frame; only explicit Nuis close may reach Finish.
        assert_eq!(stream.read(&mut [0_u8; 1]).unwrap(), 0);
    })
}

#[test]
fn compiled_nuis_window_state_can_retire_without_implicit_close_or_provider_finish() {
    let (source, target) = compiled_window();
    for cancel in [true, false] {
        let socket = Socket::new();
        let peer = start_peer(&socket, target.clone(), !cancel);
        let mut pump = ApplicationEventPump::spawn(
            source.clone(),
            ApplicationProviderSource::Ipc(&socket.0),
            "window".to_owned(),
            vec![Value::Int(640), Value::Int(400)],
        )
        .unwrap();
        let opened = pump
            .wait(Duration::from_secs(30))
            .unwrap()
            .expect("open reply");
        assert_eq!(opened.phase, ApplicationPumpPhase::Open);
        assert!(matches!(opened.state, Some(Value::Struct(_))));
        let trace = opened.trace.unwrap();
        assert!(trace.presented_frames.is_empty());
        assert!(trace.provider_completion_witnesses.is_empty());
        if cancel {
            let mut cancellation = pump.cancel().unwrap();
            let ack = cancellation
                .wait(Duration::from_secs(30))
                .unwrap()
                .expect("host retirement");
            assert!(!ack.cleanup_completed());
            assert_eq!(ack.failure_kind(), ApplicationFailureKind::None);
            assert!(pump.close(vec![Value::Int(0), Value::Int(0)]).is_err());
            assert!(pump.poll().unwrap().is_none());
            assert!(cancellation.poll().unwrap().is_none());
        } else {
            pump.close(vec![Value::Int(0), Value::Int(0)]).unwrap();
            let closed = pump
                .wait(Duration::from_secs(30))
                .unwrap()
                .expect("close reply");
            assert_eq!(closed.phase, ApplicationPumpPhase::Closed);
            assert!(closed.cleanup_completed);
            closed.trace.unwrap();
        }
        peer.join().unwrap();
    }
}

#[test]
fn compiled_nuis_window_ffi_cancel_retains_ticket_after_window_free() {
    let (source, target) = compiled_window();
    let socket = Socket::new();
    let peer = start_peer(&socket, target, false);
    let mut window = WindowSession::spawn(
        source,
        ApplicationProviderSource::Ipc(&socket.0),
        "window".to_owned(),
        640,
        400,
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    let opened = loop {
        if let Some(reply) = window.poll().unwrap() {
            break reply;
        }
        assert!(Instant::now() < deadline, "window open timed out");
        thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(opened.phase, ApplicationPumpPhase::Open);
    assert!(matches!(opened.state, Some(Value::Struct(_))));
    assert!(opened.frame.unwrap().is_none());
    let mut window = Box::into_raw(Box::new(window));
    let mut ticket = std::ptr::null_mut();
    let (mut cleanup, mut failure) = (-7, -8);
    unsafe {
        assert_eq!(nuis_window_session_cancel(window, &mut ticket), 0);
        assert_eq!(nuis_window_session_close(window), -1);
        assert_eq!(nuis_window_session_outcome_field(window, 0), -1);
        assert!((*window).take_outcome_delivery().is_none());
        nuis_window_session_free(&mut window);
        assert!(window.is_null());
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let status = nuis_application_cancellation_poll(ticket, &mut cleanup, &mut failure);
            if status != 0 {
                assert_eq!(status, 1);
                break;
            }
            assert!(Instant::now() < deadline, "window retirement timed out");
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!((cleanup, failure), (0, ApplicationFailureKind::None.code()));
        assert_eq!(
            nuis_application_cancellation_poll(ticket, &mut cleanup, &mut failure),
            0
        );
        nuis_application_cancellation_free(&mut ticket);
        assert!(ticket.is_null());
    }
    peer.join().unwrap();
}
