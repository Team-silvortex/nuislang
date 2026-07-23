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
    pub(crate) capsule_token: i64,
    pub(crate) input_role_count: i64,
    pub(crate) output_role_count: i64,
}

pub(crate) type ProviderWorkerIngressEntrypoint =
    extern "C" fn(i64, i64, i64, i64, i64, i64, i64, i64) -> i64;

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
    let capsule_token = payload_scalar(&request.payload, "capsule_token", "capsule-token:")?;
    let input_role_count = payload_scalar(&request.payload, "inputs", "")?;
    let output_role_count = payload_scalar(&request.payload, "outputs", "")?;
    let descriptor_count = semantic_descriptor_count(&request.descriptor_roles)?
        .try_into()
        .map_err(|_| "provider worker descriptor count exceeds the scalar ABI".to_owned())?;
    if capsule_token <= 0 || input_role_count != descriptor_count || output_role_count <= 0 {
        return Err("provider worker capsule ingress metadata is inconsistent".to_owned());
    }
    Ok(ProviderWorkerIngressFields {
        request_handle,
        descriptor_table_handle,
        descriptor_count,
        provider_key: registration.provider_key,
        capability_hash: registration.capability_hash,
        capsule_token,
        input_role_count,
        output_role_count,
    })
}

fn semantic_descriptor_count(roles: &[String]) -> Result<usize, String> {
    let control_positions = roles
        .iter()
        .enumerate()
        .filter_map(|(index, role)| (role == "control.adapter").then_some(index))
        .collect::<Vec<_>>();
    match control_positions.as_slice() {
        [] => Ok(roles.len()),
        [index] if *index + 1 == roles.len() => Ok(roles.len() - 1),
        _ => Err("provider worker control descriptor role must be unique and last".to_owned()),
    }
}

fn payload_scalar(payload: &[u8], key: &str, prefix: &str) -> Result<i64, String> {
    let text = std::str::from_utf8(payload)
        .map_err(|_| "provider worker capsule payload is not UTF-8".to_owned())?;
    let value = text
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .and_then(|value| value.strip_prefix(prefix))
        .ok_or_else(|| format!("provider worker capsule payload is missing `{key}`"))?;
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value >= 0)
        .ok_or_else(|| format!("provider worker capsule payload has invalid `{key}`"))
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
        fields.capsule_token,
        fields.input_role_count,
        fields.output_role_count,
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
        capsule: i64,
        inputs: i64,
        outputs: i64,
    ) -> i64 {
        request + descriptors + count + provider + capability + capsule + inputs + outputs
    }

    #[test]
    fn verified_unix_request_maps_to_eight_nuis_capsule_ingress_scalars() {
        let descriptor: OwnedFd = File::open("/dev/null").expect("descriptor").into();
        let payload = b"capsule_token=capsule-token:3030\ninputs=1\noutputs=1\n".to_vec();
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
        assert_eq!(fields.capsule_token, 3030);
        assert_eq!(fields.input_role_count, 1);
        assert_eq!(fields.output_role_count, 1);
        assert_eq!(invoke_provider_worker_ingress(ingress_probe, fields), 5675);
    }

    #[test]
    fn control_descriptor_is_not_a_semantic_capsule_input() {
        let input: OwnedFd = File::open("/dev/null").expect("input").into();
        let control: OwnedFd = File::open("/dev/null").expect("control").into();
        let payload = b"capsule_token=capsule-token:3030\ninputs=1\noutputs=1\n".to_vec();
        let request = UnixWorkerRequest {
            lease_id: "lease:test".to_owned(),
            sequence: 0,
            request_id: "request.test".to_owned(),
            payload_hash: fnv1a64_hex(&payload),
            payload,
            descriptor_roles: vec!["input.0".to_owned(), "control.adapter".to_owned()],
            descriptors: vec![input, control],
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
        .expect("control descriptor is transport-only");
        assert_eq!(fields.descriptor_count, 1);
        assert_eq!(fields.input_role_count, 1);
    }
}
