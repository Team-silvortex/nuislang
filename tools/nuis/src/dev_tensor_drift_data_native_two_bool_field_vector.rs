use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_fields.rs",
        required_patterns: &["ty == &scalar_type(\"bool\") || numeric(ty)", "!numeric(scope.get(owner)?)", "!child_scalar(ty)", "!seen.insert(name.clone())", "captures.len() > 31", "(2..64).contains(&count)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["!transport::numeric(staged.local.get(owner.name())?)", "!transport::private_field_scalar(staged.local.get(child)?)", "entering.contains_key(child)", "!staged.defined.contains(child)", "staged.constants.contains(child)", "tree::preflight(yes, no, suffix, depth)?", "Walk the original forest once"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-unit-siblings",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_two_bool_field_vector_tests.rs",
        required_patterns: &["return_two_bool_field_vector_accepts_current_sibling_reads_and_retention", "let c=decide(c!=true);let d=d&&decide(c)", "let d=decide(d!=true);let c=c||decide(d)", "scalar_type(\"bool\"),scalar_type(\"bool\")", "helper.return_type"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-unit-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_two_bool_field_vector_tests.rs",
        required_patterns: &["return_two_bool_field_vector_keeps_each_bool_private_mutable_and_exact", "for child in [\"c\", \"d\"]", "undefined.defined.remove(child)", "sealed.constants.insert(child.into())", "bool_owner.insert", "let saved=d", "reject(&root)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-unit-projections",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_two_bool_field_vector_tests.rs",
        required_patterns: &["return_two_bool_field_vector_preserves_distinct_false_neutrals_and_projections", "fields[3]", "fields[4]", "(\"child_2\".into(), NirExpr::Bool(false))", "(\"child_3\".into(), NirExpr::Bool(false))", "(\"c\", \"child_2\")", "(\"d\", \"child_3\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-unit-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_two_bool_field_vector_tests.rs",
        required_patterns: &["return_two_bool_field_vector_shares_budget_across_both_logical_roots", "logical(\"d\", 17)", "logical(\"c\", 17)", "logical(\"d\", 18)", "(6, 70, 32, true)", "(5, 70, 32, false)", "(6, 69, 32, false)", "(6, 70, 31, false)", "for count in [16, 17]", "for depth in [7, 8]", "for count in [31, 32]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-source-current-siblings",
        path: "tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector.ns",
        required_patterns: &["bool_seed: bool, peer_seed: bool", "bool_seed: bool_seed, peer_seed: peer_seed", "state.bool_seed, state.peer_seed", "let c = bool_work(c != true, 1 / bool_rhs);\n                    let d = d && peer_work(c, 1 / peer_rhs);", "let d = peer_work(d != true, 1 / peer_rhs);\n                    let c = c || bool_work(d, 1 / bool_rhs);", "after_work(local, a, b, c, d", "parent_work(local, a, b, c, d", "const peer_seal: bool = d || false"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-oracle-independent",
        path: "tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector_oracle.rs",
        required_patterns: &["pub bool_seed: bool", "pub peer_seed: bool", "for c in [false, true]", "for d in [false, true]", "d = d && if stale { bool_seed } else { c }", "c = c || if stale { peer_seed } else { d }", "STAGES: [i64; 15]", "fields = Some((local, i64::from(a), b, i64::from(c), i64::from(d)))", "Source-event positions disambiguate sentinels"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-reference-mutants",
        path: "tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector.rs",
        required_patterns: &["Value::Bool(case.peer_seed)", "count(), 22", "3 * 4 * 12 * 4 * 4", "current.stale_sibling()", "assert_ne!(current.owner, stale.owner)", "mutant differs in data, not selected calls", "d && peer_work(bool_seed,", "c || bool_work(peer_seed,", "state(&session, &stale)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-reference-faults",
        path: "tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector.rs",
        required_patterns: &["Some((70001, -70002, 96, 1, 1))", "marker.before_failure(12)", "marker.before_failure(13)", "marker.before_failure(14)", "cli.len(), 34", "inactive AND sibling RHS must stay poisoned", "inactive OR sibling RHS must stay poisoned", "assert_eq!(session.state(), &last)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-native-f32",
        path: "tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector_native_f32.rs",
        required_patterns: &["sext i32 %arg1 to i64", "bitcast float %arg2 to i32", "zext i1 %arg3 to i64", "zext i1 %arg4 to i64", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-native-f64",
        path: "tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector_native_f64.rs",
        required_patterns: &["bitcast double %arg2 to i64", "zext i1 %arg3 to i64", "zext i1 %arg4 to i64", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_two_bool_field_vector_tests.rs",
        required_patterns: &["super::return_private_fields::check_layout", "{owner:i64;child_0:i32;child_1:FLOAT;child_2:bool;child_3:bool}"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_two_bool_field_vector.rs",
        required_patterns: &["return_two_bool_field_vector_oracle.rs", "before_failure(14)", "usize::from(opened.fields.is_some()) * 5", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)", "return_two_bool_field_vector_cli_stale_sensitive_inputs", "current.stale_sibling()", "assert_ne!(current.result, stale.result)", "assert_eq!(current.trace, stale.trace)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["effectful_selections_return_two_bool_field_vector_tests.rs", "mod two_bool_vector_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_two_bool_field_vector.rs", "mod return_two_bool_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_two_bool_field_vector_tests.rs", "mod return_two_bool_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_two_bool_field_vector.rs", "mod return_two_bool_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Two-bool field-vector follow-up:", "Previous bool field-vector checkpoint:", "Previous bool field-vector task checkpoint:", "Previous bool field-vector blocker checkpoint:", "Previous bool field-vector action checkpoint:", "Previous bool field-vector validation checkpoint:", "Previous bool field-vector artifact checkpoint:", "nuis-native-two-bool-field-vector-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-receipt",
        path: "docs/reference/nuis-native-two-bool-field-vector-continuations-v1.md",
        required_patterns: &["complete 474-test predecessor", "402 distinct tests", "9216 successful native processes", "60 fatal native processes", "120 CLI event traps", "six source-equal fields", "active/99", "not formal memory safety", "32/33 shared logical edges"],
    },
    DevTensorDriftCheckSpec {
        id: "native-two-bool-field-vector-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_two_bool_field_vector.rs", "two_bool_field_vector::CHECKS"],
    },
];
