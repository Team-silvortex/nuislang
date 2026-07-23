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
    provider_sample_artifact::fnv1a64_hex,
    provider_worker_control::{
        ProviderWorkerControlCarrier, PROVIDER_WORKER_ADAPTER_CONTROL_CARRIER_CONTRACT,
        PROVIDER_WORKER_ADAPTER_CONTROL_ROLE,
    },
    provider_worker_descriptor_capability::{
        ProviderWorkerDescriptorCapability, ProviderWorkerOutputDescriptorCapability,
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
pub(crate) const PROVIDER_WORKER_ADAPTER_CONTROL_CONTRACT: &str =
    "nuis-provider-worker-adapter-control-v1";
const MAX_PROVIDER_WORKER_DISPATCH_PAYLOAD_BYTES: usize = 1800;
const MAX_INLINE_ADAPTER_CONTROL_BYTES: usize = 384;

pub(crate) struct ProviderWorkerAdapterLaunch<'a> {
    pub(crate) executable_path: &'a Path,
    pub(crate) executable_hash: &'a str,
    pub(crate) runner_contract: &'a str,
    pub(crate) cache_contract: &'a str,
    pub(crate) cache_identity: &'a str,
    pub(crate) cache_status: &'a str,
    pub(crate) arguments: &'a [String],
    pub(crate) output_byte_length: usize,
}

struct RenderedProviderDispatch {
    payload: String,
    spilled_control: Option<Vec<u8>>,
}

pub(crate) struct ProviderWorkerDispatchReceipt {
    pub(crate) lease_contract: &'static str,
    pub(crate) resolver_contract: &'static str,
    pub(crate) cache_status: &'static str,
    pub(crate) worker_pid: u32,
    pub(crate) sequence: usize,
    pub(crate) descriptor_count: usize,
    pub(crate) descriptor_capability_contract: &'static str,
    pub(crate) max_semantic_descriptors: usize,
    pub(crate) max_control_descriptors: usize,
    pub(crate) output_descriptor_capability_contract: &'static str,
    pub(crate) max_output_descriptors: usize,
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
    pub(crate) additional_worker_outputs: Vec<ProviderWorkerOutput>,
    pub(crate) worker_output_receipt_status: &'static str,
    pub(crate) adapter_control_mode: &'static str,
    pub(crate) dispatch_status: i64,
    pub(crate) dispatch_permit_contract: &'static str,
    pub(crate) dispatch_permit_status: &'static str,
}

pub(crate) struct ProviderWorkerOutput {
    pub(crate) role: String,
    pub(crate) byte_length: usize,
    pub(crate) payload_hash: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) result: Option<ProviderOutputCarrierConsumption>,
}

impl ProviderWorkerOutput {
    pub(crate) fn retention_status(&self) -> &'static str {
        if self.result.is_some() {
            "transferable-carrier"
        } else if !self.payload.is_empty() {
            "verified-payload"
        } else {
            "empty"
        }
    }
}

