#![cfg(unix)]

use std::{
    fs,
    io::Read,
    os::unix::net::UnixListener,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    thread::{self, JoinHandle},
    time::Duration,
};
use yir_core::{
    provider_runtime_ipc::{
        hash_bytes, DispatchFrame, DispatchTarget, Message, Rejection, RejectionCode,
        RejectionPhase,
    },
    ProviderPhysicalCompletion, Value,
};
use yir_runtime_host::{
    with_provider_application_session, with_registered_provider_application_session,
    ApplicationProviderSource, ApplicationSessionEntries,
};

#[path = "provider_application_session/cancellation.rs"]
mod cancellation;
#[path = "provider_application_session/event_pump.rs"]
mod event_pump;
#[path = "provider_application_session/script.rs"]
mod script;
#[path = "provider_application_session/window.rs"]
mod window;

const SOURCE: &str = include_str!("fixtures/application_session.yir");

fn entries() -> ApplicationSessionEntries<'static> {
    ApplicationSessionEntries {
        open: "open",
        event: "update",
        close: "close",
        state_parameter: "state",
    }
}

#[derive(Clone, Copy)]
enum Reply {
    Good,
    BadClose,
    RejectedFrame,
    TypedRejectedFrame(RejectionCode),
    RejectedClose,
    WrongHash,
    DisconnectedFrame,
    WrongSequence,
    DrainGood,
    DrainRejected,
    DrainWrongTarget,
    DrainDisconnected,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pause {
    Hello,
    Frame,
    Finish,
    Drain,
}

struct Gate {
    at: Pause,
    ready: std::sync::mpsc::SyncSender<()>,
    release: std::sync::mpsc::Receiver<()>,
}

impl Gate {
    fn wait(&self, at: Pause) {
        if self.at == at {
            self.ready.send(()).unwrap();
            self.release.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    }
}

struct Peer {
    path: PathBuf,
    worker: Option<JoinHandle<(usize, bool)>>,
}

impl Peer {
    fn start(reply: Reply) -> Self {
        Self::start_source(reply, SOURCE.to_owned(), None)
    }

