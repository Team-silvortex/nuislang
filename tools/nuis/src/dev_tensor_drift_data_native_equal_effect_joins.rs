use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod equal_effect_joins;", "effect_join_captures::CHECKS,", "equal_effect_joins::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-both-validated-atoms-before-selection-reduction",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["let yes = value(&yes)?", "let no = value(&no)?", "arm.scope.get(target)? != ty", "atom(value, ty, ready)", "if ready.get(condition) != Some(&scalar_type(\"bool\"))", "if matches!((&yes, &no), (Some(yes), Some(no)) if yes != no)", "(Some(yes), Some(no)) if yes == no => NirStmt::Return(Some(yes))", "condition: NirExpr::Var(joined.condition)", "condition: NirExpr::Var(gate)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-source-presence-and-data-captures-remain-independent",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["yes: Option<NirExpr>", "no: Option<NirExpr>", "if !arm.continues", "return Some(None)", "control_values::collect_inputs(value, &mut inputs)", "let mut args = vec![live.clone()]", "(Some(value), None) | (None, Some(value))", "else_body: vec![NirStmt::Return(Some(seed))]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-condition-data-hygiene-and-second-arm-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_join_capture_tests.rs",
        required_patterns: &["conditional_return_equal_effect_joins_retain_equal_condition_data_and_nonseed_atoms", "conditional_return_equal_effect_joins_do_not_skip_either_arm_validation", "NirExpr::Int(7)", "__nuis_join_live_0", "\"unready\"", "assert_eq!((ready, bindings), before, \"{mutation}\")", "shape == \"distinct\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-source-oracle-captures-budgets-and-selected-work",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_equal_join_tests.rs",
        required_patterns: &["assert_eq!(sources.len(), 384)", "assert_eq!(cases, 8)", "if shared { 2 } else { 1 }", "assert_eq!(joined.params.len(), 4)", "assert_eq!(module, once)", "verify_nir_module(&module)", "for copies in 0..=32", "preflight(then_body, 3)", "copies <= 20", "assert_eq!(module, before, \"{mutation}\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-default-source-aot-and-selected-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_equal_join_native_tests.rs",
        required_patterns: &["conditional_return_equal_effect_joins_execute_default_aot_work_exits_and_selected_traps", "result.map(|_| output.as_str())", "print(result); return 0;", "10 / left", "10 / right"],
    },
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-honest-proof-and-preserved-history",
        path: "docs/reference/nuis-native-equal-effect-result-joins-v1.md",
        required_patterns: &["Equal-atom effect-result join verification:", "389 distinct semantic sources", "778", "Eleven default-source AOT variants", "Three selected traps", "partial stdout", "`active/99`", "167-test capture receipt", "No fresh Linux/GPU run"],
    },
    DevTensorDriftCheckSpec {
        id: "native-equal-effect-joins-tensor-preserves-capture-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Equal-atom effect-result join follow-up:", "Previous effect-join capture checkpoint:", "167 distinct selected tests", "Previous one-sided result join checkpoint:", "conditional_return_equal_effect_joins"],
    },
];
