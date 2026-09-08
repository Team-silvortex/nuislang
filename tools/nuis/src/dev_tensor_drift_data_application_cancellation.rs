use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "provider-session-drain-wire-boundary",
        path: "crates/yir-core/src/provider_runtime_drain.rs",
        required_patterns: &[
            "nuis-yir-provider-session-drain-v1",
            "pub struct SessionDrain",
            "pub fn admit",
            "self.target != *target || self.sequence != sequence",
            "self.sequence > MAX_DISPATCHES",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-session-drain-lifecycle-boundary",
        path: "tools/nsdb/src/provider_runtime_ipc.rs",
        required_patterns: &[
            "complete_session(execution, close())?",
            "Message::Drained(drain.clone())",
            "drain.admit(target, count)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-session-terminal-portable-contract",
        path: "crates/yir-core/src/provider_runtime_outcome.rs",
        required_patterns: &[
            "pub enum ProviderRuntimeSessionOutcome", "pub fn into_finished_count",
            "Finished(usize)", "Drained(SessionDrain)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-session-drain-failure-evidence",
        path: "tools/nsdb/src/provider_runtime_ipc_drain_tests.rs",
        required_patterns: &[
            "drain_is_terminal_at_zero_midstream_and_exact_dispatch_limit",
            "drain_acknowledgement_waits_for_successful_provider_close",
            "drain_cleanup_or_ack_write_failure_cannot_claim_retirement",
            "disconnect_and_first_failure_are_not_reclassified_as_drain",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-session-drain-metal-evidence",
        path: "tools/nuis/src/artifact_device_sample_shader_drain_tests.rs",
        required_patterns: &[
            "drains_registered_metal_session_without_replacing_success_evidence",
            "--export-frame",
            "Message::Drain(drain.clone())",
            "Message::Drained(drain.clone())",
            "metal.command-buffer.completed",
            ".nuis-provider-worker-image",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-session-drain-host-finish-separation",
        path: "crates/yir-runtime-host/src/provider_runtime_ipc_tests.rs",
        required_patterns: &[
            "finish_cannot_consume_a_provider_drain_receipt_as_completion",
            "Message::Drained(drain)",
            "ApplicationFailureKind::ProviderContract",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-session-drain-authority-boundary",
        path: "docs/reference/nuis-ns-nova-application-lifecycle-v1.toml",
        required_patterns: &[
            "[provider_session_drain]",
            "host_cancellation_wired = true",
            "host_drain_policy = \"explicit-opt-in-validated-frontier-only-no-retry\"",
            "packaged_host_drain_wired = true",
            "windows_transport_claimed = false",
            "non_appkit_packaged_drain_claimed = false",
            "cross_session_resource_reuse = false",
            "outcome = \"typed-drained-not-finished\"",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-cancellation-host-retirement-contract",
        path: "crates/yir-runtime-host/src/application_cancellation.rs",
        required_patterns: &[
            "nuis-yir-application-cancellation-v1",
            "compare_exchange(ACTIVE, next",
            "compare_exchange(ACTIVE, FINALIZING",
            "CANCELLED_DRAIN",
            "ABANDONING",
            "pub struct ApplicationHostRetirementAck",
            "pub struct ApplicationCancellation",
            "without a host retirement acknowledgement",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-scope-admission-minimal-contract",
        path: "crates/yir-runtime-host/src/application_scope_admission.rs",
        required_patterns: &[
            "trait ScopeAdmission",
            "fn checkpoint",
            "fn admit_finalization",
            "fn admit_abandonment",
            "impl ScopeAdmission for ()",
            "does not bypass application or provider completion checks",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-scope-independent-policy-evidence",
        path: "crates/yir-runtime-host/src/provider_application_session/admission_tests.rs",
        required_patterns: &[
            "impl ScopeAdmission for RecordingAdmission",
            "provider_scope_accepts_independent_admission_without_bypassing_lifecycle",
            "independent_policy_rejects_before_provider_io",
            "independent_policy_rejects_after_provider_admission_without_application_delivery",
            "independent_abandonment_policy_can_observe_drain_without_turning_error_into_success",
            "provider_scope_has_no_direct_dependency_on_concrete_pump_or_cancellation",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-host-frontier-authority",
        path: "crates/yir-runtime-host/src/provider_runtime_ipc.rs",
        required_patterns: &[
            "fn begin_exchange", "if !self.frontier", "self.begin_exchange()?",
            "receipt.admit(&self.target, self.sequence)?", "RejectionPhase::Drain",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-host-independent-scope",
        path: "crates/yir-runtime-host/src/provider_application_session.rs",
        required_patterns: &[
            "A: ScopeAdmission", "drop(registry)",
            "control.admission.admit_abandonment() == ScopeAbandonment::Drain",
            "drain_provider_source(&provider, &control.failures)", "slot.set(observation)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-host-race-and-fault-evidence",
        path: "crates/yir-runtime-host/tests/provider_application_session/provider_drain.rs",
        required_patterns: &[
            "cancellation_observes_zero_and_two_frame_drain_without_application_success",
            "in_flight_hello_frame_and_explicit_close_drain_only_after_their_boundary",
            "drain_failures_are_separate_observations_and_do_not_replace_first_faults",
            "damaged_frame_frontiers_drop_without_attempting_drain_or_clearing_faults",
            "finish_winner_rejects_drain_cancel_without_consuming_its_close_reply",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-window-independent-ticket-evidence",
        path: "crates/yir-runtime-host/tests/provider_application_session/window/provider_drain.rs",
        required_patterns: &[
            "window_drain_ffi_outlives_window_and_preserves_independent_observations",
            "nuis_window_session_cancel_with_provider_drain",
            "nuis_application_cancellation_poll_with_provider",
            "assert_cancelled_without_outcome", "nuis_window_session_free",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-independent-ffi-observations",
        path: "crates/yir-runtime-host/src/application_cancellation/ffi.rs",
        required_patterns: &[
            "NuisApplicationCancellationReceipt", "nuis_application_cancellation_poll_with_provider",
            "provider.failure_kind().code()", "ack.failure_kind().code()",
            "if output.is_null()", "provider.completed_dispatches()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-host-metal-and-replay-evidence",
        path: "tools/nuis/src/artifact_device_sample_shader_host_drain_tests.rs",
        required_patterns: &[
            "verify_host_cancellation", "WindowSession::spawn", "cancel_with_provider_drain",
            "ProviderDrainObservation::Drained", "ProviderDrainObservation::ReplayOnly",
            "outcome.into_finished_count().is_err()", ".nuis-provider-worker-image",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-cancellation-race-and-fault-evidence",
        path: "crates/yir-runtime-host/tests/provider_application_session/cancellation.rs",
        required_patterns: &[
            "cancellation_waits_for_in_flight_open_or_event_without_fabricating_cleanup",
            "accepted_cancel_allows_explicit_close_to_return_but_prevents_finish",
            "finish_winning_admission_rejects_cancel_without_losing_the_original_reply",
            "cancellation_keeps_typed_faults_before_and_after_its_admission",
            "unconsumed_callback_reply_does_not_block_retirement_or_allow_another_event",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "window-cancellation-forwarding-contract",
        path: "crates/yir-runtime-host/src/window_session.rs",
        required_patterns: &[
            "pub fn cancel",
            "let ticket = self.pump.cancel()?",
            "self.cancellation_admitted = true",
            "&& !self.cancellation_admitted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-cancellation-ffi-contract",
        path: "crates/yir-runtime-host/src/application_cancellation/ffi.rs",
        required_patterns: &[
            "nuis_application_cancellation_poll",
            "nuis_application_cancellation_free",
            "cleanup.is_null() || failure.is_null()",
            "Ok(None) => 0",
            "ack.cleanup_completed()",
            "ack.failure_kind().code()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "window-cancellation-ownership-evidence",
        path: "crates/yir-runtime-host/tests/provider_application_session/window/cancellation.rs",
        required_patterns: &[
            "cancelled_window_open_and_event_never_manufacture_parent_outcomes",
            "window_cancellation_preserves_observed_and_in_flight_provider_failures",
            "window_cancel_rejection_keeps_explicit_close_and_finish_reply",
            "window_ffi_cancellation_ticket_outlives_window_without_losing_acknowledgement",
            "window_ticket_and_session_drop_do_not_join_an_in_flight_callback",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-cancellation-ffi-evidence",
        path: "crates/yir-runtime-host/src/application_cancellation/ffi/tests.rs",
        required_patterns: &[
            "ffi_receipt_is_once_only_and_invalid_outputs_do_not_consume_it",
            "ffi_disconnect_is_not_a_retirement_receipt",
            "ffi_ticket_drop_abandons_only_observation_and_null_handles_reject",
            "cancellation_abi_has_no_window_or_provider_dependency",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-cancellation-compiled-nuis-evidence",
        path: "tools/nuisc/tests/ns_nova_application_cancellation.rs",
        required_patterns: &[
            "compiled_nuis_window_state_can_retire_without_implicit_close_or_provider_finish",
            "compiled_nuis_window_ffi_cancel_retains_ticket_after_window_free",
            "ns_nova_image_showcase",
            "validate_window_session",
            "Message::Finish(0)",
            "pump.cancel()",
            "ack.cleanup_completed()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-cancellation-authority-boundary",
        path: "docs/reference/nuis-ns-nova-application-lifecycle-v1.toml",
        required_patterns: &[
            "[application_cancellation]",
            "host_retirement_acknowledgement = true",
            "provider_finish_after_accepted_cancel = false",
            "provider_device_retirement_claimed = false",
            "provider_wire_cancellation = false",
            "window_cancellation_wired = true",
            "appkit_cancellation_wired = true",
            "appkit_cancellation_scope = \"explicit-standalone-script-after-final-event-reply-not-in-flight-device-interruption\"",
            "appkit_cancellation_provider_boundary = \"eof-without-finish-remains-error-old-replay-preserved-no-drain-claim\"",
            "cancelled_window_outcome = \"none-even-after-rejected-window-calls-no-parent-delivery\"",
            "parent_cancellation_wired = false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "packaged-window-cancellation-ticket-only-adapter",
        path: "tools/yir-pack-aot/src/host_window_cancellation.rs",
        required_patterns: &[
            "nuis_window_session_cancel(self.session, &ticket)",
            "nuis_application_cancellation_poll(ticket, &cleanup, &failure)",
            "gNuisWindowExitStatus = 130",
            "window_session_host_retired",
            "window_session_cancel_receipt_missing",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "packaged-window-cancellation-receipt-policy-evidence",
        path: "tools/yir-pack-aot/src/host_window_cancellation_tests.rs",
        required_patterns: &[
            "generated_host_forwards_independent_ticket_and_preserves_pending_or_missing_receipts",
            "crate::host_window_session::SUPPORT",
            "crate::host_window_session::FIELDS",
            "super::METHODS",
            "pollStatus = -1",
            "cancelStatus = -1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "packaged-window-live-cancellation-evidence",
        path: "tools/nuis/src/artifact_device_sample_shader_cancellation_tests.rs",
        required_patterns: &[
            "verify_compiled_window_cancellation",
            "Message::Finish(_)",
            "--window-cancel-after-events",
            "runtime IPC idle read failed",
            "Some(130)",
            "frontdoor cancellation replaced old replay",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-portable-launch-policy",
        path: "tools/nuis/src/artifact_runtime_provider_lifecycle.rs",
        required_patterns: &[
            "enum ProviderLaunchPolicy", "enum ProviderLaunchOutcome",
            "fn admit_request", "fn admit_exit", "APPLICATION_CANCELLED_EXIT_CODE",
            "rejects Finish before publication", "runtime provider drain is terminal",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-portable-launch-policy-evidence",
        path: "tools/nuis/src/artifact_runtime_provider_lifecycle_tests.rs",
        required_patterns: &[
            "explicit_drain_needs_both_a_provider_terminal_and_the_cancelled_exit",
            "drain_policy_rejects_finish_before_the_provider_can_publish",
            "policy_mismatch_and_late_sessions_permanently_preserve_the_first_error",
            "malformed_receipts_and_counter_overflows_cannot_be_recovered_as_success",
            "policy_and_terminal_contract_do_not_depend_on_platform_or_provider_implementations",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "provider-drain-portable-exit-classifier-evidence",
        path: "crates/yir-runtime-host/src/application_cancellation/ffi/tests.rs",
        required_patterns: &[
            "drain_exit_classification_is_portable_and_never_claims_application_success",
            "drain_exit_preserves_faults_and_rejects_missing_or_contradictory_observations",
            "nuis_application_provider_drain_exit_status",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "packaged-provider-drain-capability-admission",
        path: "tools/nuis/src/artifact_runtime_window_session.rs",
        required_patterns: &[
            "application_provider_drain_contract=", "SESSION_DRAIN_CONTRACT",
            "declarations != [expected.as_str()]", "options.drain_provider",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "packaged-provider-drain-typed-frontdoor",
        path: "tools/nuis/src/artifact_runtime_launch.rs",
        required_patterns: &[
            "enum ArtifactRunOutcome", "ProviderLaunchPolicy::ExplicitDrain",
            "ProviderLaunchOutcome::Drained(receipt)", "ArtifactRunOutcome::Cancelled(receipt)",
            "Do not persist successful launch/trace evidence",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "packaged-provider-drain-thin-host-adapter",
        path: "tools/yir-pack-aot/src/host_window_cancellation.rs",
        required_patterns: &[
            "nuis_window_session_cancel_with_provider_drain(self.session, &ticket)",
            "nuis_application_cancellation_poll_with_provider(ticket, &receipt)",
            "nuis_application_provider_drain_exit_status(&receipt)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "packaged-provider-drain-metal-evidence",
        path: "tools/nuis/src/artifact_device_sample_shader_packaged_drain_tests.rs",
        required_patterns: &[
            "verify_packaged_drain", "--drain-provider", "ProviderLaunchPolicy::ExplicitDrain",
            "ArtifactRunOutcome::Cancelled(receipt)", "before publication",
            "frontdoor drain published success evidence", "window_session_provider_drain_status=2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "headless-application-scalar-script-contract",
        path: "docs/reference/nuis-yir-application-scalar-script-v1.md",
        required_patterns: &[
            "nuis-yir-application-scalar-script-v1",
            "64 ordered event deliveries",
            "180-second", "neither exit130 nor EOF proves retirement",
            "not fully native CPU", "do not yet select or admit this profile",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "headless-application-common-pump-boundary",
        path: "crates/yir-runtime-host/src/application_script.rs",
        required_patterns: &[
            "ApplicationEventPump::spawn", "MAX_SCRIPT_EVENTS: usize = 64",
            "MAX_SCRIPT_ARGUMENTS: usize = 16", "remaining(deadline)",
            "pump.cancel_with_provider_drain()", "drop(pump)",
            "ApplicationScriptOutcome::Finished", "ApplicationScriptOutcome::Cancelled",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "headless-application-packaged-protocol-evidence",
        path: "tools/yir-pack-aot/tests/headless_session.rs",
        required_patterns: &[
            "packaged_headless_session_finishes_or_drains_without_a_window_backend",
            "Message::Drained", "Message::Finish(2)", "receipt.admit(&target, 2)",
            "cpu_host_binary_mode=embedded_yir_headless", "rgba8_fnv1a64",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "headless-application-live-metal-evidence",
        path: "tools/nuis/src/artifact_device_sample_shader_headless_tests.rs",
        required_patterns: &[
            "headless_metal_session_uses_shared_lifecycle_without_appkit",
            "ProviderLaunchOutcome::Drained(receipt)", "ProviderLaunchPolicy::ExplicitDrain",
            "verify_frames", "drain replaced", "Some(130)", "provider_status=2;",
        ],
    },
];
