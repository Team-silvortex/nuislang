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

pub(crate) fn provider_completion_json_fields(
    mirror: Option<&ProviderCompletionClosureMirror>,
) -> Vec<String> {
    let provider_records = mirror
        .map(|mirror| {
            mirror
                .records
                .iter()
                .map(|record| {
                    format!(
                        "{{{},{},{},{},{}}}",
                        crate::json_field("trace_id", &record.trace_id),
                        crate::json_field("provider_family", &record.provider_family),
                        crate::json_field("output_contract", &record.output_contract),
                        crate::json_field("output_evidence", &record.output_evidence),
                        crate::json_field("record_hash", &record.record_hash),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    vec![
        json_optional_usize_field(
            "closure_summary_provider_completion_count",
            mirror.map(|mirror| mirror.count),
        ),
        crate::json_optional_string_field(
            "closure_summary_first_provider_family",
            mirror.and_then(|mirror| mirror.family.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_first_provider_output_contract",
            mirror.and_then(|mirror| mirror.output_contract.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_first_provider_output_evidence",
            mirror.and_then(|mirror| mirror.output_evidence.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_claim_authority_contract",
            mirror.and_then(|mirror| mirror.claim_authority_contract.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_claim_authority",
            mirror.and_then(|mirror| mirror.claim_authority.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_claim_authority_status",
            mirror.map(|mirror| mirror.claim_authority_status.as_str()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_signature_contract",
            mirror.and_then(|mirror| mirror.signature_contract.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_signature_public_key_id",
            mirror.and_then(|mirror| mirror.signature_public_key_id.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_signature_status",
            mirror.map(|mirror| mirror.signature_status.as_str()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_digest_contract",
            mirror.and_then(|mirror| mirror.digest_contract.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_set_hash_claim",
            mirror.and_then(|mirror| mirror.set_hash_claim.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_set_hash",
            mirror.and_then(|mirror| mirror.set_hash.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_provider_completion_set_hash_validation_status",
            mirror.map(|mirror| mirror.set_hash_validation_status.as_str()),
        ),
        crate::json_object_array_field("closure_summary_provider_completions", &provider_records),
    ]
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => format!("\"{name}\":{value}"),
        None => format!("\"{name}\":null"),
    }
}
