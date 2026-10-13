use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-early-fields-proof-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_early_fields.rs",
        required_patterns: &["depth >= 8 || body.len() > 16", "remaining.checked_sub(body.len())?", "returned(body)?", "[NirStmt::Return(Some(value))]", "!else_body.is_empty()", "owner.as_deref()?", "if owner.is_some()", "SelectedValue::StagedNested(staged.statements, Box::new(plan))", "continuation is installed once", "Numeric field records never carry"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["fn private_field_stage(", "!entering.contains_key(name) && numeric_owner(&staged.local[name])", "yes_target.as_ref().or(no_target.as_ref())?", "mixed::classify_fields(yes, no, entering, staged, &owner, depth)??", "fn continue_owner(", "!numeric_owner(scope.get(owner)?)", "Some(&Target(owner))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-field-hook",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["pub(in super::super::super::super) fn stage_selection(", "transport::prepare_fields(condition, yes, no, scope, owner.name(), &children.0)?", "staged.defined.insert(owner.name().into());", "staged.defined.extend(children.0.iter().cloned());"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["effectful_selections_return_early_fields.rs", "mod early_fields;", "if early_fields::candidate(statements)", "return early_fields::prove(statements, scope, signatures, budget, depth);"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-unit-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_early_fields_tests.rs",
        required_patterns: &["return_early_fields_retains_current_fields_and_one_guarded_continuation", "continuation.len(), 1", "[\"i64\", \"i32\", \"f32\", \"f64\", \"bool\", \"bool\"]", "return_early_fields_keeps_atomic_authority_type_and_shape_vetoes", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "return local;", "const a: i32 = small_seed;", "let small_seed = small_work"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-unit-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_early_fields_tests.rs",
        required_patterns: &["return_early_fields_charges_original_work_once_without_budget_or_depth_reset", "budget(18, 256, 32)", "assert_eq!(exact.nodes, 0)", "assert_eq!(exact.logical_edges, 29)", "budget(17, 256, 32)", "budget(18, used - 1, 32)", "budget(18, 256, 2)", "&mut budget(18, used, 3), 6", "&mut budget(18, 256, 32), 7"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_early_fields.ns",
        required_patterns: &["narrow_seed: f32, wide_seed: f64", "let c = wide_work(c, b, d, 1 / wide_rhs);", "let e = e && peer_work(d, 1 / peer_rhs);", "if exit_work(local, a, b, c, d, e, 1 / exit_guard) { return d && answer_work(local, seed, 1 / answer); }", "let ignored = after_work(local, a, b, c, d, e, 1 / after);", "return answer_work(local, seed, 1 / answer) || e;", "state.seed, state.small_seed, state.narrow_seed, state.wide_seed, state.bool_seed, state.peer_seed"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_early_fields_oracle.rs",
        required_patterns: &["pub const STAGES: [i64; 12]", "a = !a", "b ^= 1 << 31", "c ^= i64::MIN", "exited = d != e", "trace.extend([96, 95, 97])", "if event == 76", "self.trace.iter().position", "if d {", "pub fn selected()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_early_fields.rs",
        required_patterns: &["return_early_fields_reference_values_order_and_last_publication", "return_early_fields_reference_poisoned_dead_tail_and_retained_fields", "count(), 20", "assert_eq!(session.state(), &last)", "ApplicationSessionPhase::Faulted", "oracle::cases().len(), 80", "silent.words()[13], 0"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_early_fields_native.rs",
        required_patterns: &["bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "zext i1 %arg4 to i64", "zext i1 %arg5 to i64", "return_early_fields_native_exact_bits_and_order", "return_early_fields_native_real_event_traps", "Some((\"exit_work\", FIELD_PROBE))", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_early_fields_tests.rs",
        required_patterns: &["native_return_early_fields_keeps_typed_fields_guarded_continuation_and_policy_vetoes", "continuations.len(), 1", "\"{owner:i64;child_0:i32;child_1:f32;child_2:f64;child_3:bool;child_4:bool}\"", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_early_fields.rs",
        required_patterns: &["return_early_fields_cli_dead_tail_and_short_circuit_proofs_stay_poisoned", "oracle::cli_cases().len(), 16", "oracle::selected().len(), 12", "before_failure(11)", "usize::from(opened.trace.contains(&97))", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_early_fields.rs", "mod return_early_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_early_fields_tests.rs", "mod return_early_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_early_fields.rs", "mod return_early_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_early_fields.rs", "early_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Early mixed-field exit follow-up:", "Previous simultaneous float field-vector checkpoint:", "Previous simultaneous float field-vector task checkpoint:", "Previous simultaneous float field-vector blocker checkpoint:", "Previous simultaneous float field-vector action checkpoint:", "Previous simultaneous float field-vector validation checkpoint:", "Previous simultaneous float field-vector artifact checkpoint:", "nuis-native-early-mixed-field-exits-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-early-fields-receipt",
        path: "docs/reference/nuis-native-early-mixed-field-exits-v1.md",
        required_patterns: &["396 distinct tests", "complete 386-test predecessor", "160 successful native processes", "24 fatal native processes", "48 CLI event traps", "432 accepted callbacks", "active/99", "before a field join remain separate"],
    },
];