struct ProviderWorkerLease {
    provider_family: String,
    resolver_contract: &'static str,
    cache_status: &'static str,
    descriptor_capability: ProviderWorkerDescriptorCapability,
    output_descriptor_capability: ProviderWorkerOutputDescriptorCapability,
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
            let transport = UnixWorkerProcessTransport::spawn(
                &mut command,
                lease_id,
                image.registration.descriptor_capability,
                image.registration.output_descriptor_capability,
            )
            .map_err(|error| {
                    format!(
                        "provider worker `{adapter_id}` family `{provider_family}` failed to start: {error}"
                    )
                })?;
            self.leases.insert(
                adapter_id.to_owned(),
                ProviderWorkerLease {
                    provider_family: provider_family.to_owned(),
                    resolver_contract: image.resolver_contract,
                    cache_status: image.cache_status,
                    descriptor_capability: image.registration.descriptor_capability,
                    output_descriptor_capability: image.registration.output_descriptor_capability,
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
        let output_roles = request
            .output_bindings
            .iter()
            .map(|binding| binding.role.clone())
            .collect::<Vec<_>>();
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
        let dispatch = render_dispatch_payload(
            provider_family,
            request,
            &operation,
            &capsule,
            &invoker,
            adapter_launch,
        )?;
        let control_carrier = dispatch
            .spilled_control
            .as_deref()
            .map(ProviderWorkerControlCarrier::new)
            .transpose()?;
        if let Some(carrier) = control_carrier.as_ref() {
            let evidence = format!(
                "adapter_control_ref={PROVIDER_WORKER_ADAPTER_CONTROL_CARRIER_CONTRACT}\t{}\t{}",
                carrier.byte_length, carrier.payload_hash
            );
            if !dispatch.payload.lines().any(|line| line == evidence) {
                return Err(
                    "provider worker control carrier metadata is not request-bound".to_owned(),
                );
            }
        }
        let adapter_control_mode = if control_carrier.is_some() {
            "carrier"
        } else if adapter_launch.is_some() {
            "inline"
        } else {
            "none"
        };
        let mut descriptors = files
            .iter()
            .zip(&roles)
            .map(|((_, file), role)| UnixWorkerDescriptor {
                role,
                descriptor: file.as_fd(),
            })
            .collect::<Vec<_>>();
        if let Some(carrier) = control_carrier.as_ref() {
            descriptors.push(UnixWorkerDescriptor {
                role: PROVIDER_WORKER_ADAPTER_CONTROL_ROLE,
                descriptor: carrier.file().as_fd(),
            });
        }
        let reply = lease
            .transport
            .request(
                &request.kernel.id,
                dispatch.payload.as_bytes(),
                &descriptors,
            )
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
        if reply.output_descriptor_modes.len() != capsule.output_roles.len()
            || reply.output_descriptor_byte_lengths.len() != capsule.output_roles.len()
            || reply.output_descriptor_hashes.len() != capsule.output_roles.len()
            || reply.output_descriptor_payloads.len() != capsule.output_roles.len()
        {
            return Err("provider worker output descriptor evidence count mismatch".to_owned());
        }
        let adapter_protocol = reply.adapter_protocol;
        let mut worker_outputs = reply
            .output_descriptors
            .into_iter()
            .zip(reply.output_descriptor_roles)
            .zip(reply.output_descriptor_byte_lengths)
            .zip(reply.output_descriptor_hashes)
            .zip(reply.output_descriptor_modes)
            .zip(reply.output_descriptor_payloads)
            .map(
                |(((((descriptor, role), byte_length), payload_hash), mode), payload)| {
                    let result = crate::provider_worker_result::consume_worker_result_descriptor(
                        descriptor,
                        &mode,
                        byte_length,
                        &payload_hash,
                        &adapter_protocol,
                    )?;
                    Ok(ProviderWorkerOutput {
                        role,
                        byte_length,
                        payload_hash,
                        payload: if result.is_some() {
                            adapter_protocol.clone()
                        } else {
                            payload
                        },
                        result,
                    })
                },
            )
            .collect::<Result<Vec<_>, String>>()?;
        let primary_output = worker_outputs.remove(0);
        Ok(ProviderWorkerDispatchReceipt {
            lease_contract: PROVIDER_WORKER_LEASE_CONTRACT,
            resolver_contract: lease.resolver_contract,
            cache_status: lease.cache_status,
            worker_pid: reply.worker_pid,
            sequence: reply.sequence,
            descriptor_count: capsule.input_roles.len(),
            descriptor_capability_contract: lease.descriptor_capability.contract,
            max_semantic_descriptors: lease.descriptor_capability.max_semantic_descriptors,
            max_control_descriptors: lease.descriptor_capability.max_control_descriptors,
            output_descriptor_capability_contract: lease.output_descriptor_capability.contract,
            max_output_descriptors: lease.output_descriptor_capability.max_output_descriptors,
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
            worker_output_descriptor_roles: render_capsule_roles(&capsule.output_roles),
            worker_output_descriptor_count: capsule.output_roles.len(),
            worker_output_descriptor_byte_length: primary_output.byte_length,
            worker_output_descriptor_hash: primary_output.payload_hash,
            worker_output_payload: primary_output.payload,
            worker_output_result: primary_output.result,
            additional_worker_outputs: worker_outputs,
            worker_output_receipt_status: "verified",
            adapter_control_mode,
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
) -> Result<RenderedProviderDispatch, String> {
    let payload = format!(
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
    let Some(launch) = adapter_launch else {
        validate_dispatch_payload_size(&payload)?;
        return Ok(RenderedProviderDispatch {
            payload,
            spilled_control: None,
        });
    };
    attach_adapter_control(payload, launch)
}

fn attach_adapter_control(
    payload: String,
    launch: &ProviderWorkerAdapterLaunch<'_>,
) -> Result<RenderedProviderDispatch, String> {
    let control = render_adapter_control(launch);
    let inline = format!("{payload}adapter_control={control}\n");
    if control.len() <= MAX_INLINE_ADAPTER_CONTROL_BYTES
        && validate_dispatch_payload_size(&inline).is_ok()
    {
        return Ok(RenderedProviderDispatch {
            payload: inline,
            spilled_control: None,
        });
    }
    let reference = format!(
        "{PROVIDER_WORKER_ADAPTER_CONTROL_CARRIER_CONTRACT}\t{}\t{}",
        control.len(),
        fnv1a64_hex(control.as_bytes())
    );
    let spilled = format!("{payload}adapter_control_ref={reference}\n");
    validate_dispatch_payload_size(&spilled)?;
    Ok(RenderedProviderDispatch {
        payload: spilled,
        spilled_control: Some(control.into_bytes()),
    })
}

fn render_adapter_control(launch: &ProviderWorkerAdapterLaunch<'_>) -> String {
    let mut fields = vec![
        PROVIDER_WORKER_ADAPTER_CONTROL_CONTRACT.to_owned(),
        PROVIDER_WORKER_PROCESS_ADAPTER_CONTRACT.to_owned(),
        launch.executable_path.display().to_string(),
        launch.executable_hash.to_owned(),
        launch.runner_contract.to_owned(),
        launch.output_byte_length.to_string(),
        launch.arguments.len().to_string(),
    ];
    fields.extend(launch.arguments.iter().cloned());
    fields.join("\t")
}

fn validate_dispatch_payload_size(payload: &str) -> Result<(), String> {
    if payload.len() > MAX_PROVIDER_WORKER_DISPATCH_PAYLOAD_BYTES {
        return Err(format!(
            "provider worker dispatch payload is too large: {} > {MAX_PROVIDER_WORKER_DISPATCH_PAYLOAD_BYTES}",
            payload.len()
        ));
    }
    Ok(())
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
        || launch.cache_contract.is_empty()
        || !launch.cache_identity.starts_with("adapter:0x")
        || !matches!(launch.cache_status, "compiled" | "hit")
        || path.len() >= 2048
        || launch.arguments.is_empty()
        || launch.arguments.len() > 32
        || launch.output_byte_length == 0
        || launch.arguments.iter().any(|argument| {
            argument.len() >= 2048 || !is_adapter_argument(argument, descriptor_count)
        })
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
#[path = "provider_worker_lease_tests.rs"]
mod tests;
