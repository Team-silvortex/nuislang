use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_dual_i64_tests.rs", "mod dual_i64_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-exact-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["pub(super) fn numeric_pair", "owner == &scalar_type(\"i64\")", "[\"i64\", \"f32\", \"f64\"]", "child == &scalar_type(name)", "let numeric_pair = numeric_pair(owner_type, child_type)", "(wide_owner || wide_child || numeric_pair)", "!(bool_owner || i32_owner || wide_owner || numeric_pair)", "owned_fields: Option<(NirTypeRef, NirTypeRef)>", "owned_pair::install", "captures.len() > 31"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-one-outward-private-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["numeric_owner(staged.local.get(owner.name())?)", "!transport::numeric_pair(", "staged.local.get(owner.name())?", "staged.local.get(&child)?", "entering.contains_key(&child)", "!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2"],
    },
DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-exact-owned-fields",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["Representation follows independently proved field types", "fields: Vec<(String, String, NirTypeRef)>", "\"i64\" => NirExpr::Int(0)", "record(true)", "record(false)", "NirVisibility::Private", "NirExpr::FieldAccess"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-original-linear-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["tree::preflight(yes, no, suffix, depth)?", "!numeric_owner(scope.get(owner.name())?)", "No arm receives a copy of the remaining sequence"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-unit-positive-negative-and-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_dual_i64_tests.rs",
        required_patterns: &["accepts_orders_retention_nested_sequences_and_one_typed_target", "rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically", "owned_fields_neutral_guards_and_collision_free_projections", "shared_original_budget_and_capture_boundary", "(7, 7, true)", "for count in [31, 32]", "for depth in [7, 8]", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "for child_ty in [\"String\"]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-source-both-fields-and-orders",
        path: "tools/nuisc/tests/native_application_bridge/return_dual_i64_private_pair.ns",
        required_patterns: &["let child = child_initial(0 - seed - 1, 1 / child_init)", "return i64_xor(value, child)", "fn child_work(value: i64", "if child == 0", "if child == -1", "after_work(local, child, 1 / after)", "const seal: i64 = child + 0", "return answer_work(local, seed, 1 / answer) && first", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_dual_i64_oracle.rs",
        required_patterns: &["Imperative source replay never uses the compiler", "let mut child = !seed", "local ^= child", "pair = Some((local, child))", "Source-event positions cannot be confused with observed field values", "self.observe_until(position)", "i64::MIN, -4_294_967_297, -1, 0, 1, 4_294_967_296, i64::MAX", "values[stage + 2] = \"0\"", "83 => Case::new(96, true, true)", "97 => Case::new(97, true, true)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-native-observations-and-last-publication",
        path: "tools/nuisc/tests/native_application_bridge/return_dual_i64.rs",
        required_patterns: &["call void @nuis_debug_print_i64(i64 %arg0)", "call void @nuis_debug_print_i64(i64 %arg1)", "Some((\"after_work\", PAIR_PROBE))", "[96, 97, -96, -97]", "observation_prefix_uses_source_positions_not_field_markers", "case.before_failure(6)", "case.before_failure(7)", "case.before_failure(8)", "for reversed in [false, true]", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-shared-native-failure-prefix-and-canaries",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["observation: Option<(&str, &str)>", "llvm.insert_str(insertion, OWNER_PROBE)", "llvm.insert_str(insertion, observer)", "trace.extend(case.before_failure(index))", "Some(4 | 5)", "[(0, 91), (state_width + 1, 92)]", "load volatile i64, ptr %prev", "Full trace equality distinguishes AFTER from matching field words", "dynamic_loop_guard::run_command_bounded"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-aot-layout-effect-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_dual_i64_tests.rs",
        required_patterns: &["{owner:i64;child:i64}", "call_owned_struct", "cast_i64_to_i32", "yir_core::EdgeKind::Effect", "lacks dependency order", "Vec::<String>::new()", "nuis_scheduler_owned_aggregate_alloc_v1", "nuis_scheduler_owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-cli-cache-source-free-identity",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_dual_i64.rs",
        required_patterns: &["compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_dual_i64_tests.rs", "mod return_dual_i64;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_dual_i64.rs", "mod return_dual_i64;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_dual_i64.rs", "mod return_dual_i64;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-tensor-current-and-complete-predecessor",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Dual-i64 private continuation follow-up:", "Previous float-ancestor checkpoint:", "Previous float-ancestor task checkpoint:", "Previous float-ancestor blocker checkpoint:", "Previous float-ancestor action checkpoint:", "Previous float-ancestor validation checkpoint:", "Previous float-ancestor artifact checkpoint:", "nuis-native-private-dual-i64-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-bounded-contract",
        path: "docs/reference/nuis-native-private-dual-i64-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance", "owner: i64", "child: i64", "64 successful native processes", "18 fatal native processes", "36 CLI event traps", "315 distinct tests", "complete 307-test predecessor", "formal memory safety"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-ordinary-cli-and-scope",
        path: "docs/reference/nuis-native-private-dual-i64-continuations-v1.md",
        required_patterns: &["active/99", "six source-equal fields", "macOS CPU", "Other numeric/numeric", "not added to the current count"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-dual-i64-recursion-proof-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_return_dual_i64.rs", "return_dual_i64::CHECKS"],
    },
];
