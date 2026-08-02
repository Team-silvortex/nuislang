use super::link_plan_final_output_summary::NsldFinalExecutableOutputBoundarySummary;
use super::link_plan_provider_request_completion::ValidatedProviderRequestCompletionCapability;
use crate::{
    json_bool_field, json_field, json_object_array_field, json_optional_string_field,
    json_usize_field,
};

pub(super) fn final_output_provider_request_completion_json_fields(
    final_output: Option<&NsldFinalExecutableOutputBoundarySummary>,
) -> Vec<String> {
    provider_request_completion_capability_json_fields(
        final_output.map(|summary| &summary.provider_request_completion_capability),
        &[
            "nsld_final_executable_output_object_package",
            "nsld_final_executable_output_debugger_api",
        ],
        "nsld_final_executable_output_provider_request_completion_projection_source",
    )
}

pub(crate) fn provider_request_completion_capability_json_fields(
    capability: Option<&ValidatedProviderRequestCompletionCapability>,
    prefixes: &[&str],
    projection_source_field: &str,
) -> Vec<String> {
    let collections = collection_records(capability);
    let mut fields = Vec::new();
    for prefix in prefixes {
        fields.extend([
            json_optional_string_field(
                &format!("{prefix}_provider_request_completion_capability_contract"),
                capability.map(|capability| capability.contract),
            ),
            json_bool_field(
                &format!("{prefix}_provider_request_completion_ready"),
                capability.is_some_and(|capability| capability.ready),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_request_completion_status"),
                capability.map(|capability| capability.status),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_request_completion_source_contract"),
                capability.map(|capability| capability.source_contract.as_str()),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_request_completion_source_status"),
                capability.map(|capability| capability.source_status.as_str()),
            ),
            json_usize_field(
                &format!("{prefix}_provider_request_completion_collection_count"),
                capability
                    .map(|capability| capability.collection_count)
                    .unwrap_or(0),
            ),
            json_usize_field(
                &format!("{prefix}_provider_request_completion_receipt_count"),
                capability
                    .map(|capability| capability.receipt_count)
                    .unwrap_or(0),
            ),
            json_object_array_field(
                &format!("{prefix}_provider_request_completion_collections"),
                &collections,
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_request_completion_first_blocker"),
                capability.and_then(|capability| capability.first_blocker.as_deref()),
            ),
        ]);
    }
    fields.push(json_field(
        projection_source_field,
        capability
            .map(|capability| capability.projection_source)
            .unwrap_or("final_output_provider_request_completion_collections"),
    ));
    fields
}

fn collection_records(
    capability: Option<&ValidatedProviderRequestCompletionCapability>,
) -> Vec<String> {
    capability
        .map(|capability| {
            capability
                .collections
                .iter()
                .map(request_completion_collection_json)
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn request_completion_collection_json(
    collection: &super::link_plan_final_output_summary::ProviderRequestCompletionCollectionBoundarySummary,
) -> String {
    let receipts = collection
        .receipts
        .iter()
        .map(|receipt| {
            format!(
                "{{{},{},{},{},{},{},{},{},{}}}",
                json_field("contract", &receipt.contract),
                json_field("status", &receipt.status),
                json_field("request_id", &receipt.request_id),
                json_field("provider_family", &receipt.provider_family),
                json_field("dispatch_id", &receipt.dispatch_id),
                json_field("completion_clock", &receipt.completion_clock),
                json_field("output_hash", &receipt.output_hash),
                json_field("completion_token", &receipt.completion_token),
                json_field("selected_set_hash", &receipt.selected_set_hash),
            )
        })
        .collect::<Vec<_>>();
    format!(
        "{{{},{},{},{},{},{},{},{},{}}}",
        json_field("source_trace_id", &collection.source_trace_id),
        json_field("source_provider_family", &collection.source_provider_family,),
        json_field(
            "dispatch_selected_set_hash",
            &collection.dispatch_selected_set_hash,
        ),
        json_field("contract", &collection.contract),
        json_field("status", &collection.status),
        json_usize_field("count", collection.count),
        json_field("root_hash", &collection.root_hash),
        json_field("validation_status", &collection.validation_status),
        json_object_array_field("receipts", &receipts),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::link_plan_final_output_summary::{
        ProviderRequestCompletionBoundarySummary,
        ProviderRequestCompletionCollectionBoundarySummary,
    };

    #[test]
    fn package_and_debugger_share_typed_request_completion_collections() {
        let capability = ValidatedProviderRequestCompletionCapability {
            contract: "nuis-validated-provider-request-completion-capability-v1",
            source_contract: "nsdb-yir-replay-summary-v1".to_owned(),
            source_status: "replay-ready".to_owned(),
            projection_source: "final_output_provider_request_completion_collections",
            ready: true,
            status: "verified",
            collection_count: 1,
            receipt_count: 1,
            collections: vec![ProviderRequestCompletionCollectionBoundarySummary {
                source_trace_id: "trace-0001".to_owned(),
                source_provider_family: "cuda".to_owned(),
                dispatch_selected_set_hash: "fnv1a64:cccccccccccccccc".to_owned(),
                contract: "nuis-provider-request-completion-receipt-collection-v1".to_owned(),
                status: "verified".to_owned(),
                count: 1,
                root_hash: "fnv1a64:aaaaaaaaaaaaaaaa".to_owned(),
                validation_status: "verified".to_owned(),
                receipts: vec![ProviderRequestCompletionBoundarySummary {
                    contract: "nuis-provider-request-completion-receipt-v1".to_owned(),
                    status: "verified".to_owned(),
                    request_id: "request-0000".to_owned(),
                    provider_family: "cuda".to_owned(),
                    dispatch_id: "dispatch-0000".to_owned(),
                    completion_clock: "1:0".to_owned(),
                    output_hash: "fnv1a64:bbbbbbbbbbbbbbbb".to_owned(),
                    completion_token: "completion:0000".to_owned(),
                    selected_set_hash: "fnv1a64:cccccccccccccccc".to_owned(),
                }],
            }],
            first_blocker: None,
        };

        let fields = provider_request_completion_capability_json_fields(
            Some(&capability),
            &["object_package", "debugger_api"],
            "projection_source",
        );

        assert!(fields.contains(
            &"\"object_package_provider_request_completion_receipt_count\":1".to_owned()
        ));
        assert!(fields
            .contains(&"\"debugger_api_provider_request_completion_receipt_count\":1".to_owned()));
        assert!(fields.iter().any(|field| {
            field.starts_with("\"object_package_provider_request_completion_collections\"")
                && field.contains("\"request_id\":\"request-0000\"")
        }));
    }
}
