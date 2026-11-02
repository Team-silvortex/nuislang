use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-shared-proof-with-closed-ordinary-root",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs",
        required_patterns: &[
            "prove(value, depth, false, &mut work, catalog, None, &mut leaf)",
            "proof.ty == *expected && !proof.comparison_only",
            "mod epoch_local_plan;",
            "if !lazy_rhs || binding.uses != 0",
            "proof.comparison_only = true",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-definition-and-context",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_plan.rs",
        required_patterns: &[
            "scope.contains_key(name)", "definition.checked_add(1)?",
            "epoch: &'a Epoch", "body: &'a [NirStmt]", "catalog: &'a ScalarHelpers",
            "params.len() != 1", "binding.uses != 1",
            "initializer_may_fail: !initializer.total",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-original-field-and-shared-bounds",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_plan.rs",
        required_patterns: &[
            "epoch.fields.project_type(value, scope, layouts)?",
            "ready(value, depth).filter(|actual| actual == &expected)",
            "origins.insert(path, ty.clone())", "let mut work = 4096usize.checked_sub(3)?",
            "consumed.captures.len() > 31", "work: 4096 - work",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-bounded-scope-nonuse",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_scope.rs",
        required_patterns: &[
            "*work = work.checked_sub(1)?", "depth <= 32", "depth <= 64",
            "if bound != name", "if value != name", "if callee != name",
            "_ => return None", "work.saturating_sub(pending.len())",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-regression-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_plan_tests.rs",
        required_patterns: &[
            "epoch_local_plan_binds_original_fields_source_positions_and_same_epoch",
            "epoch_local_plan_keeps_fallible_initialization_eager_and_binding_reads_once_only",
            "epoch_local_plan_rejects_alias_reassignment_scope_and_noncomparison_escapes",
            "epoch_local_plan_rejects_uncertified_fields_children_and_effectful_helpers",
            "epoch_local_plan_shares_exact_work_depth_and_capture_budgets",
            "epoch_local_plan_bounds_whole_scope_nonuse_scan_without_panics",
            "epoch_local_plan_never_publishes_ordinary_availability_or_admission",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-exact-budget-controls",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_plan_tests.rs",
        required_patterns: &[
            "assert_eq!(check(&exact).unwrap().0, 4096)",
            "4097 shared expression work", "64/65 initializer depth",
            "31/32 combined captures", "locals.available.is_empty()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-plan-receipt",
        path: "docs/reference/nuis-native-epoch-local-plan-v1.md",
        required_patterns: &[
            "Analysis Only", "not called by the ordinary",
            "The caller must supply the actual complete block",
            "source `let mut` modifier", "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-epoch-local-plan-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Epoch-local initializer plan witnesses:",
            "Previous ready-local checkpoint:", "Previous ready-local task checkpoint:",
            "Previous ready-local blocker checkpoint:", "Previous ready-local action checkpoint:",
            "Previous ready-local validation checkpoint:", "Previous ready-local artifact checkpoint:",
        ],
    },
];
