use crate::artifact_nsdb_replay_cursor_lineage::DebuggerCursorLineageMirror;

pub(crate) const VALIDATED_PROVIDER_DISPATCH_IDENTITY_CAPABILITY_CONTRACT: &str =
    "nuis-validated-provider-dispatch-identity-capability-v1";

pub(crate) struct ValidatedProviderDispatchIdentityCapability {
    pub(crate) contract: &'static str,
    pub(crate) source_contract: String,
    pub(crate) source_status: String,
    pub(crate) ready: bool,
    pub(crate) status: &'static str,
    pub(crate) identity_hash: Option<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn validated_provider_dispatch_identity_capability(
    lineage: &DebuggerCursorLineageMirror,
) -> ValidatedProviderDispatchIdentityCapability {
    capability_from_validated_source(
        lineage.contract,
        lineage.status,
        lineage.ready,
        lineage.provider_dispatch_identity_hash.clone(),
        lineage.first_blocker,
    )
}

fn capability_from_validated_source(
    source_contract: &str,
    source_status: &str,
    source_ready: bool,
    identity_hash: Option<String>,
    source_blocker: Option<&str>,
) -> ValidatedProviderDispatchIdentityCapability {
    let ready = source_ready;
    ValidatedProviderDispatchIdentityCapability {
        contract: VALIDATED_PROVIDER_DISPATCH_IDENTITY_CAPABILITY_CONTRACT,
        source_contract: source_contract.to_owned(),
        source_status: source_status.to_owned(),
        ready,
        status: if !ready {
            "blocked"
        } else if identity_hash.is_some() {
            "verified"
        } else {
            "verified-empty"
        },
        identity_hash,
        first_blocker: (!ready).then(|| {
            source_blocker
                .unwrap_or("validated-provider-dispatch-identity-source-unavailable")
                .to_owned()
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_preserves_verified_identity_without_recomputing_it() {
        let capability = capability_from_validated_source(
            "nuis-debugger-cursor-lineage-mirror-v1",
            "lineage-ready",
            true,
            Some("0x0123456789abcdef".to_owned()),
            None,
        );

        assert!(capability.ready);
        assert_eq!(capability.status, "verified");
        assert_eq!(
            capability.identity_hash.as_deref(),
            Some("0x0123456789abcdef")
        );
        assert!(capability.first_blocker.is_none());
    }

    #[test]
    fn capability_distinguishes_verified_empty_from_blocked() {
        let empty = capability_from_validated_source(
            "nuis-debugger-cursor-lineage-mirror-v1",
            "lineage-ready",
            true,
            None,
            None,
        );
        let blocked = capability_from_validated_source(
            "nuis-debugger-cursor-lineage-mirror-v1",
            "lineage-invalid",
            false,
            None,
            Some("lineage-provider-dispatch-identity-mismatch"),
        );

        assert_eq!(empty.status, "verified-empty");
        assert!(empty.ready);
        assert_eq!(blocked.status, "blocked");
        assert!(!blocked.ready);
        assert_eq!(
            blocked.first_blocker.as_deref(),
            Some("lineage-provider-dispatch-identity-mismatch")
        );
    }
}
