use super::{
    container::NsldContainerMetadataBinding,
    container_provider_dispatch::{
        provider_dispatch_evidence, PROVIDER_DISPATCH_BINDING_ID, PROVIDER_DISPATCH_CONTRACT,
    },
    final_executable_provider_sample::nsld_device_provider_sample_evidence,
    fnv1a64_hex,
    link_units::nsld_sidecar_capability_diagnostics,
};
use std::{collections::BTreeSet, fs};

pub(crate) const SELECTED_PROVIDER_BUNDLE_BINDING_ID: &str =
    "identity.selected-provider-bundle-set";
pub(crate) const CLOCK_ROOT_BINDING_ID: &str = "runtime.clock-root";
pub(crate) const CLOCK_ROOT_CONTRACT: &str = "nuis-clock-protocol-v1";
pub(crate) const GLM_ROOT_BINDING_ID: &str = "runtime.glm-root";
pub(crate) const GLM_ROOT_CONTRACT: &str = "nuis-yir-glm-binding-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldContainerMetadataBindingEvidence {
    pub(crate) bindings: Vec<NsldContainerMetadataBinding>,
    pub(crate) blockers: Vec<String>,
}

pub(crate) fn container_metadata_binding_evidence(
    plan: &nuisc::linker::LinkPlan,
) -> NsldContainerMetadataBindingEvidence {
    let mut evidence = runtime_service_binding_evidence(plan);
    let provider_evidence = nsld_device_provider_sample_evidence(&plan.output_dir);
    if !provider_evidence.available || provider_evidence.record_count == 0 {
        return evidence;
    }

    if provider_evidence.selected_provider_bundle_set_validation_status != "verified" {
        evidence.blockers.push(format!(
            "metadata-binding:{SELECTED_PROVIDER_BUNDLE_BINDING_ID}:{}",
            provider_evidence
                .first_blocker
                .as_deref()
                .unwrap_or("selected-provider-bundle-set-unverified")
        ));
        return evidence;
    }

    let Some(contract) = provider_evidence.selected_provider_bundle_set_contract else {
        evidence
            .blockers
            .push(missing_selected_set_field("contract"));
        return evidence;
    };
    let Some(value_count) = provider_evidence.selected_provider_bundle_count else {
        evidence
            .blockers
            .push(missing_selected_set_field("value-count"));
        return evidence;
    };
    let Some(value_hash) = provider_evidence.selected_provider_bundle_set_hash else {
        evidence
            .blockers
            .push(missing_selected_set_field("value-hash"));
        return evidence;
    };

    let dispatch = provider_dispatch_evidence(&plan.output_dir);
    if !dispatch.blockers.is_empty() {
        evidence.blockers.extend(
            dispatch.blockers.into_iter().map(|blocker| {
                format!("metadata-binding:{PROVIDER_DISPATCH_BINDING_ID}:{blocker}")
            }),
        );
        return evidence;
    }

    evidence.bindings.extend([
        NsldContainerMetadataBinding {
            binding_id: SELECTED_PROVIDER_BUNDLE_BINDING_ID.to_owned(),
            contract,
            value_count,
            value_hash,
            validation_status: "verified".to_owned(),
            required: true,
        },
        NsldContainerMetadataBinding {
            binding_id: PROVIDER_DISPATCH_BINDING_ID.to_owned(),
            contract: PROVIDER_DISPATCH_CONTRACT.to_owned(),
            value_count: dispatch.entries.len(),
            value_hash: dispatch.table_hash,
            validation_status: "verified".to_owned(),
            required: true,
        },
    ]);
    evidence
}

