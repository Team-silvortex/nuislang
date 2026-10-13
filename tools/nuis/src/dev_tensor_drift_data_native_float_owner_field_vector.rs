use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_fields.rs",
        required_patterns: &["ty == &scalar_type(\"bool\") || numeric(ty)", "!numeric(scope.get(owner)?)", "!child_scalar(ty)", "captures.len() > 31", "(2..64).contains(&count)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-unit-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_float_owner_field_vector_tests.rs",
        required_patterns: &["return_float_owner_field_vector_keeps_typed_root_authority_and_current_children", "reject(&root)", "[\"ref\", \"optional\", \"generic\", \"bool\", \"String\", \"u32\"]", "sealed.constants.insert(\"local\".into())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-unit-layout",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_float_owner_field_vector_tests.rs",
        required_patterns: &["return_float_owner_field_vector_keeps_original_width_neutrals_and_distinct_projections", "(\"owner\".into(), zero.clone())", "(\"child_1\".into(), zero.clone())", "NirExpr::F32(\"0.0\".into())", "NirExpr::F64(\"0.0\".into())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-source",
        path: "tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector.ns",
        required_patterns: &["seed: f64, small_seed: i32, float_seed: f64, bool_seed: bool, peer_seed: bool", "peer: bool, proof: i64) -> f64", "return -value;", "after_work(local, a, b, c, d", "d && peer_work(c,", "c || bool_work(d,", "state.seed, state.small_seed, state.float_seed"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector_oracle.rs",
        required_patterns: &["v ^ sign", "Two independent child profiles per owner", "for offset in [1, 5]", "out.extend(masked_cases_with_child(ty, child_ty))", "f32::from_bits(local as u32) == f32::from_bits(seed as u32)", "f64::from_bits(local as u64) == f64::from_bits(seed as u64)", "words[10], words[11]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector.rs",
        required_patterns: &["Value::F32(f32::from_bits(case.seed as u32))", "Value::F64(f64::from_bits(case.seed as u64))", "12 * 2 * 4 * 4 * 4 + 8", "equality masks wrong owner bits", "assert_ne!(current.owner, stale.owner)", "state(&session, &stale)", "marker.owner, 95", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-native-f32",
        path: "tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector_native_f32.rs",
        required_patterns: &["bitcast float %arg0 to i32", "bitcast float %arg2 to i32", "zext i32 %raw_owner to i64", "zext i1 %arg3 to i64", "zext i1 %arg4 to i64", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")", "Case::open_words", "native_stale_mutant_with_equality_masking", "assert_eq!(current.result, wrong.result)", "&source", "&stale"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-native-f64",
        path: "tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector_native_f64.rs",
        required_patterns: &["bitcast double %arg0 to i64", "bitcast double %arg2 to i64", "zext i1 %arg3 to i64", "zext i1 %arg4 to i64", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")", "Case::open_words", "native_stale_mutant_with_equality_masking", "assert_eq!(current.result, wrong.result)", "&source", "&stale"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-owner-aware-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_private_fields_tests.rs",
        required_patterns: &["check_owner_layout(source, expected_layout, \"i64\")", "expected_owner.replace(\"FLOAT\", ty)", "result.ty == owner", "proof.parameters[0].ty, owner", "proof.result.as_ref().unwrap().ty, owner", "layout.ends_with(expected_layout)", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-aot-layout",
        path: "tools/nuisc/src/aot_application_effect_call_return_float_owner_field_vector_tests.rs",
        required_patterns: &["check_owner_layout", "{owner:FLOAT;child_0:i32;child_1:FLOAT;child_2:bool;child_3:bool}"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_float_owner_field_vector.rs",
        required_patterns: &["oracle::masked_cases(ty)", "assert_ne!(current.owner, stale.owner)", "assert_eq!(current.result, stale.result)", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["effectful_selections_return_float_owner_field_vector_tests.rs", "mod float_owner_vector_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_float_owner_field_vector.rs", "mod return_float_owner_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_float_owner_field_vector_tests.rs", "mod return_float_owner_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_float_owner_field_vector.rs", "mod return_float_owner_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Floating-owner field-vector follow-up:", "Previous two-bool field-vector checkpoint:", "Previous two-bool field-vector task checkpoint:", "Previous two-bool field-vector blocker checkpoint:", "Previous two-bool field-vector action checkpoint:", "Previous two-bool field-vector validation checkpoint:", "Previous two-bool field-vector artifact checkpoint:", "nuis-native-floating-owner-field-vector-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-receipt",
        path: "docs/reference/nuis-native-floating-owner-field-vector-continuations-v1.md",
        required_patterns: &["complete 402-test predecessor", "385 distinct tests", "6208 successful native processes", "60 fatal native processes", "120 CLI event traps", "1032 accepted callbacks", "six source-equal fields", "active/99", "not formal memory safety", "not a full IEEE Cartesian product"],
    },
    DevTensorDriftCheckSpec {
        id: "native-floating-owner-field-vector-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_float_owner_field_vector.rs", "float_owner_field_vector::CHECKS"],
    },
];
