use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-entry",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects.rs",
        required_patterns: &[
            "f.body.iter().any(|stmt| matches!(stmt, NirStmt::If { .. }))",
            "filter(|name| catalog.contains_key(name))",
            "retained.difference(&outlined.functions)",
            "Some(&loop_roots)",
            "layouts.contains_key(&ty.name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-source",
        path: "tools/nuisc/tests/native_application_bridge/state_return_direct_eager_pair.ns",
        required_patterns: &[
            "return work(0,",
            "return work(state.control,",
            "return State { total: 0.0, control: value + delta }",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-admission",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_direct_tests.rs",
        required_patterns: &[
            "direct_effectful_records_normalize_without_loop_or_pure_authority",
            "direct_effectful_records_keep_dependency_scope_and_type_rejections",
            "!control_loops::contains_loop(&f.body)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-execution",
        path: "tools/nuisc/tests/native_application_bridge/state_return_eager_pair.rs",
        required_patterns: &[
            "state_return_direct_eager_pair_keeps_eager_order_lazy_children_and_complete_state",
            "state_return_direct_eager_pair_keeps_explicit_print_and_entry_authority",
            "check_source(&source, 0, word)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-model",
        path: "tools/nuisc/tests/native_application_bridge/state_return_eager_pair_model.rs",
        required_patterns: &[
            "pub fn execute",
            "\"==\" => left == right",
            "\"!=\" => left != right",
            "\"^\" => left ^ right",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-delivery",
        path: "tools/nuis/tests/native_session_workflow/state_return_direct_eager_pair.rs",
        required_patterns: &[
            "state_return_direct_eager_pair_fresh_cache_and_source_free_delivery",
            "project_for_source_with_i64_prints(&source, 0, 192)",
            "fs::remove_dir_all(project.0.join(\".nuis\"))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-receipt",
        path: "docs/reference/nuis-native-direct-effectful-records-v1.md",
        required_patterns: &[
            "Direct Effectful Record Returns",
            "Predecessor Lineage",
            "not Linux/GPU parity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-direct-effectful-records-lineage",
        path: "tools/nuis/src/dev_tensor_native_record_lineage_tests.rs",
        required_patterns: &[
            "dev_tensor_direct_effectful_records_retains_complete_eager_lineage",
            "NUIS_DIRECT_EFFECTFUL_RECORDS_20261113_EAGER_HISTORY",
        ],
    },
];
