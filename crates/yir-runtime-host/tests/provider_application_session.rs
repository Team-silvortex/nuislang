#![cfg(unix)]

use std::{
    fs,
    os::unix::net::UnixListener,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    thread::{self, JoinHandle},
    time::Duration,
};
use yir_core::{
    provider_runtime_ipc::{hash_bytes, DispatchFrame, DispatchTarget, Message},
    ProviderPhysicalCompletion, Value,
};
use yir_runtime_host::{
    with_provider_application_session, ApplicationProviderSource, ApplicationSessionEntries,
};

const SOURCE: &str = r#"
yir 0.1
resource cpu0 cpu.arm64
resource gpu shader.metal
function open cpu helper
function-param open seed i64 value seed
function-result open Counter owned opened
function-node open seed
function-node open opened
function update cpu helper
function-param update state.count i64 value count
function-param update delta i64 value delta
function-result update Counter owned updated
function-node update count
function-node update delta
function-node update next
function-node update pass
function-node update draw
function-node update present
function-node update updated
function close cpu helper
function-param close state.count i64 value final_count
function-param close divisor i64 value divisor
function-result close Counter owned closed
function-node close final_count
function-node close divisor
function-node close quotient
function-node close closed
function main cpu entry
function-node main main_print
cpu.param_i64 seed cpu0 0
cpu.struct opened cpu0 Counter count=seed
cpu.param_i64 count cpu0 0
cpu.param_i64 delta cpu0 1
cpu.add next cpu0 count delta
shader.target target gpu rgba8_unorm 1 1
shader.viewport view gpu 1 1
shader.pipeline pipeline gpu ball triangle_strip
shader.const_i64 color gpu 1
shader.const_i64 speed gpu 2
shader.pack_ball_state packet gpu color speed
shader.begin_pass pass gpu target pipeline view
shader.draw_instanced draw gpu pass packet 4 1
cpu.present_frame present cpu0 draw
cpu.struct updated cpu0 Counter count=next
cpu.param_i64 final_count cpu0 0
cpu.param_i64 divisor cpu0 1
cpu.div quotient cpu0 final_count divisor
cpu.struct closed cpu0 Counter count=quotient
cpu.const_i64 sentinel cpu0 999
cpu.print main_print cpu0 sentinel
edge dep seed opened
edge dep count next
edge dep delta next
edge dep next updated
edge dep target pass
edge dep pipeline pass
edge dep view pass
edge dep color packet
edge dep speed packet
edge dep pass draw
edge dep packet draw
edge dep draw present
edge dep present updated
edge dep final_count quotient
edge dep divisor quotient
edge dep quotient closed
edge dep sentinel main_print
"#;

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
    WrongHash,
}

struct Peer {
    path: PathBuf,
    worker: Option<JoinHandle<(usize, bool)>>,
}

impl Peer {
    fn start(reply: Reply) -> Self {
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
                    SOURCE.as_bytes()
                }),
                module: "shader".to_owned(),
                instruction: "draw_instanced".to_owned(),
                node: "draw".to_owned(),
                resource: "gpu".to_owned(),
            };
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
                        if matches!(reply, Reply::RejectedFrame) {
                            Message::Rejected("injected device failure".to_owned())
                                .write_to(&mut stream)
                                .unwrap();
                            continue;
                        }
                        Message::Frame(DispatchFrame {
                            sequence,
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
                        Message::Closed(if matches!(reply, Reply::BadClose) {
                            count + 1
                        } else {
                            count
                        })
                        .write_to(&mut stream)
                        .unwrap();
                        return (count, true);
                    }
                    other => panic!("unexpected client message: {other:?}"),
                }
            }
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
