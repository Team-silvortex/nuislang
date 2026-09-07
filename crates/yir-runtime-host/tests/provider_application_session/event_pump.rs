use super::*;
use std::sync::mpsc::{self, Receiver, SyncSender};
use yir_runtime_host::{
    ApplicationEventPump, ApplicationPumpOperation, ApplicationPumpPhase, ApplicationPumpReply,
};

fn registered_source() -> String {
    format!(
        "application-session counter {} open update close state\n{SOURCE}",
        yir_core::APPLICATION_SESSION_CONTRACT
    )
}

fn spawn(peer: &Peer) -> ApplicationEventPump {
    ApplicationEventPump::spawn(
        registered_source(),
        ApplicationProviderSource::Ipc(&peer.path),
        "counter".to_owned(),
        vec![Value::Int(10)],
    )
    .unwrap()
}

fn receive(pump: &mut ApplicationEventPump) -> ApplicationPumpReply {
    let reply = pump
        .wait(Duration::from_secs(5))
        .unwrap()
        .expect("pump reply");
    assert_eq!(pump.phase(), reply.phase);
    assert_eq!(pump.pending(), None);
    reply
}

fn count(reply: &ApplicationPumpReply) -> i64 {
    let Some(Value::Struct(value)) = &reply.state else {
        panic!("missing state")
    };
    let [(_, Value::Int(value))] = value.fields.as_slice() else {
        panic!("missing count")
    };
    *value
}

fn gated_peer(at: Pause) -> (Peer, Receiver<()>, SyncSender<()>) {
    let (ready, waiting) = mpsc::sync_channel(1);
    let (release, resume) = mpsc::sync_channel(1);
    let peer = Peer::start_source(
        Reply::Good,
        registered_source(),
        Some(Gate {
            at,
            ready,
            release: resume,
        }),
    );
    (peer, waiting, release)
}

#[test]
fn owned_pump_keeps_one_session_across_one_hundred_host_deliveries() {
    fn assert_send<T: Send>() {}
    assert_send::<ApplicationEventPump>();
    let peer = Peer::start_source(Reply::Good, registered_source(), None);
    let mut pump = spawn(&peer);
    assert_eq!(pump.pending(), Some(ApplicationPumpOperation::Open));
    assert!(pump.event(vec![]).unwrap_err().contains("busy"));
    let opened = receive(&mut pump);
    assert_eq!(opened.operation, ApplicationPumpOperation::Open);
    assert_eq!(count(&opened), 10);
    assert!(opened.trace.unwrap().presented_frames.is_empty());
    let mut clock = 0;
    let mut witness_nodes = None;
    for index in 1..=100 {
        pump.event(vec![Value::Int(1)]).unwrap();
        // Backpressure applies even if the worker already published its reply.
        assert!(pump
            .event(vec![Value::Int(1000)])
            .unwrap_err()
            .contains("busy"));
        assert!(pump
            .close(vec![Value::Int(1)])
            .unwrap_err()
            .contains("busy"));
        let reply = receive(&mut pump);
        assert_eq!(reply.operation, ApplicationPumpOperation::Event);
        assert_eq!(reply.phase, ApplicationPumpPhase::Open);
        assert_eq!(count(&reply), 10 + index);
        let trace = reply.trace.unwrap();
        assert!(trace.values.is_empty());
        assert_eq!(trace.presented_frames.len(), 1);
        assert_eq!(
            trace.presented_frames[0].rgba8,
            Some(vec![index as u8, 2, 3, 255])
        );
        let nodes = trace
            .provider_completion_witnesses
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(expected) = &witness_nodes {
            assert_eq!(
                &nodes, expected,
                "completion snapshots must not accumulate history"
            );
        } else {
            witness_nodes = Some(nodes);
        }
        let next = trace.provider_completion_witnesses["draw"].completion_clock;
        assert!(next > clock);
        clock = next;
        assert!(!trace
            .lane_steps
            .values()
            .flatten()
            .any(|step| step.ends_with("-> main_print")));
        assert!(pump.poll().unwrap().is_none());
    }
    pump.close(vec![Value::Int(1)]).unwrap();
    let closed = receive(&mut pump);
    assert_eq!(closed.operation, ApplicationPumpOperation::Close);
    assert_eq!(closed.phase, ApplicationPumpPhase::Closed);
    assert_eq!(count(&closed), 110);
    assert!(closed.trace.unwrap().presented_frames.is_empty());
    assert!(pump.close(vec![]).is_err());
    assert!(pump.event(vec![]).is_err());
    assert!(pump.poll().unwrap().is_none());
    assert_eq!(peer.finish(), (100, true));
}

