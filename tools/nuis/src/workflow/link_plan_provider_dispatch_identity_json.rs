use super::link_plan_provider_dispatch_identity::ValidatedProviderDispatchIdentityCapability;
use crate::{json_bool_field, json_field, json_optional_string_field};

pub(super) fn provider_dispatch_identity_capability_json_fields(
    capability: Option<&ValidatedProviderDispatchIdentityCapability>,
) -> Vec<String> {
    let mut fields = Vec::new();
    for prefix in [
        "nsld_final_executable_output_object_package",
        "nsld_final_executable_output_debugger_api",
    ] {
        fields.extend([
            json_optional_string_field(
                &format!("{prefix}_provider_dispatch_identity_capability_contract"),
                capability.map(|capability| capability.contract),
            ),
            json_bool_field(
                &format!("{prefix}_provider_dispatch_identity_ready"),
                capability.is_some_and(|capability| capability.ready),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_dispatch_identity_status"),
                capability.map(|capability| capability.status),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_dispatch_identity_source_contract"),
                capability.map(|capability| capability.source_contract.as_str()),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_dispatch_identity_source_status"),
                capability.map(|capability| capability.source_status.as_str()),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_dispatch_identity_hash"),
                capability.and_then(|capability| capability.identity_hash.as_deref()),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_dispatch_identity_first_blocker"),
                capability.and_then(|capability| capability.first_blocker.as_deref()),
            ),
        ]);
    }
    fields.push(json_field(
        "nsld_final_executable_output_provider_dispatch_identity_projection_source",
        "debugger_cursor_lineage_provider_dispatch_identity_hash",
    ));
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_and_debugger_projections_share_one_validated_identity() {
        let capability = ValidatedProviderDispatchIdentityCapability {
            contract: "nuis-validated-provider-dispatch-identity-capability-v1",
            source_contract: "nuis-debugger-cursor-lineage-mirror-v1".to_owned(),
            source_status: "lineage-ready".to_owned(),
            ready: true,
            status: "verified",
            identity_hash: Some("0x0123456789abcdef".to_owned()),
            first_blocker: None,
        };

        let fields = provider_dispatch_identity_capability_json_fields(Some(&capability));

        assert!(fields.contains(&"\"nsld_final_executable_output_object_package_provider_dispatch_identity_hash\":\"0x0123456789abcdef\"".to_owned()));
        assert!(fields.contains(&"\"nsld_final_executable_output_debugger_api_provider_dispatch_identity_hash\":\"0x0123456789abcdef\"".to_owned()));
        assert!(fields.contains(&"\"nsld_final_executable_output_provider_dispatch_identity_projection_source\":\"debugger_cursor_lineage_provider_dispatch_identity_hash\"".to_owned()));
    }
}
