use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-effect-join-captures-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &[
            "mod effect_join_captures;",
            "one_sided_effect_joins::CHECKS,",
            "effect_join_captures::CHECKS,",
            ".flatten()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-join-captures-condition-validation-and-data-dependency",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &[
            "if ready.get(condition) != Some(&scalar_type(\"bool\"))",
            "let mut inputs = BTreeSet::new()",
            "control_values::collect_inputs(value, &mut inputs)",
            "if matches!((&yes, &no), (Some(yes), Some(no)) if yes != no)",
            "inputs.insert(condition.to_string())",
            "captures: captured_params(inputs, ready)",
            "ready.insert(name, ty.clone())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-join-captures-unchanged-live-authority-and-paired-selector",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &[
            "let mut args = vec![live.clone()]",
            "condition: NirExpr::Var(joined.condition)",
            "(Some(value), None) | (None, Some(value))",
            "condition: NirExpr::Var(gate)",
            "then_body: vec![selected]",
            "else_body: vec![NirStmt::Return(Some(seed))]",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-join-captures-structural-dependency-hygiene-and-atomic-veto-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_join_capture_tests.rs",
        required_patterns: &[
            "conditional_return_join_captures_drop_only_unused_single_value_selectors",
            "conditional_return_join_captures_retain_condition_as_data_and_hygienic_live_gate",
            "conditional_return_join_captures_preserve_paired_selection_and_deduplicate_atoms",
            "conditional_return_join_captures_validate_unused_condition_before_atomic_publication",
            "reads(&function.body, &mut used)",
            "assert_eq!(args, &expected)",
            "assert_eq!((ready, bindings), before)",
            "if staged { 2 } else { 1 }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-join-captures-source-installation-nir-and-idempotence",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_one_sided_join_tests.rs",
        required_patterns: &[
            "conditional_return_join_captures_match_actual_single_value_helper_reads",
            "assert_eq!(joined.params.len(), 2)",
            "__nuis_effect_condition",
            "verify_nir_module(&module)",
            "assert_eq!(module, once)",
            "assert_eq!(cases, 12)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-join-captures-honest-proof-and-next-boundary",
        path: "docs/reference/nuis-native-effect-join-captures-v1.md",
        required_patterns: &[
            "Effect-join capture verification:",
            "lose one unused bool parameter/argument",
            "data capture",
            "Two-value helpers still capture the saved selector",
            "four condition-as-data/live-name",
            "1042-test one-sided receipt",
            "`active/99`",
            "No fresh Linux/GPU run",
            "measured speedup",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-effect-join-captures-tensor-preserves-prior-receipts",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Effect-join capture follow-up:",
            "Previous one-sided result join checkpoint:",
            "1042 distinct selected tests",
            "Previous partial-result join checkpoint:",
            "conditional_return_join_captures",
        ],
    },
];
