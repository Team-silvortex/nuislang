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
    ];