    fn start_source(reply: Reply, source: String, gate: Option<Gate>) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ns-app-ipc-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = UnixListener::bind(&path).unwrap();
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let target = DispatchTarget {
                source_yir_fnv1a64: hash_bytes(if matches!(reply, Reply::WrongHash) {
                    b"other"
                } else {
                    source.as_bytes()
                }),
                module: "shader".to_owned(),
                instruction: "draw_instanced".to_owned(),
                node: "draw".to_owned(),
                resource: "gpu".to_owned(),
            };
            if let Some(gate) = &gate {
                gate.wait(Pause::Hello);
            }
            Message::Hello(target.clone())
                .write_to(&mut stream)
                .unwrap();
            let mut count = 0;
            while let Ok(message) = Message::read_from(&mut stream) {
                match message {
                    Message::Dispatch {
                        sequence,
                        target: actual,
                        arguments,
                    } => {
                        assert_eq!(actual, target);
                        assert_eq!(sequence, count);
                        count += 1;
                        if let Some(gate) = &gate {
                            gate.wait(Pause::Frame);
                        }
                        if matches!(reply, Reply::RejectedFrame | Reply::TypedRejectedFrame(_)) {
                            // Deliberately mentions a budget: clients must classify
                            // the message kind, not guess from this diagnostic.
                            let code = match reply {
                                Reply::TypedRejectedFrame(code) => code,
                                _ => RejectionCode::Request,
                            };
                            Message::Rejected(Rejection::new(
                                RejectionPhase::Dispatch,
                                sequence,
                                code,
                                "injected device failure: dispatch budget",
                            ))
                            .write_to(&mut stream)
                            .unwrap();
                            continue;
                        }
                        if matches!(reply, Reply::DisconnectedFrame) {
                            return (count, false);
                        }
                        Message::Frame(DispatchFrame {
                            sequence: if matches!(reply, Reply::WrongSequence) {
                                sequence + 1
                            } else {
                                sequence
                            },
                            arguments,
                            request_id: "render".to_owned(),
                            provider_family: "test:device".to_owned(),
                            element_type: "u8".to_owned(),
                            layout: "image-2d-row-major:pixel-format=rgba8".to_owned(),
                            shape: vec![1, 1],
                            row_stride_bytes: 4,
                            payload: vec![count as u8, 2, 3, 255],
                            completion_wire: ProviderPhysicalCompletion::new(
                                "shader.clock.frame.v1",
                                "test.clock",
                                "test.fence",
                                count as i64,
                            )
                            .unwrap()
                            .to_wire(),
                        })
                        .write_to(&mut stream)
                        .unwrap();
                    }
                    Message::Finish(sequence) => {
                        assert_eq!(sequence, count);
                        if let Some(gate) = &gate {
                            gate.wait(Pause::Finish);
                        }
                        if matches!(reply, Reply::RejectedClose) {
                            Message::Rejected(Rejection::new(
                                RejectionPhase::Finish,
                                sequence,
                                RejectionCode::Finalization,
                                "publication failed",
                            ))
                            .write_to(&mut stream)
                            .unwrap();
                            return (count, true);
                        }
                        Message::Closed(if matches!(reply, Reply::BadClose) {
                            count + 1
                        } else {
                            count
                        })
                        .write_to(&mut stream)
                        .unwrap();
                        return (count, true);
                    }
                    Message::Drain(drain)
                        if matches!(
                            reply,
                            Reply::DrainGood
                                | Reply::DrainRejected
                                | Reply::DrainWrongTarget
                                | Reply::DrainDisconnected
                        ) =>
                    {
                        drain.admit(&target, count).unwrap();
                        if let Some(gate) = &gate {
                            gate.wait(Pause::Drain);
                        }
                        match reply {
                            Reply::DrainDisconnected => return (count, false),
                            Reply::DrainRejected => Message::Rejected(Rejection::new(
                                RejectionPhase::Drain,
                                count,
                                RejectionCode::Finalization,
                                "drain close failed",
                            )),
                            Reply::DrainWrongTarget => {
                                let mut wrong = drain;
                                wrong.target.node = "other".to_owned();
                                Message::Drained(wrong)
                            }
                            _ => Message::Drained(drain),
                        }
                        .write_to(&mut stream)
                        .unwrap();
                        assert_eq!(stream.read(&mut [0]).unwrap(), 0, "drain was not terminal");
                        return (count, false);
                    }
                    other => panic!("unexpected client message: {other:?}"),
                }
            }
            // Failure/abandonment must release the transport, not merely leave
            // the peer waiting until its read deadline. EOF remains observable.
            assert_eq!(stream.read(&mut [0_u8; 1]).unwrap(), 0);
            (count, false)
        });
        Self {
            path,
            worker: Some(worker),
        }
    }
    fn finish(mut self) -> (usize, bool) {
        self.worker.take().unwrap().join().unwrap()
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn one_transport_carries_independent_events_and_finishes_only_after_close() {
    let peer = Peer::start(Reply::Good);
    let result = with_provider_application_session(
        SOURCE,
        ApplicationProviderSource::Ipc(&peer.path),
        entries(),
        vec![Value::Int(10)],
        |session, opened| {
            assert!(opened.presented_frames.is_empty());
            let mut clock = 0;
            for index in 1..=2 {
                let trace = session.event(vec![Value::Int(1)])?;
                assert_eq!(trace.presented_frames.len(), 1);
                assert_eq!(
                    trace.presented_frames[0].rgba8,
                    Some(vec![index, 2, 3, 255])
                );
                let witness = &trace.provider_completion_witnesses["draw"];
                assert!(witness.completion_clock > clock);
                clock = witness.completion_clock;
                assert!(!trace
                    .lane_steps
                    .values()
                    .flatten()
                    .any(|step| step.ends_with("-> main_print")));
            }
            let closed = session.close(vec![Value::Int(1)])?.unwrap();
            assert!(closed.presented_frames.is_empty());
            assert!(session.close(vec![Value::Int(1)])?.is_none());
            Ok(session.state().clone())
        },
    )
    .unwrap();
    assert!(
        matches!(result, Value::Struct(value) if value.fields == vec![("count".to_owned(), Value::Int(12))])
    );
    assert_eq!(peer.finish(), (2, true));
}

#[test]
fn missing_close_driver_error_and_swallowed_failures_do_not_acknowledge_success() {
    for case in 0..4 {
        let peer = Peer::start(if case == 3 {
            Reply::RejectedFrame
        } else {
            Reply::Good
        });
        let result = with_provider_application_session(
            SOURCE,
            ApplicationProviderSource::Ipc(&peer.path),
            entries(),
            vec![Value::Int(10)],
            |session, _| {
                if case == 3 {
                    assert!(session.event(vec![Value::Int(1)]).is_err());
                }
                if case == 2 {
                    assert!(session.close(vec![Value::Int(0)]).is_err());
                }
                if case == 1 || case == 3 {
                    session.close(vec![Value::Int(1)])?;
                }
                if case == 1 {
                    return Err("injected driver failure".to_owned());
                }
                Ok(())
            },
        );
        assert!(result.is_err(), "accepted failure case {case}");
        assert_eq!(peer.finish(), (usize::from(case == 3), false));
    }
}

#[test]
fn wrong_module_and_close_acknowledgement_are_rejected() {
    for reply in [Reply::WrongHash, Reply::BadClose] {
        let peer = Peer::start(reply);
        let result = with_provider_application_session(
            SOURCE,
            ApplicationProviderSource::Ipc(&peer.path),
            entries(),
            vec![Value::Int(10)],
            |session, _| {
                session.event(vec![Value::Int(1)])?;
                session.close(vec![Value::Int(1)])?;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(
            peer.finish(),
            if matches!(reply, Reply::WrongHash) {
                (0, false)
            } else {
                (1, true)
            }
        );
    }
}

#[test]
fn invalid_binding_is_rejected_before_connecting() {
    let path = PathBuf::from("missing-application-provider-socket");
    let mut binding = entries();
    binding.event = "unknown";
    let error = with_provider_application_session(
        SOURCE,
        ApplicationProviderSource::Ipc(&path),
        binding,
        vec![Value::Int(10)],
        |_, _| Ok(()),
    )
    .unwrap_err();
    assert!(error.contains("unknown function"), "{error}");
}

#[test]
fn invalid_registered_bindings_and_inputs_fail_before_transport() {
    let declaration = format!(
        "application-session app {} open update close state\n",
        yir_core::APPLICATION_SESSION_CONTRACT
    );
    let source = format!("{declaration}{SOURCE}");
    let path = PathBuf::from("missing-registered-session-provider-socket");
    for (input, id, argument) in [
        (source.clone(), "missing", Value::Int(10)),
        (source.clone(), "app", Value::Bool(false)),
        (format!("{declaration}{source}"), "app", Value::Int(10)),
        (
            source.replace("open update close state", "open missing close state"),
            "app",
            Value::Int(10),
        ),
        (
            source.replace(yir_core::APPLICATION_SESSION_CONTRACT, "future-contract"),
            "app",
            Value::Int(10),
        ),
    ] {
        let error = with_registered_provider_application_session::<()>(
            &input,
            ApplicationProviderSource::Ipc(&path),
            id,
            vec![argument],
            |_, _| panic!("invalid registration or input executed open"),
        )
        .unwrap_err();
        assert!(
            !error.contains("connection failed"),
            "preflight attempted a connection: {error}"
        );
    }
}
