use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_fields.rs",
        required_patterns: &["fn child_scalar(ty: &NirTypeRef)", "ty == &scalar_type(\"bool\") || numeric(ty)", "!numeric(scope.get(owner)?)", "!child_scalar(ty)", "!seen.insert(name.clone())", "captures.len() > 31", "(2..64).contains(&count)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["!transport::numeric(staged.local.get(owner.name())?)", "!transport::private_field_scalar(staged.local.get(child)?)", "entering.contains_key(child)", "!staged.defined.contains(child)", "staged.constants.contains(child)", "tree::preflight(yes, no, suffix, depth)?", "Walk the original forest once"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-neutral",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["\"bool\" => NirExpr::Bool(false)", "FieldAccess", "record(true)", "record(false)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-logical-budget",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_predicates.rs",
        required_patterns: &["budget.logical_edges = budget.logical_edges.checked_sub(1)?", "budget.expressions = budget.expressions.checked_sub(1)?", "if depth >= 32", "right_inputs.len() > 31", "No RHS call or fallible argument"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-old-pair",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2", "entering.contains_key(&child)", "staged.constants.contains(&child)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-unit-order",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_bool_field_vector_tests.rs",
        required_patterns: &["return_bool_field_vector_accepts_logical_updates_retention_and_orders", "let c=c&&decide(c)", "let c=c||decide(c)", "const seal:bool=c||false", "helper.return_type"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-unit-veto",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_bool_field_vector_tests.rs",
        required_patterns: &["return_bool_field_vector_preserves_owner_authority_and_exact_child_vetoes", "undefined.defined.remove(\"c\")", "sealed.constants.insert(\"c\".into())", "bool_owner.insert(\"local\".into(), scalar_type(\"bool\"))", "reject(&root)", "let saved=c", "let c:i64=relay(seed)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-unit-projection",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_bool_field_vector_tests.rs",
        required_patterns: &["return_bool_field_vector_uses_false_neutrals_and_distinct_typed_projections", "NirExpr::Bool(false)", "__nuis_effect_fields_value_1", "(\"c\", \"child_2\")", "qualified.is_ref = true"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-unit-bound",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_bool_field_vector_tests.rs",
        required_patterns: &["return_bool_field_vector_shares_logical_work_depth_and_capture_budgets", "(5, 69, 32, true)", "(4, 69, 32, false)", "(5, 68, 32, false)", "(5, 69, 31, false)", "logical(34)", "for count in [16, 17]", "for count in [64, 65]", "for depth in [7, 8]", "for captures in [31, 32]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-source",
        path: "tools/nuisc/tests/native_application_bridge/return_bool_field_vector.ns",
        required_patterns: &["bool_seed: bool", "state.bool_seed", "let c = c && bool_work(c, 1 / bool_rhs)", "let c = c || bool_work(c, 1 / bool_rhs)", "bool_work(c != true, 1 / bool_rhs)", "after_work(local, a, b, c", "parent_work(local, a, b, c", "const bool_seal: bool = c || false", "!= flag"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_bool_field_vector_oracle.rs",
        required_patterns: &["pub bool_seed: bool", "for flag in [false, true]", "fields = Some((local, i64::from(a), b, i64::from(c)))", "if !c {", "if c {", "c = !c", "Source-event positions disambiguate sentinels", "i64::from(self.bool_seed)", "STAGES: [i64; 13]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-reference",
        path: "tools/nuisc/tests/native_application_bridge/return_bool_field_vector.rs",
        required_patterns: &["Value::Bool(case.bool_seed)", "private bool must affect real computation", "Some((70001, -70002, 96, 1))", "marker.before_failure(12)", "skipped logical RHS must carry a poisoned proof", "for reversed in [false, true]", "assert_eq!(session.state(), &last)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-native-f32",
        path: "tools/nuisc/tests/native_application_bridge/return_bool_field_vector_native_f32.rs",
        required_patterns: &["sext i32 %arg1 to i64", "bitcast float %arg2 to i32", "zext i1 %arg3 to i64", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-native-f64",
        path: "tools/nuisc/tests/native_application_bridge/return_bool_field_vector_native_f64.rs",
        required_patterns: &["bitcast double %arg2 to i64", "zext i1 %arg3 to i64", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")", "Case::open_words"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_bool_field_vector_tests.rs",
        required_patterns: &["super::return_private_fields::check_layout", "{owner:i64;child_0:i32;child_1:FLOAT;child_2:bool}"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_bool_field_vector.rs",
        required_patterns: &["return_bool_field_vector_oracle.rs", "before_failure(12)", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-unit-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["effectful_selections_return_bool_field_vector_tests.rs", "mod bool_vector_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_bool_field_vector.rs", "mod return_bool_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_bool_field_vector_tests.rs", "mod return_bool_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_bool_field_vector.rs", "mod return_bool_field_vector;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Bool field-vector follow-up:", "Previous numeric field-vector checkpoint:", "Previous numeric field-vector task checkpoint:", "Previous numeric field-vector blocker checkpoint:", "Previous numeric field-vector action checkpoint:", "Previous numeric field-vector validation checkpoint:", "Previous numeric field-vector artifact checkpoint:", "nuis-native-bool-field-vector-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-receipt",
        path: "docs/reference/nuis-native-bool-field-vector-continuations-v1.md",
        required_patterns: &["complete 463-test predecessor", "474 distinct tests", "4608 successful native processes", "52 fatal native processes", "104 CLI event traps", "active/99", "not formal memory safety", "32/33 logical edges", "six source-equal fields"],
    },
    DevTensorDriftCheckSpec {
        id: "native-bool-field-vector-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_bool_field_vector.rs", "bool_field_vector::CHECKS"],
    },
];
