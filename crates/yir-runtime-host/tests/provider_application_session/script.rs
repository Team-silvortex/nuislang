use super::*;
use std::path::Path;
use yir_runtime_host::{
    run_application_script, ApplicationPumpOperation, ApplicationScript, ApplicationScriptOutcome,
    ApplicationScriptTermination, NuisApplicationCancellationReceipt, ProviderDrainObservation,
};

fn source() -> String {
    format!(
        "application-session counter {} open update close state\n{SOURCE}",
        yir_core::APPLICATION_SESSION_CONTRACT
    )
}

fn script(termination: ApplicationScriptTermination) -> ApplicationScript {
    ApplicationScript {
        id: "counter".to_owned(),
        open: vec![10],
        events: vec![vec![1], vec![2]],
        termination,
    }
}

#[test]
fn headless_script_checks_all_scalar_calls_before_effects_or_provider_setup() {
    let valid = script(ApplicationScriptTermination::Close(vec![1]));
    valid.validate_source(&source()).unwrap();
    for case in 0..4 {
        let mut invalid = valid.clone();
        match case {
            0 => invalid.id = "missing".to_owned(),
            1 => invalid.open.clear(),
            2 => invalid.events.push(vec![]),
            _ => invalid.termination = ApplicationScriptTermination::Close(vec![]),
        }
        assert!(invalid.validate_source(&source()).is_err());
        let error = run_application_script(
            source(),
            ApplicationProviderSource::Replay(Path::new("missing-script-test-replay")),
            invalid,
            Duration::from_secs(1),
            |_| panic!("invalid script observed effects"),
        )
        .unwrap_err();
        assert!(!error.contains("missing-script-test-replay"), "{error}");
    }
}

