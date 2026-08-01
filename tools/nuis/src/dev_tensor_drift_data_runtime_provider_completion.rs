use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_PROVIDER_COMPLETION_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] =
    &[
        DevTensorDriftCheckSpec {
            id: "nuis-provider-completion-closure-mirror",
            path: "tools/nuis/src/closure_summary_provider_completion.rs",
            required_patterns: &[
                "ProviderCompletionClosureMirror",
                "ProviderCompletionRecordClosureMirror",
                "from_final_output",
                "nsdb_provider_completions",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "nuis-provider-completion-closure-json",
            path: "tools/nuis/src/closure_summary_provider_completion.rs",
            required_patterns: &[
                "provider_completion_json_fields",
                "closure_summary_provider_completion_count",
                "closure_summary_first_provider_family",
                "closure_summary_first_provider_output_contract",
                "closure_summary_first_provider_output_evidence",
                "closure_summary_provider_completion_claim_authority_contract",
                "closure_summary_provider_completion_claim_authority",
                "closure_summary_provider_completion_claim_authority_status",
                "closure_summary_provider_completion_digest_contract",
                "closure_summary_provider_completion_set_hash_claim",
                "closure_summary_provider_completion_set_hash",
                "closure_summary_provider_completion_set_hash_validation_status",
                "closure_summary_provider_completions",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "nuis-provider-completion-workflow-projection",
            path: "tools/nuis/src/workflow/link_plan_json.rs",
            required_patterns: &["append_projection_json_fields"],
        },
        DevTensorDriftCheckSpec {
            id: "nsdb-provider-request-completion-receipts",
            path: "tools/nsdb/src/provider_request_completion.rs",
            required_patterns: &[
                "nuis-provider-request-completion-receipt-collection-v1",
                "bind_final_image_dispatch",
                "request_completion_root_hash",
                "request-completion-dispatch:entry-missing",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "nuis-independent-request-completion-audit",
            path: "tools/nuis/src/artifact_nsdb_handoff_request_completion.rs",
            required_patterns: &[
                "parse_and_append",
                "receipt_root_hash",
                "receipts_verified",
                "legacy-unavailable",
            ],
        },
        DevTensorDriftCheckSpec {
            id: "provider-request-completion-real-mixed-smoke",
            path: "tools/nuis/tests/official_galaxy_hetero_smoke/linux_vulkan.rs",
            required_patterns: &[
                "request_completion_count",
                "request_completion_{index}_request_id",
                "request_completion_{index}_provider_family",
                "request_completion_{index}_dispatch_id",
                "request_completion_{index}_selected_set_hash",
            ],
        },
    ];
