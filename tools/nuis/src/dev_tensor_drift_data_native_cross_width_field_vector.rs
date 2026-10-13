use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_fields.rs",
        required_patterns: &["ty == &scalar_type(\"bool\") || numeric(ty)", "!numeric(scope.get(owner)?)", "!child_scalar(ty)", "captures.len() > 31", "(2..64).contains(&count)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-unit-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_cross_width_field_vector_tests.rs",
        required_patterns: &["return_cross_width_field_vector_keeps_typed_root_authority_and_current_children", "source(child, body)", "scope(child)", "reject(&root)", "[\"ref\", \"optional\", \"generic\", \"bool\", \"String\", \"u32\"]", "sealed.constants.insert(\"local\".into())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-unit-layout",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_cross_width_field_vector_tests.rs",
        required_patterns: &["return_cross_width_field_vector_keeps_original_width_neutrals_and_distinct_projections", "scalar_type(child)", "assert_ne!(zero, child_zero)", "(\"owner\".into(), zero.clone())", "(\"child_1\".into(), child_zero.clone())", "(\"child_2\".into(), NirExpr::Bool(false))", "(\"child_3\".into(), NirExpr::Bool(false))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-source",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector.ns",
        required_patterns: &["seed: f32, small_seed: i32, float_seed: f64", "parent_work(value: f32, small: i32, child: f64", "after_work(value: f32, small: i32, child: f64", "answer_work(value: f32, expected: f32", "return -value;", "d && peer_work(c,", "c || bool_work(d,", "state.seed, state.small_seed, state.float_seed"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-oracle-widths",
        path: "tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector_oracle.rs",
        required_patterns: &["pub child_ty: &'static str", "cases_with_child(ty, ty)", "Self::cross(", "self.child_ty", "let child_sign = if child_ty == \"f32\"", "if child_ty == \"f32\"", "b ^= child_sign", "v ^ sign", "let children = inputs::words(child_ty)", "children[(index + offset) % children.len()]", "out.extend(masked_cases_with_child(ty, child_ty))", "format!(\"float_seed: {}{}\", self.state_decimal(), self.child_ty)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-oracle-source",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector_oracle.rs",
        required_patterns: &["return_float_owner_field_vector_oracle.rs", "\"f32\" => \"f64\"", "\"f64\" => \"f32\"", "replace(\"f32\", \"__OWNER__\")", "replace(\"f64\", \"__CHILD__\")", "shared::cases_with_child(owner, child_type(owner))", "shared::native_selected_with_child(owner, child_type(owner))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector.rs",
        required_patterns: &["if case.child_ty == \"f32\"", "Value::F32(f32::from_bits(case.seed as u32))", "Value::F64(f64::from_bits(case.float_seed as u64))", "12 * 2 * 4 * 4 * 4 + 8", "equality masks wrong owner bits", "state(&session, &stale)", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "marker.owner, 95"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-reference-model-widths",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector.rs",
        required_patterns: &["return_cross_width_field_vector_oracle_keeps_widths_and_raw_words_independent", "assert_eq!(case.previous().child_ty, child)", "assert_eq!(case.stale_sibling().child_ty, child)", "case.open_words()[0], case.seed", "case.open_words()[2], case.float_seed", "u32::try_from(case.seed)", "u32::try_from(case.float_seed)", "assert_ne!(same.float_seed, cross.float_seed)", "assert_eq!(same.owner, cross.owner)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-native-owner-f32-child-f64",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector_native_f32.rs",
        required_patterns: &["bitcast float %arg0 to i32", "bitcast double %arg2 to i64", "zext i32 %raw_owner to i64", "zext i1 %arg3 to i64", "zext i1 %arg4 to i64", "native_stale_mutant_with_equality_masking", "assert_eq!(current.result, wrong.result)", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-native-owner-f64-child-f32",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector_native_f64.rs",
        required_patterns: &["bitcast double %arg0 to i64", "bitcast float %arg2 to i32", "zext i32 %raw_float to i64", "zext i1 %arg3 to i64", "zext i1 %arg4 to i64", "native_stale_mutant_with_equality_masking", "assert_eq!(current.result, wrong.result)", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-shared-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_private_fields_tests.rs",
        required_patterns: &["check_typed_layout(source: &str, expected_layout: &str, owner: &str)", "compiled(source)", "layout.ends_with(expected_layout)", "check_owner_layout(source, expected_layout, \"i64\")", "expected_owner.replace(\"FLOAT\", ty)", "proof.parameters[0].ty, owner", "proof.result.as_ref().unwrap().ty, owner", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-aot-layout",
        path: "tools/nuisc/src/aot_application_effect_call_return_cross_width_field_vector_tests.rs",
        required_patterns: &["check_typed_layout", "oracle::child_type(owner)", "oracle::source(owner)", "owner:{owner};child_0:i32;child_1:{child};child_2:bool;child_3:bool"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_cross_width_field_vector.rs",
        required_patterns: &["return_cross_width_field_vector_oracle.rs", "oracle::source(ty)", "oracle::bits(oracle::child_type(ty), value)", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)", "oracle::cases(ty)", "raw.len(), 12 * 2 * 4 * 4 * 4 + 8", "oracle::STAGES.len(), 15"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-cli-legacy-identity",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_float_owner_field_vector.rs",
        required_patterns: &["Case::new(", "legacy.child_ty, ty", "legacy.state_words(), current.state_words()", "legacy.state_text(\"state\"), current.state_text(\"state\")", "legacy.open_args(), current.open_args()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["effectful_selections_return_cross_width_field_vector_tests.rs", "mod cross_width_vector_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_cross_width_field_vector.rs", "mod return_cross_width_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_cross_width_field_vector_tests.rs", "mod return_cross_width_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_cross_width_field_vector.rs", "mod return_cross_width_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Cross-width field-vector follow-up:", "Previous floating-owner field-vector checkpoint:", "Previous floating-owner field-vector task checkpoint:", "Previous floating-owner field-vector blocker checkpoint:", "Previous floating-owner field-vector action checkpoint:", "Previous floating-owner field-vector validation checkpoint:", "Previous floating-owner field-vector artifact checkpoint:", "nuis-native-cross-width-field-vector-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-receipt",
        path: "docs/reference/nuis-native-cross-width-field-vector-continuations-v1.md",
        required_patterns: &["complete 385-test predecessor", "396 distinct tests", "6208 successful native processes", "60 fatal native processes", "120 CLI event traps", "1032 accepted callbacks", "six source-equal fields", "active/99", "not formal memory safety", "not a full IEEE Cartesian product"],
    },
    DevTensorDriftCheckSpec {
        id: "native-cross-width-field-vector-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_cross_width_field_vector.rs", "cross_width_field_vector::CHECKS"],
    },
];
