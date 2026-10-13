use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-in-fields-shape",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["early_fields::preflight(body, &mut remaining, depth)?", "(Some(split), None) if continuing(else_body)", "(None, Some(split)) if continuing(then_body)", "[NirStmt::Return(Some(value))]", "else_body.is_empty() && continuing(before) && continuing(after)", "continuing(&prefix[index + 1..])"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-control",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["private_field_certificate(yes, no, scope, &staged, depth)?", "__nuis_effect_field_exit_gate", "branches::collect_bindings(body, &mut bindings)", "source_condition", "staged.constants.insert(gate.clone())", "This generated edge is transport, not another original logical operator", "selected: true", "exit_reads.len() > 31"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-continuation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs",
        required_patterns: &["(split.before, other)", "(other, split.before)", "(split.after, &[][..])", "(&[][..], split.after)", "fields.tail(", "SelectedValue::StagedPredicate(tail.statements, value)", "SelectedValue::StagedNested(staged.statements, Box::new(plan))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["struct PrivateFields", "fn private_field_certificate(", "!entering.contains_key(name) && numeric_owner(&staged.local[name])", "mixed::classify_fields(yes, no, entering, staged, &owner, depth)??", "fn append_owner(", "Some(&Target(owner))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-split",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["fn stage_split(", "staged.local.get(gate)? != &scalar_type(\"bool\")", "The source selection/condition was charged and bound before splitting", "fn prepared(", "transport::prepare_fields(condition, yes, no, scope, owner.name(), &children.0)?", "staged.defined.extend(children.0.iter().cloned())"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["effectful_selections_return_in_fields.rs", "mod in_fields;", "if in_fields::candidate(statements)", "return in_fields::prove(statements, scope, signatures, budget, depth);"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields_tests.rs",
        required_patterns: &["return_in_fields_retains_prefix_then_guarded_suffix_without_control_in_records", "oracle::mirror_source()", "records.len(), 2", "continuations.len(), 1", "[\"exit\", \"result\", \"returned\"]", "__nuis_effect_field_exit_gate_1", "return_in_fields_keeps_atomic_authority_type_and_shape_vetoes", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "return_in_fields_charges_original_work_once_without_budget_or_depth_reset", "assert_eq!(exact.nodes, 0)", "assert_eq!(exact.logical_edges, 29)", "budget(17, 256, 32)", "budget(18, used - 1, 32)", "budget(18, 256, 2)", "&mut budget(18, used, 3), 5", "&mut budget(18, 256, 32), 6"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-source",
        path: "tools/nuisc/tests/native_application_bridge/return_in_fields.ns",
        required_patterns: &["let d = bool_work(d, 1 / bool_rhs);\n                if exit_work", "return d && answer_work(local, seed, 1 / answer);", "let c = wide_work(c, b, d, 1 / wide_rhs);", "let e = e && peer_work(d, 1 / peer_rhs);", "after_work(local, a, b, c, d, e, 1 / after)", "state.seed, state.small_seed, state.narrow_seed, state.wide_seed, state.bool_seed, state.peer_seed"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_in_fields_oracle.rs",
        required_patterns: &["trace.extend([94, 93, 81])", "prefix_fields = Some(", "exited = d != e", "if !exited", "c ^= i64::MIN", "event == 76 && self.observe_exit", "event == 96 && !self.observe_exit", "previous.observe_exit = self.observe_exit", "pub fn mirror_source()", "if condition(second != true, 1 / cond)", "cases[10] = marker.clone()", "cases[11] = marker"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_in_fields.rs",
        required_patterns: &["return_in_fields_reference_values_order_and_last_publication", "oracle::mirror_source()", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "return_in_fields_reference_prefix_and_joined_observations_and_dead_work", "[5, 7, 8, 10]", "retained.words()[11], 0", "Some((70001, -70002, 76, 96, 1, 1))", "marker.before_failure(10).contains(&96)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-native",
        path: "tools/nuisc/tests/native_application_bridge/return_in_fields_native.rs",
        required_patterns: &["return_i32_ancestor_probe.rs", "bitcast float %arg2 to i32", "bitcast double %arg3 to i64", "oracle::mirror_source()", "for prefix in [false, true]", "oracle::native_cases(prefix)", "oracle::native_selected(prefix)", "\"exit_work\"", "\"after_work\"", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_in_fields_tests.rs",
        required_patterns: &["oracle::mirror_source()", "{owner:i64;child_0:i32;child_1:f32;child_2:f64;child_3:bool;child_4:bool}", "continuations.len(), 1", "\"guard_return\"", "seen.contains(pair[1].name.as_str())", "owned_aggregate_alloc_v1", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_in_fields.rs",
        required_patterns: &["return_in_fields_cli_dead_suffix_and_source_arm_gate_proofs_stay_zero", "oracle::cli_cases().len(), 16", "oracle::selected().len(), 12", "oracle::mirror_source()", "project_for_source(&source)", "compile_cache: hit", "fresh.host_objects == cached.host_objects", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_in_fields.rs", "mod return_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_in_fields_tests.rs", "mod return_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_in_fields.rs", "mod return_in_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_in_fields.rs", "in_fields::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["In-selection field exit follow-up:", "Previous early mixed-field exit checkpoint:", "Previous early mixed-field exit task checkpoint:", "Previous early mixed-field exit blocker checkpoint:", "Previous early mixed-field exit action checkpoint:", "Previous early mixed-field exit validation checkpoint:", "Previous early mixed-field exit artifact checkpoint:", "nuis-native-in-selection-field-exits-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-in-fields-receipt",
        path: "docs/reference/nuis-native-in-selection-field-exits-v1.md",
        required_patterns: &["404 distinct tests", "complete 396-test predecessor", "640 successful native processes", "96 fatal native processes", "96 CLI event traps", "864 accepted callbacks", "active/99", "completed prefix transport", "not simultaneous observations", "nested/multiple exits remain separate"],
    },
];
