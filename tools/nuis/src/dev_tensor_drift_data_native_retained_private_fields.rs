use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-owner-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["candidates.len() < 2", "candidates.len() == 2 && candidates.contains(owner.name())", "candidates.remove(owner.name());", "!transport::private_field_arity(candidates.len())", "The Target already delegates", "retains that value", "entering.contains_key(child)", "!staged.defined.contains(child)", "staged.constants.contains(child)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-root-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["Root candidates have no delegation", "actual write evidence", "let delegated = authorized.map", "!staged.defined.contains(name)", "authorized.is_some_and(|target| target.name() == name)", "staged.constants.contains(name)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-unit-selected-and-retained",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_retained_private_fields_tests.rs",
        required_patterns: &["return_retained_private_fields_accepts_child_only_initial_arms_and_shared_owner_suffix", "let a=small(a);let b=floating(b);", "let b=floating(b);let a=small(a);", "let ignored=relay(local)", "verify_nir_module"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-unit-authority-vetoes",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_retained_private_fields_tests.rs",
        required_patterns: &["return_retained_private_fields_requires_existing_target_and_two_lexical_mutable_children", "assert!(!stage(&s).defined.contains(\"local\"))", "undefined.defined.remove(name)", "sealed.constants.insert(name.into())", "absent.remove(\"local\")", "classify(&pair", "no_authority", "A root child-only body"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-unit-atomic-scope",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_retained_private_fields_tests.rs",
        required_patterns: &["return_retained_private_fields_rejects_seals_leaks_and_other_entering_writes_atomically", "let seed=local", "const local:i64=local", "const a:i32=int_seed", "let secret=seed", "print(80)", "reject(&text)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-unit-original-work",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_retained_private_fields_tests.rs",
        required_patterns: &["return_retained_private_fields_counts_original_work_and_captures_retention", "(4, 4, true)", "(3, 4, false)", "(4, 3, false)", "proof.inputs.contains(\"local\")", "for count in [16, 17]", "for depth in [7, 8]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-native-source",
        path: "tools/nuisc/tests/native_application_bridge/return_retained_private_fields.ns",
        required_patterns: &["seed: i64, small_seed: i32, float_seed: f64", "let a = small_initial(small_seed", "let b = float_initial(float_seed", "if condition(b < 0.0", "if a == i32_from_i64(0)", "if a == i32_from_i64(-1)", "let local = after_work(local, a, b, 1 / after);", "let local = parent_work(local, a, b, 1 / parent_rhs);", "const small_seal: i32", "const float_seal: f64", "print(80)", "                if condition(b < 0.0, 1 / cond) {\n                    let a = small_work(a, 1 / small_rhs);\n                    let b = float_work(b, 1 / float_rhs);\n                    if a == i32_from_i64(0) {\n                        let a = small_work(a, 1 / small_rhs);\n                        let b = float_work(b, 1 / float_rhs);\n                    }\n                } else {\n                    let b = float_work(b, 1 / float_rhs);\n                    let a = small_work(a, 1 / small_rhs);\n                    if a == i32_from_i64(-1) {\n                        let b = float_work(b, 1 / float_rhs);\n                        let a = small_work(a, 1 / small_rhs);\n                    }\n                }\n"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-shared-independent-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_private_fields_oracle.rs",
        required_patterns: &["retain_owner: bool", "pub fn retained", "fn calculate", "Self::calculate(ty, seed, small_seed, float_seed, first, second, true)", "if !retain_owner", "if retain_owner", "fields = Some((local, i64::from(a), b))", "self.retain_owner", "Independent i64/i32 complements and raw IEEE sign bits"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-oracle-inventories",
        path: "tools/nuisc/tests/native_application_bridge/return_retained_private_fields_oracle.rs",
        required_patterns: &["return_private_fields_oracle.rs", "return_retained_private_fields.ns", "Case::retained", "base::cases(ty)", "base::cli_cases(ty)", "base::selected(ty)", "base::native_selected(ty)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-reference-and-sentinel",
        path: "tools/nuisc/tests/native_application_bridge/return_retained_private_fields.rs",
        required_patterns: &["return_retained_private_fields_reference_values_order_and_last_publication", "return_retained_private_fields_independent_extrema_and_marker_fault_prefixes", "for reversed in [false, true]", "3 * 4 * 12 * 4", "Some((70001, -70002, 96))", "marker.before_failure(8)", "marker.before_failure(9)", "marker.before_failure(10)", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-f32-live-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_retained_private_fields_native_f32.rs",
        required_patterns: &["i64 %arg0", "sext i32 %arg1 to i64", "bitcast float %arg2 to i32", "zext i32 %observed_bits to i64", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")", "Case::open_words", "Some((\"after_work\", FIELD_PROBE))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-f64-live-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_retained_private_fields_native_f64.rs",
        required_patterns: &["i64 %arg0", "sext i32 %arg1 to i64", "bitcast double %arg2 to i64", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")", "Case::open_words", "Some((\"after_work\", FIELD_PROBE))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-aot-shared-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_retained_private_fields_tests.rs",
        required_patterns: &["native_return_retained_private_fields_exact_layout_guarded_order_and_policy_vetoes", "super::return_private_fields::check_source", "return_retained_private_fields.ns"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-cli-eight-images",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_retained_private_fields.rs",
        required_patterns: &["return_retained_private_fields_build_cache_restore_exact_outcomes_and_event_failure_publication", "return_retained_private_fields_oracle.rs", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_retained_private_fields.rs", "mod return_retained_private_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_retained_private_fields_tests.rs", "mod return_retained_private_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_retained_private_fields.rs", "mod return_retained_private_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["effectful_selections_return_retained_private_fields_tests.rs", "mod retained_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-complete-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Retained-owner three-field follow-up:", "Previous three-field checkpoint:", "Previous three-field task checkpoint:", "Previous three-field blocker checkpoint:", "Previous three-field action checkpoint:", "Previous three-field validation checkpoint:", "Previous three-field artifact checkpoint:", "nuis-native-retained-owner-three-field-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-bounded-receipt",
        path: "docs/reference/nuis-native-retained-owner-three-field-continuations-v1.md",
        required_patterns: &["## Existing Owner Authority", "## Executed Acceptance", "452 distinct tests", "2304 successful native processes", "44 fatal native processes", "88 CLI event traps", "complete 441-test predecessor", "formal memory safety", "active/99", "six source-equal fields"],
    },
    DevTensorDriftCheckSpec {
        id: "native-retained-private-fields-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_retained_private_fields.rs", "retained_private_fields::CHECKS"],
    },
];
