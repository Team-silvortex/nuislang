use super::*;
use std::sync::{mpsc, Arc, Barrier};

#[test]
fn cancellation_and_finalization_have_exactly_one_admission_winner() {
    for iteration in 0..64 {
        let mode = if iteration % 2 == 0 {
            CancellationMode::HostOnly
        } else {
            CancellationMode::ProviderDrain
        };
        let control = Arc::new(CancellationControl::default());
        let barrier = Arc::new(Barrier::new(2));
        let other = Arc::clone(&control);
        let start = Arc::clone(&barrier);
        let worker = std::thread::spawn(move || {
            start.wait();
            other.admit_finalization().is_ok()
        });
        barrier.wait();
        let cancelled = control.request(mode).is_ok();
        let finishing = worker.join().unwrap();
        assert_ne!(cancelled, finishing);
        assert_eq!(control.cancelled(), cancelled);
        assert!(control.request(mode).is_err());
        assert!(control.admit_finalization().is_err());
        assert_eq!(control.retire(), cancelled.then_some(mode));
        assert!(control.retire().is_none());
        assert!(control.request(mode).is_err());
    }
}

#[test]
fn abandonment_seals_admission_before_the_provider_can_be_dropped() {
    for _ in 0..64 {
        let control = Arc::new(CancellationControl::default());
        let start = Arc::new(Barrier::new(2));
        let worker_control = Arc::clone(&control);
        let worker_start = Arc::clone(&start);
        let worker = std::thread::spawn(move || {
            worker_start.wait();
            worker_control.admit_abandonment() == ScopeAbandonment::Drain
        });
        start.wait();
        let cancelled = control.request(CancellationMode::ProviderDrain).is_ok();
        assert_eq!(worker.join().unwrap(), cancelled);
        assert!(control.request(CancellationMode::HostOnly).is_err());
        assert!(control.admit_finalization().is_err());
        assert_eq!(
            control.retire(),
            cancelled.then_some(CancellationMode::ProviderDrain)
        );
    }
}

#[test]
fn early_scope_exit_reports_no_observation_instead_of_inventing_drain() {
    let ack = ApplicationHostRetirementAck::new(false, ApplicationFailureKind::Host);
    assert_eq!(
        ack.for_cancellation(CancellationMode::HostOnly)
            .provider_drain(),
        ProviderDrainObservation::NotRequested
    );
    assert_eq!(
        ack.for_cancellation(CancellationMode::ProviderDrain)
            .provider_drain(),
        ProviderDrainObservation::NotObserved
    );
}

#[test]
fn timeout_is_not_retirement_and_receipt_is_delivered_once() {
    let (send, receive) = mpsc::sync_channel(1);
    let mut cancellation = ApplicationCancellation::new(receive);
    assert!(cancellation.poll().unwrap().is_none());
    assert!(cancellation.wait(Duration::ZERO).unwrap().is_none());
    let ack = ApplicationHostRetirementAck::new(false, ApplicationFailureKind::ProviderBudget);
    send.send(ack).unwrap();
    assert_eq!(cancellation.wait(Duration::ZERO).unwrap(), Some(ack));
    assert!(cancellation.poll().unwrap().is_none());
    assert!(cancellation.wait(Duration::ZERO).unwrap().is_none());
}

#[test]
fn missing_worker_acknowledgement_is_an_error_not_retirement() {
    for waiting in [false, true] {
        let (send, receive) = mpsc::sync_channel(1);
        let mut cancellation = ApplicationCancellation::new(receive);
        drop(send);
        let result = if waiting {
            cancellation.wait(Duration::ZERO)
        } else {
            cancellation.poll()
        };
        assert!(result
            .unwrap_err()
            .contains("without a host retirement acknowledgement"));
        assert!(cancellation.poll().unwrap().is_none());
    }
}
