use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_i64_ancestor_tests.rs", "mod i64_ancestor_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-exact-integer-owner-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["pub(super) fn numeric_owner", "[\"i32\", \"i64\", \"f32\", \"f64\"]", "ty == &scalar_type(name)", "authorized.is_none_or(|target| target.name() == *name)", "!staged.constants.contains(*name)", "ty: scope.get(name)?.clone()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-private-bool-only-no-third-write",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["numeric_owner(staged.local.get(owner.name())?)", "staged.local.get(&child)? != &scalar_type(\"bool\")", "!writes.remove(owner.name()) || writes.len() != 1", "entering.contains_key(&child)", "if writes.len() > 2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-typed-transport-not-bit-packing",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["owned_fields: Option<(NirTypeRef, NirTypeRef)>", "[\"i64\", \"f32\", \"f64\"]", "owner_type == &scalar_type(name)", "child_type == &scalar_type(\"bool\")", "!(bool_owner || i32_owner || wide_owner || numeric_pair)", "(owner_type.clone(), child_type.clone())", "captures.len() > 31", "owned_pair::install"],
    },
DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-exact-fields-neutral-and-projections",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["for (field, name, ty) in fields", "fields: Vec<(String, String, NirTypeRef)>", "\"bool\" => NirExpr::Bool(false)", "\"i64\" => NirExpr::Int(0)", "record(true)", "NirVisibility::Private", "NirExpr::FieldAccess", "record(false)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-linear-forest-shared-original-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["!numeric_owner(scope.get(owner.name())?)", "tree::preflight(yes, no, suffix, depth)?", "ty: scope.get(owner.name())?.clone()", "No arm receives a copy of the remaining sequence"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-four-unit-proof-families",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_i64_ancestor_tests.rs",
        required_patterns: &["accepts_orders_retention_nested_sequences_and_one_typed_target", "rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically", "owned_fields_neutral_guards_and_collision_free_projections", "shared_original_budget_and_capture_boundary", "(7, 7, true)", "for count in [31, 32]", "for depth in [7, 8]", "assert_eq!(module, before)", "assert_eq!(names, before_names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-nuis-source-current-owner-and-private-child",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_ancestor_private_bool.ns",
        required_patterns: &["fn initialize(value: i64, proof: i64) -> i64", "value + (proof - 1)", "fn parent_work(value: i64", "fn child_work(value: bool", "let child = child_initial(local < 0, 1 / child_init)", "const seal: bool = child || false", "return answer_work(local, seed, 1 / answer) && first", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-independent-oracle-wide-extremes",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_ancestor_oracle.rs",
        required_patterns: &["Imperative source replay never uses the compiler", "pub seed: i64", "local = !local", "i64::MIN, -4_294_967_297, -1, 0, 1, 4_294_967_296, i64::MAX", "Self::new(self.seed, !self.first, !self.second)", "values[stage + 2] = \"0\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-native-callee-argument-and-reference-publication",
        path: "tools/nuisc/tests/native_application_bridge/return_i64_ancestor.rs",
        required_patterns: &["call void @nuis_debug_print_i64(i64 %arg0)", "path = \"return_i32_ancestor_probe.rs\"", "for reversed in [false, true]", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-shared-native-probe-traps-canaries-no-allocation",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["llvm.insert_str(insertion, OWNER_PROBE)", "Source computations, private joins and real traps remain intact", "Some(4 | 5)", "[(0, 91), (state_width + 1, 92)]", "load volatile i64, ptr %prev", "Full trace equality distinguishes AFTER from matching field words", "nuis_scheduler_owned_aggregate_alloc_v1", "dynamic_loop_guard::run_command_bounded"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-artifact-owned-layout-effect-and-policy-veto",
        path: "tools/nuisc/src/aot_application_effect_call_return_i64_ancestor_tests.rs",
        required_patterns: &["result.ty == \"i64\"", "{owner:i64;child:bool}", "call_owned_struct", "cast_i64_to_i32", "yir_core::EdgeKind::Effect", "function.body_nodes.contains(&edge.to)", "lacks dependency order", "Vec::<String>::new()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-cli-four-images-cache-source-free-failure-publication",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_i64_ancestor.rs",
        required_patterns: &["compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_i64_ancestor_tests.rs", "mod return_i64_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_i64_ancestor.rs", "mod return_i64_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_i64_ancestor.rs", "mod return_i64_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-predecessor-and-next-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["I64-ancestor private bool follow-up:", "Previous i32-ancestor checkpoint:", "Previous i32-ancestor task checkpoint:", "Previous i32-ancestor blocker checkpoint:", "Previous i32-ancestor action checkpoint:", "Previous i32-ancestor validation checkpoint:", "Previous i32-ancestor artifact checkpoint:", "nuis-native-private-i64-ancestor-continuations-v1.md", "Extend terminal mixed private continuations to one exact f32 or f64 outward ancestor beside a private bool child"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i64-ancestor-bounded-contract-receipt",
        path: "docs/reference/nuis-native-private-i64-ancestor-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance", "owner: i64", "child: bool", "seven nodes / seven expressions / zero logical edges", "56 successful native processes", "18 fatal native processes", "36 CLI event traps", "bool-returning", "formal memory safety"],
    },
];
