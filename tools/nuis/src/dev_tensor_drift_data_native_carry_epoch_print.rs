use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-entry-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "pub(super) struct Epoch",
            ".intersection(writable)",
            "scope.get(*name) == Some(&scalar_type(\"i64\"))",
            "pub(super) fn reads",
            ".intersection(updates)",
            "self.seeds.contains(name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-local-read-lens",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "print_epoch::Epoch::enter(scope, writable, updates)",
            "iteration_prints::admit(value, &locals.scope, catalog, layouts)?",
            "print_epoch.reads(value, &locals.scope, &locals.available, updates)?",
            "std::borrow::Cow::Borrowed(&*locals)",
            "view.to_mut().available.extend(reads)",
            "expression(value, &view, updates, None, catalog, layouts, 0)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-child-isolation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops.rs",
        required_patterns: &[
            "child.available.retain(|name| !changed.contains(name))",
            "&print_epoch::Epoch::default()",
            "locals.available.extend(changed)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-source-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_tests.rs",
        required_patterns: &[
            "print_epoch_certifies_only_initialized_exact_i64_mutable_entry_carries",
            "print_epoch_lends_reads_only_to_one_print_without_mutating_availability",
            "print_epoch_keeps_assignment_condition_forward_local_and_child_permissions_separate",
            "print_epoch_executes_leading_trailing_and_branch_versions_in_source_order",
            "assert!(locals.available.is_empty())",
            "vec![7, 7, 9, 9, 9, 11]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/carry_epoch_print.ns",
        required_patterns: &[
            "let mut current = seed.total",
            "print(observe(current, divisor - index))",
            "print(current)",
            "current = current + delta",
            "finish(current, tail)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/carry_epoch_print_cases.rs",
        required_patterns: &[
            "previous.unwrap_or(self.initial)",
            "0i64.checked_div(self.divisor.wrapping_sub(index))",
            "current.wrapping_add(self.delta)",
            "0i64.checked_div(self.tail)",
            "divisor: 2",
            "delta: -37",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-native-equivalence",
        path: "tools/nuisc/tests/native_application_bridge/carry_epoch_print.rs",
        required_patterns: &[
            "carry_epoch_print_native_keeps_current_versions_and_selected_fault_prefixes",
            "carry_epoch_print_reference_preserves_callee_counts_and_last_state",
            "check_sequences_with_i64_prints",
            "compiled.yir.functions.len() <= 32",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/carry_epoch_print.rs",
        required_patterns: &[
            "carry_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "project_for_source_with_i64_prints",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Current-epoch carry-print witnesses:",
            "Previous computed-print checkpoint:",
            "Previous computed-print task checkpoint:",
            "Previous computed-print blocker checkpoint:",
            "Previous computed-print action checkpoint:",
            "Previous computed-print validation checkpoint:",
            "Previous computed-print artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-carry-epoch-print-receipt",
        path: "docs/reference/nuis-native-carry-epoch-print-v1.md",
        required_patterns: &[
            "One Print Read Lens",
            "six failure rows",
            "64 -> 72 -> -2",
            "active/99",
        ],
    },
];
