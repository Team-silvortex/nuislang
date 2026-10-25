use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-fresh-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs",
        required_patterns: &[
            "pub(super) fn enter_child",
            "changed.intersection(available)",
            "Self::enter(scope, writable, &ready, layouts)",
            ".intersection(writable)",
            "scope.get(*name) == Some(&scalar_type(\"i64\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-before-readiness-reset",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops.rs",
        required_patterns: &[
            "print_epoch::Epoch::enter_child(",
            "&locals.available",
            "child.available.retain(|name| !changed.contains(name))",
            "&print_epoch,",
            "locals.available.extend(changed)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-controls",
        path:
            "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/child_print_epoch_tests.rs",
        required_patterns: &[
            "child_print_epoch_requires_ready_exact_mutable_changed_entry_names",
            "child_print_epoch_lends_no_readiness_to_assignments_or_grandchildren",
            "child_print_epoch_keeps_unavailable_ancestors_and_ordinary_reads_closed",
            "child_print_epoch_executes_fresh_entries_for_leading_and_trailing_loops",
            "child_print_epoch_executes_unsimplified_nir_without_hoisting_poisoned_children",
            "assert!(locals.available.is_empty())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-existing-value-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "iteration_prints::admit(value, &locals.scope, catalog, layouts)?",
            "print_epoch.reads(value, &locals.scope, &locals.available, updates)?",
            "view.to_mut().available.extend(reads)",
            "expression_with_fields(",
            "Some(print_epoch)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-nuis-witness",
        path: "tools/nuisc/tests/native_application_bridge/child_epoch_print.ns",
        required_patterns: &[
            "let mut local = current",
            "while child < children",
            "print(observe(local, divisor - index - child))",
            "local = local + delta",
            "current = local",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/child_epoch_print_cases.rs",
        required_patterns: &[
            "LOOP_LIMIT: u64 = 12",
            "for index in 0..self.trips",
            "let mut local = current",
            "for child in 0..self.children",
            "self.divisor.wrapping_sub(index).wrapping_sub(child)",
            "0i64.checked_div(divisor)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-native-equivalence",
        path: "tools/nuisc/tests/native_application_bridge/child_epoch_print.rs",
        required_patterns: &[
            "child_epoch_print_native_keeps_current_versions_and_selected_fault_prefixes",
            "child_epoch_print_reference_preserves_callee_counts_and_last_state",
            "check_sequences_with_i64_prints",
            "compiled.yir.functions.len() <= 32",
            "ApplicationSessionPhase::Faulted",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-delivery",
        path: "tools/nuis/tests/native_session_workflow/child_epoch_print.rs",
        required_patterns: &[
            "child_epoch_print_fresh_cache_and_source_free_delivery_keep_selected_checks",
            "project_for_source_with_i64_prints",
            "compile_cache: hit",
            "for i in 0..2",
            "fresh.host_objects == cached.host_objects",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Fresh child-epoch print witnesses:",
            "Previous carry-epoch checkpoint:",
            "Previous carry-epoch task checkpoint:",
            "Previous carry-epoch blocker checkpoint:",
            "Previous carry-epoch action checkpoint:",
            "Previous carry-epoch validation checkpoint:",
            "Previous carry-epoch artifact checkpoint:",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-print-receipt",
        path: "docs/reference/nuis-native-child-epoch-print-v1.md",
        required_patterns: &[
            "Fresh Child Entry",
            "nine failure rows",
            "64 -> 72 -> -2",
            "active/99",
        ],
    },
];
