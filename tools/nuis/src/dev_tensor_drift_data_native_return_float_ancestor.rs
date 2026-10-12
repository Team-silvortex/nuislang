use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["mod float_ancestor_tests;", "pub(super) fn numeric_owner", "[\"i32\", \"i64\", \"f32\", \"f64\"]", "ty == &scalar_type(name)", "authorized.is_none_or(|target| target.name() == *name)", "!staged.constants.contains(*name)", "ty: scope.get(name)?.clone()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-one-outward-private-bool",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &["numeric_owner(staged.local.get(owner.name())?)", "staged.local.get(&child)? != &scalar_type(\"bool\")", "entering.contains_key(&child)", "!writes.remove(owner.name()) || writes.len() != 1", "if writes.len() > 2"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-typed-transport",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &["[\"i64\", \"f32\", \"f64\"]", "owner_type == &scalar_type(name)", "&& child_type == &scalar_type(\"bool\")", "(owner_type.clone(), child_type.clone())", "captures.len() > 31", "owned_pair::install"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-exact-inactive-fields",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport_owned.rs",
        required_patterns: &["\"f32\" => NirExpr::F32(\"0.0\".into())", "\"f64\" => NirExpr::F64(\"0.0\".into())", "\"bool\" => NirExpr::Bool(false)", "record(zero.0, zero.1)", "NirExpr::FieldAccess", "NirVisibility::Private"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-whole-original-forest",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &["!numeric_owner(scope.get(owner.name())?)", "tree::preflight(yes, no, suffix, depth)?", "ty: scope.get(owner.name())?.clone()", "No arm receives a copy of the remaining sequence"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-both-width-unit-boundaries",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_float_ancestor_tests.rs",
        required_patterns: &["float_cases!(f32_cases, \"f32\"", "float_cases!(f64_cases, \"f64\"", "(7, 7, true)", "for count in [31, 32]", "for depth in [7, 8]", "assert_eq!(module, before)", "assert_eq!(names, before_names)", "for kind in [\"ref\", \"optional\", \"generic\", \"u32\", \"String\"]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-source-work",
        path: "tools/nuisc/tests/native_application_bridge/return_float_ancestor_private_bool.ns",
        required_patterns: &["seed: f64", "fn initialize(value: f64", "fn parent_work(value: f64", "return -value;", "let child = child_initial(local < 0.0, 1 / child_init)", "const seal: bool = child || false", "return answer_work(local, seed, 1 / answer) && first", "print(80)", "print(79)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-independent-ieee-oracle",
        path: "tools/nuisc/tests/native_application_bridge/return_float_ancestor_oracle.rs",
        required_patterns: &["Independent IEEE comparisons and sign-bit updates", "f32::from_bits(bits as u32)", "f64::from_bits(bits as u64)", "local ^= sign", "0x8000_0000", "0x7fc0_1234", "0xff80_0001", "0x7ff8_1234_5678_abcd", "0xfff0_0000_0000_0001", "values[stage + 2] = \"0\""],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-f32-actual-argument",
        path: "tools/nuisc/tests/native_application_bridge/return_float_ancestor_native_f32.rs",
        required_patterns: &["bitcast float %arg0 to i32", "zext i32 %observed_bits to i64", "call void @nuis_debug_print_i64(i64 %observed_owner)", "return_i32_ancestor_probe.rs", "native_exact_bits_and_order", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-f64-actual-argument",
        path: "tools/nuisc/tests/native_application_bridge/return_float_ancestor_native_f64.rs",
        required_patterns: &["bitcast double %arg0 to i64", "call void @nuis_debug_print_i64(i64 %observed_owner)", "return_i32_ancestor_probe.rs", "native_exact_bits_and_order", "native_real_event_traps"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-reference-last-publication",
        path: "tools/nuisc/tests/native_application_bridge/return_float_ancestor.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "for reversed in [false, true]", "oracle::cli_cases(ty)", "ApplicationSessionPhase::Faulted", "assert_eq!(session.state(), &last)", "session.completion_status().is_err()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-native-fatal-canaries",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &["pub(super) fn check(", "observation: Option<(&str, &str)>", "llvm.insert_str(insertion, OWNER_PROBE)", "Source computations, private joins and real traps remain intact", "Some(4 | 5)", "[(0, 91), (14, 92)]", "load volatile i64, ptr %prev", "assert!(!actual.contains(&AFTER))"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-aot-policy-effect-veto",
        path: "tools/nuisc/src/aot_application_effect_call_return_float_ancestor_tests.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "result.ty == ty", "{{owner:{ty};child:bool}}", "call_owned_struct", "cast_f32_to_i64", "cast_f64_to_i64", "yir_core::EdgeKind::Effect", "lacks dependency order", "Vec::<String>::new()"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-cli-eight-images-publication",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_float_ancestor.rs",
        required_patterns: &["for ty in [\"f32\", \"f64\"]", "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects", "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)", "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "previous.state_text(\"event\")", "SIGILL", "SIGTRAP", "healthy(project, output, opened)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-aot-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_float_ancestor_tests.rs", "mod return_float_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-native-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_float_ancestor.rs", "mod return_float_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-cli-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_float_ancestor.rs", "mod return_float_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-receipt",
        path: "docs/reference/nuis-native-private-float-ancestor-continuations-v1.md",
        required_patterns: &["## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance", "bool-returning", "negative zero", "NaN", "formal memory safety"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-float-ancestor-tensor-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Float-ancestor private bool follow-up:", "Previous i64-ancestor checkpoint:", "Previous i64-ancestor task checkpoint:", "Previous i64-ancestor blocker checkpoint:", "Previous i64-ancestor action checkpoint:", "Previous i64-ancestor validation checkpoint:", "Previous i64-ancestor artifact checkpoint:", "nuis-native-private-float-ancestor-continuations-v1.md"],
    },
];
