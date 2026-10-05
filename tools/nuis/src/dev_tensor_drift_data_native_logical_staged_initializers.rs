use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-static-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod logical_staged_initializers;", "logical_staged_initializers::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-root-only-original-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["!conditional_values::prefix::computed_logical_root(value)", "!entry::admitted(condition, scope, catalog, layouts)", "control_values::value_type(value, &scope, catalog, layouts)", "suffix::validate(&body[end..], scope, result, catalog, layouts)", "aliases::rewrite(&mut tail, &aliases)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-shared-expanded-reservation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_suffix.rs",
        required_patterns: &["pub(super) fn reserve_staged_prefix", "32usize.checked_sub(prefix.len())", "plan_expressions(&plan)", "expressions.push((value, 0, true))", "NirStmt::Print(value) => expressions.push((value, 0, false))", "expressions.extend(prefix.iter().map(|value| (*value, 0, false)))", "computed_expression_roots(expressions)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-existing-short-circuit-composition",
        path: "tools/nuisc/src/lowering/buffer_loop_outline.rs",
        required_patterns: &["conditional_returns::outline(", "selections.extend(conditional_values::outline(", "Stored return signals introduce private typed result layouts.", "scalar_helpers::collect_typed_values(module, &value_layouts, &control_catalog)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-independent-oracle-and-snapshots",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_logical_tests.rs",
        required_patterns: &["assert_eq!(cases, 864)", "before_print_source(input)", "chain_source(gate, left)", "prints[1] < observe && observe < prints[2]", "hidden-comparison", "hidden-call", "hidden-print", "total-only", "assert_eq!(module, before, \"{mutation}\")", "assert_eq!(module, once)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-original-and-expanded-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_logical_budget_tests.rs",
        required_patterns: &["for edges in [16, 17]", "for edges in [10, 11]", "for args in [4092, 4093]", "for depth in [63, 64]", "[27, 28]", "[24, 25]", "assert_eq!(module, before)", "Print roots do not inherit initializer authority"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-default-source-aot",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_logical_native_tests.rs",
        required_patterns: &["conditional_return_logical_initializers_execute_default_aot_short_circuit_and_traps", "chain_source(gate, 2)", "before_print_source(input)", "run(&text, result.map(|_| output.as_str()))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-proof-and-honest-receipt",
        path: "docs/reference/nuis-native-logical-staged-initializers-v1.md",
        required_patterns: &["Logical staged-initializer verification:", "864 core cases", "1776 ordinary/reversed semantic executions", "Twenty-four default-source AOT variants", "Seven selected failures", "no partial stdout", "Multiple initializers do not", "`active/98`", "historical"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-tensor-preserves-ordinary-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Logical staged-initializer follow-up:", "Previous ordinary staged-initializer checkpoint:", "Staged initializer acceptance:", "2094 distinct tests", "conditional_return_logical_initializers"],
    },
    DevTensorDriftCheckSpec {
        id: "native-logical-staged-initializers-mainline-and-boundaries",
        path: "docs/current-mainline-map.md",
        required_patterns: &["nuis-native-logical-staged-initializers-v1.md", "Ordinary", "print/call leaves and original scalar capture authority are not widened.", "further changing backedges and wide private captures"],
    },
];
