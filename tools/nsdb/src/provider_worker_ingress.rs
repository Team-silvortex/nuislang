#[cfg(unix)]
use crate::provider_worker_transport_unix::UnixWorkerRequest;

pub(crate) const PROVIDER_WORKER_INGRESS_CONTRACT: &str = "nuis-provider-worker-ingress-adapter-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderWorkerIngressRegistration {
    pub(crate) provider_key: i64,
    pub(crate) capability_hash: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderWorkerIngressFields {
    pub(crate) request_handle: i64,
    pub(crate) descriptor_table_handle: i64,
    pub(crate) descriptor_count: i64,
    pub(crate) provider_key: i64,
    pub(crate) capability_hash: i64,
}

pub(crate) type ProviderWorkerIngressEntrypoint = extern "C" fn(i64, i64, i64, i64, i64) -> i64;

#[cfg(unix)]
pub(crate) fn map_verified_unix_request_to_ingress(
    request: &UnixWorkerRequest,
    request_handle: i64,
    descriptor_table_handle: i64,
    registration: ProviderWorkerIngressRegistration,
) -> Result<ProviderWorkerIngressFields, String> {
    if request.payload_hash.is_empty()
        || request.descriptor_roles.len() != request.descriptors.len()
    {
        return Err("provider worker request is not a verified NUISPWU2 envelope".to_owned());
    }
    if request_handle <= 0 {
        return Err("provider worker request registry returned an invalid handle".to_owned());
    }
    if !request.descriptors.is_empty() && descriptor_table_handle <= 0 {
        return Err(
            "provider worker descriptor registry returned an invalid table handle".to_owned(),
        );
    }
    if registration.provider_key <= 0 || registration.capability_hash <= 0 {
        return Err("provider worker ingress registration is invalid".to_owned());
    }
    let descriptor_count = request
        .descriptors
        .len()
        .try_into()
        .map_err(|_| "provider worker descriptor count exceeds the scalar ABI".to_owned())?;
    Ok(ProviderWorkerIngressFields {
        request_handle,
        descriptor_table_handle,
        descriptor_count,
        provider_key: registration.provider_key,
        capability_hash: registration.capability_hash,
    })
}

pub(crate) fn invoke_provider_worker_ingress(
    entrypoint: ProviderWorkerIngressEntrypoint,
    fields: ProviderWorkerIngressFields,
) -> i64 {
    entrypoint(
        fields.request_handle,
        fields.descriptor_table_handle,
        fields.descriptor_count,
        fields.provider_key,
        fields.capability_hash,
    )
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::{
        provider_sample_artifact::fnv1a64_hex, provider_worker_transport_unix::UnixWorkerRequest,
    };
    use std::{fs::File, os::fd::OwnedFd};

    extern "C" fn ingress_probe(
        request: i64,
        descriptors: i64,
        count: i64,
        provider: i64,
        capability: i64,
    ) -> i64 {
        request + descriptors + count + provider + capability
    }

    #[test]
    fn verified_unix_request_maps_to_five_nuis_ingress_scalars() {
        let descriptor: OwnedFd = File::open("/dev/null").expect("descriptor").into();
        let payload = vec![0, 17, 255];
        let request = UnixWorkerRequest {
            lease_id: "lease:test".to_owned(),
            sequence: 0,
            request_id: "request.test".to_owned(),
            payload_hash: fnv1a64_hex(&payload),
            payload,
            descriptor_roles: vec!["input.primary".to_owned()],
            descriptors: vec![descriptor],
        };
        let fields = map_verified_unix_request_to_ingress(
            &request,
            101,
            501,
            ProviderWorkerIngressRegistration {
                provider_key: 20,
                capability_hash: 2020,
            },
        )
        .expect("verified ingress mapping");

        assert_eq!(fields.descriptor_count, 1);
        assert_eq!(invoke_provider_worker_ingress(ingress_probe, fields), 2643);
    }
}