fn runtime_service_binding_evidence(
    plan: &nuisc::linker::LinkPlan,
) -> NsldContainerMetadataBindingEvidence {
    let mut blockers = Vec::new();
    let clock_verified =
        plan.clock_protocol.validation.valid && plan.clock_protocol.schema == CLOCK_ROOT_CONTRACT;
    if !clock_verified {
        blockers.push(format!(
            "metadata-binding:{CLOCK_ROOT_BINDING_ID}:clock-protocol-unverified"
        ));
    }
    let clock_hash = clock_protocol_hash(&plan.clock_protocol);
    let clock_count = 1usize
        .saturating_add(plan.clock_protocol.domains.len())
        .saturating_add(plan.clock_protocol.edges.len());

    let (glm_count, glm_hash, glm_verified, glm_blockers) = glm_binding_material(plan);
    blockers.extend(glm_blockers);
    NsldContainerMetadataBindingEvidence {
        bindings: vec![
            NsldContainerMetadataBinding {
                binding_id: CLOCK_ROOT_BINDING_ID.to_owned(),
                contract: CLOCK_ROOT_CONTRACT.to_owned(),
                value_count: clock_count,
                value_hash: clock_hash,
                validation_status: if clock_verified {
                    "verified"
                } else {
                    "unverified"
                }
                .to_owned(),
                required: true,
            },
            NsldContainerMetadataBinding {
                binding_id: GLM_ROOT_BINDING_ID.to_owned(),
                contract: GLM_ROOT_CONTRACT.to_owned(),
                value_count: glm_count,
                value_hash: glm_hash,
                validation_status: if glm_verified {
                    "verified"
                } else {
                    "unverified"
                }
                .to_owned(),
                required: true,
            },
        ],
        blockers,
    }
}

fn clock_protocol_hash(clock: &nuisc::linker::LinkPlanClockProtocol) -> String {
    let mut material = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\n",
        clock.schema,
        clock.mode,
        clock.source,
        clock.default_time_mode,
        clock.lifecycle_tick_policy,
        clock.validation.checked
    );
    for domain in &clock.domains {
        material.push_str(&format!(
            "domain\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            domain.domain_family,
            domain.package_id,
            domain.clock_domain_id,
            domain.clock_kind,
            domain.clock_epoch_kind,
            domain.clock_resolution,
            domain.clock_bridge_default,
            domain.lifecycle_hook
        ));
    }
    for edge in &clock.edges {
        material.push_str(&format!(
            "edge\t{}\t{}\t{}\t{}\n",
            edge.from, edge.to, edge.relation, edge.source
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

fn glm_binding_material(plan: &nuisc::linker::LinkPlan) -> (usize, String, bool, Vec<String>) {
    let mut blockers = Vec::new();
    let artifact_hash = match fs::read(&plan.compiled_artifact.path) {
        Ok(bytes) => fnv1a64_hex(&bytes),
        Err(_) => {
            blockers.push(format!(
                "metadata-binding:{GLM_ROOT_BINDING_ID}:compiled-artifact-unreadable"
            ));
            "missing".to_owned()
        }
    };
    if plan.compiled_artifact.section_table_valid != Some(true)
        || !plan.artifact_lowering_alignment.consistent
    {
        blockers.push(format!(
            "metadata-binding:{GLM_ROOT_BINDING_ID}:artifact-contract-unverified"
        ));
    }

    let mut glm_contracts = BTreeSet::new();
    for capability in nsld_sidecar_capability_diagnostics(plan) {
        let contracts = capability
            .validation_contracts
            .iter()
            .filter(|contract| contract.starts_with("glm."))
            .cloned()
            .collect::<Vec<_>>();
        if !capability.valid || contracts.is_empty() {
            blockers.push(format!(
                "metadata-binding:{GLM_ROOT_BINDING_ID}:{}:{}:glm-contract-unverified",
                capability.domain_family, capability.package_id
            ));
        }
        glm_contracts.extend(contracts);
    }

    let mut material = format!("artifact\t{artifact_hash}\n");
    for contract in &glm_contracts {
        material.push_str("contract\t");
        material.push_str(contract);
        material.push('\n');
    }
    (
        1usize.saturating_add(glm_contracts.len()),
        fnv1a64_hex(material.as_bytes()),
        blockers.is_empty(),
        blockers,
    )
}

fn missing_selected_set_field(field: &str) -> String {
    format!("metadata-binding:{SELECTED_PROVIDER_BUNDLE_BINDING_ID}:missing-{field}")
}
