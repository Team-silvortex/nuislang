use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "tensor-recalibration-native-resource-plan",
        path: "docs/reference/nuis-development-tensor.mainline.toml",
        required_patterns: &[
            "compiler-toolchain/nuisc/native-resource-session-callbacks",
            "compiler-toolchain/nuisc/native-scalar-record-session-profile",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "tensor-recalibration-drift-readiness",
        path: "tools/nuis/src/dev_tensor.rs",
        required_patterns: &[
            "dev_tensor_summary_with_drift(dev_tensor_drift_summary().status)",
            "task card blocked by evidence drift",
            "&& drift_status == \"clean\"",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "tensor-recalibration-scope-and-lineage",
        path: "tools/nuis/src/dev_tensor_recalibration_tests.rs",
        required_patterns: &[
            "dev_tensor_recalibration_drift_blocks_ready_cards_without_changing_the_frontier",
            "dev_tensor_recalibration_keeps_scopes_and_preparation_separate",
            "dev_tensor_recalibration_preserves_all_six_complete_direct_entry_fields",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "tensor-recalibration-dependency-sequence",
        path: "tools/nuis/src/dev_tensor_mainline_tests.rs",
        required_patterns: &[
            "mainline_recalibration_advances_resources_frames_sessions_then_failure_safety",
            "compiler-toolchain/nuisc/native-scalar-record-session-profile",
            "compiler-toolchain/nuisc/native-resource-session-callbacks",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "tensor-recalibration-native-resource-rejection",
        path: "crates/yir-lower-llvm/src/native_session/resource_boundary_tests.rs",
        required_patterns: &[
            "native_session_resource_boundary_is_not_implied_by_scalar_record_admission",
            "alloc_buffer",
            "drop_owned_bytes",
            "ScalarStateLayout::parse",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "tensor-recalibration-audit",
        path: "docs/reference/nuis-development-tensor-recalibration-2026-11-13.md",
        required_patterns: &[
            "## Inventory Review",
            "early/0",
            "baseline safety gate",
            "All six complete preceding persistent-session fields",
            "does not rerun GPU hardware",
        ],
    },
];
