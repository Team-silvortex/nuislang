use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "scoped-capture-loop-written-input-protection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped.rs",
        required_patterns: &[
            "protected_inputs(",
            "branches::collect_bindings(body, &mut written)",
            "protected.entry(callee.clone()).or_default()",
            "access(arg).is_none_or(|path| written.contains(&path[0]))",
            "inputs.insert(index)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-control-identity-preservation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs",
        required_patterns: &[
            "if !scoped.contains(&name) {",
            "Scoped helpers retain the outliner's control identities",
            "aliases::normalize(&mut candidate, layouts)",
            "valid_caller(&module.functions[*i].body, &name, &plan)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-invariant-field-argument-admission",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering/arguments.rs",
        required_patterns: &[
            "invariant.remove(name.as_str())",
            "while let NirExpr::FieldAccess { base, .. } = root",
            "invariant.contains(name.as_str())",
            "scoped_arguments_only_hoist_ready_invariant_field_paths",
            "scoped_arguments_exclude_nested_writes_and_unknown_roots",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-driver-map-after-projection",
        path: "tools/nuisc/src/lowering/scoped_loop_lowering.rs",
        required_patterns: &[
            "arguments::invariant_bindings(body, bindings)",
            "arguments::ready(arg, &invariant_inputs)",
            "scalar_carries::argument_index(&result, param, arg)",
            "action_args.push(\"$current\".to_owned())",
            "action_args.push(\"$carry\".to_owned())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-projection-and-control-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/capture_scoped_tests.rs",
        required_patterns: &[
            "scoped_projection_keeps_rebound_record_seeds_by_argument_identity",
            "scoped_projection_unions_writes_across_callers_and_nested_scopes",
            "scoped_projection_keeps_computed_and_whole_uses_transactional",
            "scoped_field_inputs_preserve_zero_trips_and_induction_carry_maps",
            "scoped_field_inputs_preserve_bool_record_and_break_carry_slots",
            "scoped_field_inputs_keep_nested_break_controls_local",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "scoped-capture-wide-source-not-narrow-wrapper",
        path: "tools/nuisc/tests/native_application_bridge/typed_alias_capture_fixture.rs",
        required_patterns: &[
            "pub fn wide_loop_source()",
            "let saved = fallback; let second = saved;",
            "let result = relay(second.f63) / second.f1;",
        ],
    },
];
