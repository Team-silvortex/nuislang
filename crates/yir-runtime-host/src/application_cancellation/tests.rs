use super::*;
use std::sync::{mpsc, Arc, Barrier};

#[test]
fn cancellation_and_finalization_have_exactly_one_admission_winner() {
    for _ in 0..64 {
        let control = Arc::new(CancellationControl::default());
        let barrier = Arc::new(Barrier::new(2));
        let other = Arc::clone(&control);
        let start = Arc::clone(&barrier);
        let worker = std::thread::spawn(move || {
            start.wait();
            other.admit_finalization().is_ok()
        });
        barrier.wait();
        let cancelled = control.request().is_ok();
        let finishing = worker.join().unwrap();
        assert_ne!(cancelled, finishing);
        assert_eq!(control.cancelled(), cancelled);
        assert!(control.request().is_err());
        assert!(control.admit_finalization().is_err());
        assert_eq!(control.retire(), cancelled);
        assert!(!control.retire());
        assert!(control.request().is_err());
    }
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
