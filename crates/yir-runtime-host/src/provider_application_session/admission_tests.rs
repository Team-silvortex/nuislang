use super::*;
use std::{
    cell::RefCell,
    fs,
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};
use yir_core::provider_runtime_ipc::{hash_bytes, DispatchTarget, Message};

fn source() -> String {
    format!(
        "application-session counter {} open update close state\n{}",
        yir_core::APPLICATION_SESSION_CONTRACT,
        include_str!("../../tests/fixtures/application_session.yir")
    )
}

// Independent admission policy: no cancellation handle, atomics or GUI state.
#[derive(Default)]
struct RecordingAdmission {
    calls: RefCell<Vec<&'static str>>,
    reject_checkpoint: Option<usize>,
    reject_finalization: bool,
}

impl ScopeAdmission for RecordingAdmission {
    fn checkpoint(&self) -> Result<(), String> {
        let mut calls = self.calls.borrow_mut();
        calls.push("checkpoint");
        if self.reject_checkpoint == Some(calls.len()) {
            Err("independent policy denied scope entry".to_owned())
        } else {
            Ok(())
        }
    }

    fn admit_finalization(&self) -> Result<(), String> {
        self.calls.borrow_mut().push("finalization");
        if self.reject_finalization {
            Err("independent policy denied finalization".to_owned())
        } else {
            Ok(())
        }
    }
}

struct Peer {
    path: PathBuf,
    worker: Option<thread::JoinHandle<()>>,
}

impl Peer {
    fn start(source: &str, finish: bool) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ns-admission-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let target = DispatchTarget {
            source_yir_fnv1a64: hash_bytes(source.as_bytes()),
            module: "shader".to_owned(),
            instruction: "draw_instanced".to_owned(),
            node: "draw".to_owned(),
            resource: "gpu".to_owned(),
        };
        let worker = thread::spawn(move || {
            let mut stream = accept(listener);
            Message::Hello(target).write_to(&mut stream).unwrap();
            if finish {
                assert!(matches!(
                    Message::read_from(&mut stream).unwrap(),
                    Message::Finish(0)
                ));
                Message::Closed(0).write_to(&mut stream).unwrap();
            }
            assert_eq!(stream.read(&mut [0_u8; 1]).unwrap(), 0);
        });
        Self {
            path,
            worker: Some(worker),
        }
    }

    fn finish(mut self) {
        self.worker.take().unwrap().join().unwrap();
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn accept(listener: UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "scope never connected");
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("{error}"),
        }
    };
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}

#[test]
fn provider_scope_accepts_independent_admission_without_bypassing_lifecycle() {
    for (reject_finalization, driver_failure, divisor) in [
        (false, false, 1),
        (true, false, 1),
        (false, true, 1),
        (false, false, 0),
    ] {
        let source = source();
        let success = !reject_finalization && !driver_failure && divisor != 0;
        let peer = Peer::start(&source, success);
        let admission = RecordingAdmission {
            reject_finalization,
            ..Default::default()
        };
        let result = with_registered_provider_application_session_checked(
            &source,
            ApplicationProviderSource::Ipc(&peer.path),
            "counter",
            vec![Value::Int(10)],
            |_, _| Ok(()),
            SessionControl {
                admission: &admission,
                failures: FailureState::default(),
            },
            |session, _| {
                if driver_failure {
                    return Err("driver failed".to_owned());
                }
                session.close(vec![Value::Int(divisor)])?;
                Ok(())
            },
        );
        assert_eq!(result.is_ok(), success);
        if reject_finalization {
            assert!(result
                .unwrap_err()
                .contains("independent policy denied finalization"));
        }
        let mut expected = vec!["checkpoint", "checkpoint"];
        if !driver_failure && divisor != 0 {
            expected.push("finalization");
        }
        assert_eq!(*admission.calls.borrow(), expected);
        peer.finish();
    }
}

#[test]
fn independent_policy_rejects_before_provider_io() {
    let admission = RecordingAdmission {
        reject_checkpoint: Some(1),
        ..Default::default()
    };
    let result: Result<(), String> = with_registered_provider_application_session_checked(
        &source(),
        ApplicationProviderSource::Replay(Path::new("unopened-provider-replay")),
        "counter",
        vec![Value::Int(10)],
        |_, _| Ok(()),
        SessionControl {
            admission: &admission,
            failures: FailureState::default(),
        },
        |_, _| panic!("denied scope executed application driver"),
    );
    assert!(result
        .unwrap_err()
        .contains("independent policy denied scope entry"));
    assert_eq!(*admission.calls.borrow(), ["checkpoint"]);
}

#[test]
fn independent_policy_rejects_after_provider_admission_without_application_delivery() {
    let source = source();
    let peer = Peer::start(&source, false);
    let admission = RecordingAdmission {
        reject_checkpoint: Some(2),
        ..Default::default()
    };
    let result: Result<(), String> = with_registered_provider_application_session_checked(
        &source,
        ApplicationProviderSource::Ipc(&peer.path),
        "counter",
        vec![Value::Int(10)],
        |_, _| Ok(()),
        SessionControl {
            admission: &admission,
            failures: FailureState::default(),
        },
        |_, _| panic!("denied scope executed application driver"),
    );
    assert!(result
        .unwrap_err()
        .contains("independent policy denied scope entry"));
    assert_eq!(*admission.calls.borrow(), ["checkpoint", "checkpoint"]);
    peer.finish();
}

#[test]
fn provider_scope_has_no_direct_dependency_on_concrete_pump_or_cancellation() {
    // A direct-import regression guard, complemented by the replacement-policy
    // execution tests above; not a full transitive dependency proof.
    let source = include_str!("../provider_application_session.rs");
    for forbidden in [
        "application_cancellation",
        "CancellationControl",
        "ApplicationEventPump",
        "WindowSession",
        "ApplicationHostRetirementAck",
        "dyn ScopeAdmission",
    ] {
        assert!(
            !source.contains(forbidden),
            "provider scope depends on {forbidden}"
        );
    }
    let boundary = include_str!("../application_scope_admission.rs");
    for forbidden in [
        "CancellationControl",
        "ApplicationProviderSource",
        "WindowSession",
        "ModRegistry",
        "UnixStream",
        "Metal",
    ] {
        assert!(
            !boundary.contains(forbidden),
            "shared admission depends on {forbidden}"
        );
    }
}
