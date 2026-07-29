use crate::workflow::NsldFinalExecutableOutputBoundarySummary;

#[derive(Clone)]
pub(crate) struct ProviderDispatchIdentityClosureMirror {
    contract: &'static str,
    source_contract: String,
    source_status: String,
    projection_source: &'static str,
    ready: bool,
    status: &'static str,
    identity_hash: Option<String>,
    first_blocker: Option<String>,
}

impl ProviderDispatchIdentityClosureMirror {
    pub(crate) fn from_final_output(
        final_output: &NsldFinalExecutableOutputBoundarySummary,
    ) -> Self {
        let capability = &final_output.provider_dispatch_identity_capability;
        Self {
            contract: capability.contract,
            source_contract: capability.source_contract.clone(),
            source_status: capability.source_status.clone(),
            projection_source: capability.projection_source,
            ready: capability.ready,
            status: capability.status,
            identity_hash: capability.identity_hash.clone(),
            first_blocker: capability.first_blocker.clone(),
        }
    }

    pub(crate) fn json_fields(mirror: Option<&Self>) -> Vec<String> {
        let mut fields = Vec::new();
        for prefix in [
            "closure_summary_object_package",
            "closure_summary_debugger_api",
        ] {
            fields.extend([
                crate::json_optional_string_field(
                    &format!("{prefix}_provider_dispatch_identity_capability_contract"),
                    mirror.map(|mirror| mirror.contract),
                ),
                crate::json_optional_bool_field(
                    &format!("{prefix}_provider_dispatch_identity_ready"),
                    mirror.map(|mirror| mirror.ready),
                ),
                crate::json_optional_string_field(
                    &format!("{prefix}_provider_dispatch_identity_status"),
                    mirror.map(|mirror| mirror.status),
                ),
                crate::json_optional_string_field(
                    &format!("{prefix}_provider_dispatch_identity_source_contract"),
                    mirror.map(|mirror| mirror.source_contract.as_str()),
                ),
                crate::json_optional_string_field(
                    &format!("{prefix}_provider_dispatch_identity_source_status"),
                    mirror.map(|mirror| mirror.source_status.as_str()),
                ),
                crate::json_optional_string_field(
                    &format!("{prefix}_provider_dispatch_identity_hash"),
                    mirror.and_then(|mirror| mirror.identity_hash.as_deref()),
                ),
                crate::json_optional_string_field(
                    &format!("{prefix}_provider_dispatch_identity_first_blocker"),
                    mirror.and_then(|mirror| mirror.first_blocker.as_deref()),
                ),
            ]);
        }
        fields.push(crate::json_field(
            "closure_summary_provider_dispatch_identity_projection_source",
            mirror
                .map(|mirror| mirror.projection_source)
                .unwrap_or("debugger_cursor_lineage_provider_dispatch_identity_hash"),
        ));
        fields
    }
}
