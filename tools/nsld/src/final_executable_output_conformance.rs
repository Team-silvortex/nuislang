pub(crate) fn validate_provider_conformance_evidence(
    completions: &[nsdb::PayloadExecutionProviderCompletion],
) -> Result<(), String> {
    for completion in completions {
        validate_completion(&completion.provider_family, &completion.conformance).map_err(
            |reason| {
                format!(
                    "provider-conformance-lifecycle-invalid:{}:{reason}",
                    completion.trace_id
                )
            },
        )?;
    }
    Ok(())
}

fn validate_completion(
    provider_family: &str,
    evidence: &nsdb::ProviderConformanceLifecycleEvidence,
) -> Result<(), &'static str> {
    if evidence.physical_execution_claimed {
        return Err("physical-execution-claim-forbidden");
    }
    if evidence.status == "not-applicable" {
        return if evidence == &Default::default() {
            Ok(())
        } else {
            Err("not-applicable-evidence-not-empty")
        };
    }
    if evidence.status != "verified" || evidence.replay_status != "verified" {
        return Err("status-invalid");
    }
    if evidence.capsule_contract != nsdb::PROVIDER_CONFORMANCE_CAPSULE_CONTRACT
        || evidence.replay_contract != nsdb::PROVIDER_CONFORMANCE_REPLAY_CONTRACT
    {
        return Err("contract-invalid");
    }
    if evidence.execution_authority != nsdb::PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY {
        return Err("authority-invalid");
    }
    if evidence.provider_family != provider_family {
        return Err("provider-family-mismatch");
    }
    for identity in [
        evidence.scenario_contract.as_str(),
        evidence.scenario_id.as_str(),
        evidence.package_id.as_str(),
        evidence.provider_id.as_str(),
        evidence.bundle_id.as_str(),
        evidence.provider_family.as_str(),
    ] {
        if !valid_identity(identity) {
            return Err("identity-invalid");
        }
    }
    for hash in [
        evidence.capability_selection_hash.as_str(),
        evidence.capsule_hash.as_str(),
        evidence.replay_hash.as_str(),
    ] {
        if !valid_fnv1a64(hash) {
            return Err("hash-invalid");
        }
    }
    Ok(())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value != "none"
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
}

fn valid_fnv1a64(value: &str) -> bool {
    value
        .strip_prefix("fnv1a64:")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::validate_completion;

    fn evidence() -> nsdb::ProviderConformanceLifecycleEvidence {
        nsdb::ProviderConformanceLifecycleEvidence {
            capsule_contract: nsdb::PROVIDER_CONFORMANCE_CAPSULE_CONTRACT.to_owned(),
            status: "verified".to_owned(),
            scenario_contract: "nuis-data-reference-copy-conformance-v1".to_owned(),
            scenario_id: "data.copy.binary-octets.v1".to_owned(),
            package_id: "official.data".to_owned(),
            provider_id: "data.cpu-memory.reference.v1".to_owned(),
            bundle_id: "data.host.bundle.v1".to_owned(),
            provider_family: "data:host".to_owned(),
            capability_selection_hash: "fnv1a64:6d712122a1132927".to_owned(),
            capsule_hash: "fnv1a64:82270e31b99f2c0b".to_owned(),
            replay_contract: nsdb::PROVIDER_CONFORMANCE_REPLAY_CONTRACT.to_owned(),
            replay_status: "verified".to_owned(),
            replay_hash: "fnv1a64:7ee93c8f8a4ae011".to_owned(),
            execution_authority: nsdb::PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY.to_owned(),
            physical_execution_claimed: false,
        }
    }

    #[test]
    fn accepts_verified_or_not_applicable_conformance() {
        assert!(validate_completion("data:host", &evidence()).is_ok());
        assert!(validate_completion("metal:gpu", &Default::default()).is_ok());

        let hidden_claim = nsdb::ProviderConformanceLifecycleEvidence {
            execution_authority: "execution-authority".to_owned(),
            ..Default::default()
        };
        assert_eq!(
            validate_completion("metal:gpu", &hidden_claim),
            Err("not-applicable-evidence-not-empty")
        );
    }

    #[test]
    fn rejects_authority_family_hash_and_physical_claim_drift() {
        let mut candidate = evidence();
        candidate.execution_authority = "execution-authority".to_owned();
        assert_eq!(
            validate_completion("data:host", &candidate),
            Err("authority-invalid")
        );

        let mut candidate = evidence();
        candidate.provider_family = "data:device".to_owned();
        assert_eq!(
            validate_completion("data:host", &candidate),
            Err("provider-family-mismatch")
        );

        let mut candidate = evidence();
        candidate.replay_hash = "fnv1a64:short".to_owned();
        assert_eq!(
            validate_completion("data:host", &candidate),
            Err("hash-invalid")
        );

        let mut candidate = evidence();
        candidate.physical_execution_claimed = true;
        assert_eq!(
            validate_completion("data:host", &candidate),
            Err("physical-execution-claim-forbidden")
        );
    }
}
