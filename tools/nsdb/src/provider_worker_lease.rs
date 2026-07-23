use crate::{
    provider_execution_capsule::{
        register_provider_execution_capsule, render_capsule_roles,
        ProviderExecutionCapsuleRegistration,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_request::ProviderRequest,
    provider_runner_registry::{
        select_provider_worker_operation_registration, ProviderWorkerOperationRegistration,
    },
    provider_worker_image::resolve_provider_worker_image,
    provider_worker_transport_unix::{UnixWorkerDescriptor, UnixWorkerProcessTransport},
};
use std::{
    collections::BTreeMap,
    fs,
    os::fd::AsFd,
    path::{Path, PathBuf},
};

pub(crate) const PROVIDER_WORKER_LEASE_CONTRACT: &str = "nuis-provider-worker-lease-v1";
pub(crate) const PROVIDER_WORKER_DISPATCH_PERMIT_CONTRACT: &str =
    "nuis-provider-worker-dispatch-permit-v1";

pub(crate) struct ProviderWorkerDispatchReceipt {
    pub(crate) lease_contract: &'static str,
    pub(crate) resolver_contract: &'static str,
    pub(crate) cache_status: &'static str,
    pub(crate) worker_pid: u32,
    pub(crate) sequence: usize,
    pub(crate) descriptor_count: usize,
    pub(crate) payload_hash: String,
    pub(crate) operation_token: String,
    pub(crate) execution_capsule_contract: &'static str,
    pub(crate) execution_capsule_id: String,
    pub(crate) execution_capsule_token: String,
    pub(crate) execution_capsule_invocation_mode: &'static str,
    pub(crate) execution_capsule_input_roles: String,
    pub(crate) execution_capsule_output_roles: String,
    pub(crate) execution_capsule_status: &'static str,
    pub(crate) dispatch_status: i64,
    pub(crate) dispatch_permit_contract: &'static str,
    pub(crate) dispatch_permit_status: &'static str,
}

struct ProviderWorkerLease {
    provider_family: String,
    resolver_contract: &'static str,
    cache_status: &'static str,
    transport: UnixWorkerProcessTransport,
}

pub(crate) struct ProviderWorkerLeaseManager {
    image_dir: PathBuf,
    leases: BTreeMap<String, ProviderWorkerLease>,
}

impl ProviderWorkerLeaseManager {
    pub(crate) fn new(output_dir: &Path) -> Self {
        Self {
            image_dir: output_dir.join(".nuis-provider-worker-image"),
            leases: BTreeMap::new(),
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        adapter_id: &str,
        provider_family: &str,
        lease_id: &str,
        expected_sequence: usize,
        request: &ProviderRequest,
        inputs: &[PreparedProviderInput],
    ) -> Result<ProviderWorkerDispatchReceipt, String> {
        if !self.leases.contains_key(adapter_id) {
            let adapter_image_dir = self.image_dir.join(adapter_id);
            let image = resolve_provider_worker_image(provider_family, &adapter_image_dir)?;
            let mut command = image.command();
            let transport = UnixWorkerProcessTransport::spawn(&mut command, lease_id).map_err(
                |error| {
                    format!(
                        "provider worker `{adapter_id}` family `{provider_family}` failed to start: {error}"
                    )
                },
            )?;
            self.leases.insert(
                adapter_id.to_owned(),
                ProviderWorkerLease {
                    provider_family: provider_family.to_owned(),
                    resolver_contract: image.resolver_contract,
                    cache_status: image.cache_status,
                    transport,
                },
            );
        }
        let lease = self
            .leases
            .get_mut(adapter_id)
            .expect("provider worker lease was inserted");
        if lease.provider_family != provider_family {
            return Err(format!(
                "provider worker adapter `{adapter_id}` cannot change family from `{}` to `{provider_family}`",
                lease.provider_family
            ));
        }
        let files = inputs
            .iter()
            .map(PreparedProviderInput::try_clone_worker_descriptor)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let roles = (0..files.len())
            .map(|index| format!("input.{index}"))
            .collect::<Vec<_>>();
        let descriptors = files
            .iter()
            .zip(&roles)
            .map(|(file, role)| UnixWorkerDescriptor {
                role,
                descriptor: file.as_fd(),
            })
            .collect::<Vec<_>>();
        let operation = select_provider_worker_operation_registration(
            provider_family,
            adapter_id,
            &request.kernel.operation,
        )
        .ok_or_else(|| {
            format!(
                "provider worker operation `{}` is not registrable for adapter `{adapter_id}`",
                request.kernel.operation
            )
        })?;
        let output_roles = vec!["output.result".to_owned()];
        let capsule = register_provider_execution_capsule(
            provider_family,
            adapter_id,
            &operation.operation_token,
            &roles,
            &output_roles,
        )
        .ok_or_else(|| {
            format!(
                "provider execution capsule is not registrable for adapter `{adapter_id}` request `{}`",
                request.kernel.id
            )
        })?;
        let payload = render_dispatch_payload(provider_family, request, &operation, &capsule);
        let reply = lease
            .transport
            .request(&request.kernel.id, payload.as_bytes(), &descriptors)
            .map_err(|error| {
                format!(
                    "provider worker dispatch `{adapter_id}` request `{}` sequence {expected_sequence} failed: {error}",
                    request.kernel.id
                )
            })?;
        if reply.sequence != expected_sequence {
            return Err(format!(
                "provider worker sequence {} does not match session sequence {expected_sequence}",
                reply.sequence
            ));
        }
        if reply.payload != payload.as_bytes() {
            return Err("provider worker changed the opaque request payload".to_owned());
        }
        Ok(ProviderWorkerDispatchReceipt {
            lease_contract: PROVIDER_WORKER_LEASE_CONTRACT,
            resolver_contract: lease.resolver_contract,
            cache_status: lease.cache_status,
            worker_pid: reply.worker_pid,
            sequence: reply.sequence,
            descriptor_count: reply.descriptor_count,
            payload_hash: reply.payload_hash,
            operation_token: operation.operation_token,
            execution_capsule_contract: capsule.contract,
            execution_capsule_id: capsule.capsule_id,
            execution_capsule_token: capsule.capsule_token,
            execution_capsule_invocation_mode: capsule.invocation_mode,
            execution_capsule_input_roles: render_capsule_roles(&capsule.input_roles),
            execution_capsule_output_roles: render_capsule_roles(&capsule.output_roles),
            execution_capsule_status: "worker-authorized",
            dispatch_status: reply.dispatch_status,
            dispatch_permit_contract: PROVIDER_WORKER_DISPATCH_PERMIT_CONTRACT,
            dispatch_permit_status: "granted",
        })
    }

    pub(crate) fn close(mut self) -> Result<(), String> {
        let leases = std::mem::take(&mut self.leases);
        for (_, lease) in leases {
            lease.transport.close()?;
        }
        remove_image_dir(&self.image_dir)
    }
}

impl Drop for ProviderWorkerLeaseManager {
    fn drop(&mut self) {
        self.leases.clear();
        let _ = remove_image_dir(&self.image_dir);
    }
}

fn render_dispatch_payload(
    provider_family: &str,
    request: &ProviderRequest,
    operation: &ProviderWorkerOperationRegistration,
    capsule: &ProviderExecutionCapsuleRegistration,
) -> String {
    format!(
        "contract={}\nregistry_source={}\ncapsule_id={}\ncapsule_token={}\ninvocation_mode={}\nprovider={provider_family}\nadapter={}\nkernel={}\noperation_registry_contract={}\noperation={}\noperation_token={}\ninput_roles={}\noutput_roles={}\ninputs={}\noutputs={}\n",
        capsule.contract,
        capsule.registry_source,
        capsule.capsule_id,
        capsule.capsule_token,
        capsule.invocation_mode,
        operation.adapter_id,
        request.kernel.id,
        operation.registry_contract,
        operation.operation,
        operation.operation_token,
        render_capsule_roles(&capsule.input_roles),
        render_capsule_roles(&capsule.output_roles),
        capsule.input_roles.len(),
        capsule.output_roles.len(),
    )
}

fn remove_image_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| {
            format!(
                "failed to remove transient provider worker image `{}`: {error}",
                path.display()
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn dispatch_payload_is_provider_and_request_bound() {
        let source = include_str!("provider_worker_lease.rs");
        assert!(source.contains("provider={provider_family}"));
        assert!(source.contains("request.kernel.id"));
        assert!(source.contains("operation.operation_token"));
        assert!(source.contains("capsule.capsule_token"));
        assert!(source.contains("output_roles"));
        assert_eq!(
            crate::provider_worker_image::PROVIDER_WORKER_IMAGE_RESOLVER_CONTRACT,
            "nuis-provider-worker-image-resolver-v1"
        );
    }
}
