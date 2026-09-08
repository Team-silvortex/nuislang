use super::*;
use yir_runtime_host::ProviderDrainObservation;

#[test]
fn cancellation_observes_zero_and_two_frame_drain_without_application_success() {
    for count in [0, 2] {
        let peer = Peer::start_source(Reply::DrainGood, source(false), None);
        let mut pump = spawn(&peer, source(false));
        receive(&mut pump).trace.unwrap();
        for _ in 0..count {
            pump.event(vec![Value::Int(1)]).unwrap();
            assert_eq!(receive(&mut pump).trace.unwrap().presented_frames.len(), 1);
        }
        let mut ticket = pump.cancel_with_provider_drain().unwrap();
        assert_eq!(pump.phase(), ApplicationPumpPhase::Stopped);
        assert!(pump.close(vec![Value::Int(1)]).is_err());
        assert!(pump.poll().unwrap().is_none());
        drop(pump);
        let ack = retired(&mut ticket);
        assert!(!ack.cleanup_completed());
        assert_eq!(ack.failure_kind(), ApplicationFailureKind::None);
        assert_eq!(
            ack.provider_drain(),
            ProviderDrainObservation::Drained {
                completed_dispatches: count
            }
        );
        assert_eq!(peer.finish(), (count, false));
    }
}

#[test]
fn in_flight_hello_frame_and_explicit_close_drain_only_after_their_boundary() {
    for (at, closing) in [
        (Pause::Hello, false),
        (Pause::Frame, false),
        (Pause::Frame, true),
    ] {
        let (peer, waiting, release) = gated_peer(at, Reply::DrainGood, source(closing));
        let mut pump = spawn(&peer, source(closing));
        if at == Pause::Frame {
            receive(&mut pump).trace.unwrap();
            if closing {
                pump.close(vec![Value::Int(1)]).unwrap();
            } else {
                pump.event(vec![Value::Int(1)]).unwrap();
            }
        }
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        let mut ticket = pump.cancel_with_provider_drain().unwrap();
        assert!(ticket.wait(Duration::from_millis(1)).unwrap().is_none());
        drop(pump);
        release.send(()).unwrap();
        let ack = retired(&mut ticket);
        assert_eq!(ack.cleanup_completed(), closing);
        assert_eq!(ack.failure_kind(), ApplicationFailureKind::None);
        let count = usize::from(at == Pause::Frame);
        assert_eq!(
            ack.provider_drain(),
            ProviderDrainObservation::Drained {
                completed_dispatches: count
            }
        );
        assert_eq!(peer.finish(), (count, false));
    }
}

#[test]
fn drain_receipt_waits_are_observation_only_and_never_retry() {
    let (peer, waiting, release) = gated_peer(Pause::Drain, Reply::DrainGood, source(false));
    let mut pump = spawn(&peer, source(false));
    receive(&mut pump).trace.unwrap();
    let mut ticket = pump.cancel_with_provider_drain().unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    for _ in 0..3 {
        assert!(ticket.poll().unwrap().is_none());
        assert!(ticket.wait(Duration::from_millis(1)).unwrap().is_none());
    }
    release.send(()).unwrap();
    assert_eq!(
        retired(&mut ticket).provider_drain(),
        ProviderDrainObservation::Drained {
            completed_dispatches: 0
        }
    );
    assert_eq!(peer.finish(), (0, false));
}

#[test]
fn drain_failures_are_separate_observations_and_do_not_replace_first_faults() {
    for (reply, kind) in [
        (
            Reply::DrainRejected,
            ApplicationFailureKind::ProviderFinalization,
        ),
        (
            Reply::DrainWrongTarget,
            ApplicationFailureKind::ProviderContract,
        ),
        (
            Reply::DrainDisconnected,
            ApplicationFailureKind::ProviderExchange,
        ),
    ] {
        for prior_fault in [false, true] {
            let source = source(false)
                .replace(
                    "cpu.add next cpu0 count delta",
                    "cpu.div next cpu0 count delta",
                )
                .replace(
                    "edge dep target pass",
                    "edge dep target pass\nedge dep next pass",
                );
            let peer = Peer::start_source(reply, source.clone(), None);
            let mut pump = spawn(&peer, source);
            receive(&mut pump).trace.unwrap();
            if prior_fault {
                // An actual callback fault precedes the draw, not an ingress
                // rejection (which is intentionally recoverable and unlatched).
                pump.event(vec![Value::Int(0)]).unwrap();
                let failed = receive(&mut pump);
                assert!(failed.trace.is_err());
                assert_eq!(failed.failure_kind, ApplicationFailureKind::Callback);
            }
            let ack = retired(&mut pump.cancel_with_provider_drain().unwrap());
            assert!(!ack.cleanup_completed());
            assert_eq!(ack.provider_drain(), ProviderDrainObservation::Failed(kind));
            assert_eq!(
                ack.failure_kind(),
                if prior_fault {
                    ApplicationFailureKind::Callback
                } else {
                    kind
                }
            );
            assert_eq!(peer.finish(), (0, false));
        }
    }
}

#[test]
fn damaged_frame_frontiers_drop_without_attempting_drain_or_clearing_faults() {
    for (reply, kind) in [
        (
            Reply::WrongSequence,
            ApplicationFailureKind::ProviderContract,
        ),
        (
            Reply::DisconnectedFrame,
            ApplicationFailureKind::ProviderExchange,
        ),
        (
            Reply::TypedRejectedFrame(RejectionCode::Execution),
            ApplicationFailureKind::ProviderExecution,
        ),
    ] {
        let (peer, waiting, release) = gated_peer(Pause::Frame, reply, source(false));
        let mut pump = spawn(&peer, source(false));
        receive(&mut pump).trace.unwrap();
        pump.event(vec![Value::Int(1)]).unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        let mut ticket = pump.cancel_with_provider_drain().unwrap();
        release.send(()).unwrap();
        let ack = retired(&mut ticket);
        assert_eq!(ack.provider_drain(), ProviderDrainObservation::Unavailable);
        assert_eq!(ack.failure_kind(), kind);
        assert_eq!(peer.finish(), (1, false));
    }
}

#[test]
fn finish_winner_rejects_drain_cancel_without_consuming_its_close_reply() {
    for reply in [Reply::Good, Reply::RejectedClose] {
        let (peer, waiting, release) = gated_peer(Pause::Finish, reply, source(false));
        let mut pump = spawn(&peer, source(false));
        receive(&mut pump).trace.unwrap();
        pump.close(vec![Value::Int(1)]).unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(pump
            .cancel_with_provider_drain()
            .unwrap_err()
            .contains("finalization already admitted"));
        assert_eq!(pump.pending(), Some(ApplicationPumpOperation::Close));
        release.send(()).unwrap();
        let closed = receive(&mut pump);
        assert!(closed.cleanup_completed);
        assert_eq!(closed.trace.is_ok(), matches!(reply, Reply::Good));
        assert_eq!(peer.finish(), (0, true));
    }
}

#[test]
fn dropping_a_drain_ticket_does_not_join_or_revoke_an_admitted_drain() {
    let (peer, waiting, release) = gated_peer(Pause::Drain, Reply::DrainGood, source(false));
    let mut pump = spawn(&peer, source(false));
    receive(&mut pump).trace.unwrap();
    let ticket = pump.cancel_with_provider_drain().unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(ticket);
    drop(pump);
    release.send(()).unwrap();
    assert_eq!(peer.finish(), (0, false));
}
