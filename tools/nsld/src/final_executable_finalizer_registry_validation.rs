use super::*;
use std::collections::BTreeSet;

pub(crate) fn executable_finalizer_registry_validation() -> ExecutableFinalizerRegistryValidation {
    let mut issues = Vec::new();
    let mut provider_ids = BTreeSet::new();
    let mut target_keys = BTreeSet::new();
    let mut publication_capability_ids = BTreeSet::new();
    let mut loader_probe_capability_ids = BTreeSet::new();

    for registration in REGISTERED_FINALIZERS {
        if !provider_ids.insert(registration.provider_id) {
            issues.push(format!(
                "duplicate executable finalizer provider id `{}`",
                registration.provider_id
            ));
        }
        let route_key = registration_route_key(registration);
        if !target_keys.insert(route_key.clone()) {
            issues.push(format!(
                "duplicate executable finalizer route `{route_key}`"
            ));
        }
        if registration.provider_status == "ready" && registration.executor.is_none() {
            issues.push(format!(
                "ready executable finalizer provider `{}` has no executor",
                registration.provider_id
            ));
        }
        if registration.input_summary_contract == Some("") {
            issues.push(format!(
                "executable finalizer provider `{}` declares an empty input summary contract",
                registration.provider_id
            ));
        }
        validate_private_image_publication_registration(
            registration,
            &mut publication_capability_ids,
            &mut issues,
        );
        validate_loader_probe_registration(
            registration,
            &mut loader_probe_capability_ids,
            &mut issues,
        );
        if registration.provider_status != "ready"
            && registration.provider_status != "registered-not-implemented"
        {
            issues.push(format!(
                "executable finalizer provider `{}` has invalid status `{}`",
                registration.provider_id, registration.provider_status
            ));
        }
    }

    ExecutableFinalizerRegistryValidation {
        contract: EXECUTABLE_FINALIZER_CONTRACT,
        registry_hash: executable_finalizer_registry_hash(),
        registration_count: REGISTERED_FINALIZERS.len(),
        valid: issues.is_empty(),
        issues,
    }
}

fn validate_private_image_publication_registration(
    registration: &ExecutableFinalizerRegistration,
    capability_ids: &mut BTreeSet<&'static str>,
    issues: &mut Vec<String>,
) {
    match (
        registration.private_image_publication_capability,
        registration.private_image_publisher,
    ) {
        (Some(""), _) => issues.push(format!(
            "executable finalizer provider `{}` declares an empty private-image publication capability",
            registration.provider_id
        )),
        (Some(capability), Some(_)) => {
            if !capability_ids.insert(capability) {
                issues.push(format!(
                    "duplicate private-image publication capability id `{capability}`"
                ));
            }
            validate_ready_capability_provider(registration, "private-image publication", issues);
        }
        (None, None) => {}
        _ => issues.push(format!(
            "executable finalizer provider `{}` has an incomplete private-image publication registration",
            registration.provider_id
        )),
    }
}

fn validate_loader_probe_registration(
    registration: &ExecutableFinalizerRegistration,
    capability_ids: &mut BTreeSet<&'static str>,
    issues: &mut Vec<String>,
) {
    match (
        registration.loader_probe_capability,
        registration.loader_probe,
    ) {
        (Some(""), _) => issues.push(format!(
            "executable finalizer provider `{}` declares an empty loader-probe capability",
            registration.provider_id
        )),
        (Some(capability), Some(_)) => {
            if !capability_ids.insert(capability) {
                issues.push(format!(
                    "duplicate executable loader-probe capability id `{capability}`"
                ));
            }
            validate_ready_capability_provider(registration, "loader-probe", issues);
        }
        (None, None) => {}
        _ => issues.push(format!(
            "executable finalizer provider `{}` has an incomplete loader-probe registration",
            registration.provider_id
        )),
    }
}

fn validate_ready_capability_provider(
    registration: &ExecutableFinalizerRegistration,
    capability_kind: &str,
    issues: &mut Vec<String>,
) {
    if registration.provider_status != "ready" || registration.executor.is_none() {
        issues.push(format!(
            "{capability_kind} provider `{}` is not a ready executable finalizer",
            registration.provider_id
        ));
    }
}

fn executable_finalizer_registry_hash() -> String {
    let mut registrations = REGISTERED_FINALIZERS.iter().collect::<Vec<_>>();
    registrations.sort_by_key(|registration| registration.provider_id);
    let mut material = format!("contract={EXECUTABLE_FINALIZER_CONTRACT}\n");
    for registration in registrations {
        material.push_str(&format!(
            "provider={}\nroute={}\nstatus={}\nexecution={}\ninput={}\ninput_summary={}\nhost_driver={}\nprivate_image_publication={}\nloader_probe={}\n",
            registration.provider_id,
            registration_route_key(registration),
            registration.provider_status,
            registration.execution_kind,
            registration.input_kind,
            registration.input_summary_contract.unwrap_or("none"),
            registration.requires_host_driver,
            registration
                .private_image_publication_capability
                .unwrap_or("none"),
            registration.loader_probe_capability.unwrap_or("none")
        ));
    }
    crate::fnv1a64_hex(material.as_bytes())
}
