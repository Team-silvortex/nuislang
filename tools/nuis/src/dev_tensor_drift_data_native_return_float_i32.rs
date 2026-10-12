use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-exact-orientation-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["let float_i32 = [\"f32\", \"f64\"]", "&& child == &scalar_type(\"i32\")", "i64_owner || float_owner || i32_wide || i32_same || i32_float || float_i32", "owned_fields: Option<(NirTypeRef, NirTypeRef)>", "(wide_owner || wide_child || numeric_pair)", "owned_pair::install", "i32_pair::pack(child, owner)", "i32_pair::restore(carrier, child, owner)", "captures.len() > 31"],
    },
DevTensorDriftCheckSpec {
        id: "native-return-float-i32-independent-i32-neutral-and-projection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["\"i32\" => NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0)))", "\"i64\" => NirExpr::Int(0)", "fields: Vec<(String, String, NirTypeRef)>", "record(true)", "NirVisibility::Private", "NirExpr::FieldAccess"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-one-outward-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["!transport::numeric_pair(", "entering.contains_key(&child)", "!staged.defined.contains(&child)", "staged.constants.contains(&child)", "!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-four-unit-proof-families",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_float_i32_tests.rs",
        required_patterns: &["float_owner_cases!(", "f32_owner", "f64_owner", "NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0)))", "accepts_orders_retention_nested_sequences_and_one_typed_target", "rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically", "owned_fields_neutral_guards_and_collision_free_projections", "shared_original_budget_and_capture_boundary", "(7, 7, true)", "for count in [16, 17]", "for count in [31, 32]", "for depth in [7, 8]", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "for child_ty in [\"u32\", \"u64\", \"String\"]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-original-forest-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["tree::preflight(yes, no, suffix, depth)?", "!numeric_owner(scope.get(owner.name())?)", "No arm receives a copy of the remaining sequence"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-nuis-current-independent-integer-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i32_private_pair.ns",
        required_patterns: &["seed: f64, child_seed: i32", "fn initialize(value: f64", "fn child_initial(value: i32", "child_initial(child_seed, 1 / child_init)", "fn parent_work(value: f64, child: i32", "const minus_one: i32 = i32_from_i64(-1)", "return minus_one - value", "if child == i32_from_i64(0)", "if child == i32_from_i64(-1)", "after_work(local, child, 1 / after)", "const seal: i32 = child", "return answer_work(local, seed, 1 / answer) && first", "state.child_seed", "print(80)", "print(79)", "return -value"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-independent-cartesian-ieee-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i32_oracle.rs",
        required_patterns: &["Independent IEEE sign bits and i32 complements never use the compiler carrier", "seed: i64", "child_seed: i32", "i32::MIN", "i32::MAX", "0x7fc0_1234", "0xff80_0001", "0x7ff8_1234_5678_abcd", "0xfff0_0000_0000_0001", "bits ^ sign", "child = !child", "pair = Some((local, i64::from(child)))", "Source-event positions cannot be confused", "Case::new(ty, sign | 96, 70001, true, true)", "-70002", "pub fn native_selected", "pub fn cli_cases", "self.state_decimal()", "f64::from(f32::from_bits(self.seed as u32)).to_string()", "(-2.5, -1)", "(1.5, 0)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-reference-exact-values-and-collision-prefixes",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i32.rs",
        required_patterns: &["Value::I32(case.child_seed)", "f32::from_bits(case.seed", "f64::from_bits(case.seed", "for opened in oracle::cases(ty)", "for reversed in [false, true]", "count(), 13", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "11 * 12 * 4", "Some((96, -70002))", "oracle::native_selected(ty)[7]", "case.before_failure(6)", "case.before_failure(7)", "case.before_failure(8)", "assert_eq!(tiny.state_words()[0], 1)", "assert_ne!(tiny.decimal(), tiny.state_decimal())", "assert_eq!(cli.len(), 24)", "for child_seed in [-1, 0]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-f32-actual-pair-and-final-owner",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i32_native_f32.rs",
        required_patterns: &["bitcast float %arg0 to i32", "zext i32 %observed_pair_bits to i64", "sext i32 %arg1 to i64", "check_with_open_words", "Case::open_words", "Some((\"after_work\", PAIR_PROBE))", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-f64-actual-pair-and-final-owner",
        path: "tools/nuisc/tests/native_application_bridge/return_float_i32_native_f64.rs",
        required_patterns: &["bitcast double %arg0 to i64", "sext i32 %arg1 to i64", "check_with_open_words", "Case::open_words", "Some((\"after_work\", PAIR_PROBE))", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-shared-width-canaries-and-sentinel-safe-trace",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["let state_width = cases[0].state_words().len()", "let open_width = open_words(&cases[0]).len()", "state_width + 2", "[(0, 91), (state_width + 1, 92)]", "load volatile i64, ptr %prev", "Full trace equality distinguishes AFTER from matching field words", "expected(case, failure, index)", "trace.extend(case.before_failure(index))", "Some(4 | 5)", "dynamic_loop_guard::run_command_bounded", "llvm.insert_str(insertion, observer)", "nuis_scheduler_owned_aggregate_alloc_v1", "nuis_scheduler_owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-aot-layout-literal-casts-effect-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_float_i32_tests.rs",
        required_patterns: &["{owner:{ty};child:i32}", "result.ty == ty", "call_owned_struct", "cast_i64_to_i32", "fn width_safe_narrow", "assert!(!width_safe_narrow", "depth >= 8", "Some(0 | -1)", "yir_core::EdgeKind::Effect", "lacks dependency order", "Vec::<String>::new()", "f.name == \"initialize\"", "&proof.parameters[1].node", "fn canonical_i32_codec", "nonparticipating_coercions", "cast_f32_to_i64", "cast_f64_to_i64", "fptrunc", "ret double %1", "ret i64 %2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-cli-eight-source-free-images",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_float_i32.rs",
        required_patterns: &["oracle::cli_cases(ty)", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_float_i32_tests.rs", "mod float_i32_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_float_i32_tests.rs", "mod return_float_i32;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_float_i32.rs", "mod return_float_i32;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_float_i32.rs", "mod return_float_i32;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-tensor-complete-predecessor",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Float-i32 private continuation follow-up:", "Previous i32-float checkpoint:", "Previous i32-float task checkpoint:", "Previous i32-float blocker checkpoint:", "Previous i32-float action checkpoint:", "Previous i32-float validation checkpoint:", "Previous i32-float artifact checkpoint:", "nuis-native-private-float-i32-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-bounded-contract-receipt",
        path: "docs/reference/nuis-native-private-float-i32-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance", "owner: f32 or f64 / child: i32", "430 distinct tests", "2112 successful native processes", "36 fatal native processes", "72 CLI event traps", "complete 415-test predecessor", "formal memory safety"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-receipt-scope",
        path: "docs/reference/nuis-native-private-float-i32-continuations-v1.md",
        required_patterns: &["macOS CPU", "active/99", "six source-equal fields", "Two private children", "not added to the current count", "readonly entering values", "diagnostic sentinel"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-i32-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_float_i32.rs", "return_float_i32::CHECKS"],
    },
];