#[test]
fn delayed_admission_and_frame_do_not_block_polling_or_allow_retries() {
    for at in [Pause::Hello, Pause::Frame] {
        let (peer, waiting, release) = gated_peer(at);
        let mut pump = spawn(&peer);
        if at == Pause::Frame {
            receive(&mut pump).trace.unwrap();
            pump.event(vec![Value::Int(1)]).unwrap();
        }
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        let pending = pump.pending();
        assert!(pump.poll().unwrap().is_none());
        assert!(pump.wait(Duration::from_millis(1)).unwrap().is_none());
        assert_eq!(pump.pending(), pending);
        assert!(pump.event(vec![Value::Int(1)]).is_err());
        assert!(pump.close(vec![Value::Int(1)]).is_err());
        release.send(()).unwrap();
        receive(&mut pump).trace.unwrap();
        pump.close(vec![Value::Int(1)]).unwrap();
        assert_eq!(receive(&mut pump).phase, ApplicationPumpPhase::Closed);
        assert_eq!(peer.finish(), (usize::from(at == Pause::Frame), true));
    }
}

#[test]
fn close_reply_waits_for_provider_acknowledgement() {
    let (peer, waiting, release) = gated_peer(Pause::Finish);
    let mut pump = spawn(&peer);
    receive(&mut pump).trace.unwrap();
    pump.close(vec![Value::Int(1)]).unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(pump.pending(), Some(ApplicationPumpOperation::Close));
    assert_ne!(pump.phase(), ApplicationPumpPhase::Closed);
    assert!(pump.poll().unwrap().is_none());
    assert!(pump.wait(Duration::from_millis(1)).unwrap().is_none());
    assert!(pump.close(vec![Value::Int(1)]).is_err());
    release.send(()).unwrap();
    let reply = receive(&mut pump);
    assert_eq!(reply.phase, ApplicationPumpPhase::Closed);
    reply.trace.unwrap();
    assert_eq!(peer.finish(), (0, true));
}

#[test]
fn invalid_ingress_can_be_corrected_without_faulting_or_provider_effects() {
    let peer = Peer::start_source(Reply::Good, registered_source(), None);
    let mut pump = spawn(&peer);
    receive(&mut pump).trace.unwrap();
    for closing in [false, true] {
        for arguments in [vec![], vec![Value::Bool(true)], vec![Value::Pointer(None)]] {
            if closing {
                pump.close(arguments)
            } else {
                pump.event(arguments)
            }
            .unwrap();
            let reply = receive(&mut pump);
            assert_eq!(reply.phase, ApplicationPumpPhase::Open);
            assert_eq!(count(&reply), 10);
            assert!(reply.trace.is_err());
        }
    }
    pump.event(vec![Value::Int(2)]).unwrap();
    assert_eq!(count(&receive(&mut pump)), 12);
    pump.close(vec![Value::Int(1)]).unwrap();
    assert_eq!(receive(&mut pump).phase, ApplicationPumpPhase::Closed);
    assert_eq!(peer.finish(), (1, true));
}

#[test]
fn failed_event_latches_and_successful_cleanup_cannot_certify_completion() {
    let peer = Peer::start_source(Reply::RejectedFrame, registered_source(), None);
    let mut pump = spawn(&peer);
    receive(&mut pump).trace.unwrap();
    pump.event(vec![Value::Int(1)]).unwrap();
    let failed = receive(&mut pump);
    assert_eq!(failed.phase, ApplicationPumpPhase::Faulted);
    assert_eq!(count(&failed), 10);
    assert!(failed
        .trace
        .unwrap_err()
        .contains("injected device failure"));
    assert!(pump.event(vec![Value::Int(1)]).is_err());
    pump.close(vec![Value::Bool(true)]).unwrap();
    assert_eq!(receive(&mut pump).phase, ApplicationPumpPhase::Faulted);
    pump.close(vec![Value::Int(1)]).unwrap();
    let closed = receive(&mut pump);
    assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
    assert_eq!(count(&closed), 10);
    assert!(closed
        .trace
        .unwrap_err()
        .contains("did not complete successfully"));
    assert!(pump.close(vec![Value::Int(1)]).is_err());
    assert_eq!(peer.finish(), (1, false));
}

