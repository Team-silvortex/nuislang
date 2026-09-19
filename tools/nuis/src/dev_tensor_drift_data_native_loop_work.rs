use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-loop-work-reservation-policy",
        path: "crates/yir-lower-llvm/src/native_session/loop_work.rs",
        required_patterns: &[
            "DEFAULT_LOOP_WORK_LIMIT: u64 = 1_048_576",
            "icmp uge i64 {remaining}, {trips}",
            "sub i64 {remaining}, {trips}",
            "call void @llvm.trap()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-callback-owner",
        path: "crates/yir-lower-llvm/src/native_session/emit.rs",
        required_patterns: &[
            "%nuis_loop_work = alloca i64, align 8",
            "store i64 {loop_work_limit}, ptr %nuis_loop_work, align 8",
            "parameters.push(super::COUNTER_PARAMETER.to_owned())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-selected-induction",
        path: "crates/yir-lower-llvm/src/native_session/loops/dynamic.rs",
        required_patterns: &[
            "loop_work::reserve(&trips.to_string(), body, next_reg, next_block)",
            "loop_work::reserve(&trips, ir.body, ir.next_reg, next_block)",
            "validated constant induction",
            "label %{reserve}, label %{trap}",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-synchronous-forwarding",
        path: "crates/yir-lower-llvm/src/call_lowering.rs",
        required_patterns: &["signature.call_arguments(lowered_args)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-scoped-forwarding",
        path: "crates/yir-lower-llvm/src/loop_effect_action.rs",
        required_patterns: &["signature.call_arguments(lowered)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-no-deferred-escape",
        path: "crates/yir-lower-llvm/src/scalar_task_invoker.rs",
        required_patterns: &["!signature.implicit_parameters.is_empty()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-execution-probes",
        path: "tools/nuisc/tests/native_application_bridge/loop_work.rs",
        required_patterns: &[
            "whole_callback_loop_work_is_shared_and_resets_for_every_lifecycle_entry",
            "whole_callback_loop_work_traps_before_an_over_budget_child_or_outer_body",
            "whole_callback_loop_work_checks_induction_before_debit_and_skips_dead_work",
            "whole_callback_loop_work_does_not_refund_early_break",
            "whole_callback_loop_work_counts_constant_and_sequential_calls",
            "whole_callback_loop_work_uses_unsigned_budget_without_wrapping",
            "whole_callback_loop_work_reentrant_entry_cannot_reset_its_callers_budget",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-frontdoor",
        path: "tools/nuis/tests/native_session_workflow/loop_work.rs",
        required_patterns: &[
            "whole_callback_loop_work_survives_build_cache_and_standalone_restoration",
            "compile_cache: hit",
            "materialize-artifact",
            "1048576",
            "assert_eq!(states(&run), actual[..1])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-loop-work-policy-boundary",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Shared Callback Loop-Work Reservations",
            "emit_registered_with_loop_work_limit",
            "early-exit iteration",
            "compiled artifacts are not retroactively budgeted",
            "helper-entry accounting for loop-free fanout",
        ],
    },
];
