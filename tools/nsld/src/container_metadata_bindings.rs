use super::{
    container::NsldContainerMetadataBinding,
    final_executable_provider_sample::nsld_device_provider_sample_evidence,
};

pub(crate) const SELECTED_PROVIDER_BUNDLE_BINDING_ID: &str =
    "identity.selected-provider-bundle-set";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerMetadataBindingEvidence {
    pub(crate) bindings: Vec<NsldContainerMetadataBinding>,
    pub(crate) blockers: Vec<String>,
}

pub(crate) fn container_metadata_binding_evidence(
    output_dir: &str,
) -> NsldContainerMetadataBindingEvidence {
    let evidence = nsld_device_provider_sample_evidence(output_dir);
    if !evidence.available || evidence.record_count == 0 {
        return NsldContainerMetadataBindingEvidence {
            bindings: Vec::new(),
            blockers: Vec::new(),
        };
    }

    if evidence.selected_provider_bundle_set_validation_status != "verified" {
        return NsldContainerMetadataBindingEvidence {
            bindings: Vec::new(),
            blockers: vec![format!(
                "metadata-binding:{SELECTED_PROVIDER_BUNDLE_BINDING_ID}:{}",
                evidence
                    .first_blocker
                    .as_deref()
                    .unwrap_or("selected-provider-bundle-set-unverified")
            )],
        };
    }

    let Some(contract) = evidence.selected_provider_bundle_set_contract else {
        return missing_selected_set_field("contract");
    };
    let Some(value_count) = evidence.selected_provider_bundle_count else {
        return missing_selected_set_field("value-count");
    };
    let Some(value_hash) = evidence.selected_provider_bundle_set_hash else {
        return missing_selected_set_field("value-hash");
    };

    NsldContainerMetadataBindingEvidence {
        bindings: vec![NsldContainerMetadataBinding {
            binding_id: SELECTED_PROVIDER_BUNDLE_BINDING_ID.to_owned(),
            contract,
            value_count,
            value_hash,
            validation_status: "verified".to_owned(),
            required: true,
        }],
        blockers: Vec::new(),
    }
}

fn missing_selected_set_field(field: &str) -> NsldContainerMetadataBindingEvidence {
    NsldContainerMetadataBindingEvidence {
        bindings: Vec::new(),
        blockers: vec![format!(
            "metadata-binding:{SELECTED_PROVIDER_BUNDLE_BINDING_ID}:missing-{field}"
        )],
    }
}
