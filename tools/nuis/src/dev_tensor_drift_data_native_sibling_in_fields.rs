use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["early_fields::preflight(body, &mut remaining, depth)?", ".position(|s| !continuing(std::slice::from_ref(s)))?", "else_body.is_empty() && continuing(prefix) && continuing(before) && continuing(after)", "continuing(then_body) && continuing(else_body)", "_ => false", "(Some(split), None) if continuing(else_body)", "(None, Some(split)) if continuing(then_body)", "continuing(&prefix[index + 1..])"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-transport",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["fn stage_split(", "isolated(yes, no, future, staged, owner, children, depth)?", "nested::consume_node(budget)?", "predicates::prove(condition, &staged.local, signatures, budget)?", "Walk the original forest once; never copy a continuation into both arms.", "staged.defined.extend(children.0.iter().cloned())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-unit-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_sibling_in_fields_tests.rs",
        required_patterns: &["records.len(), 5", "continuations.len(), 1", "\"nested_work\"", "[\"exit\", \"result\", \"returned\"]", "__nuis_effect_field_exit_gate_1", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "callee == \"prefix_work\"", "callee == \"sibling_work\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-unit-veto",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_sibling_in_fields_tests.rs",
        required_patterns: &["return_sibling_in_fields_rejects_sibling_exits_leaks_and_uncertified_writes_atomically", "\"if first { return e; }\"", "\"const c: f64 = wide_seed;\"", "\"let c: f32 = b;\"", "sibling_owner_work(local, a, b, leaked, d, e", "let seed = initialize(seed", "if d { let b = narrow_work", "return d; } if exit_work", "let c = sibling_wide_work", "let a = sibling_small_work", "let local = sibling_owner_work", "let a = sibling_small_work(a, 1 / sibling_small);"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-unit-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_sibling_in_fields_tests.rs",
        required_patterns: &["return_sibling_in_fields_charges_original_work_once_without_budget_or_depth_reset", "assert_eq!(exact.nodes, 0)", "assert_eq!(exact.logical_edges, 29)", "budget(27, 256, 32)", "budget(28, used - 1, 32)", "budget(28, 256, 2)", "&mut budget(28, used, 3), 5", "&mut budget(28, 256, 32), 6", "budget(29, 256, 32)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_sibling_in_fields.ns",
        required_patterns: &["if exit_work(local, a, b, c, d, e, 1 / exit_guard) { return d && answer_work", "if nested_work(d, 1 / nested)", "let c = wide_work(c, b, d, 1 / wide_rhs)", "let e = e && peer_work(d, 1 / peer_rhs)", "let e = peer_work(d != true, 1 / peer_rhs)", "state.seed, state.small_seed, state.narrow_seed, state.wide_seed, state.bool_seed, state.peer_seed", "nested: nested", "if prefix_work(d, 1 / prefix)", "let c = prefix_wide_work(c, b, d, 1 / prefix_wide)", "let e = prefix_peer_work(e, 1 / prefix_peer)", "prefix: prefix, prefix_wide: prefix_wide, prefix_peer: prefix_peer", "if sibling_work(e, 1 / sibling)", "let a = sibling_small_work(a, 1 / sibling_small)", "let c = sibling_wide_work(c, b, d, 1 / sibling_wide)", "let local = sibling_owner_work(local, a, b, c, d, e, 1 / sibling_owner)", "sibling_owner: sibling_owner"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_sibling_in_fields_oracle.rs",
        required_patterns: &["STAGES: [i64; 20]", "trace.push(75)", "exited = d != e", "if !exited", "e = !d", "c ^= i64::MIN", "event == 76 && self.observe_exit", "event == 96 && !self.observe_exit", "cases[18] = marker.clone()", "cases[19] = marker", "previous.observe_exit = self.observe_exit", "trace.push(73)", "trace.push(72)", "trace.push(77)", "e = !e", "trace.push(69)", "trace.extend([70, 81])", "trace.push(68)", "trace.push(67)", "no_start", "if condition(second != true, 1 / cond)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_sibling_in_fields.rs",
        required_patterns: &["return_sibling_in_fields_reference_values_order_and_last_publication", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "[5, 7, 8, 10, 14, 15, 16, 17, 18]", "continuing.len(), 2", "assert_eq!(after_wide, before_wide)", "assert_eq!(after_wide, before_wide ^ i64::MIN)", "assert_eq!(after_peer, 1)", "marker.before_failure(18).contains(&96)", "assert_ne!(current.prefix_fields.unwrap().3, current.wide_seed)", "assert_eq!(c.exited, c.peer_seed)", "i64::from(!c.peer_seed)", "siblings.len(), 4", "flag != peer", "gated.words()[11], 0", "assert!(!gated.trace.contains(&76))", "assert_eq!(wide, c.wide_seed)", "assert_eq!(flag, i64::from(!c.bool_seed))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_sibling_in_fields_native.rs",
        required_patterns: &["return_i32_ancestor_probe.rs", "bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "oracle::mirror_source()", "for prefix in [false, true]", "oracle::native_cases(prefix)", "oracle::native_selected(prefix)", "\"exit_work\"", "\"after_work\"", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_sibling_in_fields_tests.rs",
        required_patterns: &["oracle::mirror_source()", "{owner:i64;child_0:i32;child_1:f32;child_2:f64;child_3:bool;child_4:bool}", "continuations.len(), 1", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_sibling_in_fields.rs",
        required_patterns: &["return_sibling_in_fields_cli_dead_suffix_and_source_arm_gate_proofs_stay_zero", "oracle::selected().len(), 20", "project_for_source(&source)", "oracle::mirror_source()", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)", "compile_cache: hit", "assert!(!stderr.contains(\"native_session_completed=1\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["effectful_selections_return_sibling_in_fields_tests.rs", "mod sibling_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_sibling_in_fields.rs", "mod return_sibling_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_sibling_in_fields_tests.rs", "mod return_sibling_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_sibling_in_fields.rs", "mod return_sibling_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_sibling_in_fields.rs", "sibling_in_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Nested sibling field selection follow-up:", "Previous nested pre-exit selection checkpoint:", "Previous nested pre-exit selection task checkpoint:", "Previous nested pre-exit selection blocker checkpoint:", "Previous nested pre-exit selection action checkpoint:", "Previous nested pre-exit selection validation checkpoint:", "Previous nested pre-exit selection artifact checkpoint:", "nuis-native-nested-sibling-field-selections-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sibling-in-fields-receipt",
        path: "docs/reference/nuis-native-nested-sibling-field-selections-v1.md",
        required_patterns: &["414 distinct tests", "complete 409-test predecessor", "640 successful native processes", "160 fatal native processes", "160 CLI event traps", "1184 accepted callbacks", "active/99", "not simultaneous observations", "multiple exits remain separate", "direct predecessor"],
    },
];
