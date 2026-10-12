use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-exact-original-owner-child-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["effectful_selections_return_i64_pair_tests.rs", "transport::numeric(ty)", "staged.defined.contains(*name)", "!staged.constants.contains(*name)", "!writes.remove(owner.name()) || writes.len() != 1", "entering.contains_key(&child)", "!staged.defined.contains(&child)", "staged.constants.contains(&child)", "Do not manufacture a bool Target token", "staged.defined.extend(next.defined)"],
    },
DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-independent-full-width-owned-fields-and-zero-seed",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["Representation follows independently proved field types", "fresh_name(type_prefix, names)", "fields: Vec<(String, String, NirTypeRef)>", "NirVisibility::Private", "SelectedValue::Region(_, value)", "record(false)", "record(true)", "install_with_neutral", "NirExpr::FieldAccess", "plan.destination = Some((carrier.clone(), false))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-shared-installer-and-complete-private-layout-publication",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["names.extend(module.structs.iter()", "names.extend(module.enums.iter()", "module.structs.extend(structs)", "fn install_with_neutral", "then_body: vec![NirStmt::Return(Some(neutral.clone()))]", "Descendant predicates are emitted only after this ancestor guard", "body.extend(regions::install(\n                    statements, names, bindings, helpers, structs,\n                ))", "function.return_type = Some(plan.ty.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-exact-family-retention-and-recursive-carrier-installation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["effectful_selections_transport_owned.rs", "owned_fields: Option<(NirTypeRef, NirTypeRef)>", "[\"i64\", \"f32\", \"f64\"]", "owner_type == &scalar_type(\"bool\")", "!arm.defined.contains(*n)", "captures.len() > 31", "Installed as a typed two-field result, never scalar packing", "plan, owner, child, fields, names, bindings, helpers, structs,"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-six-source-budget-scope-layout-and-linear-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_i64_pair_tests.rs",
        required_patterns: &["keep_both_orders_nested_sequences_retention_and_one_bool_export", "keep_full_width_fields_literal_zero_guards_and_collision_free_installation", "reject_incoming_third_writes_seals_hidden_calls_and_exits_atomically", "charge_original_nodes_expressions_and_edges_once", "keep_lexical_and_capture_limits_with_retained_versions", "install_linear_helpers_and_private_layouts_without_tail_copy", "(6, 12, true)", "for count in [31, 32]", "for count in [1, 4, 8]", "crate::nir_verify::verify_nir_module(&module)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-native-current-reads-signed-flips-retention-and-source-exit",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_pair_mixed_logical_effectful_scalar_selections.ns",
        required_patterns: &["seed: i64, first: bool, second: bool", "fn child_work(value: i64, proof: i64) -> i64", "let child = child_initial(seed, 1 / child_init)", "0 - child - 1", "if child < 0", "let local = after_work(child, seed, 1 / after)", "let ignored = unused_work(1 / unused)", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-artifact-owned-layout-no-narrowing-guard-effect-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_i64_pair_mixed_logical_tests.rs",
        required_patterns: &["native_return_i64_pair_mixed_logical_effectful_scalar_selections", "State{seed:i64;first:bool;second:bool", "call_owned_struct", "{owner:bool;child:i64}", "checked_functions >= 9", "guarded_calls >= 16", "function.body_nodes.contains(&edge.to)", "verify_nuis_compiled_artifact", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-independent-wide-oracle-cache-restoration-and-traps",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_i64_pair_mixed_logical.rs",
        required_patterns: &["return_i64_pair_oracle.rs", "use pair_oracle::oracle", "const SEEDS: [i64; 7]", "i64::MIN", "i64::MAX", "-4_294_967_297", "4_294_967_296", "const STAGES: [i32; 9]", "compile_cache: hit", "for _ in 0..3", "fs::remove_file(project.0.join(\"main.ns\"))", "fs::remove_file(project.0.join(\"nuis.toml\"))", "fs::remove_dir_all(project.0.join(\".nuis\"))", "rejected_before_open(run)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-artifact-proof-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_i64_pair_mixed_logical_tests.rs", "mod return_i64_pair_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_i64_pair_mixed_logical.rs", "mod return_i64_pair_mixed_logical;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-complete-predecessor-history-and-bit-preservation-next",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["I64-pair mixed return bool-update follow-up:", "Previous i32-pair checkpoint:", "Previous i32-pair task checkpoint:", "Previous i32-pair blocker checkpoint:", "Previous i32-pair action checkpoint:", "Previous i32-pair validation checkpoint:", "Previous i32-pair artifact checkpoint:", "Prove exact bool/f32 and bool/f64 private continuations", "nuis-native-private-i64-pair-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-bounded-owned-contract-and-drift-registration",
        path: "docs/reference/nuis-native-private-i64-pair-continuations-v1.md",
        required_patterns: &["## Exact Owned Carrier", "owner: bool", "child: i64", "six nodes / twelve expressions / zero logical edges", "31 captures plus a gate", "420 lifecycle transitions", "## Original Authority", "formal memory safety", "strict Clippy", "dev_tensor_drift_data_native_return_i64_pair_mixed_logical.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-pair-shared-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_pair_oracle.rs",
        required_patterns: &["Independent imperative replay uses signed i64 values, never the compiler codec", "child = !child"],
    },
];
