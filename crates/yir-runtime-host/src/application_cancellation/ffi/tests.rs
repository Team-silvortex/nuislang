use super::*;
use crate::{ApplicationHostRetirementAck, ProviderDrainObservation};
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

#[test]
fn provider_poll_validates_outputs_and_delivers_each_observation_once() {
    for (observation, code, dispatches, failure) in [
        (
            ProviderDrainObservation::NotRequested,
            0,
            -1,
            ApplicationFailureKind::None,
        ),
        (
            ProviderDrainObservation::NotObserved,
            1,
            -1,
            ApplicationFailureKind::None,
        ),
        (
            ProviderDrainObservation::ReplayOnly,
            2,
            -1,
            ApplicationFailureKind::None,
        ),
        (
            ProviderDrainObservation::Unavailable,
            3,
            -1,
            ApplicationFailureKind::None,
        ),
        (
            ProviderDrainObservation::Failed(ApplicationFailureKind::ProviderExchange),
            4,
            -1,
            ApplicationFailureKind::ProviderExchange,
        ),
        (
            ProviderDrainObservation::Drained {
                completed_dispatches: 0,
            },
            5,
            0,
            ApplicationFailureKind::None,
        ),
        (
            ProviderDrainObservation::Drained {
                completed_dispatches: 256,
            },
            5,
            256,
            ApplicationFailureKind::None,
        ),
    ] {
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut ticket = Box::into_raw(Box::new(ApplicationCancellation::new(receiver)));
        let sentinel = NuisApplicationCancellationReceipt {
            cleanup_completed: -7,
            provider_status: -7,
            failure_kind: -7,
            provider_failure_kind: -7,
            completed_dispatches: -7,
        };
        let mut output = sentinel;
        unsafe {
            assert_eq!(
                nuis_application_cancellation_poll_with_provider(ticket, &mut output),
                0
            );
            assert_eq!(output, sentinel);
            sender
                .send(
                    ApplicationHostRetirementAck::new(false, ApplicationFailureKind::Host)
                        .with_provider_drain(observation),
                )
                .unwrap();
            assert_eq!(
                nuis_application_cancellation_poll_with_provider(ticket, ptr::null_mut()),
                -1
            );
            assert_eq!(
                nuis_application_cancellation_poll_with_provider(ptr::null_mut(), &mut output),
                -1
            );
            assert_eq!(output, sentinel);
            assert_eq!(
                nuis_application_cancellation_poll_with_provider(ticket, &mut output),
                1
            );
            assert_eq!(
                output,
                NuisApplicationCancellationReceipt {
                    cleanup_completed: 0,
                    provider_status: code,
                    failure_kind: ApplicationFailureKind::Host.code(),
                    provider_failure_kind: failure.code(),
                    completed_dispatches: dispatches,
                }
            );
            output = sentinel;
            assert_eq!(
                nuis_application_cancellation_poll_with_provider(ticket, &mut output),
                0
            );
            assert_eq!(output, sentinel);
            nuis_application_cancellation_free(&mut ticket);
        }
    }
}
