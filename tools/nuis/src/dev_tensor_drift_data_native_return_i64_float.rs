use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-admission-and-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_i64_float_tests.rs", "mod i64_float_tests;", "pub(super) fn numeric_owner", "authorized.is_none_or(|target| target.name() == *name)", "!staged.constants.contains(*name)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-exact-child-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["pub(super) fn numeric_pair", "owner == &scalar_type(\"i64\")", "[\"i64\", \"f32\", \"f64\"]", "child == &scalar_type(name)", "owned_fields: Option<(NirTypeRef, NirTypeRef)>", "(wide_owner || wide_child || numeric_pair)", "captures.len() > 31", "owned_pair::install"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-one-outward-private-child",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["!transport::numeric_pair(", "entering.contains_key(&child)", "!staged.defined.contains(&child)", "staged.constants.contains(&child)", "!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-typed-neutral-and-same-carrier",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_owned.rs",
        required_patterns: &["No bit is borrowed from either field", "[(\"owner\", field_types.0), (\"child\", field_types.1)]", "\"f32\" => NirExpr::F32(\"0.0\".into())", "\"f64\" => NirExpr::F64(\"0.0\".into())", "\"i64\" => NirExpr::Int(0)", "record(zero.0, zero.1)", "record(NirExpr::Var(owner.clone()), NirExpr::Var(child.clone()))", "NirVisibility::Private", "NirExpr::FieldAccess"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-eight-unit-proofs",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_i64_float_tests.rs",
        required_patterns: &["float_child_cases!(f32_child", "float_child_cases!(f64_child", "accepts_orders_retention_nested_sequences_and_one_typed_target", "rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically", "owned_fields_neutral_guards_and_collision_free_projections", "shared_original_budget_and_capture_boundary", "(7, 7, true)", "for count in [16, 17]", "for count in [31, 32]", "for depth in [7, 8]", "assert_eq!(module, before)", "assert_eq!(names, before_names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-original-sequence-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["tree::preflight(yes, no, suffix, depth)?", "!numeric_owner(scope.get(owner.name())?)", "No arm receives a copy of the remaining sequence"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-nuis-source-reads-current-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_float_private_pair.ns",
        required_patterns: &["seed: f64", "if value == 0.0 { return 96; }", "return 9223372036854775807", "if child < 0.0 { return 0 - value - proof; }", "return -value", "after_work(local, child, 1 / after)", "const seal: f64 = child", "return answer_work(local, 1 / answer) && first", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-independent-ieee-and-marker-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_float_oracle.rs",
        required_patterns: &["Independent integer updates and IEEE sign-bit changes never use the compiler carrier", "f32::from_bits(bits as u32)", "f64::from_bits(bits as u64)", "child ^= sign", "pair = Some((local, child))", "Source-event positions cannot be confused with observed field values", "0x7fc0_1234", "0xfff0_0000_0000_0001_u64 as i64", "stage == 83 || stage == 97", "Case::new(ty, 0, true, true)", "self.observe_until(position)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-reference-and-late-marker-failure",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_float.rs",
        required_patterns: &["Value::F32", "Value::F64", "for reversed in [false, true]", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "observation_prefix_keeps_marker_values_before_real_late_faults", "oracle::selected(ty)[7].pair", "oracle::selected(ty)[8].pair", "case.before_failure(6)", "case.before_failure(7)", "case.before_failure(8)", "call void @nuis_debug_print_i64(i64 %arg0)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-native-f32-actual-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_float_native_f32.rs",
        required_patterns: &["call void @nuis_debug_print_i64(i64 %arg0)", "bitcast float %arg1 to i32", "zext i32 %observed_child_bits to i64", "Some((\"after_work\", PAIR_PROBE))", "&oracle::cases(\"f32\")", "&oracle::selected(\"f32\")", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-native-f64-actual-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_float_native_f64.rs",
        required_patterns: &["call void @nuis_debug_print_i64(i64 %arg0)", "bitcast double %arg1 to i64", "Some((\"after_work\", PAIR_PROBE))", "&oracle::cases(\"f64\")", "&oracle::selected(\"f64\")", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-shared-actual-callback-failure-policy",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["llvm.insert_str(insertion, observer)", "trace.extend(case.before_failure(index))", "load volatile i64, ptr %prev", "[(0, 91), (14, 92)]", "Some(4 | 5)", "dynamic_loop_guard::run_command_bounded", "nuis_scheduler_owned_aggregate_alloc_v1", "nuis_scheduler_owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-aot-exact-types-effects-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_i64_float_tests.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "{{owner:i64;child:{ty}}}", "result.ty == \"i64\"", "call_owned_struct", "cast_f32_to_i64", "cast_i64_to_f32", "yir_core::EdgeKind::Effect", "lacks dependency order", "Vec::<String>::new()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-cli-eight-images-source-free-identities",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_i64_float.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_i64_float_tests.rs", "mod return_i64_float;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_i64_float.rs", "mod return_i64_float;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_i64_float.rs", "mod return_i64_float;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-tensor-current-and-full-predecessor",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["I64-float private continuation follow-up:", "Previous dual-i64 checkpoint:", "Previous dual-i64 task checkpoint:", "Previous dual-i64 blocker checkpoint:", "Previous dual-i64 action checkpoint:", "Previous dual-i64 validation checkpoint:", "Previous dual-i64 artifact checkpoint:", "nuis-native-private-i64-float-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-bounded-contract",
        path: "docs/reference/nuis-native-private-i64-float-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance", "owner: i64", "child: f32 or f64", "328 distinct tests", "192 successful native processes", "36 fatal native processes", "72 CLI event traps", "complete 315-test predecessor", "formal memory safety"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-receipt-scope-and-drift",
        path: "docs/reference/nuis-native-private-i64-float-continuations-v1.md",
        required_patterns: &["macOS CPU", "active/99", "six source-equal fields", "Other numeric/numeric", "not added to the current count", "readonly entering values"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-float-recursive-registry",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_i64_float.rs", "return_i64_float::CHECKS"],
    },
];
