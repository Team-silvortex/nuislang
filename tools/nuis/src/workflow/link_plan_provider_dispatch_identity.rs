use crate::artifact_nsdb_handoff_dispatch::PersistedProviderDispatchIdentity;
use crate::artifact_nsdb_replay_cursor_lineage::DebuggerCursorLineageMirror;

pub(crate) const VALIDATED_PROVIDER_DISPATCH_IDENTITY_CAPABILITY_CONTRACT: &str =
    "nuis-validated-provider-dispatch-identity-capability-v1";
const LINEAGE_PROJECTION_SOURCE: &str = "debugger_cursor_lineage_provider_dispatch_identity_hash";
const FINAL_OUTPUT_PROJECTION_SOURCE: &str =
    "final_output_provider_completion_dispatch_identity_hash";

pub(crate) struct ValidatedProviderDispatchIdentityCapability {
    pub(crate) contract: &'static str,
    pub(crate) source_contract: String,
    pub(crate) source_status: String,
    pub(crate) projection_source: &'static str,
    pub(crate) ready: bool,
    pub(crate) status: &'static str,
    pub(crate) identity_hash: Option<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn validated_provider_dispatch_identity_capability(
    lineage: &DebuggerCursorLineageMirror,
    final_output_dispatch: &PersistedProviderDispatchIdentity,
) -> ValidatedProviderDispatchIdentityCapability {
    if !lineage.ready
        && lineage.status == "lineage-unavailable"
        && final_output_dispatch.status == "verified"
        && final_output_dispatch.identity_hash != "none"
    {
        return capability_from_validated_source(
            final_output_dispatch.contract.as_str(),
            final_output_dispatch.status.as_str(),
            FINAL_OUTPUT_PROJECTION_SOURCE,
            true,
            Some(final_output_dispatch.identity_hash.clone()),
            None,
        );
    }
    capability_from_validated_source(
        lineage.contract,
        lineage.status,
        LINEAGE_PROJECTION_SOURCE,
        lineage.ready,
        lineage.provider_dispatch_identity_hash.clone(),
        lineage.first_blocker,
    )
}

fn capability_from_validated_source(
    source_contract: &str,
    source_status: &str,
    projection_source: &'static str,
    source_ready: bool,
    identity_hash: Option<String>,
    source_blocker: Option<&str>,
) -> ValidatedProviderDispatchIdentityCapability {
    let ready = source_ready;
    ValidatedProviderDispatchIdentityCapability {
        contract: VALIDATED_PROVIDER_DISPATCH_IDENTITY_CAPABILITY_CONTRACT,
        source_contract: source_contract.to_owned(),
        source_status: source_status.to_owned(),
        projection_source,
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
    use crate::artifact_nsdb_handoff_dispatch::PersistedProviderDispatchIdentity;
    use crate::artifact_nsdb_replay_cursor_lineage::{
        DebuggerCursorLineageMirror, DebuggerCursorLineageRepairAction,
        DebuggerCursorLineageRepairMirror,
    };

    #[test]
    fn capability_preserves_verified_identity_without_recomputing_it() {
        let capability = capability_from_validated_source(
            "nuis-debugger-cursor-lineage-mirror-v1",
            "lineage-ready",
            LINEAGE_PROJECTION_SOURCE,
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
            LINEAGE_PROJECTION_SOURCE,
            true,
            None,
            None,
        );
        let blocked = capability_from_validated_source(
            "nuis-debugger-cursor-lineage-mirror-v1",
            "lineage-invalid",
            LINEAGE_PROJECTION_SOURCE,
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

    #[test]
    fn capability_can_fallback_to_verified_final_output_dispatch_before_cursor_lineage_exists() {
        let capability = validated_provider_dispatch_identity_capability(
            &unavailable_lineage(),
            &PersistedProviderDispatchIdentity {
                contract: "nuis-provider-completion-dispatch-authority-v1".to_owned(),
                status: "verified".to_owned(),
                table_hash: "0xaaaabbbbccccdddd".to_owned(),
                selected_set_hash: "fnv1a64:f8efa211643f7bcd".to_owned(),
                identity_hash: "0xfedcba9876543210".to_owned(),
            },
        );

        assert!(capability.ready);
        assert_eq!(capability.status, "verified");
        assert_eq!(capability.projection_source, FINAL_OUTPUT_PROJECTION_SOURCE);
        assert_eq!(
            capability.identity_hash.as_deref(),
            Some("0xfedcba9876543210")
        );
    }

    fn unavailable_lineage() -> DebuggerCursorLineageMirror {
        DebuggerCursorLineageMirror {
            contract: "nuis-debugger-cursor-lineage-mirror-v1",
            source_protocol: "nsdb-yir-replay-cursor-lineage-v2",
            path: "out/nuis.nsdb.replay-cursor.lineage.toml".to_owned(),
            ready: false,
            status: "lineage-unavailable",
            entry_count: 0,
            latest_hash: None,
            provider_dispatch_identity_hash: None,
            first_blocker: None,
            next_action: None,
            next_command: None,
            repair: DebuggerCursorLineageRepairMirror {
                contract: "nuis-debugger-cursor-lineage-repair-mirror-v1",
                path: "out/nuis.nsdb.replay-cursor.lineage-repairs.toml".to_owned(),
                status: "repair-history-unavailable",
                entry_count: 0,
                rotation_generation: None,
                evicted_prefix_hash: None,
                window_hash: None,
                latest_mutated: None,
                latest_event_status: None,
                latest_lineage_mutated: None,
                latest_repair_journal_mutated: None,
                latest_archived_path: None,
                latest_archived_hash: None,
                latest_archived_repair_journal_path: None,
                latest_archived_repair_journal_hash: None,
                latest_rebuilt_hash: None,
                action: DebuggerCursorLineageRepairAction {
                    first_blocker: None,
                    next_action: None,
                    next_command: None,
                },
            },
        }
    }
}
