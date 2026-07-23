use crate::{
    provider_execution_capsule::{
        register_provider_execution_capsule, register_provider_execution_capsule_invoker,
        render_capsule_roles, ProviderExecutionCapsuleInvokerRegistration,
        ProviderExecutionCapsuleRegistration,
    },
    provider_output_carrier_registry::ProviderOutputCarrierConsumption,
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
pub(crate) const PROVIDER_WORKER_PROCESS_ADAPTER_CONTRACT: &str =
    "nuis-provider-worker-process-adapter-v4";

pub(crate) struct ProviderWorkerAdapterLaunch<'a> {
    pub(crate) executable_path: &'a Path,
    pub(crate) executable_hash: &'a str,
    pub(crate) runner_contract: &'a str,
    pub(crate) arguments: &'a [String],
    pub(crate) output_byte_length: usize,
}

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
    pub(crate) execution_capsule_invoker_contract: &'static str,
    pub(crate) execution_capsule_invoker_id: String,
    pub(crate) execution_capsule_invoker_status: &'static str,
    pub(crate) worker_output_descriptor_contract: &'static str,
    pub(crate) worker_output_descriptor_roles: String,
    pub(crate) worker_output_descriptor_count: usize,
    pub(crate) worker_output_descriptor_byte_length: usize,
    pub(crate) worker_output_descriptor_hash: String,
    pub(crate) worker_output_payload: Vec<u8>,
    pub(crate) worker_output_result: Option<ProviderOutputCarrierConsumption>,
    pub(crate) worker_output_receipt_status: &'static str,
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
        adapter_launch: Option<&ProviderWorkerAdapterLaunch<'_>>,
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
            .enumerate()
            .map(|(index, input)| {
                input
                    .try_clone_worker_descriptor()
                    .map(|file| file.map(|file| (index, file)))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let roles = files
            .iter()
            .map(|(index, _)| format!("input.{index}"))
            .collect::<Vec<_>>();
        let descriptors = files
            .iter()
            .zip(&roles)
            .map(|((_, file), role)| UnixWorkerDescriptor {
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
        let invoker = register_provider_execution_capsule_invoker(&capsule, adapter_id)
            .ok_or_else(|| {
                format!(
                    "provider execution capsule invoker is not registrable for adapter `{adapter_id}` request `{}`",
                    request.kernel.id
                )
            })?;
        validate_adapter_launch(adapter_launch, files.len())?;
        let payload = render_dispatch_payload(
            provider_family,
            request,
            &operation,
            &capsule,
            &invoker,
            adapter_launch,
        );
        let mut reply = lease
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
        if reply.output_descriptor_roles != capsule.output_roles
            || reply.output_descriptors.len() != capsule.output_roles.len()
        {
            return Err("provider worker output descriptor roles do not match capsule".to_owned());
        }
        let worker_output_result = crate::provider_worker_result::consume_worker_result(
            &mut reply.output_descriptors,
            &reply.output_descriptor_mode,
            reply.output_descriptor_byte_length,
            &reply.output_descriptor_hash,
            &reply.adapter_protocol,
        )?;
        let worker_output_payload = if worker_output_result.is_some() {
            reply.adapter_protocol
        } else {
            reply.output_descriptor_payload
        };
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
            execution_capsule_invocation_mode: if adapter_launch.is_some() {
                "worker-process-adapter-v4"
            } else {
                capsule.invocation_mode
            },
            execution_capsule_input_roles: render_capsule_roles(&capsule.input_roles),
            execution_capsule_output_roles: render_capsule_roles(&capsule.output_roles),
            execution_capsule_status: "worker-invoked",
            execution_capsule_invoker_contract: invoker.contract,
            execution_capsule_invoker_id: invoker.invoker_id,
            execution_capsule_invoker_status: "registered-invoked",
            worker_output_descriptor_contract: invoker.output_carrier_contract,
            worker_output_descriptor_roles: render_capsule_roles(&reply.output_descriptor_roles),
            worker_output_descriptor_count: capsule.output_roles.len(),
            worker_output_descriptor_byte_length: reply.output_descriptor_byte_length,
            worker_output_descriptor_hash: reply.output_descriptor_hash,
            worker_output_payload,
            worker_output_result,
            worker_output_receipt_status: "verified",
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
    invoker: &ProviderExecutionCapsuleInvokerRegistration,
    adapter_launch: Option<&ProviderWorkerAdapterLaunch<'_>>,
) -> String {
    let mut payload = format!(
        "contract={}\nregistry_source={}\ncapsule_id={}\ncapsule_token={}\ninvocation_mode={}\ninvoker_contract={}\ninvoker_registry_source={}\ninvoker_id={}\ninvoker_token={}\noutput_carrier_contract={}\nprovider={provider_family}\nadapter={}\nkernel={}\noperation_registry_contract={}\noperation={}\noperation_token={}\ninput_roles={}\noutput_roles={}\ninputs={}\noutputs={}\n",
        capsule.contract,
        capsule.registry_source,
        capsule.capsule_id,
        capsule.capsule_token,
        capsule.invocation_mode,
        invoker.contract,
        invoker.registry_source,
        invoker.invoker_id,
        invoker.invoker_token,
        invoker.output_carrier_contract,
        operation.adapter_id,
        request.kernel.id,
        operation.registry_contract,
        operation.operation,
        operation.operation_token,
        render_capsule_roles(&capsule.input_roles),
        render_capsule_roles(&capsule.output_roles),
        capsule.input_roles.len(),
        capsule.output_roles.len(),
    );
    if let Some(launch) = adapter_launch {
        payload.push_str(&format!(
            "adapter_launch_contract={PROVIDER_WORKER_PROCESS_ADAPTER_CONTRACT}\nadapter_executable={}\nadapter_executable_hash={}\nadapter_runner_contract={}\nadapter_argument_count={}\n",
            launch.executable_path.display(),
            launch.executable_hash,
            launch.runner_contract,
            launch.arguments.len(),
        ));
        payload.push_str(&format!(
            "adapter_output_byte_length={}\n",
            launch.output_byte_length
        ));
        for (index, argument) in launch.arguments.iter().enumerate() {
            payload.push_str(&format!("adapter_argument_{index}={argument}\n"));
        }
    }
    payload
}

fn validate_adapter_launch(
    launch: Option<&ProviderWorkerAdapterLaunch<'_>>,
    descriptor_count: usize,
) -> Result<(), String> {
    let Some(launch) = launch else {
        return Ok(());
    };
    let path = launch
        .executable_path
        .to_str()
        .ok_or_else(|| "provider worker adapter executable path is not UTF-8".to_owned())?;
    if path.is_empty()
        || path.contains(['\t', '\r', '\n'])
        || launch.executable_hash.len() != 18
        || !launch.executable_hash.starts_with("0x")
        || !launch.executable_hash[2..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || !launch
            .runner_contract
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
        || launch.arguments.is_empty()
        || launch.arguments.len() > 32
        || launch.output_byte_length == 0
        || launch
            .arguments
            .iter()
            .any(|argument| !is_adapter_argument(argument, descriptor_count))
    {
        return Err("provider worker adapter launch descriptor is invalid".to_owned());
    }
    Ok(())
}

fn is_adapter_argument(value: &str, descriptor_count: usize) -> bool {
    if value.contains(['\t', '\r', '\n']) {
        return false;
    }
    if let Some(literal) = value.strip_prefix("literal:") {
        return !literal.is_empty();
    }
    if let Some(binding) = value.strip_prefix("verified-path:") {
        return binding
            .split_once(':')
            .is_some_and(|(hash, path)| is_fnv_hash(hash) && !path.is_empty());
    }
    if let Some(index) = value.strip_prefix("descriptor-path:") {
        return valid_descriptor_index(index, descriptor_count);
    }
    value
        .strip_prefix("descriptor-carrier:")
        .and_then(|metadata| {
            let mut fields = metadata.split(':');
            Some((
                fields.next()?,
                fields.next()?,
                fields.next()?,
                fields.next()?,
                fields.next(),
            ))
        })
        .is_some_and(|(index, frame, length, hash, extra)| {
            extra.is_none()
                && valid_descriptor_index(index, descriptor_count)
                && [frame, length, hash].iter().all(|field| is_decimal(field))
        })
}

fn valid_descriptor_index(value: &str, descriptor_count: usize) -> bool {
    is_decimal(value)
        && value
            .parse::<usize>()
            .is_ok_and(|index| index < descriptor_count)
}

fn is_decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_fnv_hash(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
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
    use super::{validate_adapter_launch, ProviderWorkerAdapterLaunch};
    use std::path::Path;

    #[test]
    fn dispatch_payload_is_provider_and_request_bound() {
        let source = include_str!("provider_worker_lease.rs");
        assert!(source.contains("provider={provider_family}"));
        assert!(source.contains("request.kernel.id"));
        assert!(source.contains("operation.operation_token"));
        assert!(source.contains("capsule.capsule_token"));
        assert!(source.contains("invoker.invoker_token"));
        assert!(source.contains("output_roles"));
        assert_eq!(
            crate::provider_worker_image::PROVIDER_WORKER_IMAGE_RESOLVER_CONTRACT,
            "nuis-provider-worker-image-resolver-v1"
        );
    }

    #[test]
    fn adapter_launch_rejects_unbound_or_frame_unsafe_identity() {
        let invalid_hash = ProviderWorkerAdapterLaunch {
            executable_path: Path::new("adapter"),
            executable_hash: "not-a-hash",
            runner_contract: "runner.v1",
            arguments: &["descriptor-path:0".to_owned()],
            output_byte_length: 4,
        };
        assert!(validate_adapter_launch(Some(&invalid_hash), 1).is_err());

        let invalid_literal = ProviderWorkerAdapterLaunch {
            executable_path: Path::new("adapter"),
            executable_hash: "0x0123456789abcdef",
            runner_contract: "runner.v1",
            arguments: &["literal:15\nnext".to_owned()],
            output_byte_length: 4,
        };
        assert!(validate_adapter_launch(Some(&invalid_literal), 1).is_err());

        let invalid_descriptor = ProviderWorkerAdapterLaunch {
            executable_path: Path::new("adapter"),
            executable_hash: "0x0123456789abcdef",
            runner_contract: "runner.v1",
            arguments: &["descriptor-carrier:1:0:4096:42".to_owned()],
            output_byte_length: 4,
        };
        assert!(validate_adapter_launch(Some(&invalid_descriptor), 1).is_err());

        let ordered_arguments = ProviderWorkerAdapterLaunch {
            executable_path: Path::new("adapter"),
            executable_hash: "0x0123456789abcdef",
            runner_contract: "runner.v1",
            arguments: &[
                "verified-path:0x0123456789abcdef:model.mlmodel".to_owned(),
                "literal:--multi".to_owned(),
                "descriptor-carrier:0:0:4096:42".to_owned(),
            ],
            output_byte_length: 4,
        };
        assert!(validate_adapter_launch(Some(&ordered_arguments), 1).is_ok());
    }
}
