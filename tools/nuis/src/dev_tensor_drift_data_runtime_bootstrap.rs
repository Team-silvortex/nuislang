use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_BOOTSTRAP_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "runtime-lifecycle-bootstrap-plan",
        path: "crates/nuis-runtime/src/lifecycle_bootstrap.rs",
        required_patterns: &[
            "nuis-runtime-lifecycle-bootstrap-plan-v1",
            "nuis-runtime-lifecycle-bootstrap-plan-identity-v1",
            "plan_lifecycle_bootstrap",
            "map-section",
            "apply-relocation",
            "bind-loader-entry",
            "bind-runtime-service",
            "runtime.clock-root",
            "runtime.glm-root",
            "enter-nuis-bootstrap",
            "bind-provider-dispatch",
            "activate-scheduler",
            "runtime-bootstrap:entry-relocation-target-mismatch",
            "runtime-bootstrap:mapped-section-count-mismatch",
            "runtime-bootstrap:applied-relocation-count-mismatch",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-runtime-service-bindings",
        path: "tools/nsld/src/container_metadata_bindings.rs",
        required_patterns: &[
            "runtime.clock-root",
            "runtime.glm-root",
            "clock_protocol_hash",
            "glm_binding_material",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsdb-runtime-only-binding-proof",
        path: "tools/nsdb/src/handoff_binding.rs",
        required_patterns: &[
            "provider_selection_absent",
            "provider_selection_verified",
            "claim_builder_accepts_verified_runtime_only_binding_table",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-runtime-only-binding-proof",
        path: "tools/nuis/src/artifact_nsdb_handoff_binding.rs",
        required_patterns: &[
            "provider_selection_absent",
            "provider_selection_verified",
            "independently_verifies_runtime_only_binding_proof",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "host-runner-runtime-bootstrap-adapter",
        path: "tools/nuis-host-runner/src/runtime_bootstrap.rs",
        required_patterns: &[
            "plan_lifecycle_bootstrap",
            "LifecycleBootstrapFacts",
            "MappedSectionFacts",
            "AppliedRelocationFacts",
            "runtime_service_binding_facts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-final-output-runtime-bootstrap-identity",
        path: "tools/nsld/src/final_executable_bootstrap.rs",
        required_patterns: &[
            "nsld_final_output_bootstrap_plan",
            "actual_relocation_patch_application_status",
            "actual_relocation_patch_byte_audit_status",
            "MappedSectionFacts",
            "AppliedRelocationFacts",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsdb-runtime-bootstrap-replay-identity",
        path: "tools/nsdb/src/lib.rs",
        required_patterns: &[
            "nuis-runtime-lifecycle-bootstrap-plan-identity-v1",
            "runtime_bootstrap_identity_status",
            "runtime-bootstrap-identity:invalid",
        ],
    },
];
