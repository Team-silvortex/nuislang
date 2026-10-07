use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-flat-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["mod typed_effect_snapshots;", "equal_effect_joins::CHECKS,", "typed_effect_snapshots::CHECKS,", ".flatten()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-exact-owned-data-profile-and-neutral-kinds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_scalars.rs",
        required_patterns: &["ty == &scalar_type(&ty.name)", "\"bool\" | \"i64\" | \"i32\" | \"f32\" | \"f64\"", "assert!(admitted(ty))", "control_values::zero_value(ty, &control_values::TypedLayouts::default())", "borrowed.is_ref = true", "assert!(!admitted(&borrowed))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-original-rewritten-types-and-bool-paths",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_regions.rs",
        required_patterns: &["mod data_scalars;", "control_values::value_type(value, scope, catalog, layouts)", "!data_scalars::admitted(&ty) || declared.is_some_and(|declared| declared != &ty)", "rewrite_value(value, values)", "!= Some(&ty)", "!ready.get(name).is_some_and(data_scalars::admitted)", "!= Some(scalar_type(\"bool\"))", "let seed = data_scalars::seed(&ty)", "if !preflight(body, end)", "suffix::validate(&body[end..], scope, result, catalog, layouts)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-independent-join-presence-and-exact-atoms",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_region_joins.rs",
        required_patterns: &["if !data_scalars::admitted(ty)", "arm.scope.get(target)? != ty", "let yes = value(&yes)?", "let no = value(&no)?", "NirExpr::F32(_) => ty == &scalar_type(\"f32\")", "NirExpr::F64(_) => ty == &scalar_type(\"f64\")", "if ready.get(condition) != Some(&scalar_type(\"bool\"))", "let seed = data_scalars::seed(&joined.ty)", "(Some(yes), Some(no)) if yes == no"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-oracle-seeds-typed-traces-and-atomic-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_typed_snapshot_tests.rs",
        required_patterns: &["assert_eq!(sources.len(), 288)", "for input in [INPUTS[1], INPUTS[2]]", "NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0)))", "NirExpr::F32(\"-0.0\".into())", "assert!(!scalar(&scalar_type(\"f64\")))", "assert_eq!(module, before, \"{ty} {mutation}\")", "assert_eq!(module, once)", "\"legacy-tail\"", "for copies in 0..=32"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-default-source-aot-paths-and-traps",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_typed_snapshot_native_tests.rs",
        required_patterns: &["conditional_return_typed_snapshots_execute_default_aot_paths_and_selected_traps", "for ty in [\"i32\", \"f32\", \"f64\"]", "result.map(|_| output.as_str())", "assert_eq!(variants, 18)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-honest-bounded-receipt-and-source-negation-gap",
        path: "docs/reference/nuis-native-typed-effect-snapshots-v1.md",
        required_patterns: &["Typed effect-snapshot acceptance:", "288 distinct core sources", "576 executions", "Eighteen default-source native AOT variants", "three selected traps", "native float bit-pattern certificate", "Source unary `-0.0`", "`active/99`", "No fresh Linux/GPU run"],
    },
    DevTensorDriftCheckSpec {
        id: "native-typed-effect-snapshots-tensor-preserves-equal-atom-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Typed effect-snapshot follow-up:", "Previous equal-atom checkpoint:", "173 distinct selected tests", "Equal-atom effect-result join follow-up:", "conditional_return_typed_snapshots"],
    },
];
