use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-dispatch",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["early_fields::preflight(body, &mut remaining, depth)?", "(Some(yes), Some(no))", "return paired::prove(", "effectful_selections_return_paired_in_fields.rs", "effectful_selections_return_paired_in_fields_tests.rs", "!continuing(&prefix[index + 1..])"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_paired_in_fields.rs",
        required_patterns: &["updates::private_field_certificate(", "&clean(&yes)", "&clean(&no)", "yes.before", "no.before", "yes.after, no.after", "fields.tail(", "staged.constants.insert(gate.clone())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-lazy-plans",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_paired_in_fields.rs",
        required_patterns: &["for (selected, arm) in [(false, no), (true, yes)]", "right: Box::new(predicate)", "rhs: Box::new(NirExpr::Bool(selected))", "reads.len() > 31", "std::iter::once(gate.clone())", "SelectedValue::StagedPredicate(tail.statements, value)", "SelectedValue::Nested(Box::new(plan))", "SelectedValue::StagedNested(staged.statements, plan)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_paired_in_fields_tests.rs",
        required_patterns: &["records.len(), 7", "continuations.len(), 1", "p.name == name", "f.params.len() <= 32", "\"sibling_exit_work\"", "one_direct_exit_per_arm_without_new_authority", "for removed in [yes, no]", "assert_eq!(module, before)", "assert_eq!(names, before_names)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_paired_in_fields_tests.rs",
        required_patterns: &["budget(39, 256, 32)", "budget(38, 256, 32)", "budget(39, used - 1, 32)", "budget(39, 256, 3)", "budget(39, used, 4)", "budget(40, 256, 32)", "exact.logical_edges, 28"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_paired_in_fields.ns",
        required_patterns: &["print(58); return ((small < i32_from_i64(0)) != flag)", "1 / sibling_exit", "return (e != true) && answer_work(local, seed, 1 / answer);", "let local = sibling_owner_work(local, a, b, c, d, e, 1);", "sibling_exit: sibling_exit", "let e = tail_fifth_work(e, b);"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_paired_in_fields_oracle.rs",
        required_patterns: &["STAGES: [i64; 24]", "68, 58, 96", "\"sibling_exit\"", "trace.push(58)", "exited = (a < 0) ^ d", "let enabled = if second { d } else { !e }", "[76, 58].contains(&event)", "event == 96 && !self.observe_exit", "if !exited {"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_paired_in_fields.rs",
        required_patterns: &["count(), 32", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "for second in [false, true]", "c.trace.contains(&76)", "c.trace.contains(&58)", "stale_small_condition", "assert!(!stale_small_condition)", "witness.exited && !witness.result", "missed_exit_result", "assert_ne!(missed_exit_result, witness.result)", "!witness.trace.contains(&67)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_paired_in_fields_native.rs",
        required_patterns: &["probe::check_with_observations(", "(\"sibling_exit_work\", FIELD_PROBE)", "(\"exit_work\", FIELD_PROBE)", "(\"after_work\", FIELD_PROBE)", "bitcast float %arg2 to i32", "oracle::native_selected(prefix)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-probe",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["for &(callee, observer) in observations", "observation.into_iter().collect", "pub(super) fn check_with_observations(", "check_with_observations(source, cases, failure, &observations, open_words)", "expected(case, failure, index)", "case.before_failure(index)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_paired_in_fields_tests.rs",
        required_patterns: &["continuations.len(), 1", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "owned_aggregate_drop_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_paired_in_fields.rs",
        required_patterns: &["oracle::selected().len(), 24", "for second in [false, true]", "c.words()[11]", "c.words()[19]", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "compile_cache: hit"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_paired_in_fields_tests.rs", "mod return_paired_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_paired_in_fields.rs", "mod return_paired_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_paired_in_fields.rs", "mod return_paired_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_paired_in_fields.rs", "paired_in_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Paired source-arm exits follow-up:", "Previous five-child common-tail checkpoint:", "Previous five-child common-tail task checkpoint:", "Previous five-child common-tail blocker checkpoint:", "Previous five-child common-tail action checkpoint:", "Previous five-child common-tail validation checkpoint:", "Previous five-child common-tail artifact checkpoint:", "nuis-native-paired-field-arm-exits-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-paired-in-fields-receipt",
        path: "docs/reference/nuis-native-paired-field-arm-exits-v1.md",
        required_patterns: &["458 distinct tests", "complete 451-test predecessor", "640 successful native processes", "192 fatal native processes", "192 CLI event traps", "1344 accepted callbacks", "active/99", "not simultaneous observations", "more than one exit per arm", "marker 67 is not an independent new trap", "healthy continuing proof operands"],
    },
];