#[test]
fn headless_script_observes_state_and_live_frames_before_validated_finish() {
    let peer = Peer::start_source(Reply::Good, source(), None);
    let mut states = Vec::new();
    let mut frames = Vec::new();
    let outcome = run_application_script(
        source(),
        ApplicationProviderSource::Ipc(&peer.path),
        script(ApplicationScriptTermination::Close(vec![1])),
        Duration::from_secs(5),
        |reply| {
            states.push(reply.state.clone().unwrap());
            for frame in &reply.trace.as_ref().unwrap().presented_frames {
                frames.push(frame.rgba8.clone().unwrap());
            }
        },
    )
    .unwrap();
    let ApplicationScriptOutcome::Finished(outcome) = outcome else {
        panic!("expected completion")
    };
    assert_eq!(outcome.codes(), [1, 1, 0]);
    let counts = states
        .iter()
        .map(|state| {
            let Value::Struct(state) = state else {
                panic!("missing state")
            };
            state.fields[0].1.clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        counts,
        [
            Value::Int(10),
            Value::Int(11),
            Value::Int(13),
            Value::Int(13)
        ]
    );
    assert_eq!(frames, [vec![1, 2, 3, 255], vec![2, 2, 3, 255]]);
    assert_eq!(peer.finish(), (2, true));
}

#[test]
fn headless_script_drain_never_runs_close_or_substitutes_finish() {
    for (reply, success) in [
        (Reply::DrainGood, true),
        (Reply::DrainRejected, false),
        (Reply::DrainWrongTarget, false),
        (Reply::DrainDisconnected, false),
    ] {
        let peer = Peer::start_source(reply, source(), None);
        let mut observed = Vec::new();
        let outcome = run_application_script(
            source(),
            ApplicationProviderSource::Ipc(&peer.path),
            script(ApplicationScriptTermination::Cancel {
                drain_provider: true,
            }),
            Duration::from_secs(5),
            |reply| observed.push(reply.operation),
        )
        .unwrap();
        let ApplicationScriptOutcome::Cancelled(ack) = outcome else {
            panic!("expected cancellation")
        };
        assert!(!ack.cleanup_completed());
        assert_eq!(
            observed,
            [
                ApplicationPumpOperation::Open,
                ApplicationPumpOperation::Event,
                ApplicationPumpOperation::Event
            ]
        );
        assert_eq!(
            NuisApplicationCancellationReceipt::from(ack).provider_drain_exit_status(),
            if success { 130 } else { 1 }
        );
        assert_eq!(peer.finish(), (2, false));
    }
}

#[test]
fn headless_script_host_only_cancellation_has_no_provider_retirement_claim() {
    let peer = Peer::start_source(Reply::Good, source(), None);
    let outcome = run_application_script(
        source(),
        ApplicationProviderSource::Ipc(&peer.path),
        script(ApplicationScriptTermination::Cancel {
            drain_provider: false,
        }),
        Duration::from_secs(5),
        |_| {},
    )
    .unwrap();
    let ApplicationScriptOutcome::Cancelled(ack) = outcome else {
        panic!("expected cancellation")
    };
    assert_eq!(ack.provider_drain(), ProviderDrainObservation::NotRequested);
    assert!(!ack.cleanup_completed());
    assert_eq!(peer.finish(), (2, false));
}

#[test]
fn headless_script_callback_failure_aborts_without_retry_or_implicit_cleanup() {
    for termination in [
        ApplicationScriptTermination::Close(vec![1]),
        ApplicationScriptTermination::Cancel {
            drain_provider: true,
        },
    ] {
        let peer = Peer::start_source(Reply::RejectedFrame, source(), None);
        let mut observed = Vec::new();
        assert!(run_application_script(
            source(),
            ApplicationProviderSource::Ipc(&peer.path),
            script(termination),
            Duration::from_secs(5),
            |reply| observed.push(reply.operation),
        )
        .is_err());
        assert_eq!(
            observed,
            [
                ApplicationPumpOperation::Open,
                ApplicationPumpOperation::Event
            ]
        );
        assert_eq!(peer.finish(), (1, false));
    }
}

#[test]
fn headless_script_late_finish_failure_cannot_become_completion() {
    for reply in [Reply::BadClose, Reply::RejectedClose] {
        let peer = Peer::start_source(reply, source(), None);
        assert!(run_application_script(
            source(),
            ApplicationProviderSource::Ipc(&peer.path),
            script(ApplicationScriptTermination::Close(vec![1])),
            Duration::from_secs(5),
            |_| {},
        )
        .is_err());
        assert_eq!(peer.finish(), (2, true));
    }
}

#[test]
fn headless_script_deadline_is_not_reset_for_each_event_or_terminal_operation() {
    let peer = Peer::start_source(Reply::Good, source(), None);
    let error = run_application_script(
        source(),
        ApplicationProviderSource::Ipc(&peer.path),
        script(ApplicationScriptTermination::Close(vec![1])),
        Duration::from_millis(20),
        |_| thread::sleep(Duration::from_millis(25)),
    )
    .unwrap_err();
    assert!(error.contains("deadline"));
    assert_eq!(peer.finish(), (0, false));
}

#[test]
fn headless_script_rejects_oversized_input_and_duration_before_provider_admission() {
    let missing = PathBuf::from("provider-that-must-not-be-opened");
    let mut oversized = script(ApplicationScriptTermination::Close(vec![]));
    oversized.events = vec![vec![]; 65];
    assert!(run_application_script(
        source(),
        ApplicationProviderSource::Ipc(&missing),
        oversized,
        Duration::from_secs(1),
        |_| {}
    )
    .unwrap_err()
    .contains("event limit"));
    for timeout in [Duration::ZERO, Duration::from_secs(181)] {
        assert!(run_application_script(
            source(),
            ApplicationProviderSource::Ipc(&missing),
            script(ApplicationScriptTermination::Close(vec![])),
            timeout,
            |_| {}
        )
        .unwrap_err()
        .contains("duration"));
    }
}
