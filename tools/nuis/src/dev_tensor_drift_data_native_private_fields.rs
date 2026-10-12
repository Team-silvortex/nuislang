use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-private-fields-lexical-three-write-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["struct Children(Vec<String>)", "candidates.len() < 2", "candidates.len() == 2 && candidates.contains(owner.name())", "candidates.remove(owner.name());", "!transport::private_field_arity(candidates.len())", "entering.contains_key(child)", "!staged.defined.contains(child)", "staged.constants.contains(child)", "!transport::private_field_scalar(staged.local.get(child)?)", "*remaining = remaining.checked_sub(body.len())?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-unchanged-pair-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2", "entering.contains_key(&child)", "staged.constants.contains(&child)", "!transport::numeric_pair("],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-field-driven-record",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_record.rs",
        required_patterns: &["fields: Vec<(String, String, NirTypeRef)>", "Representation follows independently proved field types", "\"i32\" => NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0)))", "\"f32\" => NirExpr::F32(\"0.0\".into())", "\"f64\" => NirExpr::F64(\"0.0\".into())", "NirVisibility::Private", "NirExpr::Var(binding.clone())", "record(false)", "record(true)", "install_with_neutral", "NirExpr::FieldAccess", "for (field, name, ty) in fields"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-legacy-pair-adapter",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_owned.rs",
        required_patterns: &["[(\"owner\", field_types.0), (\"child\", field_types.1)]", "super::record::install", "zip([owner, child])", "\"__nuis_effect_pair_value\"", "\"__nuis_effect_call_pair\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-typed-distinct-captures",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_fields.rs",
        required_patterns: &["!arity(children.len())", "!numeric(scope.get(owner)?)", "!seen.insert(name.clone())", "!child_scalar(ty)", "format!(\"child_{index}\")", "captures.len() > 31", "scalar(ty).then", "SelectedValue::Region(yes.statements", "record::install", "\"__nuis_effect_fields_value\"", "\"__nuis_effect_call_fields\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-shared-original-forest-and-versions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields.rs",
        required_patterns: &["tree::preflight(yes, no, suffix, depth)?", "tree::isolated(yes, scope, &future)?", "tree::isolated(no, scope, &future)?", "nested::consume_node(budget)?", ".any(|name| staged.constants.contains(name))", "transport::prepare_fields", "regions::Statement::PrivateFields", "staged.defined.extend(children.0.iter().cloned())", "Walk the original forest once", "Only certified private bindings advance", "append_prefix"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-region-installation",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs",
        required_patterns: &["PrivateFields(Box<transport::FieldsSelection>)", "Statement::PrivateFields(plan)", "transport::install_fields("],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-no-continuation-cloning",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_continuations.rs",
        required_patterns: &["fn copy_flat", "regions::Statement::Selection(_)", "regions::Statement::PrivatePair(_)", "regions::Statement::PrivateFields(_)", "return None", "classify as classify_fields", "prove as continue_fields"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-single-outward-integration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["mixed::classify_fields(then_body, else_body, scope, &staged, &authorized, depth)?", "mixed::continue_fields", "return Some(finish(staged, proof))", "mixed::classify(then_body, else_body, scope, &staged, &authorized, depth)?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-four-unit-proof-families",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_private_fields_tests.rs",
        required_patterns: &["return_private_fields_accepts_orders_retention_nested_sequences_and_one_outward_target", "return_private_fields_rejects_entering_sealed_qualified_extra_writes_hidden_work_and_leaks_atomically", "return_private_fields_generic_record_has_independent_neutrals_projections_and_no_pair_codec", "return_private_fields_original_forest_capture_statement_and_depth_bounds", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "for count in [64, 65]", "for count in [16, 17]", "for count in [31, 32]", "for depth in [7, 8]", "(8, 8, true)", "__nuis_effect_call_fields_1", "not a whitelist of tested backend/type triples"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-nuis-three-independent-seeds",
        path: "tools/nuisc/tests/native_application_bridge/return_private_fields.ns",
        required_patterns: &["seed: i64, small_seed: i32, float_seed: f64", "fn small_initial(value: i32", "fn float_initial(value: f64", "let a = small_initial(small_seed", "let b = float_initial(float_seed", "if (small < i32_from_i64(0)) != (child < 0.0)", "return minus_one - value", "if a == i32_from_i64(0)", "if a == i32_from_i64(-1)", "after_work(local, a, b", "const small_seal: i32", "const float_seal: f64", "state.small_seed", "state.float_seed", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-independent-ieee-integer-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_private_fields_oracle.rs",
        required_patterns: &["Independent i64/i32 complements and raw IEEE sign bits", "pub small_seed: i32", "pub float_seed: i64", "pub fields: Option<(i64, i64, i64)>", "i64::MIN", "i64::MAX", "i32::MIN", "i32::MAX", "0x7fc0_1234", "0xff80_0001", "0x7ff8_1234_5678_abcd", "0xfff0_0000_0000_0001", "fields = Some((local, i64::from(a), b))", "Source-event positions remain distinct", "Case::new(ty, 70001, 70001, sign | 96, true, true)", "pub fn native_selected", "pub fn cli_cases", "self.state_decimal()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-reference-values-and-marker-prefixes",
        path: "tools/nuisc/tests/native_application_bridge/return_private_fields.rs",
        required_patterns: &["Value::Int(case.seed)", "Value::I32(case.small_seed)", "f32::from_bits(case.float_seed", "f64::from_bits(case.float_seed", "count(), 16", "for opened in oracle::cases(ty)", "for reversed in [false, true]", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "3 * 4 * 12 * 4", "Some((-70002, -70002, 96))", "marker.before_failure(8)", "marker.before_failure(9)", "marker.before_failure(10)", "assert_eq!(tiny.state_words()[2], 1)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-f32-actual-three-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_private_fields_native_f32.rs",
        required_patterns: &["i64 %arg0", "sext i32 %arg1 to i64", "bitcast float %arg2 to i32", "zext i32 %observed_bits to i64", "check_with_open_words", "Case::open_words", "Some((\"after_work\", FIELD_PROBE))", "oracle::cases(\"f32\")", "oracle::native_selected(\"f32\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-f64-actual-three-fields",
        path: "tools/nuisc/tests/native_application_bridge/return_private_fields_native_f64.rs",
        required_patterns: &["i64 %arg0", "sext i32 %arg1 to i64", "bitcast double %arg2 to i64", "check_with_open_words", "Case::open_words", "Some((\"after_work\", FIELD_PROBE))", "oracle::cases(\"f64\")", "oracle::native_selected(\"f64\")"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-dynamic-word-width-and-real-state",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["let state_width = cases[0].state_words().len()", "let open_width = open_words(&cases[0]).len()", "state_width + 2", "[(0, 91), (state_width + 1, 92)]", "load volatile i64, ptr %prev", "Full trace equality distinguishes AFTER from matching field words", "trace.extend(case.before_failure(index))", "Some(4 | 5)", "dynamic_loop_guard::run_command_bounded", "nuis_scheduler_owned_aggregate_alloc_v1", "nuis_scheduler_owned_aggregate_drop_v1"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-aot-exact-layout-and-policy",
        path: "tools/nuisc/src/aot_application_effect_call_return_private_fields_tests.rs",
        required_patterns: &["{owner:i64;child_0:i32;child_1:FLOAT}", "result.ty == \"i64\"", "call_owned_struct", "&proof.parameters[0].node", "&proof.parameters[1].node", "fn width_safe_narrow", "depth >= 8", "yir_core::EdgeKind::Effect", "lacks dependency order", "Vec::<String>::new()", "canonical_i32_codec", "nonparticipating_coercions", "fptrunc", "ret double %1", "ret i64 %2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-cli-eight-source-free-images",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_private_fields.rs",
        required_patterns: &["oracle::cli_cases(ty)", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)", "usize::from(opened.fields.is_some()) * 3"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_private_fields.rs", "mod return_private_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_private_fields_tests.rs", "mod return_private_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_private_fields.rs", "mod return_private_fields;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-tensor-complete-predecessor",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Private three-field continuation follow-up:", "Previous float-i32 checkpoint:", "Previous float-i32 task checkpoint:", "Previous float-i32 blocker checkpoint:", "Previous float-i32 action checkpoint:", "Previous float-i32 validation checkpoint:", "Previous float-i32 artifact checkpoint:", "nuis-native-private-three-field-continuations-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-bounded-contract-receipt",
        path: "docs/reference/nuis-native-private-three-field-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Field Driven Transport", "## Executed Acceptance", "441 distinct tests", "2304 successful native processes", "44 fatal native processes", "88 CLI event traps", "complete 430-test predecessor", "formal memory safety"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-receipt-scope",
        path: "docs/reference/nuis-native-private-three-field-continuations-v1.md",
        required_patterns: &["macOS CPU", "active/99", "six source-equal fields", "not added to the current count", "readonly entering values", "diagnostic sentinel", "three private children"],
    },
    DevTensorDriftCheckSpec {
        id: "native-private-fields-recursive-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_private_fields.rs", "private_fields::CHECKS"],
    },
];
