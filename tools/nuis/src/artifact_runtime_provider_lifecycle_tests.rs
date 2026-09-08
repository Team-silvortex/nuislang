use super::*;
use yir_core::provider_runtime_ipc::{hash_bytes, DispatchTarget, MAX_DISPATCHES};

fn drain(sequence: usize) -> SessionDrain {
    SessionDrain {
        sequence,
        target: DispatchTarget {
            source_yir_fnv1a64: hash_bytes(b"registered application"),
            module: "registered-family".to_owned(),
            instruction: "work".to_owned(),
            node: "application.node".to_owned(),
            resource: "registered-resource".to_owned(),
        },
    }
}

#[test]
fn completion_accounts_for_sessions_without_changing_child_exit_semantics() {
    let mut observer = ProviderLaunchLifecycle::new(ProviderLaunchPolicy::CompletionOnly);
    for count in [0, 2, 3] {
        assert!(!observer
            .observe(ProviderRuntimeSessionOutcome::Finished(count))
            .unwrap());
    }
    let outcome = observer.finish().unwrap();
    assert_eq!(
        outcome,
        ProviderLaunchOutcome::Finished {
            sessions: 3,
            invocations: 5
        }
    );
    assert_eq!(outcome.clone().into_finished_count().unwrap(), 5);
    for code in [Some(0), Some(1), Some(130), None] {
        // The ordinary launcher still checks success; lifecycle admission does not.
        ProviderLaunchPolicy::CompletionOnly
            .admit_exit(&outcome, code)
            .unwrap();
    }
}

#[test]
fn explicit_drain_needs_both_a_provider_terminal_and_the_cancelled_exit() {
    for count in [0, 2, MAX_DISPATCHES] {
        let receipt = drain(count);
        let mut observer = ProviderLaunchLifecycle::new(ProviderLaunchPolicy::ExplicitDrain);
        assert!(observer
            .observe(ProviderRuntimeSessionOutcome::Drained(receipt.clone()))
            .unwrap());
        let outcome = observer.finish().unwrap();
        assert_eq!(outcome, ProviderLaunchOutcome::Drained(receipt));
        assert!(outcome.clone().into_finished_count().is_err());
        ProviderLaunchPolicy::ExplicitDrain
            .admit_exit(
                &outcome,
                Some(yir_runtime_host::APPLICATION_CANCELLED_EXIT_CODE),
            )
            .unwrap();
        for code in [Some(0), Some(1), Some(2), Some(-1), None] {
            assert!(ProviderLaunchPolicy::ExplicitDrain
                .admit_exit(&outcome, code)
                .is_err());
        }
        assert!(ProviderLaunchPolicy::CompletionOnly
            .admit_exit(&outcome, Some(130))
            .is_err());
    }
    for policy in [
        ProviderLaunchPolicy::CompletionOnly,
        ProviderLaunchPolicy::ExplicitDrain,
    ] {
        assert!(ProviderLaunchLifecycle::new(policy).finish().is_err());
    }
    assert!(ProviderLaunchPolicy::ExplicitDrain
        .admit_exit(
            &ProviderLaunchOutcome::Finished {
                sessions: 1,
                invocations: 0
            },
            Some(130)
        )
        .is_err());
}

#[test]
fn drain_policy_rejects_finish_before_the_provider_can_publish() {
    for count in [0, MAX_DISPATCHES] {
        let finish = Message::Finish(count);
        assert!(ProviderLaunchPolicy::ExplicitDrain
            .admit_request(&finish)
            .unwrap_err()
            .contains("before publication"));
        ProviderLaunchPolicy::CompletionOnly
            .admit_request(&finish)
            .unwrap();
        ProviderLaunchPolicy::ExplicitDrain
            .admit_request(&Message::Drain(drain(count)))
            .unwrap();
    }
}

#[test]
fn policy_mismatch_and_late_sessions_permanently_preserve_the_first_error() {
    for (policy, invalid) in [
        (
            ProviderLaunchPolicy::CompletionOnly,
            ProviderRuntimeSessionOutcome::Drained(drain(0)),
        ),
        (
            ProviderLaunchPolicy::ExplicitDrain,
            ProviderRuntimeSessionOutcome::Finished(1),
        ),
    ] {
        let mut observer = ProviderLaunchLifecycle::new(policy);
        let error = observer.observe(invalid).unwrap_err();
        assert_eq!(
            observer
                .observe(ProviderRuntimeSessionOutcome::Finished(0))
                .unwrap_err(),
            error
        );
        assert_eq!(
            observer
                .observe(ProviderRuntimeSessionOutcome::Drained(drain(0)))
                .unwrap_err(),
            error
        );
        assert_eq!(observer.finish().unwrap_err(), error);
    }
    for late in [
        ProviderRuntimeSessionOutcome::Finished(0),
        ProviderRuntimeSessionOutcome::Drained(drain(1)),
    ] {
        let mut observer = ProviderLaunchLifecycle::new(ProviderLaunchPolicy::ExplicitDrain);
        observer
            .observe(ProviderRuntimeSessionOutcome::Drained(drain(1)))
            .unwrap();
        let error = observer.observe(late).unwrap_err();
        assert!(error.contains("terminal"));
        assert_eq!(observer.finish().unwrap_err(), error);
    }
}

#[test]
fn malformed_receipts_and_counter_overflows_cannot_be_recovered_as_success() {
    let mut bad_hash = drain(1);
    bad_hash.target.source_yir_fnv1a64 = "invalid".to_owned();
    let mut bad_node = drain(1);
    bad_node.target.node = "injected\nnode".to_owned();
    for invalid in [drain(MAX_DISPATCHES + 1), bad_hash, bad_node] {
        let mut observer = ProviderLaunchLifecycle::new(ProviderLaunchPolicy::ExplicitDrain);
        let error = observer
            .observe(ProviderRuntimeSessionOutcome::Drained(invalid))
            .unwrap_err();
        assert_eq!(
            observer
                .observe(ProviderRuntimeSessionOutcome::Drained(drain(0)))
                .unwrap_err(),
            error
        );
        assert_eq!(observer.finish().unwrap_err(), error);
    }
    for (sessions, invocations) in [(usize::MAX, 0), (1, usize::MAX)] {
        let mut observer = ProviderLaunchLifecycle::new(ProviderLaunchPolicy::CompletionOnly);
        observer.outcome = Some(ProviderLaunchOutcome::Finished {
            sessions,
            invocations,
        });
        let error = observer
            .observe(ProviderRuntimeSessionOutcome::Finished(1))
            .unwrap_err();
        assert!(error.contains("overflow"));
        assert_eq!(observer.finish().unwrap_err(), error);
    }
}

#[test]
fn policy_and_terminal_contract_do_not_depend_on_platform_or_provider_implementations() {
    for source in [
        include_str!("artifact_runtime_provider_lifecycle.rs"),
        include_str!("../../../crates/yir-core/src/provider_runtime_outcome.rs"),
    ] {
        for forbidden in [
            "cfg(unix)",
            "target_os",
            "std::os",
            "UnixStream",
            "AppKit",
            "Metal",
            "Cuda",
            "nsdb::",
            "ModRegistry",
            "yir_domain_",
        ] {
            assert!(
                !source.contains(forbidden),
                "shared policy depends on {forbidden}"
            );
        }
    }
}
