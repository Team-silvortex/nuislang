use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-effect-prefix-admission-and-preparation-gates",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs",
        required_patterns: &[
            "mod effect_prefix;",
            "fn normalize_effects",
            "fn prepare_effects",
            "effect_prefix::verify_independent(function, &plan, layouts)?",
            "scalar_helpers::validate_iteration_body(function, &plan.body, catalog, layouts)?",
            "coalesced storage needs a different certificate",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-prefix-independent-structural-certificate",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_prefix.rs",
        required_patterns: &[
            "pub(super) fn verify_independent",
            "bounds::preflight(&function.body",
            "bounds::preflight(&plan.body",
            "names.contains(payload_name)",
            "before.leading != after.leading || before.step != after.step",
            "let right = self.block(else_body, right, in_loop)?",
            "self.propagation(output.next()?, in_loop)?",
            "output.next().is_none().then_some(returns)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-prefix-bounded-closed-vocabulary",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_prefix_bounds.rs",
        required_patterns: &[
            "const MAX_NODES: usize = 65_536",
            "const MAX_DEPTH: usize = 64",
            "body.len() > budget.0",
            "pending.len() >= budget.0",
            "budget.0.checked_sub(1)?",
            "Node::Type(ty)",
            "_ => return None",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-prefix-mutation-and-authority-regressions",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_prefix_tests.rs",
        required_patterns: &[
            "return_effect_prefix_certifies_all_existing_typed_selected_and_local_exit_matrices",
            "return_effect_prefix_rejects_removed_replayed_reordered_and_hoisted_prefix_work",
            "return_effect_prefix_rejects_early_pending_wrong_payload_and_changed_call_arguments",
            "return_effect_prefix_requires_each_immediate_scoped_pending_propagation",
            "return_effect_prefix_keeps_leading_trailing_steps_and_local_exits_exact",
            "return_effect_prefix_requires_fresh_independent_storage_and_exact_seed_types",
            "return_effect_prefix_bounds_fail_closed_without_mutating_or_granting_source",
            "return_effect_prefix_is_not_call_type_or_purity_admission",
            "inactive-arm",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-prefix-bounded-receipt",
        path: "docs/reference/nuis-native-return-effect-prefix-v1.md",
        required_patterns: &[
            "Separate Source And Rewrite Checks",
            "does not call the return rewriter",
            "65,536-node",
            "not a general",
            "Remaining Optimization Boundary",
            "explicitly rejects reused source storage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-effect-prefix-six-field-lineage",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Independent return effect-prefix certificate follow-up:",
            "Previous AOT-packer modularization checkpoint:",
            "Previous AOT-packer modularization task checkpoint:",
            "Previous AOT-packer modularization blocker checkpoint:",
            "Previous AOT-packer modularization action checkpoint:",
            "Previous AOT-packer modularization validation checkpoint:",
            "Previous AOT-packer modularization artifact checkpoint:",
        ],
    },
];
