#[derive(Clone)]
pub(crate) struct ProviderCompletionClosureMirror {
    pub(crate) count: usize,
    pub(crate) family: Option<String>,
    pub(crate) output_contract: Option<String>,
    pub(crate) output_evidence: Option<String>,
    pub(crate) claim_authority_contract: Option<String>,
    pub(crate) claim_authority: Option<String>,
    pub(crate) claim_authority_status: String,
    pub(crate) signature_contract: Option<String>,
    pub(crate) signature_public_key_id: Option<String>,
    pub(crate) signature_status: String,
    pub(crate) digest_contract: Option<String>,
    pub(crate) set_hash_claim: Option<String>,
    pub(crate) set_hash: Option<String>,
    pub(crate) set_hash_validation_status: String,
    pub(crate) records: Vec<ProviderCompletionRecordClosureMirror>,
}

#[derive(Clone)]
pub(crate) struct ProviderCompletionRecordClosureMirror {
    pub(crate) trace_id: String,
    pub(crate) provider_family: String,
    pub(crate) output_contract: String,
    pub(crate) output_evidence: String,
    pub(crate) record_hash: String,
}

impl ProviderCompletionClosureMirror {
    pub(crate) fn from_final_output(
        final_output: &crate::workflow::NsldFinalExecutableOutputBoundarySummary,
    ) -> Option<Self> {
        (final_output.nsdb_provider_completion_count > 0).then(|| Self {
            count: final_output.nsdb_provider_completion_count,
            family: final_output.nsdb_first_provider_family.clone(),
            output_contract: final_output.nsdb_first_provider_output_contract.clone(),
            output_evidence: final_output.nsdb_first_provider_output_evidence.clone(),
            claim_authority_contract: final_output
                .nsdb_provider_completion_claim_authority_contract
                .clone(),
            claim_authority: final_output
                .nsdb_provider_completion_claim_authority
                .clone(),
            claim_authority_status: final_output
                .nsdb_provider_completion_claim_authority_status
                .clone(),
            signature_contract: final_output
                .nsdb_provider_completion_signature_contract
                .clone(),
            signature_public_key_id: final_output
                .nsdb_provider_completion_signature_public_key_id
                .clone(),
            signature_status: final_output
                .nsdb_provider_completion_signature_status
                .clone(),
            digest_contract: final_output
                .nsdb_provider_completion_digest_contract
                .clone(),
            set_hash_claim: final_output.nsdb_provider_completion_set_hash_claim.clone(),
            set_hash: final_output.nsdb_provider_completion_set_hash.clone(),
            set_hash_validation_status: final_output
                .nsdb_provider_completion_set_hash_validation_status
                .clone(),
            records: final_output
                .nsdb_provider_completions
                .iter()
                .map(|completion| ProviderCompletionRecordClosureMirror {
                    trace_id: completion.trace_id.clone(),
                    provider_family: completion.provider_family.clone(),
                    output_contract: completion.output_contract.clone(),
                    output_evidence: completion.output_evidence.clone(),
                    record_hash: completion.record_hash.clone(),
                })
                .collect(),
        })
    }
}
