use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &["effectful_selections_return_plain_prefix_tests.rs", "let (_, yes_prefix) = returned(yes)?", "if yes_prefix.is_empty() && no_prefix.is_empty()", "prove(condition, yes, no, scope, signatures, pure, &mut budget)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-typed-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_predicates.rs",
        required_patterns: &["ty == scalar_type(\"bool\")", "nested::inspect_at_depth(condition, scope, signatures, budget, depth)?"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-effect-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_returns.rs",
        required_patterns: &["return_type != Some(&scalar_type(\"bool\"))", "any(|name| !pure.contains(name))", "captures.len() > 31"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-units",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_plain_prefix_tests.rs",
        required_patterns: &["return_plain_prefix_calls_literals_and_names_use_the_typed_route", "!SOURCE.contains(\"&& true\")", "BTreeSet<_>>().len(), 4", "return_plain_prefix_non_bool_and_authority_vetoes_are_atomic", "assert_eq!(module, original)", "assert_eq!(names, original_names)", "return_plain_prefix_exact_budget_and_depth_need_no_logical_edge", "logical_edges: 0", "budget(38, 256)", "return_plain_prefix_keeps_effect_admission_prefix_requirement_and_capture_cap", "for count in [31, 32]"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-source",
        path: "tools/nuisc/tests/native_application_bridge/return_plain_exit_prefix.ns",
        required_patterns: &["let finished = tail_work(local, 1 / tail);", "return answer_work(local, a, b, c, d, e, 1 / answer);", "sibling_owner(local, 1 / no_owner)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-variants",
        path: "tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix_cases.rs",
        required_patterns: &["PLAIN_SOURCE: &str = include_str!(\"return_plain_exit_prefix.ns\")", "pub fn source_variants() -> [String; 4]", "SOURCE.to_owned()", "mirror_source()", "PLAIN_SOURCE.to_owned()", "mirror(PLAIN_SOURCE)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-native",
        path: "tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix.rs",
        required_patterns: &["for source in cases::source_variants()", "cases::OBSERVER", "cases::failures()", "c.before_failure(stage)"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-aot",
        path: "tools/nuisc/src/aot_application_effect_call_return_typed_exit_prefix_tests.rs",
        required_patterns: &["for source in cases::source_variants()", "\"guard_return\"", "lacks dependency order"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-cli",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_typed_exit_prefix.rs",
        required_patterns: &["for source in cases::source_variants()", "cases::delivery_cases()", "compile_cache: hit", "for _ in 0..2", "fresh.host_objects == cached.host_objects"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_plain_prefix.rs", "plain_prefix::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-history",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Ordinary bool-prefix return follow-up:", "Previous typed returning-prefix checkpoint:", "Previous typed returning-prefix task checkpoint:", "Previous typed returning-prefix blocker checkpoint:", "Previous typed returning-prefix action checkpoint:", "Previous typed returning-prefix validation checkpoint:", "Previous typed returning-prefix artifact checkpoint:", "nuis-native-plain-bool-prefix-returns-v1.md"],
    },
    DevTensorDriftCheckSpec {
        id: "native-plain-prefix-receipt",
        path: "docs/reference/nuis-native-plain-bool-prefix-returns-v1.md",
        required_patterns: &["nonempty prefix", "bool/effect proof", "zero original logical edges", "sixteen delivery images", "full-return-tree", "whole-workspace formatting"],
    },
];
