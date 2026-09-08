use super::*;
use std::sync::mpsc::{self, Receiver, SyncSender};
use yir_core::ApplicationFailureKind;
use yir_runtime_host::{
    ApplicationCancellation, ApplicationEventPump, ApplicationHostRetirementAck,
    ApplicationPumpOperation, ApplicationPumpPhase, ApplicationPumpReply,
};

#[path = "provider_drain.rs"]
mod provider_drain;

fn source(drawing_close: bool) -> String {
    let callbacks = if drawing_close {
        "open close update"
    } else {
        "open update close"
    };
    format!(
        "application-session counter {} {callbacks} state\n{SOURCE}",
        yir_core::APPLICATION_SESSION_CONTRACT
    )
}

fn spawn(peer: &Peer, source: String) -> ApplicationEventPump {
    ApplicationEventPump::spawn(
        source,
        ApplicationProviderSource::Ipc(&peer.path),
        "counter".to_owned(),
        vec![Value::Int(10)],
    )
    .unwrap()
}

fn receive(pump: &mut ApplicationEventPump) -> ApplicationPumpReply {
    pump.wait(Duration::from_secs(5))
        .unwrap()
        .expect("pump reply")
}

fn retired(cancellation: &mut ApplicationCancellation) -> ApplicationHostRetirementAck {
    let ack = cancellation
        .wait(Duration::from_secs(5))
        .unwrap()
        .expect("host retirement");
    assert!(cancellation.poll().unwrap().is_none());
    assert!(cancellation.wait(Duration::ZERO).unwrap().is_none());
    ack
}

fn gated_peer(at: Pause, reply: Reply, source: String) -> (Peer, Receiver<()>, SyncSender<()>) {
    let (ready, waiting) = mpsc::sync_channel(1);
    let (release, resume) = mpsc::sync_channel(1);
    let peer = Peer::start_source(
        reply,
        source,
        Some(Gate {
            at,
            ready,
            release: resume,
        }),
    );
    (peer, waiting, release)
}

#[test]
fn cancellation_waits_for_in_flight_open_or_event_without_fabricating_cleanup() {
    for at in [Pause::Hello, Pause::Frame] {
        let (peer, waiting, release) = gated_peer(at, Reply::Good, source(false));
        let mut pump = spawn(&peer, source(false));
        if at == Pause::Frame {
            receive(&mut pump).trace.unwrap();
            pump.event(vec![Value::Int(1)]).unwrap();
        }
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        let mut cancellation = pump.cancel().unwrap();
        assert_eq!(pump.phase(), ApplicationPumpPhase::Stopped);
        assert_eq!(pump.pending(), None);
        assert!(pump.event(vec![Value::Int(1)]).is_err());
        assert!(pump.close(vec![Value::Int(1)]).is_err());
        assert!(pump.cancel().is_err());
        assert!(pump.poll().unwrap().is_none());
        assert!(cancellation.poll().unwrap().is_none());
        assert!(cancellation
            .wait(Duration::from_millis(1))
            .unwrap()
            .is_none());
        // Admission did not interrupt the in-flight exchange or join its worker.
        release.send(()).unwrap();
        let ack = retired(&mut cancellation);
        assert!(!ack.cleanup_completed());
        assert_eq!(ack.failure_kind(), ApplicationFailureKind::None);
        assert_eq!(peer.finish(), (usize::from(at == Pause::Frame), false));
    }
}

#[test]
fn idle_cancel_retires_without_close_and_ticket_drop_does_not_join() {
    let peer = Peer::start_source(Reply::Good, source(false), None);
    let mut pump = spawn(&peer, source(false));
    receive(&mut pump).trace.unwrap();
    assert!(!retired(&mut pump.cancel().unwrap()).cleanup_completed());
    assert_eq!(peer.finish(), (0, false));

    let (peer, waiting, release) = gated_peer(Pause::Frame, Reply::Good, source(false));
    let mut pump = spawn(&peer, source(false));
    receive(&mut pump).trace.unwrap();
    pump.event(vec![Value::Int(1)]).unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(pump.cancel().unwrap());
    drop(pump);
    release.send(()).unwrap();
    assert_eq!(peer.finish(), (1, false));
}

