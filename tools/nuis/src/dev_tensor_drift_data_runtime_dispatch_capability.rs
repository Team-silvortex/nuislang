use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_DISPATCH_CAPABILITY_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] =
    &[
        DevTensorDriftCheckSpec {
            id: "validated-provider-dispatch-identity-capability",
            path: "tools/nuis/src/workflow/link_plan_provider_dispatch_identity.rs",
            required_patterns: &[
                "nuis-validated-provider-dispatch-identity-capability-v1",
                "validated_provider_dispatch_identity_capability",
                "capability_from_validated_source",
                "\"verified\"",
                "\"verified-empty\"",
                "\"blocked\"",
                "capability_preserves_verified_identity_without_recomputing_it",
                "capability_distinguishes_verified_empty_from_blocked",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "provider-dispatch-identity-package-debugger-projections",
            path: "tools/nuis/src/workflow/link_plan_provider_dispatch_identity_json.rs",
            required_patterns: &[
                "nsld_final_executable_output_object_package",
                "nsld_final_executable_output_debugger_api",
                "provider_dispatch_identity_capability_contract",
                "provider_dispatch_identity_hash",
                "debugger_cursor_lineage_provider_dispatch_identity_hash",
                "package_and_debugger_projections_share_one_validated_identity",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "provider-dispatch-identity-closure-projections",
            path: "tools/nuis/src/closure_summary_dispatch_identity.rs",
            required_patterns: &[
                "ProviderDispatchIdentityClosureMirror",
                "closure_summary_object_package",
                "closure_summary_debugger_api",
                "provider_dispatch_identity_capability_contract",
                "debugger_cursor_lineage_provider_dispatch_identity_hash",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "provider-dispatch-identity-workflow-closure-integration",
            path: "tools/nuis/src/workflow/link_plan_json.rs",
            required_patterns: &[
                "ProviderDispatchIdentityClosureMirror::json_fields",
                "closure_summary.provider_dispatch_identity.as_ref()",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "provider-dispatch-identity-closure-regression",
            path: "tools/nuis/src/closure_summary_tests.rs",
            required_patterns: &[
                "closure_summary_object_package_provider_dispatch_identity_hash",
                "closure_summary_debugger_api_provider_dispatch_identity_hash",
                "closure_summary_provider_dispatch_identity_projection_source",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "provider-dispatch-identity-official-heterogeneous-proof",
            path: "tools/nuis/tests/official_galaxy_hetero_smoke/replay.rs",
            required_patterns: &[
                "replay cursor lineage provider dispatch identity hash",
                "nsld_final_executable_output_object_package_provider_dispatch_identity_hash",
                "nsld_final_executable_output_debugger_api_provider_dispatch_identity_hash",
                "closure_summary_object_package_provider_dispatch_identity_hash",
                "closure_summary_debugger_api_provider_dispatch_identity_hash",
                "debugger_cursor_lineage_provider_dispatch_identity_hash",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "provider-dispatch-identity-capability-doc",
            path: "docs/reference/nuis-development-tensor.md",
            required_patterns: &[
                "nuis-validated-provider-dispatch-identity-capability-v1",
                "verified-empty",
                "object_package_provider_dispatch_identity",
                "debugger_api_provider_dispatch_identity",
                "debugger_cursor_lineage_provider_dispatch_identity_hash",
            ],
        },
    ];
