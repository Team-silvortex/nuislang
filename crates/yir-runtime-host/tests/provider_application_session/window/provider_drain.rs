use super::*;
use yir_runtime_host::{
    nuis_application_cancellation_poll_with_provider,
    nuis_window_session_cancel_with_provider_drain, NuisApplicationCancellationReceipt,
};

#[test]
fn window_drain_ffi_outlives_window_and_preserves_independent_observations() {
    for (reply, status, failure) in [
        (Reply::DrainGood, 5, ApplicationFailureKind::None),
        (
            Reply::DrainRejected,
            4,
            ApplicationFailureKind::ProviderFinalization,
        ),
    ] {
        let (peer, waiting, release) = gated(Pause::Drain, reply);
        let mut session = spawn(&peer);
        receive(&mut session).frame.unwrap();
        session.event(0, 0).unwrap();
        assert!(receive(&mut session).frame.unwrap().is_some());
        let mut session = Box::into_raw(Box::new(session));
        let mut ticket = ptr::null_mut();
        let sentinel = NuisApplicationCancellationReceipt {
            cleanup_completed: -7,
            provider_status: -7,
            failure_kind: -7,
            provider_failure_kind: -7,
            completed_dispatches: -7,
        };
        let mut receipt = sentinel;
        unsafe {
            assert_eq!(
                nuis_window_session_cancel_with_provider_drain(ptr::null_mut(), &mut ticket),
                -1
            );
            assert_eq!(
                nuis_window_session_cancel_with_provider_drain(session, ptr::null_mut()),
                -1
            );
            let occupied = ptr::NonNull::<ApplicationCancellation>::dangling().as_ptr();
            let mut output = occupied;
            assert_eq!(
                nuis_window_session_cancel_with_provider_drain(session, &mut output),
                -1
            );
            assert_eq!(output, occupied);
            assert_eq!((*session).phase(), ApplicationPumpPhase::Open);
            assert_eq!(
                nuis_window_session_cancel_with_provider_drain(session, &mut ticket),
                0
            );
            assert_cancelled_without_outcome(&mut *session);
            waiting.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(
                nuis_application_cancellation_poll_with_provider(ticket, &mut receipt),
                0
            );
            assert_eq!(receipt, sentinel);
            nuis_window_session_free(&mut session);
            assert!(session.is_null());
            assert_eq!(
                nuis_application_cancellation_poll_with_provider(ticket, ptr::null_mut()),
                -1
            );
            release.send(()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                let poll = nuis_application_cancellation_poll_with_provider(ticket, &mut receipt);
                if poll != 0 {
                    assert_eq!(poll, 1);
                    break;
                }
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(receipt.cleanup_completed, 0);
            assert_eq!(receipt.provider_status, status);
            assert_eq!(receipt.failure_kind, failure.code());
            assert_eq!(receipt.provider_failure_kind, failure.code());
            assert_eq!(
                receipt.completed_dispatches,
                if status == 5 { 1 } else { -1 }
            );
            let mut cleanup = -7;
            let mut fault = -7;
            assert_eq!(
                nuis_application_cancellation_poll(ticket, &mut cleanup, &mut fault),
                0
            );
            assert_eq!((cleanup, fault), (-7, -7));
            nuis_application_cancellation_free(&mut ticket);
            assert!(ticket.is_null());
        }
        assert_eq!(peer.finish(), (1, false));
    }
}
