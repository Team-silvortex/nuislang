use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_DEV_LINEAGE_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "dev-tensor-task-card-recursive-lineage",
        path: "tools/nuis/src/dev_tensor_task_card_lineage.rs",
        required_patterns: &[
            "nuis-dev-tensor-task-card-lineage-v1",
            "validate_dev_tensor_task_card_lineage",
            "resolve_leaf_ancestry",
            "find_node_ancestry",
            "common_ancestor_index",
            "recursive_lineage_resolves_current_task_and_handoff_leaves",
            "direct_lineage_has_zero_transition_depth",
            "lineage_rejects_an_unreachable_handoff_leaf",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "dev-tensor-task-card-lineage-render",
        path: "tools/nuis/src/dev_tensor_task_card_lineage_render.rs",
        required_patterns: &[
            "weakest_bootstrap_task_card_lineage_protocol",
            "weakest_bootstrap_task_card_lineage_status",
            "weakest_bootstrap_task_card_lineage_errors",
            "weakest_bootstrap_task_card_task_ancestry",
            "weakest_bootstrap_task_card_handoff_ancestry",
            "weakest_bootstrap_task_card_common_ancestor_path",
            "weakest_bootstrap_task_card_transition_depth",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "dev-tensor-task-card-lineage-regression",
        path: "tools/nuis/src/dev_tensor_tests.rs",
        required_patterns: &[
            "nuis-dev-tensor-task-card-lineage-v1",
            "weakest_bootstrap_task_card_lineage_status",
            "weakest_bootstrap_task_card_lineage_errors",
            "weakest_bootstrap_task_card_task_ancestry",
            "weakest_bootstrap_task_card_handoff_ancestry",
            "weakest_bootstrap_task_card_transition_depth",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "dev-tensor-task-card-lineage-doc",
        path: "docs/reference/nuis-development-tensor.md",
        required_patterns: &[
            "nuis-dev-tensor-task-card-lineage-v1",
            "weakest_bootstrap_task_card_lineage_status",
            "weakest_bootstrap_task_card_task_ancestry",
            "weakest_bootstrap_task_card_handoff_ancestry",
            "weakest_bootstrap_task_card_common_ancestor_path",
            "weakest_bootstrap_task_card_transition_depth",
            "transition depth zero",
            "different reachable leaf",
        ],
    },
];
