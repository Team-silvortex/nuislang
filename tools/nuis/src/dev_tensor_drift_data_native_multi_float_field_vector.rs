use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_fields.rs",
        required_patterns: &["ty == &scalar_type(\"bool\") || numeric(ty)", "!numeric(scope.get(owner)?)", "captures.len() > 31", "(2..64).contains(&count)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-record-types",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["\"f32\" => NirExpr::F32(\"0.0\".into())", "\"f64\" => NirExpr::F64(\"0.0\".into())", "Representation follows independently proved field types", "for (field, name, ty) in fields", "ty: Some(ty)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-unit-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_multi_float_field_vector_tests.rs",
        required_patterns: &["return_multi_float_field_vector_keeps_five_children_private_and_one_numeric_owner", "children.0, [\"a\", \"b\", \"c\", \"d\", \"e\"]", "&mixed_source(body)", "[\"ref\", \"optional\", \"generic\", \"String\", \"u32\"]", "undefined.defined.remove(name)", "sealed.constants.insert(name.into())", "bad.insert(\"local\".into(), scalar_type(\"bool\"))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-unit-layout",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_multi_float_field_vector_tests.rs",
        required_patterns: &["return_multi_float_field_vector_has_six_exact_neutrals_and_distinct_projections", "(\"child_1\".into(), NirExpr::F32(\"0.0\".into()))", "(\"child_2\".into(), NirExpr::F64(\"0.0\".into()))", "(\"child_3\".into(), NirExpr::Bool(false))", "(\"child_4\".into(), NirExpr::Bool(false))", "output[output.len() - 6..]", "for count in [31, 32]", "count == 31"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-source",
        path: "tools/nuisc/tests/native_application_bridge/return_multi_float_field_vector.ns",
        required_patterns: &["narrow_seed: f32, wide_seed: f64", "wide_work(value: f64, narrow: f32, flag: bool", "if (narrow < 0.0) != flag", "let c = wide_work(c, b, d, 1 / wide_rhs)", "let c = wide_work(c, b, e, 1 / wide_rhs)", "after_work(local, a, b, c, d, e", "parent_work(local, a, b, c, d, e", "const narrow_seal: f32 = b", "const wide_seal: f64 = c"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_multi_float_field_vector_oracle.rs",
        required_patterns: &["negative32(if stale { narrow_seed } else { b })", "value ^ i64::MIN", "b ^= 1 << 31", "c = wide(c, b, d)", "c = wide(c, b, e)", "i64::from(d)", "i64::from(e)", "STAGES: [i64; 17]", "Two companion wide profiles per narrow word"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_multi_float_field_vector.rs",
        required_patterns: &["Value::F32(f32::from_bits(case.narrow_seed as u32))", "Value::F64(f64::from_bits(case.wide_seed as u64))", "count(), 25", "12 * 2 * 2 * 2 * 4 * 4 + 8 + 4", "NaN/zero comparisons hide a stale wide sign", "current.stale_narrow()", "wide_work(c, narrow_seed,", "state(&session, &stale)", "Some((70001, -70002, 93, 96, 1, 1))", "marker.before_failure(16)", "assert_eq!(session.state(), &last)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-native",
        path: "tools/nuisc/tests/native_application_bridge/return_multi_float_field_vector_native.rs",
        required_patterns: &["bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "zext i1 %arg4 to i64", "zext i1 %arg5 to i64", "oracle::cases()", "oracle::native_selected()", "Case::open_words", "native_stale_mutant_with_masked_wide_signs", "assert_eq!(current.owner, wrong.owner)", "assert_ne!(current.fields, wrong.fields)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_multi_float_field_vector_tests.rs",
        required_patterns: &["check_typed_layout", "{owner:i64;child_0:i32;child_1:f32;child_2:f64;child_3:bool;child_4:bool}"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_multi_float_field_vector.rs",
        required_patterns: &["return_multi_float_field_vector_oracle.rs", "before_failure(16)", "usize::from(opened.fields.is_some()) * 6", "oracle::STAGES.len(), 17", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)", "assert_eq!(current.owner, stale.owner)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["effectful_selections_return_multi_float_field_vector_tests.rs", "mod multi_float_vector_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_multi_float_field_vector.rs", "mod return_multi_float_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_multi_float_field_vector_tests.rs", "mod return_multi_float_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_multi_float_field_vector.rs", "mod return_multi_float_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Simultaneous float field-vector follow-up:", "Previous cross-width field-vector checkpoint:", "Previous cross-width field-vector task checkpoint:", "Previous cross-width field-vector blocker checkpoint:", "Previous cross-width field-vector action checkpoint:", "Previous cross-width field-vector validation checkpoint:", "Previous cross-width field-vector artifact checkpoint:", "nuis-native-simultaneous-float-field-vector-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-receipt",
        path: "docs/reference/nuis-native-simultaneous-float-field-vector-continuations-v1.md",
        required_patterns: &["complete 396-test predecessor", "386 distinct tests", "3112 successful native processes", "34 fatal native processes", "68 CLI event traps", "556 accepted callbacks", "six source-equal fields", "active/99", "not formal memory safety", "not a full IEEE Cartesian product"],
    },
    DevTensorDriftCheckSpec {
        id: "native-multi-float-field-vector-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_multi_float_field_vector.rs", "multi_float_field_vector::CHECKS"],
    },
];
