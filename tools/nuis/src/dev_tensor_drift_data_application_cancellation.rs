use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "application-cancellation-host-retirement-contract",
        path: "crates/yir-runtime-host/src/application_cancellation.rs",
        required_patterns: &[
            "nuis-yir-application-cancellation-v1",
            "compare_exchange(ACTIVE, CANCELLED",
            "compare_exchange(ACTIVE, FINALIZING",
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
            "provider_scope_has_no_direct_dependency_on_concrete_pump_or_cancellation",
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
            "appkit_cancellation_wired = false",
            "cancelled_window_outcome = \"none-even-after-rejected-window-calls-no-parent-delivery\"",
            "parent_cancellation_wired = false",
        ],
    },
];
