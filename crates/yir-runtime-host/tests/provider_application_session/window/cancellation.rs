use super::*;
use std::{ptr, sync::mpsc};
use yir_runtime_host::{
    nuis_application_cancellation_free, nuis_application_cancellation_poll,
    nuis_window_session_cancel, nuis_window_session_close, nuis_window_session_event,
    nuis_window_session_free, nuis_window_session_outcome_field, ApplicationCancellation,
};

fn gated(at: Pause, reply: Reply) -> (Peer, mpsc::Receiver<()>, mpsc::SyncSender<()>) {
    let (ready, waiting) = mpsc::sync_channel(1);
    let (release, resume) = mpsc::sync_channel(1);
    (
        Peer::start_source(
            reply,
            source(),
            Some(Gate {
                at,
                ready,
                release: resume,
            }),
        ),
        waiting,
        release,
    )
}

fn spawn(peer: &Peer) -> WindowSession {
    WindowSession::spawn(
        source(),
        ApplicationProviderSource::Ipc(&peer.path),
        "ui".to_owned(),
        640,
        400,
    )
    .unwrap()
}

fn assert_cancelled_without_outcome(session: &mut WindowSession) {
    assert_eq!(session.phase(), ApplicationPumpPhase::Stopped);
    assert!(!session.pending());
    assert!(session.event(0, 0).is_err());
    assert!(session.close().is_err());
    assert!(session.cancel().is_err());
    assert!(session.poll().unwrap().is_none());
    assert!(session.outcome().is_none());
    assert!(session.take_outcome_delivery().is_none());
    assert!(!session.cleanup_completed());
}

#[test]
fn cancelled_window_open_and_event_never_manufacture_parent_outcomes() {
    for at in [Pause::Hello, Pause::Frame] {
        let (peer, waiting, release) = gated(at, Reply::Good);
        let mut session = spawn(&peer);
        if at == Pause::Frame {
            receive(&mut session).frame.unwrap();
            session.event(0, 0).unwrap();
        }
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(session.pending());
        let mut ticket = session.cancel().unwrap();
        assert_cancelled_without_outcome(&mut session);
        assert_eq!(session.close_reason(), None);
        assert_eq!(session.failure_kind(), ApplicationFailureKind::None);
        assert!(ticket.poll().unwrap().is_none());
        release.send(()).unwrap();
        let ack = ticket.wait(Duration::from_secs(5)).unwrap().unwrap();
        assert!(!ack.cleanup_completed());
        assert_eq!(ack.failure_kind(), ApplicationFailureKind::None);
        assert_cancelled_without_outcome(&mut session);
        assert!(ticket.poll().unwrap().is_none());
        assert_eq!(peer.finish(), (usize::from(at == Pause::Frame), false));
    }
}

#[test]
fn window_cancellation_preserves_observed_and_in_flight_provider_failures() {
    for observed in [false, true] {
        let (peer, waiting, release) = gated(
            Pause::Frame,
            Reply::TypedRejectedFrame(RejectionCode::Execution),
        );
        let mut session = spawn(&peer);
        receive(&mut session).frame.unwrap();
        session.event(0, 0).unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        if observed {
            release.send(()).unwrap();
            let reply = receive(&mut session);
            assert_eq!(reply.phase, ApplicationPumpPhase::Faulted);
            assert!(reply.frame.is_err());
            assert_eq!(
                session.failure_kind(),
                ApplicationFailureKind::ProviderExecution
            );
        }
        let observed_failure = session.failure_kind();
        let mut ticket = session.cancel().unwrap();
        assert_cancelled_without_outcome(&mut session);
        if !observed {
            release.send(()).unwrap();
        }
        let ack = ticket.wait(Duration::from_secs(5)).unwrap().unwrap();
        assert!(!ack.cleanup_completed());
        assert_eq!(
            ack.failure_kind(),
            ApplicationFailureKind::ProviderExecution
        );
        // The independent receipt does not mutate an abandoned window snapshot.
        assert_eq!(session.failure_kind(), observed_failure);
        assert_eq!(peer.finish(), (1, false));
    }
}