#[test]
fn accepted_cancel_allows_explicit_close_to_return_but_prevents_finish() {
    let (peer, waiting, release) = gated_peer(Pause::Frame, Reply::Good, source(true));
    let mut pump = spawn(&peer, source(true));
    receive(&mut pump).trace.unwrap();
    pump.close(vec![Value::Int(1)]).unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut cancellation = pump.cancel().unwrap();
    assert!(cancellation.poll().unwrap().is_none());
    release.send(()).unwrap();
    let ack = retired(&mut cancellation);
    assert!(
        ack.cleanup_completed(),
        "only the explicitly admitted close ran"
    );
    assert_eq!(ack.failure_kind(), ApplicationFailureKind::None);
    assert_eq!(peer.finish(), (1, false), "cancel must prevent Finish");
}

#[test]
fn finish_winning_admission_rejects_cancel_without_losing_the_original_reply() {
    for reply in [Reply::Good, Reply::RejectedClose] {
        let (peer, waiting, release) = gated_peer(Pause::Finish, reply, source(false));
        let mut pump = spawn(&peer, source(false));
        receive(&mut pump).trace.unwrap();
        pump.close(vec![Value::Int(1)]).unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(pump
            .cancel()
            .unwrap_err()
            .contains("finalization already admitted"));
        assert_eq!(pump.pending(), Some(ApplicationPumpOperation::Close));
        assert!(pump.poll().unwrap().is_none());
        release.send(()).unwrap();
        let closed = receive(&mut pump);
        assert!(closed.cleanup_completed);
        match reply {
            Reply::Good => {
                assert_eq!(closed.phase, ApplicationPumpPhase::Closed);
                closed.trace.unwrap();
            }
            _ => {
                assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
                assert_eq!(
                    closed.failure_kind,
                    ApplicationFailureKind::ProviderFinalization
                );
                assert!(closed.trace.is_err());
            }
        }
        assert!(pump.cancel().is_err());
        assert_eq!(peer.finish(), (0, true));
    }
}

#[test]
fn cancellation_keeps_typed_faults_before_and_after_its_admission() {
    for before_cancel in [false, true] {
        let (peer, waiting, release) = gated_peer(
            Pause::Frame,
            Reply::TypedRejectedFrame(RejectionCode::Execution),
            source(false),
        );
        let mut pump = spawn(&peer, source(false));
        receive(&mut pump).trace.unwrap();
        pump.event(vec![Value::Int(1)]).unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        if before_cancel {
            release.send(()).unwrap();
            let failure = receive(&mut pump);
            assert_eq!(failure.phase, ApplicationPumpPhase::Faulted);
            assert_eq!(
                failure.failure_kind,
                ApplicationFailureKind::ProviderExecution
            );
        }
        let mut cancellation = pump.cancel().unwrap();
        if !before_cancel {
            release.send(()).unwrap();
        }
        let ack = retired(&mut cancellation);
        assert!(!ack.cleanup_completed());
        assert_eq!(
            ack.failure_kind(),
            ApplicationFailureKind::ProviderExecution
        );
        assert_eq!(peer.finish(), (1, false));
    }
}

#[test]
fn unconsumed_callback_reply_does_not_block_retirement_or_allow_another_event() {
    let peer = Peer::start_source(Reply::Good, source(false), None);
    let mut pump = spawn(&peer, source(false));
    receive(&mut pump).trace.unwrap();
    pump.event(vec![Value::Int(1)]).unwrap();
    // Whether queued, executing or already replied, no second callback is admitted.
    let mut cancellation = pump.cancel().unwrap();
    assert!(pump.event(vec![Value::Int(1)]).is_err());
    assert!(!retired(&mut cancellation).cleanup_completed());
    let (dispatches, finished) = peer.finish();
    assert!(dispatches <= 1);
    assert!(!finished);
}
