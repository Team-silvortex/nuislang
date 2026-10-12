use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-bounded-arity",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_fields.rs",
        required_patterns: &["fn arity(count: usize)", "(2..64).contains(&count)", "Arity is representation capacity, never lexical write authority", "!arity(children.len())", "!seen.insert(name.clone())", "!child_scalar(ty)", "format!(\"child_{index}\")", "captures.len() > 31"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-shared-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["struct Children(Vec<String>)", "*remaining = remaining.checked_sub(body.len())?", "let mut remaining = 64", "candidates.len() == 2 && candidates.contains(owner.name())", "candidates.remove(owner.name());", "!transport::private_field_arity(candidates.len())", "!transport::private_field_arity(children.0.len())", "entering.contains_key(child)", "!staged.defined.contains(child)", "staged.constants.contains(child)", "tree::preflight(yes, no, suffix, depth)?", "Walk the original forest once"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-old-pair-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2", "entering.contains_key(&child)", "staged.constants.contains(&child)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-unit-orders-and-owner",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_numeric_field_vector_tests.rs",
        required_patterns: &["return_numeric_field_vector_admits_three_children_in_original_and_retained_orders", "s.fields.len()==4", "let c=relay(c)", "helper.return_type", "[\"local\",\"a\",\"b\",\"c\",\"ignored\"]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-unit-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_numeric_field_vector_tests.rs",
        required_patterns: &["return_numeric_field_vector_retains_authority_scope_types_and_atomic_rejection", "undefined.defined.remove(n)", "sealed.constants.insert(n.into())", "qualifier", "reject(&root)", "const c:i64=c", "let seed=local"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-unit-types-and-capacity",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_numeric_field_vector_tests.rs",
        required_patterns: &["return_numeric_field_vector_is_type_driven_and_uses_original_width_neutrals", "Every exact ordered numeric quadruple prepares", "Field capacity and captures are independent", "[0, 1, 2, 3, 16, 31, 63, 64]", "\"child_2\"", "__nuis_effect_fields_value_1", "NirExpr::CastI64ToI32"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-unit-budgets",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_numeric_field_vector_tests.rs",
        required_patterns: &["return_numeric_field_vector_preserves_original_work_forest_capture_statement_and_depth_bounds", "(5, 5, true)", "(4, 5, false)", "(5, 4, false)", "for count in [64, 65]", "for count in [16, 17]", "for count in [31, 32]", "for depth in [7, 8]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-source-four-inputs",
        path: "tools/nuisc/tests/native_application_bridge/return_numeric_field_vector.ns",
        required_patterns: &["other_seed: i64", "other_seed: other_seed", "state.other_seed", "other_initial(other_seed, 1 / other_init)", "other_work(c, 1 / other_rhs)", "after_work(local, a, b, c", "parent_work(local, a, b, c", "const other_seal: i64 = c", "((small < i32_from_i64(0)) != (child < 0.0)) != (other < 0)", "                if condition(b < 0.0, 1 / cond) {\n                    let a = small_work(a, 1 / small_rhs);\n                    let b = float_work(b, 1 / float_rhs);\n                    let c = other_work(c, 1 / other_rhs);\n                    if a == i32_from_i64(0) {\n                        let a = small_work(a, 1 / small_rhs);\n                        let c = other_work(c, 1 / other_rhs);\n                    }\n                } else {\n                    let c = other_work(c, 1 / other_rhs);\n                    let b = float_work(b, 1 / float_rhs);\n                    let a = small_work(a, 1 / small_rhs);\n                    if a == i32_from_i64(-1) {\n                        let b = float_work(b, 1 / float_rhs);\n                        let c = other_work(c, 1 / other_rhs);\n                    }\n                }\n"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_numeric_field_vector_oracle.rs",
        required_patterns: &["Independent signed complements and IEEE sign XORs", "pub other_seed: i64", "pub fields: Option<(i64, i64, i64, i64)>", "fields = Some((local, i64::from(a), b, c))", "for other in [i64::MIN, i64::MAX]", "STAGES: [i64; 13]", "Source-event positions disambiguate sentinels", "self.other_seed", "for index in [11, 12]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-reference-real-dependency",
        path: "tools/nuisc/tests/native_application_bridge/return_numeric_field_vector.rs",
        required_patterns: &["Value::Int(case.other_seed)", "count(), 19", "3 * 4 * 12 * 2 * 4", "288", "third child must affect real computation", "Some((70001, -70002, 96, -70002))", "marker.before_failure(10)", "marker.before_failure(11)", "marker.before_failure(12)", "for reversed in [false, true]", "assert_eq!(session.state(), &last)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-f32-actual-four-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_numeric_field_vector_native_f32.rs",
        required_patterns: &["i64 %arg0", "sext i32 %arg1 to i64", "bitcast float %arg2 to i32", "zext i32 %observed_bits to i64", "i64 %arg3", "Case::open_words", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-f64-actual-four-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_numeric_field_vector_native_f64.rs",
        required_patterns: &["i64 %arg0", "sext i32 %arg1 to i64", "bitcast double %arg2 to i64", "i64 %arg3", "Case::open_words", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-aot-layout",
        path: "tools/nuisc/src/aot_application_effect_call_return_numeric_field_vector_tests.rs",
        required_patterns: &["native_return_numeric_field_vector_exact_layout_guarded_order_and_policy_vetoes", "super::return_private_fields::check_layout", "{owner:i64;child_0:i32;child_1:FLOAT;child_2:i64}"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-aot-exact-expected-layout",
        path: "tools/nuisc/src/aot_application_effect_call_return_private_fields_tests.rs",
        required_patterns: &["expected_layout: &str", "expected_layout.replace(\"FLOAT\", ty)", "fn width_safe_narrow", "nonparticipating_coercions", "lacks dependency order", "&proof.parameters[0].node", "&proof.parameters[1].node"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-cli-eight-images",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_numeric_field_vector.rs",
        required_patterns: &["return_numeric_field_vector_oracle.rs", "before_failure(12)", "usize::from(opened.fields.is_some()) * 4", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["effectful_selections_return_numeric_field_vector_tests.rs", "mod vector_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_numeric_field_vector.rs", "mod return_numeric_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_numeric_field_vector_tests.rs", "mod return_numeric_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_numeric_field_vector.rs", "mod return_numeric_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-complete-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Numeric field-vector follow-up:", "Previous retained-owner checkpoint:", "Previous retained-owner task checkpoint:", "Previous retained-owner blocker checkpoint:", "Previous retained-owner action checkpoint:", "Previous retained-owner validation checkpoint:", "Previous retained-owner artifact checkpoint:", "nuis-native-numeric-field-vector-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-bounded-receipt",
        path: "docs/reference/nuis-native-numeric-field-vector-continuations-v1.md",
        required_patterns: &["## Field List And Authority", "## Executed Acceptance", "463 distinct tests", "4608 successful native processes", "52 fatal native processes", "104 CLI event traps", "complete 452-test predecessor", "not formal memory safety", "active/99", "six source-equal fields"],
    },
    DevTensorDriftCheckSpec {
        id: "native-numeric-field-vector-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_numeric_field_vector.rs", "numeric_field_vector::CHECKS"],
    },
];
