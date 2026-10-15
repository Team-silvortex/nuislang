use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-sealed-binding-destination-preflight",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs",
        required_patterns: &["effectful_selections_sealed_binding_tests.rs", "pub(super) fn writes_constants", "A sealed destination stays sealed regardless of its RHS syntax.", "NirStmt::Let { name, .. } | NirStmt::Const { name, .. }", "if constants.contains(name)", "if constants.is_empty()", "yes.len() + no.len() > 64", "visited + pending.len() + then_body.len() + else_body.len() > 64"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sealed-binding-before-planning-and-installed-seals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs",
        required_patterns: &["regions::writes_constants(then_body, else_body, &constants)", "let mut constants = BTreeSet::new()", "constants.insert(name.clone())", ".is_some_and(|(_, constant)| *constant)", "Reserve source types only on installation"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sealed-binding-ordinary-root-and-installed-seal-matrix",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_sealed_binding_tests.rs",
        required_patterns: &["sealed_binding_effectful_selections_veto_ordinary_writes_in_all_scalar_branches_atomically", "sealed_binding_effectful_selections_veto_literal_name_comparison_call_and_logical_rhs", "sealed_binding_effectful_selections_keep_installed_const_seals_without_partial_rewrites", "assert_eq!(module, before", "assert_eq!(names, before_names", "assert_eq!(module, expected", "assert_eq!(actual_names, expected_names", "[\"bool\", \"i32\", \"i64\", \"f32\", \"f64\"]", "late NIR destination mutation"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sealed-binding-positive-controls-and-bounded-nir-checks",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_sealed_binding_tests.rs",
        required_patterns: &["sealed_binding_effectful_selections_preserve_read_only_constants_and_unrelated_updates", "verify_nir_module(&module).unwrap()", "sealed_binding_effectful_selections_preflight_depends_on_destination_not_rhs_or_type", "sealed_binding_effectful_selections_preflight_keeps_exact_64_node_pending_and_depth_bound", "(64, false), (65, true)", "(31, false), (32, true)", "sealed_binding_effectful_selections_preflight_does_not_spend_expression_budget_or_follow_reads", "for _ in 0..64"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sealed-binding-drift-registration",
        path: "tools/nuis/src/dev_tensor_drift_data_native_iteration_calls.rs",
        required_patterns: &["dev_tensor_drift_data_native_sealed_binding.rs", "sealed_binding::CHECKS"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sealed-binding-history-and-next-stateful-proof",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &["Sealed-binding preflight repair:", "Previous ordinary bool-const checkpoint:", "Previous ordinary bool-const task checkpoint:", "Previous ordinary bool-const blocker checkpoint:", "Previous ordinary bool-const action checkpoint:", "Previous ordinary bool-const validation checkpoint:", "Previous ordinary bool-const artifact checkpoint:", "nuis-native-sealed-binding-preflight-v1.md", "actual prior bool-State consumption"],
    },
    DevTensorDriftCheckSpec {
        id: "native-sealed-binding-scoped-receipt",
        path: "docs/reference/nuis-native-sealed-binding-preflight-v1.md",
        required_patterns: &["## Reproduced Bypass", "pass-local admission gap", "64/65-node", "Seven unit tests", "no partial suffix helpers or names", "## Next Boundary", "whole-workspace"],
    },
];