#[test]
fn close_callback_failure_and_bad_finish_ack_are_terminal_not_retried() {
    for (reply, divisor, finished) in [(Reply::Good, 0, false), (Reply::BadClose, 1, true)] {
        let peer = Peer::start_source(reply, registered_source(), None);
        let mut pump = spawn(&peer);
        receive(&mut pump).trace.unwrap();
        pump.close(vec![Value::Int(divisor)]).unwrap();
        let closed = receive(&mut pump);
        assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
        assert!(closed.trace.is_err());
        assert!(pump.close(vec![Value::Int(1)]).is_err());
        assert!(pump.event(vec![Value::Int(1)]).is_err());
        assert_eq!(peer.finish(), (0, finished));
    }
}

#[test]
fn drop_and_abort_do_not_implicitly_close_an_idle_session() {
    for abort in [false, true] {
        let peer = Peer::start_source(Reply::Good, registered_source(), None);
        let mut pump = spawn(&peer);
        receive(&mut pump).trace.unwrap();
        if abort {
            pump.abort();
            assert_eq!(pump.phase(), ApplicationPumpPhase::Stopped);
            assert!(pump.poll().unwrap().is_none());
            assert!(pump.event(vec![Value::Int(1)]).is_err());
        }
        drop(pump);
        assert_eq!(peer.finish(), (0, false));
    }
}

#[test]
fn abort_does_not_join_an_in_flight_callback_or_queue_cleanup() {
    let (peer, waiting, release) = gated_peer(Pause::Frame);
    let mut pump = spawn(&peer);
    receive(&mut pump).trace.unwrap();
    pump.event(vec![Value::Int(1)]).unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    // The peer cannot resume until abort returns; joining here would deadlock.
    pump.abort();
    assert_eq!(pump.pending(), None);
    release.send(()).unwrap();
    assert_eq!(peer.finish(), (1, false));
}

#[test]
fn opening_failures_are_reported_without_fallback_or_retry() {
    let path = PathBuf::from("missing-pump-provider-socket");
    let mut pump = ApplicationEventPump::spawn(
        registered_source(),
        ApplicationProviderSource::Ipc(&path),
        "unknown".to_owned(),
        vec![Value::Int(10)],
    )
    .unwrap();
    let reply = receive(&mut pump);
    assert_eq!(reply.operation, ApplicationPumpOperation::Open);
    assert_eq!(reply.phase, ApplicationPumpPhase::Stopped);
    assert!(reply.state.is_none());
    let error = reply.trace.unwrap_err();
    assert!(!error.contains("connection failed"), "{error}");
    assert!(pump.event(vec![]).is_err());

    let peer = Peer::start_source(Reply::WrongHash, registered_source(), None);
    let mut pump = spawn(&peer);
    let reply = receive(&mut pump);
    assert_eq!(reply.phase, ApplicationPumpPhase::Stopped);
    assert!(reply.trace.unwrap_err().contains("different YIR module"));
    assert_eq!(peer.finish(), (0, false));
}

#[test]
fn provider_dispatch_budget_is_not_reset_between_host_events() {
    let peer = Peer::start_source(Reply::Good, registered_source(), None);
    let mut pump = spawn(&peer);
    receive(&mut pump).trace.unwrap();
    let limit = yir_core::provider_runtime_ipc::MAX_DISPATCHES;
    for _ in 0..limit {
        pump.event(vec![Value::Int(1)]).unwrap();
        receive(&mut pump).trace.unwrap();
    }
    pump.event(vec![Value::Int(1)]).unwrap();
    let reply = receive(&mut pump);
    assert_eq!(reply.phase, ApplicationPumpPhase::Faulted);
    assert_eq!(count(&reply), 10 + limit as i64);
    assert!(reply
        .trace
        .unwrap_err()
        .contains("invocation limit rejected"));
    pump.close(vec![Value::Int(1)]).unwrap();
    let reply = receive(&mut pump);
    assert_eq!(reply.phase, ApplicationPumpPhase::Stopped);
    assert!(reply.trace.is_err());
    assert_eq!(peer.finish(), (limit, false));
}

#[test]
fn abort_cannot_retract_an_already_admitted_close() {
    let (peer, waiting, release) = gated_peer(Pause::Finish);
    let mut pump = spawn(&peer);
    receive(&mut pump).trace.unwrap();
    pump.close(vec![Value::Int(1)]).unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    pump.abort();
    release.send(()).unwrap();
    assert_eq!(peer.finish(), (0, true));
}
