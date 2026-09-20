use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-policy",
        path: "crates/yir-lower-llvm/src/native_session/helper_entries.rs",
        required_patterns: &[
            "DEFAULT_HELPER_ENTRY_LIMIT: u64 = 1_048_576",
            "icmp uge i64 {remaining}, 1",
            "sub i64 {remaining}, 1",
            "call void @llvm.trap()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-producer-api",
        path: "crates/yir-lower-llvm/src/native_session/mod.rs",
        required_patterns: &[
            "emit_registered_with_work_limits",
            "pub helper_entry_limit: u64",
            "guard helpers that immediately return a neutral value",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-stack-owner",
        path: "crates/yir-lower-llvm/src/native_session/emit.rs",
        required_patterns: &[
            "%nuis_helper_entries = alloca i64, align 8",
            "store i64 {helper_entry_limit}, ptr %nuis_helper_entries, align 8",
            "parameters.push(super::HELPER_ENTRY_PARAMETER.to_owned())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-before-body",
        path: "crates/yir-lower-llvm/src/function_lowering.rs",
        required_patterns: &["native_session::helper_entries::enter("],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-no-pure-call-speculation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &[
            "Pure calls can still expand into substantial work",
            "scalar_helpers::contains_calls(&function.body)",
            "scalar_control::outline(",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-execution-probes",
        path: "tools/nuisc/tests/native_application_bridge/helper_entries.rs",
        required_patterns: &[
            "helper_entries_share_exact_budget_and_reset_at_lifecycle_roots",
            "helper_entries_reject_before_body_but_after_argument_evaluation",
            "helper_entries_skip_unselected_paths_and_validate_transport_first",
            "helper_entries_use_unsigned_limits_without_wrapping",
            "helper_entries_count_loop_free_roots_and_unused_results",
            "helper_entries_include_scoped_iterations_without_resetting_loop_reservations",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-reentrant-isolation",
        path: "tools/nuisc/tests/native_application_bridge/loop_work.rs",
        required_patterns: &[
            "probe_entries_before",
            "probe_entries_after",
            "probe_budgets_unchanged",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-source-predicate-observation",
        path: "tools/nuisc/tests/native_application_bridge/predicate_probe.rs",
        required_patterns: &[
            "predicate_probe_excludes_work_guards_by_provenance_not_comparison_opcode",
            "non_source_operands.insert(register)",
            "%nuis_helper_entries",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-frontdoor",
        path: "tools/nuis/tests/native_session_workflow/helper_entries.rs",
        required_patterns: &[
            "loop_free_helper_fanout_is_bounded_through_build_cache_and_standalone",
            "compile_cache: hit",
            "materialize-artifact",
            "fork19",
            "assert_eq!(states(&rejected), actual[..1])",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-rehash-rejection",
        path: "tools/nuisc/src/aot_application_bundle_tests.rs",
        required_patterns: &[
            "store i64 1048576, ptr %nuis_helper_entries",
            "store i64 18446744073709551615, ptr %nuis_helper_entries",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-helper-entry-documentation",
        path: "docs/reference/nuis-native-scalar-session-bridge-v1.md",
        required_patterns: &[
            "### Shared Callback Helper-Entry Accounting",
            "Every **actual admitted YIR function entry** consumes one unit",
            "emit_registered_with_work_limits",
            "outlined guard that immediately",
            "### Iteration-Local Bool Rebinding",
        ],
    },
];
