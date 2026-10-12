use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-registration-and-owner-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_float_i64_tests.rs", "mod float_i64_tests;", "pub(super) fn numeric_owner", "authorized.is_none_or(|target| target.name() == *name)", "!staged.constants.contains(*name)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-exact-orientation-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["pub(super) fn numeric_pair", "let float_owner = [\"f32\", \"f64\"]", "owner == &scalar_type(name)", "child == &scalar_type(\"i64\")", "i64_owner || float_owner", "owned_fields: Option<(NirTypeRef, NirTypeRef)>", "(wide_owner || wide_child || numeric_pair)", "captures.len() > 31", "owned_pair::install"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-one-outward-private-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["!transport::numeric_pair(", "entering.contains_key(&child)", "!staged.defined.contains(&child)", "staged.constants.contains(&child)", "!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-independent-neutral-and-projection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_owned.rs",
        required_patterns: &["No bit is borrowed from either field", "[(\"owner\", field_types.0), (\"child\", field_types.1)]", "\"f32\" => NirExpr::F32(\"0.0\".into())", "\"f64\" => NirExpr::F64(\"0.0\".into())", "\"i64\" => NirExpr::Int(0)", "record(zero.0, zero.1)", "record(NirExpr::Var(owner.clone()), NirExpr::Var(child.clone()))", "NirVisibility::Private", "NirExpr::FieldAccess"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-eight-unit-proof-families",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_float_i64_tests.rs",
        required_patterns: &["float_owner_cases!(f32_owner", "float_owner_cases!(f64_owner", "accepts_orders_retention_nested_sequences_and_one_typed_target", "rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically", "owned_fields_neutral_guards_and_collision_free_projections", "shared_original_budget_and_capture_boundary", "(7, 7, true)", "for count in [16, 17]", "for count in [31, 32]", "for depth in [7, 8]", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "for child_ty in [\"i32\", \"f32\", \"f64\", \"String\"]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-original-whole-forest-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["tree::preflight(yes, no, suffix, depth)?", "!numeric_owner(scope.get(owner.name())?)", "No arm receives a copy of the remaining sequence"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-nuis-current-owner-child-sequence",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i64_private_pair.ns",
        required_patterns: &["fn initialize(value: f64, proof: i64) -> f64", "fn child_initial(value: f64, proof: i64) -> i64", "return 0 - 9223372036854775807 - proof", "if child < 0 { return -value; }", "fn child_work(value: i64", "return 0 - value - proof", "if child == 96", "if child == -97", "after_work(local, child, 1 / after)", "const seal: i64 = child", "return answer_work(local, seed, 1 / answer) && first", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-independent-ieee-owner-and-integer-child",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i64_oracle.rs",
        required_patterns: &["Independent IEEE owner bits and full-width integer child updates never use the compiler carrier", "i64::MIN", "i64::MAX", "bits ^ sign", "child = !child", "pair = Some((local, child))", "Source-event positions cannot be confused with observed field values", "0x7fc0_1234", "0xfff0_0000_0000_0001_u64 as i64", "stage == 83 || stage == 97", "Case::new(ty, 0, true, true)", "self.observe_until(position)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-reference-last-publication-and-marker-values",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i64.rs",
        required_patterns: &["Value::F32", "Value::F64", "for reversed in [false, true]", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "observation_prefix_keeps_marker_values_before_real_late_faults", "Some((0, 96))", "oracle::selected(ty)[7].pair", "oracle::selected(ty)[8].pair", "case.before_failure(6)", "case.before_failure(7)", "case.before_failure(8)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-native-f32-actual-pair-and-final-owner",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i64_native_f32.rs",
        required_patterns: &["bitcast float %arg0 to i32", "zext i32 %observed_bits to i64", "call void @nuis_debug_print_i64(i64 %arg1)", "Some((\"after_work\", PAIR_PROBE))", "&oracle::cases(\"f32\")", "&oracle::selected(\"f32\")", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-native-f64-actual-pair-and-final-owner",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i64_native_f64.rs",
        required_patterns: &["bitcast double %arg0 to i64", "call void @nuis_debug_print_i64(i64 %arg1)", "Some((\"after_work\", PAIR_PROBE))", "&oracle::cases(\"f64\")", "&oracle::selected(\"f64\")", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-shared-native-callback-canaries-traps",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["llvm.insert_str(insertion, OWNER_PROBE)", "llvm.insert_str(insertion, observer)", "trace.extend(case.before_failure(index))", "load volatile i64, ptr %prev", "[(0, 91), (14, 92)]", "Some(4 | 5)", "dynamic_loop_guard::run_command_bounded", "nuis_scheduler_owned_aggregate_alloc_v1", "nuis_scheduler_owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-aot-typed-layout-effect-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_float_i64_tests.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "{{owner:{ty};child:i64}}", "result.ty == ty", "call_owned_struct", "cast_f32_to_i64", "cast_i64_to_f32", "yir_core::EdgeKind::Effect", "lacks dependency order", "Vec::<String>::new()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-cli-eight-images-source-free-identities",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_float_i64.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_float_i64_tests.rs", "mod return_float_i64;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_float_i64.rs", "mod return_float_i64;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_float_i64.rs", "mod return_float_i64;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-tensor-current-and-complete-predecessor",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Float-i64 private continuation follow-up:", "Previous i64-float checkpoint:", "Previous i64-float task checkpoint:", "Previous i64-float blocker checkpoint:", "Previous i64-float action checkpoint:", "Previous i64-float validation checkpoint:", "Previous i64-float artifact checkpoint:", "nuis-native-private-float-i64-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-bounded-contract-receipt",
        path: "docs/reference/nuis-native-private-float-i64-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance", "owner: f32 or f64", "child: i64", "343 distinct tests", "192 successful native processes", "36 fatal native processes", "72 CLI event traps", "complete 328-test predecessor", "formal memory safety"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-receipt-scope-and-drift",
        path: "docs/reference/nuis-native-private-float-i64-continuations-v1.md",
        required_patterns: &["macOS CPU", "active/99", "six source-equal fields", "Other numeric/numeric", "not added to the current count", "readonly entering values"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i64-recursive-guard-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_float_i64.rs", "return_float_i64::CHECKS"],
    },
];
