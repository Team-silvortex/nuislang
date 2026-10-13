use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["early_fields::preflight(body, &mut remaining, depth)?", "fields.tail(", "SelectedValue::StagedPredicate(tail.statements, value)", "effectful_selections_return_quad_tail_in_fields_tests.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["fn tail_children(", "Option<BTreeSet<String>>", "written.remove(owner.name())", "written.len() > 4", "!self.0.contains(name)", "Some(written)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-dispatch",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["tail_children(statements, &[], &staged.local, &owner, depth)?", "!selected.is_subset(&children)", "!selected.is_empty()", "&statements[index + 1..]", "mixed::stage_fields(", "append_owner_tail("],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_quad_tail_in_fields_tests.rs",
        required_patterns: &["records.len(), 7", "continuations.len(), 1", "certificate_and_four_children", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "budget(35, 256, 32)", "budget(36, used - 1, 32)", "budget(36, 256, 2)", "budget(37, 256, 32)", "\"tail_pair_work\"", "let e = false; } let ignored = after_work", "\"tail_third_work\"", "let separate", "\"tail_fourth_work\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_quad_tail_in_fields.ns",
        required_patterns: &["let c = tail_child_work(c, 1 / tail_child);", "let d = tail_pair_work(d, c, 1 / tail_pair);", "let local = tail_yes_work(local, 1);", "print(62); return value != (wide < 0.0)", "tail_pair: tail_pair", "if exit_work(local, a, b, c, d, e, 1 / exit_guard)", "let a = tail_third_work(a, d);", "print(61);", "if flag { return value;", "let b = tail_fourth_work(b, a);", "print(60);", "if small < i32_from_i64(0)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_quad_tail_in_fields_oracle.rs",
        required_patterns: &["STAGES: [i64; 24]", "FIELDS: [&str; 26]", "\"tail_pair\"", "trace.extend([63, 62, 61, 60, 65])", "d ^= f64::from_bits(c as u64) < 0.0", "trace.extend([96, 95, 97])", "event == 96 && !self.observe_exit", "if !d {", "a = !a;", "if a >= 0 {", "b ^= 1 << 31;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_quad_tail_in_fields.rs",
        required_patterns: &["count(), 32", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "current_four_and_stale_dependencies", "assert_eq!(after.4, 0)", "assert_eq!(witness.owner, !witness.seed)", "assert!(!witness.result)", "let stale_second", "assert!(stale_second)", "assert_eq!(after.5, 0)", "expected_flag", "let stale_fourth_parity", "assert!(!stale_fourth_parity)", "after.1, i64::from(i32::MAX)", "let stale_third_a", "assert_ne!(stale_third_a, after.1)", "after.2, i64::from(2.5f32.to_bits())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_quad_tail_in_fields_native.rs",
        required_patterns: &["return_i32_ancestor_probe.rs", "bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "oracle::mirror_source()", "for prefix in [false, true]", "\"exit_work\"", "\"after_work\"", "oracle::native_selected(prefix)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_quad_tail_in_fields_tests.rs",
        required_patterns: &["oracle::mirror_source()", "continuations.len(), 1", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_quad_tail_in_fields.rs",
        required_patterns: &["oracle::selected().len(), 24", "witness.before_tail.unwrap().4, 1", "witness.fields.unwrap().4, 0", "assert!(!witness.result)", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "compile_cache: hit", "witness.trace.contains(&60)", "assert_ne!(witness.before_tail.unwrap().2, witness.fields.unwrap().2)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_quad_tail_in_fields_tests.rs", "mod return_quad_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_quad_tail_in_fields.rs", "mod return_quad_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_quad_tail_in_fields.rs", "mod return_quad_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_quad_tail_in_fields.rs", "quad_tail_in_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Certified four-child common-tail follow-up:", "Previous three-child common-tail checkpoint:", "Previous three-child common-tail task checkpoint:", "Previous three-child common-tail blocker checkpoint:", "Previous three-child common-tail action checkpoint:", "Previous three-child common-tail validation checkpoint:", "Previous three-child common-tail artifact checkpoint:", "nuis-native-certified-quad-tail-selections-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-quad-tail-in-fields-receipt",
        path: "docs/reference/nuis-native-certified-quad-tail-selections-v1.md",
        required_patterns: &["444 distinct tests", "complete 438-test predecessor", "640 successful native processes", "192 fatal native processes", "192 CLI event traps", "1344 accepted callbacks", "active/99", "not simultaneous observations", "multiple exits remain separate", "markers 60, 61 and 65 are not independent traps"],
    },
];
