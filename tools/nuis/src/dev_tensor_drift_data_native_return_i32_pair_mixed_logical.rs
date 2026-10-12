use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-exact-private-owner-child-classification",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["effectful_selections_return_i32_pair_tests.rs", "transport::numeric(ty)", "staged.defined.contains(*name)", "!staged.constants.contains(*name)", "!writes.remove(owner.name()) || writes.len() != 1", "entering.contains_key(&child)", "staged.local.get(&child)? == &scalar_type(\"bool\")", "transport::numeric(staged.local.get(&child)?)", "Only the classified prefix-private child gets this scalar write", "Do not manufacture a bool Target token", "staged.constants", "staged.defined.extend(next.defined)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-closed-carrier-exact-scalar-family-and-retention",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["effectful_selections_transport_i32.rs", "i32_child: bool", "let child_type = scope.get(child)?", "owner_type == &scalar_type(\"bool\")", "child_type != &scalar_type(\"bool\")", "captures.len() > 31", "!arm.defined.contains(*n)", "i32_pair::pack(owner, child)", "i32_pair::restore(carrier, owner, child)", "plan.destination = Some((carrier.clone(), false))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-full-signed-33-bit-range-nontrapping-codec",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_i32.rs",
        required_patterns: &["const BIAS: i64 = 2_147_483_648", "const FLAG: i64 = 4_294_967_296", "0..=2^33-1", "NirExpr::CastI32ToI64", "NirExpr::CastBoolToI64", "NirBinaryOp::Ge", "NirBinaryOp::Sub", "NirExpr::CastI64ToI32", "no division, remainder or fallible source work"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-bounded-terminal-nested-and-sequential-routing",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["suffix.is_empty()", "!transport::numeric(scope.get(&child.0)?)", "suffix.len() > 16", "tree::preflight(yes, no, suffix, depth)?", "tree::isolated(body, scope, future)", "depth >= 8", "No arm receives a copy of the remaining sequence", "transport::prepare"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-six-original-budget-range-veto-and-linear-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_i32_pair_tests.rs",
        required_patterns: &["keep_both_orders_nested_sequences_retention_and_one_bool_export", "roundtrip_full_signed_extremes_both_flags_without_traps_or_name_collision", "reject_wide_children_third_writes_seals_hidden_calls_and_exits_atomically", "charge_original_casts_nodes_and_edges_once", "keep_lexical_and_capture_bounds_with_retained_versions", "install_two_guarded_helpers_per_original_selection_without_tail_copy", "i32::try_from(v).unwrap()", "(6, 14, true)", "for count in [31, 32]", "for count in [1, 4, 8]", "reject_module(parsed(&text))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-native-signed-child-flips-current-reads-and-unused-effects",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_pair_mixed_logical_effectful_scalar_selections.ns",
        required_patterns: &["seed: i64, first: bool, second: bool", "fn child_work(value: i32", "let child = i32_from_i64(child_initial(seed, 1 / child_init))", "i32_from_i64(0) - child - i32_from_i64(1)", "if child < i32_from_i64(0)", "let local = after_work(child, i32_from_i64(seed), 1 / after)", "let ignored = unused_work(1 / unused)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-artifact-exact-casts-guards-effects-policy-and-veto",
        path: "tools/nuisc/src/aot_application_effect_call_return_i32_pair_mixed_logical_tests.rs",
        required_patterns: &["native_return_i32_pair_mixed_logical_effectful_scalar_selections", "State{seed:i64;first:bool;second:bool", "cast_i32_to_i64", "cast_i64_to_i32", "checked_functions >= 9", "guarded_calls >= 16", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-independent-signed-oracle-cache-restoration-and-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_i32_pair_mixed_logical.rs",
        required_patterns: &["return_i32_pair_oracle.rs", "use pair_oracle::oracle", "const SEEDS: [i64; 5]", "i32::MIN as i64", "i32::MAX as i64", "const STAGES: [i32; 9]", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-artifact-and-proxy-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_i32_pair_mixed_logical_tests.rs", "mod return_i32_pair_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_i32_pair_mixed_logical.rs", "mod return_i32_pair_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-full-predecessor-history-and-wide-transport-next",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["I32-pair mixed return bool-update follow-up:", "Previous nested-sequential-mixed checkpoint:", "Previous nested-sequential-mixed task checkpoint:", "Previous nested-sequential-mixed blocker checkpoint:", "Previous nested-sequential-mixed action checkpoint:", "Previous nested-sequential-mixed validation checkpoint:", "Previous nested-sequential-mixed artifact checkpoint:", "Prove full-width bool/i64 private continuation transport", "nuis-native-private-i32-pair-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-contract-and-drift-registration",
        path: "docs/reference/nuis-native-private-i32-pair-continuations-v1.md",
        required_patterns: &["## Exact Carrier", "0..=8589934591", "six nodes / fourteen expressions / zero logical edges", "## Original Authority", "full-width bool/i64", "formal memory safety", "strict Clippy", "dev_tensor_drift_data_native_return_i32_pair_mixed_logical.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-pair-shared-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_pair_oracle.rs",
        required_patterns: &["Independent imperative replay uses signed i32 values, never the compiler codec", "child = !child"],
    },
];
