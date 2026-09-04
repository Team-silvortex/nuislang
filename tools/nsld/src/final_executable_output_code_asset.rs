pub(crate) fn validate_compiled_code_asset_selections(
    completions: &[nsdb::PayloadExecutionProviderCompletion],
) -> Result<(), String> {
    for completion in completions {
        let selection = &completion.compiled_code_asset_selection;
        if selection == &nsdb::CompiledCodeAssetSelectionEvidence::default() {
            continue;
        }
        let selected_count = selection.selections.len();
        let contract_valid = if selected_count == 1 {
            matches!(
                selection.contract.as_str(),
                "nuis-provider-code-asset-contribution-selection-v1"
                    | "nuis-provider-code-asset-contribution-selection-set-v1"
            )
        } else {
            selected_count > 1
                && selection.contract == "nuis-provider-code-asset-contribution-selection-set-v1"
        };
        let unique = selection
            .selections
            .iter()
            .map(|item| item.contribution_index)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == selected_count;
        let alias_valid = selection.selections.first().is_some_and(|first| {
            first.contribution_index == selection.contribution_index
                && first.asset_id == selection.asset_id
                && first.identity_hash == selection.identity_hash
        });
        if !contract_valid
            || selection.status != "verified"
            || selection.table_contract != "nuis-domain-code-asset-contribution-table-v1"
            || selection.contribution_count == 0
            || selection.contribution_count > 64
            || selection.contribution_index >= selection.contribution_count
            || !(1..=64).contains(&selected_count)
            || !unique
            || !alias_valid
            || selection.selections.iter().any(|item| {
                item.contribution_index >= selection.contribution_count
                    || !valid_token(&item.asset_id)
                    || !valid_hash(&item.identity_hash)
            })
            || !valid_hash(&selection.table_hash)
            || !valid_hash(&selection.identity_set_root_hash)
            || !valid_token(&selection.asset_id)
            || !valid_hash(&selection.identity_hash)
        {
            return Err(format!(
                "compiled-code-asset-selection-invalid:{}",
                completion.trace_id
            ));
        }
    }
    Ok(())
}

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_verified_or_not_applicable_selection() {
        let mut completion = completion();
        assert!(validate_compiled_code_asset_selections(&[completion.clone()]).is_ok());
        completion.compiled_code_asset_selection.contract =
            "nuis-provider-code-asset-contribution-selection-set-v1".to_owned();
        assert!(validate_compiled_code_asset_selections(&[completion.clone()]).is_ok());
        completion.compiled_code_asset_selection =
            nsdb::CompiledCodeAssetSelectionEvidence::default();
        assert!(validate_compiled_code_asset_selections(&[completion.clone()]).is_ok());
        completion.compiled_code_asset_selection.status = "verified".to_owned();
        assert!(validate_compiled_code_asset_selections(&[completion]).is_err());
    }

    fn completion() -> nsdb::PayloadExecutionProviderCompletion {
        nsdb::PayloadExecutionProviderCompletion {
            trace_id: "trace.cuda".to_owned(),
            provider_family: "cuda:nvidia-gpu".to_owned(),
            output_contract: "output".to_owned(),
            output_evidence: "output.toml".to_owned(),
            completion_evidence_contract: "completion".to_owned(),
            completion_evidence_status: "verified".to_owned(),
            completion_evidence_count: 1,
            completion_clock_evidence: "clock".to_owned(),
            completion_tokens: "token".to_owned(),
            glm_release_contract: "glm".to_owned(),
            glm_release_tokens: "release".to_owned(),
            glm_release_status: "released".to_owned(),
            code_asset_identity_contract: "semantic".to_owned(),
            code_asset_identity_status: "verified".to_owned(),
            code_asset_identity_asset_id: "kernel.cuda.project.test".to_owned(),
            code_asset_identity_hash: "0x0123456789abcdef".to_owned(),
            code_asset_identity_set_contract: "set".to_owned(),
            code_asset_identity_set_status: "verified".to_owned(),
            code_asset_identity_set_count: 1,
            code_asset_identity_set_root_hash: "0x0123456789abcdef".to_owned(),
            conformance: Default::default(),
            compiled_code_asset_selection: nsdb::CompiledCodeAssetSelectionEvidence {
                contract: "nuis-provider-code-asset-contribution-selection-v1".to_owned(),
                status: "verified".to_owned(),
                table_contract: "nuis-domain-code-asset-contribution-table-v1".to_owned(),
                table_hash: "0x1111111111111111".to_owned(),
                contribution_count: 1,
                identity_set_root_hash: "0x2222222222222222".to_owned(),
                contribution_index: 0,
                asset_id: "kernel.cuda.project.test".to_owned(),
                identity_hash: "0x3333333333333333".to_owned(),
                selections: vec![nsdb::CompiledCodeAssetSelectionItem {
                    contribution_index: 0,
                    asset_id: "kernel.cuda.project.test".to_owned(),
                    identity_hash: "0x3333333333333333".to_owned(),
                }],
            },
            request_completion_contract: "request-completion".to_owned(),
            request_completion_status: "not-applicable".to_owned(),
            request_completion_count: 0,
            request_completion_root_hash: "none".to_owned(),
            request_completions: Vec::new(),
            dispatch_authority_contract: "dispatch".to_owned(),
            dispatch_authority_status: "verified".to_owned(),
            dispatch_table_hash: "table".to_owned(),
            dispatch_selected_set_hash: "set".to_owned(),
            dispatch_id: "dispatch".to_owned(),
            dispatch_package_id: "official.kernel".to_owned(),
            dispatch_bundle_id: "bundle".to_owned(),
            dispatch_runner_adapter_id: "adapter".to_owned(),
            record_hash: "record".to_owned(),
        }
    }
}
