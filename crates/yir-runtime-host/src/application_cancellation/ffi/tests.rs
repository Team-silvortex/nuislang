use super::*;
use crate::ApplicationHostRetirementAck;
use std::sync::mpsc;
use yir_core::ApplicationFailureKind;

#[test]
fn ffi_receipt_is_once_only_and_invalid_outputs_do_not_consume_it() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let mut slot = Box::into_raw(Box::new(ApplicationCancellation::new(receiver)));
    let (mut cleanup, mut failure) = (-7, -8);
    unsafe {
        assert_eq!(
            nuis_application_cancellation_poll(slot, &mut cleanup, &mut failure),
            0
        );
        assert_eq!((cleanup, failure), (-7, -8));
        sender
            .send(ApplicationHostRetirementAck::new(
                true,
                ApplicationFailureKind::Host,
            ))
            .unwrap();
        assert_eq!(
            nuis_application_cancellation_poll(slot, ptr::null_mut(), &mut failure),
            -1
        );
        assert_eq!(
            nuis_application_cancellation_poll(slot, &mut cleanup, ptr::null_mut()),
            -1
        );
        assert_eq!((cleanup, failure), (-7, -8));
        assert_eq!(
            nuis_application_cancellation_poll(slot, &mut cleanup, &mut failure),
            1
        );
        assert_eq!((cleanup, failure), (1, ApplicationFailureKind::Host.code()));
        cleanup = -7;
        failure = -8;
        for _ in 0..2 {
            assert_eq!(
                nuis_application_cancellation_poll(slot, &mut cleanup, &mut failure),
                0
            );
            assert_eq!((cleanup, failure), (-7, -8));
        }
        nuis_application_cancellation_free(&mut slot);
        assert!(slot.is_null());
        nuis_application_cancellation_free(&mut slot);
    }
}

#[test]
fn ffi_disconnect_is_not_a_retirement_receipt() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let mut slot = Box::into_raw(Box::new(ApplicationCancellation::new(receiver)));
    let (mut cleanup, mut failure) = (-7, -8);
    drop(sender);
    unsafe {
        assert_eq!(
            nuis_application_cancellation_poll(slot, &mut cleanup, &mut failure),
            -1
        );
        assert_eq!((cleanup, failure), (-7, -8));
        assert_eq!(
            nuis_application_cancellation_poll(slot, &mut cleanup, &mut failure),
            0
        );
        assert_eq!((cleanup, failure), (-7, -8));
        nuis_application_cancellation_free(&mut slot);
    }
}

#[test]
fn ffi_ticket_drop_abandons_only_observation_and_null_handles_reject() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let mut slot = Box::into_raw(Box::new(ApplicationCancellation::new(receiver)));
    let (mut cleanup, mut failure) = (-7, -8);
    unsafe {
        nuis_application_cancellation_free(&mut slot);
        nuis_application_cancellation_free(ptr::null_mut());
        assert!(slot.is_null());
        assert_eq!(
            nuis_application_cancellation_poll(slot, &mut cleanup, &mut failure),
            -1
        );
        assert_eq!((cleanup, failure), (-7, -8));
    }
    assert!(sender
        .send(ApplicationHostRetirementAck::new(
            false,
            ApplicationFailureKind::None
        ))
        .is_err());
}

#[test]
fn cancellation_abi_has_no_window_or_provider_dependency() {
    let source = include_str!("../ffi.rs");
    for forbidden in [
        "WindowSession",
        "ApplicationEventPump",
        "CancellationControl",
        "ScopeAdmission",
        "ApplicationProviderSource",
        "ModRegistry",
        "UnixStream",
    ] {
        assert!(
            !source.contains(forbidden),
            "ticket ABI depends on {forbidden}"
        );
    }
}
