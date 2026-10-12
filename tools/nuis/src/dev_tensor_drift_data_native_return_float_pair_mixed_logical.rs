use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-exact-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["effectful_selections_return_float_pair_tests.rs", "transport::numeric(ty)", "!writes.remove(owner.name()) || writes.len() != 1", "entering.contains_key(&child)", "!staged.constants.contains(*name)", "Do not manufacture a bool Target token"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-private-kind-and-capture-union",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["[\"i32\", \"i64\", \"f32\", \"f64\"]", "[\"i64\", \"f32\", \"f64\"]", "owned_fields: Option<(NirTypeRef, NirTypeRef)>", "captures.len() > 31", "owned_pair::install", "!arm.defined.contains(*n)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-typed-owned-neutral-and-projections",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_owned.rs",
        required_patterns: &["\"i64\" => NirExpr::Int(0)", "\"f32\" => NirExpr::F32(\"0.0\".into())", "\"f64\" => NirExpr::F64(\"0.0\".into())", "[(\"owner\", field_types.0), (\"child\", field_types.1)]", "NirVisibility::Private", "record(zero.0, zero.1)", "NirExpr::FieldAccess"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-exact-signed-word-expressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["NirExpr::PackF32Word(v) => (next(v)? == scalar_type(\"f32\"))", "NirExpr::UnpackF32Word(v) => (next(v)? == scalar_type(\"i64\"))", "NirExpr::PackF64Word(v) => (next(v)? == scalar_type(\"f64\"))", "NirExpr::UnpackF64Word(v) => (next(v)? == scalar_type(\"i64\"))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-inference-nested-boolean-regression",
        path: "tools/nuisc/src/frontend/types/nir_float_compare_tests.rs",
        required_patterns: &["float_comparison_inference_keeps_six_exact_bool_results", "float_comparison_inference_rejects_cross_width_resources", "modified.generic_args.push(i64_type())", "NirBinaryOp::Ge", "comparison"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-both-width-source-budget-scope-linear-and-neutral-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_float_pair_tests.rs",
        required_patterns: &["float_cases!(f32_cases, \"f32\")", "float_cases!(f64_cases, \"f64\")", "keep_both_orders_nested_sequences_retention_and_one_bool_export", "keep_full_width_fields_literal_zero_guards_and_collision_free_installation", "reject_incoming_third_writes_seals_hidden_calls_and_exits_atomically", "(6, 12, true)", "for count in [31, 32]", "install_linear_helpers_and_private_layouts_without_tail_copy"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-nuis-source-sign-flips-retention-and-unused-stage",
        path: "tools/nuisc/tests/native_application_bridge/return_float_pair_mixed_logical_effectful_scalar_selections.ns",
        required_patterns: &["seed: f64, first: bool, second: bool", "let child = child_work(-child, 1 / child_rhs)", "if child < 0.0", "let local = after_work(child, seed, 1 / after)", "let ignored = unused_work(1 / unused)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-artifact-effect-policy-and-cast-veto",
        path: "tools/nuisc/src/aot_application_effect_call_return_float_pair_mixed_logical_tests.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "call_owned_struct", "checked_functions >= 9", "guarded_calls >= 16", "cast_f32_to_i64", "cast_i64_to_f64", "lacks dependency order", "verify_nuis_compiled_artifact"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-raw-native-carried-bits-and-zero-allocation-probe",
        path: "tools/nuisc/tests/native_application_bridge/return_float_pair.rs",
        required_patterns: &["0x8000_0000", "0x7ff8_1234_5678_abcd", "0xfff0_0000_0000_0001", "aggregate_values::definition(&llvm, \"after_work\")", "bitcast float %arg0 to i32", "bitcast double %arg0 to i64", "assert_eq!(case, 144)", "for reversed in [false, true]", "assert_eq!(actual, expected", "return_float_pair_mixed_logical_effectful_scalar_selections_f32_reject_numeric_word_casts", "return_float_pair_mixed_logical_effectful_scalar_selections_f64_reject_numeric_word_casts"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-independent-ieee-comparison-and-sign-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_float_pair_oracle.rs",
        required_patterns: &["Independent IEEE comparisons and sign-bit updates, not a compiler codec", "f32::from_bits", "f64::from_bits", "child ^= sign", "observed = Some(child)", "pub const STAGES: [i32; 9]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-cli-source-free-and-before-publication-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_float_pair_mixed_logical.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "const SEEDS: [f64; 4]", "NaN payloads belong", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-pair-complete-i64-predecessor-and-event-atomicity-next",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Float-pair mixed return bool-update follow-up:", "Previous i64-pair checkpoint:", "Previous i64-pair task checkpoint:", "Previous i64-pair blocker checkpoint:", "Previous i64-pair action checkpoint:", "Previous i64-pair validation checkpoint:", "Previous i64-pair artifact checkpoint:", "Prove event-time failure atomicity for typed private continuations", "nuis-native-private-float-pair-continuations-v1.md"],
    },
];
