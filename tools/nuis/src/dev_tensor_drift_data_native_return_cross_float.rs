use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-exact-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["let float_owner = [\"f32\", \"f64\"]", "&& [\"i64\", \"f32\", \"f64\"]", ".any(|name| child == &scalar_type(name))", "i64_owner || float_owner", "owned_fields: Option<(NirTypeRef, NirTypeRef)>", "captures.len() > 31", "owned_pair::install"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-one-outward-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["!transport::numeric_pair(", "entering.contains_key(&child)", "!staged.defined.contains(&child)", "staged.constants.contains(&child)", "!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2"],
    },
DevTensorDriftCheckSpec {
        id: "native-return-cross-float-typed-independent-transport",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["fields: Vec<(String, String, NirTypeRef)>", "\"f32\" => NirExpr::F32(\"0.0\".into())", "\"f64\" => NirExpr::F64(\"0.0\".into())", "record(true)", "NirVisibility::Private", "NirExpr::FieldAccess"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-eight-unit-proof-families",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_cross_float_tests.rs",
        required_patterns: &["cross_width_cases!(", "f32_owner", "f64_owner", "const CHILD_TY: &str = $child_ty", "child_neutral()", "accepts_orders_retention_nested_sequences_and_one_typed_target", "rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically", "owned_fields_neutral_guards_and_collision_free_projections", "shared_original_budget_and_capture_boundary", "(7, 7, true)", "for count in [16, 17]", "for count in [31, 32]", "for depth in [7, 8]", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "for child_ty in [\"u32\", \"u64\", \"String\"]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-original-forest-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["tree::preflight(yes, no, suffix, depth)?", "!numeric_owner(scope.get(owner.name())?)", "No arm receives a copy of the remaining sequence"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-nuis-independent-inputs-and-current-reads",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_float_private_pair.ns",
        required_patterns: &["seed: f32, child_seed: f64", "fn child_initial(value: f64, proof: i64) -> f64", "child_initial(child_seed, 1 / child_init)", "fn parent_work(value: f32, child: f64", "fn child_work(value: f64", "fn after_work(value: f32, child: f64", "return -value", "if child < 0.0", "if child == 0.0", "after_work(local, child, 1 / after)", "const seal: f64 = child", "return answer_work(local, seed, 1 / answer) && first", "state.child_seed", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-independent-ieee-cartesian-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_float_oracle.rs",
        required_patterns: &["Independent mixed-width input words retain their own sign/payload bits", "pub fn child_type", "let child_sign = if child_ty == \"f32\"", "for seed in words(ty)", "for child_seed in words(child_type(ty))", "child ^= child_sign", "number(child, child_ty)", "pair = Some((local, child))", "[self.seed, self.child_seed]", "Field words may equal event markers", "self.observe_until(position)", "0x7fc0_1234", "0xfff0_0000_0000_0001_u64 as i64", "child_sign | 96", "for index in [7, 8]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-reference-publication-and-marker-prefixes",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_float.rs",
        required_patterns: &["[case.seed, case.child_seed]", ".zip([case.ty, case.child_ty()])", "Value::F32", "Value::F64", "for reversed in [false, true]", "count(), 13", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "12 * 12 * 4", "Some((sign, 96))", "case.before_failure(6)", "case.before_failure(7)", "case.before_failure(8)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-native-f32-actual-mixed-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_float_native_f32.rs",
        required_patterns: &["bitcast float %arg0 to i32", "bitcast double %arg1 to i64", "%observed_pair_owner", "%observed_child", "Some((\"after_work\", PAIR_PROBE))", "check_with_open_words", "Case::open_words", "&oracle::cases(\"f32\")", "&oracle::native_selected(\"f32\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-native-f64-actual-mixed-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_cross_float_native_f64.rs",
        required_patterns: &["bitcast double %arg0 to i64", "bitcast float %arg1 to i32", "%observed_pair_owner", "%observed_child", "Some((\"after_work\", PAIR_PROBE))", "check_with_open_words", "Case::open_words", "&oracle::cases(\"f64\")", "&oracle::native_selected(\"f64\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-shared-width-driven-callback-canaries",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["let state_width = cases[0].state_words().len()", "let open_width = open_words(&cases[0]).len()", "state_width + event_width", "state_width + 2", "[(0, 91), (state_width + 1, 92)]", "load volatile i64, ptr %prev", "trace.extend(case.before_failure(index))", "Some(4 | 5)", "dynamic_loop_guard::run_command_bounded", "llvm.insert_str(insertion, observer)", "nuis_scheduler_owned_aggregate_alloc_v1", "nuis_scheduler_owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-aot-typed-layout-effect-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_cross_float_tests.rs",
        required_patterns: &["{{owner:{ty};child:{child_ty}}}", "result.ty == ty", "call_owned_struct", "cast_f32_to_i64", "yir_core::EdgeKind::Effect", "lacks dependency order", "Vec::<String>::new()", "fn nonparticipating_coercions", "line.contains(\"fptrunc \")", "conversion.contains(\" = fptosi double \")", "Legacy last-value bookkeeping", "live float conversion", "\"ret double %1\", \"ret i64 %2\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-cli-eight-source-free-images",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_cross_float.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "&oracle::source(ty)", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_cross_float_tests.rs", "mod cross_float_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_cross_float_tests.rs", "mod return_cross_float;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_cross_float.rs", "mod return_cross_float;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_cross_float.rs", "mod return_cross_float;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-tensor-complete-predecessor",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Cross-width float private continuation follow-up:", "Previous same-float checkpoint:", "Previous same-float task checkpoint:", "Previous same-float blocker checkpoint:", "Previous same-float action checkpoint:", "Previous same-float validation checkpoint:", "Previous same-float artifact checkpoint:", "nuis-native-private-cross-float-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-bounded-contract-receipt",
        path: "docs/reference/nuis-native-private-cross-float-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance", "owner: f32 / child: f64", "owner: f64 / child: f32", "373 distinct tests", "2304 successful native processes", "36 fatal native processes", "72 CLI event traps", "complete 358-test predecessor", "formal memory safety"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-receipt-scope",
        path: "docs/reference/nuis-native-private-cross-float-continuations-v1.md",
        required_patterns: &["macOS CPU", "active/99", "six source-equal fields", "i32 numeric pairs", "not added to the current count", "readonly entering values"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-cross-float-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_cross_float.rs", "return_cross_float::CHECKS"],
    },
];
