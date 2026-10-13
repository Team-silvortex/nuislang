use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-boundary",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["!flat(&prefix[..index]) || !continuing(&prefix[index + 1..])", "fields.tail(", "early_fields::preflight(body, &mut remaining, depth)?", "SelectedValue::StagedPredicate(tail.statements, value)", "effectful_selections_return_tail_in_fields_tests.rs"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-writes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_owner_tail.rs",
        required_patterns: &["fn writes(", "local.contains(name)", "name != owner || !matches!(statement, NirStmt::Let { .. })", "writes(then_body, &local, owner, remaining, depth + 1)?", "writes(else_body, &local, owner, remaining, depth + 1)?", "_ => return None", "staged.constants.contains(owner)", "Select this exact owner", "regions::Statement::Selection(plan)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-module",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_owner_tail.rs", "pub(super) use owner_tail::append as append_owner_tail;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_tail_in_fields_tests.rs",
        required_patterns: &["records.len(), 5", "continuations.len(), 1", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "return_tail_in_fields_confines_nested_writes_to_exact_unsealed_owner", "let d = true;", "let seed = tail_yes_work", "const local: i64", "budget(31, 256, 32)", "budget(32, used - 1, 32)", "budget(32, 256, 2)", "unknown value `private`"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_tail_in_fields.ns",
        required_patterns: &["if tail_work(local, a, b, c, d, e, 1 / tail_cond)", "if d { let local = tail_yes_work(local, 1 / tail_yes); }", "let local = tail_no_work(local, a, b, c, d, e, 1 / tail_no);", "if exit_work(local, a, b, c, d, e, 1 / exit_guard)", "tail_cond: tail_cond", "state.seed, state.small_seed"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_tail_in_fields_oracle.rs",
        required_patterns: &["STAGES: [i64; 23]", "FIELDS: [&str; 25]", "trace.extend([96, 66])", "let selected = parent(0, a, b, c, d, e) != 0", "trace.push(65)", "trace.push(64)", "event == 66 && !self.observe_exit", "cases[18] = marker.clone()", "if condition(second != true, 1 / cond)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_tail_in_fields.rs",
        required_patterns: &["count(), 31", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "return_tail_in_fields_reference_readonly_fields_retention_and_dead_choices", "[20, 21, 22]", "parity && flag == 0", "assert!(!c.trace.contains(&66))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_tail_in_fields_native.rs",
        required_patterns: &["return_i32_ancestor_probe.rs", "bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "oracle::mirror_source()", "for prefix in [false, true]", "\"exit_work\"", "\"tail_work\"", "oracle::native_selected(prefix)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_tail_in_fields_tests.rs",
        required_patterns: &["oracle::mirror_source()", "continuations.len(), 1", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_tail_in_fields.rs",
        required_patterns: &["oracle::selected().len(), 23", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "compile_cache: hit", "assert!(!stderr.contains(\"native_session_completed=1\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_tail_in_fields_tests.rs", "mod return_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_tail_in_fields.rs", "mod return_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_tail_in_fields.rs", "mod return_tail_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_tail_in_fields.rs", "tail_in_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Nested owner-only common-tail follow-up:", "Previous nested sibling selection checkpoint:", "Previous nested sibling selection task checkpoint:", "Previous nested sibling selection blocker checkpoint:", "Previous nested sibling selection action checkpoint:", "Previous nested sibling selection validation checkpoint:", "Previous nested sibling selection artifact checkpoint:", "nuis-native-nested-owner-tail-selections-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-tail-in-fields-receipt",
        path: "docs/reference/nuis-native-nested-owner-tail-selections-v1.md",
        required_patterns: &["420 distinct tests", "complete 414-test predecessor", "640 successful native processes", "184 fatal native processes", "184 CLI event traps", "1304 accepted callbacks", "active/99", "not simultaneous observations", "multiple exits remain separate"],
    },
];
