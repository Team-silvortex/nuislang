use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["early_fields::preflight(body, &mut remaining, depth)?", "fields.tail(", "SelectedValue::StagedPredicate(tail.statements, value)", "effectful_selections_return_quint_tail_in_fields_tests.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["fn tail_children(", "Option<BTreeSet<String>>", "written.remove(owner.name())", "written.len() > 5", "!self.0.contains(name)", "Some(written)", "effectful_selections_return_tail_child_bound_tests.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-dispatch",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["tail_children(statements, &[], &staged.local, &owner, depth)?", "!selected.is_subset(&children)", "!selected.is_empty()", "&statements[index + 1..]", "mixed::stage_fields(", "append_owner_tail("],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_quint_tail_in_fields_tests.rs",
        required_patterns: &["records.len(), 7", "continuations.len(), 1", "certificate_and_five_children", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "budget(36, 256, 32)", "budget(37, used - 1, 32)", "budget(37, 256, 2)", "budget(38, 256, 32)", "\"tail_pair_work\"", "let e = false; let seed = local; } let ignored = after_work", "\"tail_third_work\"", "let separate", "\"tail_fourth_work\"", "\"tail_fifth_work\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_quint_tail_in_fields.ns",
        required_patterns: &["let c = tail_child_work(c, 1 / tail_child);", "let d = tail_pair_work(d, c, 1 / tail_pair);", "let local = tail_yes_work(local, 1);", "print(62); return value != (wide < 0.0)", "tail_pair: tail_pair", "if exit_work(local, a, b, c, d, e, 1 / exit_guard)", "let a = tail_third_work(a, d);", "print(61);", "if flag { return value;", "let b = tail_fourth_work(b, a);", "print(60);", "if small < i32_from_i64(0)", "let e = tail_fifth_work(e, b);", "print(59);", "if narrow < 0.0", "return answer_work(local, seed, 1 / answer) && e;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_quint_tail_in_fields_oracle.rs",
        required_patterns: &["STAGES: [i64; 24]", "FIELDS: [&str; 26]", "\"tail_pair\"", "trace.extend([63, 62, 61, 60, 59, 65])", "d ^= f64::from_bits(c as u64) < 0.0", "trace.extend([96, 95, 97])", "event == 96 && !self.observe_exit", "if !d {", "a = !a;", "if a >= 0 {", "b ^= 1 << 31;", "e ^= !(f32::from_bits(b as u32) < 0.0)", "result = owner == seed && e;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_quint_tail_in_fields.rs",
        required_patterns: &["count(), 32", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "current_five_and_stale_dependencies", "(before.4, after.4), (1, 0)", "assert_eq!(witness.owner, witness.seed)", "assert!(witness.result)", "let stale_second", "assert!(stale_second)", "(before.5, after.5), (0, 1)", "expected_flag", "let stale_fifth_parity", "assert!(stale_fifth_parity)", "i64::from(i32::MAX)", "let stale_third_a", "assert_ne!(stale_third_a, after.1)", "after.2, i64::from(2.5f32.to_bits())", "let stale_fifth_peer", "assert!(!stale_fifth_peer)", "let stale_fourth_b", "assert_ne!(stale_fourth_b, after.2)", "expected_peer"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_quint_tail_in_fields_native.rs",
        required_patterns: &["return_i32_ancestor_probe.rs", "bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "oracle::mirror_source()", "for prefix in [false, true]", "\"exit_work\"", "\"after_work\"", "oracle::native_selected(prefix)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_quint_tail_in_fields_tests.rs",
        required_patterns: &["oracle::mirror_source()", "continuations.len(), 1", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_quint_tail_in_fields.rs",
        required_patterns: &["oracle::selected().len(), 24", "witness.before_tail.unwrap().4, 1", "witness.fields.unwrap().4, 0", "assert!(witness.result)", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "compile_cache: hit", "witness.trace.contains(&60)", "assert_ne!(witness.before_tail.unwrap().2, witness.fields.unwrap().2)", "witness.trace.contains(&59)", "witness.fields.unwrap().5, 1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_quint_tail_in_fields_tests.rs", "mod return_quint_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_quint_tail_in_fields.rs", "mod return_quint_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_quint_tail_in_fields.rs", "mod return_quint_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_quint_tail_in_fields.rs", "quint_tail_in_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Certified five-child common-tail follow-up:", "Previous four-child common-tail checkpoint:", "Previous four-child common-tail task checkpoint:", "Previous four-child common-tail blocker checkpoint:", "Previous four-child common-tail action checkpoint:", "Previous four-child common-tail validation checkpoint:", "Previous four-child common-tail artifact checkpoint:", "nuis-native-certified-quint-tail-selections-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-receipt",
        path: "docs/reference/nuis-native-certified-quint-tail-selections-v1.md",
        required_patterns: &["451 distinct tests", "complete 444-test predecessor", "640 successful native processes", "192 fatal native processes", "192 CLI event traps", "1344 accepted callbacks", "active/99", "not simultaneous observations", "multiple exits remain separate", "markers 59, 60, 61 and 65 are not independent traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quint-tail-in-fields-certified-capacity",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_tail_child_bound_tests.rs",
        required_patterns: &["return_quint_tail_in_fields_bounds_six_authorized_children_across_the_whole_tail", "let children = classify(", "let five =", "tail_children(&original,", "tail_children(&across,", "tail_children(&including_owner,", "tail_children(&root,", "assert_eq!(five, names[..5]", "(0..6)"],
    },
];