#[test]
fn window_cancel_rejection_keeps_explicit_close_and_finish_reply() {
    for reply in [Reply::Good, Reply::RejectedClose] {
        let (peer, waiting, release) = gated(Pause::Finish, reply);
        let mut session = spawn(&peer);
        receive(&mut session).frame.unwrap();
        session.close().unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(session
            .cancel()
            .unwrap_err()
            .contains("finalization already admitted"));
        let mut ticket = ptr::null_mut();
        assert_eq!(
            unsafe { nuis_window_session_cancel(&mut session, &mut ticket) },
            -1
        );
        assert!(ticket.is_null());
        assert!(session.pending());
        assert_eq!(
            session.close_reason(),
            Some(ApplicationCloseReason::Requested)
        );
        assert!(session.outcome().is_none());
        release.send(()).unwrap();
        let closed = receive(&mut session);
        assert!(closed.cleanup_completed);
        match reply {
            Reply::Good => {
                assert_eq!(closed.phase, ApplicationPumpPhase::Closed);
                closed.frame.unwrap();
                assert_eq!(closed.outcome.unwrap().codes(), [1, 1, 0]);
            }
            _ => {
                assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
                assert!(closed.frame.is_err());
                assert_eq!(
                    closed.failure_kind,
                    ApplicationFailureKind::ProviderFinalization
                );
                assert_eq!(closed.outcome.unwrap().codes()[0], 2);
            }
        }
        assert_eq!(
            session.take_outcome_delivery().unwrap().outcome(),
            closed.outcome.unwrap()
        );
        assert!(session.take_outcome_delivery().is_none());
        assert_eq!(peer.finish(), (0, true));
    }
}

#[test]
fn window_ffi_cancellation_ticket_outlives_window_without_losing_acknowledgement() {
    let (peer, waiting, release) = gated(Pause::Frame, Reply::Good);
    let mut session = spawn(&peer);
    receive(&mut session).frame.unwrap();
    session.event(0, 0).unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut session = Box::into_raw(Box::new(session));
    let mut ticket = ptr::null_mut();
    let (mut cleanup, mut failure) = (-7, -8);
    unsafe {
        assert_eq!(nuis_window_session_cancel(ptr::null_mut(), &mut ticket), -1);
        assert_eq!(nuis_window_session_cancel(session, ptr::null_mut()), -1);
        // A nonempty slot is never dereferenced, overwritten or silently freed.
        let sentinel = ptr::NonNull::<ApplicationCancellation>::dangling().as_ptr();
        let mut occupied = sentinel;
        assert_eq!(nuis_window_session_cancel(session, &mut occupied), -1);
        assert_eq!(occupied, sentinel);
        assert!((*session).pending());
        assert_eq!((*session).phase(), ApplicationPumpPhase::Open);
        assert_eq!(nuis_window_session_cancel(session, &mut ticket), 0);
        let admitted = ticket;
        assert_eq!(nuis_window_session_cancel(session, &mut ticket), -1);
        assert_eq!(ticket, admitted);
        assert_eq!(nuis_window_session_event(session, 0, 0), -1);
        assert_eq!(nuis_window_session_close(session), -1);
        for field in 0..3 {
            assert_eq!(nuis_window_session_outcome_field(session, field), -1);
        }
        assert!((*session).take_outcome_delivery().is_none());
        assert_eq!(
            nuis_application_cancellation_poll(ticket, &mut cleanup, &mut failure),
            0
        );
        assert_eq!((cleanup, failure), (-7, -8));
        nuis_window_session_free(&mut session);
        assert!(session.is_null());
        release.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let status = nuis_application_cancellation_poll(ticket, &mut cleanup, &mut failure);
            if status != 0 {
                assert_eq!(status, 1);
                break;
            }
            assert!(Instant::now() < deadline, "no retirement receipt");
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!((cleanup, failure), (0, 0));
        assert_eq!(
            nuis_application_cancellation_poll(ticket, &mut cleanup, &mut failure),
            0
        );
        nuis_application_cancellation_free(&mut ticket);
        assert!(ticket.is_null());
        nuis_application_cancellation_free(&mut ticket);
    }
    assert_eq!(peer.finish(), (1, false));
}

#[test]
fn window_ticket_and_session_drop_do_not_join_an_in_flight_callback() {
    let (peer, waiting, release) = gated(Pause::Frame, Reply::Good);
    let mut session = spawn(&peer);
    receive(&mut session).frame.unwrap();
    session.event(0, 0).unwrap();
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut session = Box::into_raw(Box::new(session));
    let mut ticket = ptr::null_mut();
    unsafe {
        assert_eq!(nuis_window_session_cancel(session, &mut ticket), 0);
        nuis_application_cancellation_free(&mut ticket);
        nuis_window_session_free(&mut session);
    }
    release.send(()).unwrap();
    assert_eq!(peer.finish(), (1, false));
}
