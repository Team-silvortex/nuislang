use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["early_fields::preflight(body, &mut remaining, depth)?", "fields.tail(", "&staged.local", "SelectedValue::StagedPredicate(tail.statements, value)", "effectful_selections_return_child_tail_in_fields_tests.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["fn tail_child(", "let mut written = writes(yes, no, scope, depth)?", "written.remove(owner.name())", "written.len() > 1", "!self.0.contains(name)", "Some(written.pop_first())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-dispatch",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["pub(super) fn tail(", "tail_child(statements, &[], &staged.local, &owner, depth)?", "child.as_ref() != Some(&selected)", "&statements[index + 1..]", "&self.children", "mixed::stage_fields(", "append_owner_tail("],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-proof",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_child_tail_in_fields_tests.rs",
        required_patterns: &["records.len(), 7", "continuations.len(), 1", "return_child_tail_in_fields_confines_nested_writes_to_certificate_and_one_child", "\"let a = small_work(a, 1 / small_rhs);\"", "\"let b = narrow_work(b, 1 / narrow_rhs);\"", "\"let d = bool_work(d, 1 / bool_rhs);\"", "\"let e = peer_work(d, 1 / peer_rhs);\"", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "budget(32, 256, 32)", "budget(33, used - 1, 32)", "budget(33, 256, 2)", "budget(34, 256, 32)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_child_tail_in_fields.ns",
        required_patterns: &["let c = tail_child_work(c, 1 / tail_child);", "tail_child_work(value: f64", "print(63); return -value", "if tail_work(local, a, b, c, d, e, 1 / tail_cond)", "let ignored = after_work(local, a, b, c, d, e, 1 / after)", "tail_child: tail_child"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_child_tail_in_fields_oracle.rs",
        required_patterns: &["STAGES: [i64; 24]", "FIELDS: [&str; 26]", "pub before_tail", "trace.extend([63, 65])", "c ^= i64::MIN", "trace.extend([96, 95, 97])", "event == 96 && !self.observe_exit", "let tail = \"\\n            }\\n            if tail_work\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_child_tail_in_fields.rs",
        required_patterns: &["count(), 32", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "return_child_tail_in_fields_reference_current_child_and_stale_counterexample", "assert_ne!(before.3, after.3)", "assert_eq!(after.5, 0)", "let stale_owner = !after.0", "after_wide, wide ^ if changed { i64::MIN }"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_child_tail_in_fields_native.rs",
        required_patterns: &["return_i32_ancestor_probe.rs", "bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "oracle::mirror_source()", "for prefix in [false, true]", "\"exit_work\"", "\"after_work\"", "oracle::native_selected(prefix)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_child_tail_in_fields_tests.rs",
        required_patterns: &["oracle::mirror_source()", "continuations.len(), 1", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_child_tail_in_fields.rs",
        required_patterns: &["oracle::selected().len(), 24", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "compile_cache: hit", "assert!(!stderr.contains(\"native_session_completed=1\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_child_tail_in_fields_tests.rs", "mod return_child_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_child_tail_in_fields.rs", "mod return_child_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_child_tail_in_fields.rs", "mod return_child_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_child_tail_in_fields.rs", "child_tail_in_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Certified common-tail child follow-up:", "Previous owner-only common-tail checkpoint:", "Previous owner-only common-tail task checkpoint:", "Previous owner-only common-tail blocker checkpoint:", "Previous owner-only common-tail action checkpoint:", "Previous owner-only common-tail validation checkpoint:", "Previous owner-only common-tail artifact checkpoint:", "nuis-native-certified-child-tail-selections-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-tail-in-fields-receipt",
        path: "docs/reference/nuis-native-certified-child-tail-selections-v1.md",
        required_patterns: &["426 distinct tests", "complete 420-test predecessor", "640 successful native processes", "192 fatal native processes", "192 CLI event traps", "1344 accepted callbacks", "active/99", "not simultaneous observations", "multiple exits remain separate"],
    },
];
