use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_prints.rs",
        required_patterns: &[
            "pub(super) fn bounded",
            "computed_call_expression_roots",
            "pub(super) fn admit",
            "completed_pure_dependencies",
            "scalar_type(\"i64\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-completed-pure-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs",
        required_patterns: &[
            "pure_value: bool",
            "pure_value: false",
            "let pure = collect_with_layouts(module, layouts)",
            "helper.pure_value = matches!(profile, Profile::Pure)",
            "fn completed_pure_dependencies",
            "helper.pure_value",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-source-preflight",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_body.rs",
        required_patterns: &[
            "!iteration_prints::bounded(&function.body, catalog)",
            "!iteration_prints::bounded(body, catalog)",
            "Profile::IterationPrints",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_prints_tests.rs",
        required_patterns: &[
            "computed_iteration_prints_require_independently_completed_pure_call_dependencies",
            "computed_iteration_prints_admit_exact_i64_values_without_hidden_logical_or_reference_roots",
            "computed_iteration_prints_share_one_root_budget_across_branches_and_loops",
            "computed_iteration_prints_preserve_legacy_literal_buffer_and_pure_boundaries",
            "computed_iteration_prints_execute_only_selected_values_with_current_iteration_inputs",
            "computed_iteration_prints_keep_scalar_branch_capture_hygiene_and_late_failure_order",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-explicit-llvm-policy",
        path: "crates/yir-lower-llvm/src/native_session/i64_prints.rs",
        required_patterns: &[
            "pub struct I64PrintPolicy",
            "LiteralPrintPolicy::new(sites)?",
            "inner.i64_values = true",
            "pub fn emit_registered_with_i64_prints",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-persisted-mode",
        path: "crates/yir-lower-llvm/src/native_session/build_policy.rs",
        required_patterns: &[
            "nuis-native-i64-print-build-policy-v1",
            "pub fn new_i64",
            "pub fn is_i64",
            "\"i64-v1\"",
            "native_session_i64_print_bound",
            "i64_build_policy_has_distinct_canonical_identity_without_upgrading_v1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-packaging-kind-isolation",
        path: "tools/yir-pack-aot/src/host_application_script.rs",
        required_patterns: &[
            "--native-i64-print-policy",
            "native print policy option/token kind mismatch",
            "computed_print_packaging_requires_matching_explicit_option_and_policy_kind",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/computed_iteration_print.ns",
        required_patterns: &[
            "let merged: Record = if outer",
            "print(local_after(post_div))",
            "let final_record = merged",
            "print(83)",
            "print(84)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/computed_iteration_print_cases.rs",
        required_patterns: &[
            "origin_record_join_cases.rs",
            "checked_div(self.input.post_div)",
            "if value == 84",
            "self.input",
            "self.printed(trace)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-native-and-negative-controls",
        path: "tools/nuisc/tests/native_application_bridge/computed_iteration_print.rs",
        required_patterns: &[
            "computed_iteration_print_native_keeps_one_predicate_and_selected_fault_prefixes",
            "computed_iteration_print_reference_preserves_predicate_counts_and_last_state",
            "computed_iteration_print_requires_distinct_exact_native_policy_and_bound_identity",
            "computed_iteration_print_policy_rejects_wrong_kinds_unselected_sites_and_unordered_work",
            "direct i64 constant",
            "unordered.edges.clear()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-source-free-delivery",
        path: "tools/nuis/tests/native_session_workflow/computed_iteration_print.rs",
        required_patterns: &[
            "project_for_source_with_i64_prints",
            "computed_iteration_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
            "trace.starts_with",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Computed i64 iteration-print witnesses:",
            "Previous origin-join checkpoint:",
            "Previous origin-join task checkpoint:",
            "Previous origin-join blocker checkpoint:",
            "Previous origin-join action checkpoint:",
            "Previous origin-join validation checkpoint:",
            "Previous origin-join artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-computed-iteration-print-receipt",
        path: "docs/reference/nuis-native-computed-iteration-print-v1.md",
        required_patterns: &[
            "Completed Pure Values",
            "Separate Native Grant",
            "Thirty-two core failures",
            "36 native/reference failures",
            "active/99",
        ],
    },
];
