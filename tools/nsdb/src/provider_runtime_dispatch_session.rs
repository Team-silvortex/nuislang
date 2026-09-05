use crate::{
    model::{CompiledCodeAssetSelectionEvidence, NsdbDeviceProviderSampleRecordInfo},
    provider_code_asset_identity::ProviderCodeAssetIdentity,
    provider_edge_transport::ProviderEdgeTransportReceipt,
    provider_graph_output::{
        bind_output_binding_summary, bind_provider_completion_evidence, CompletedProviderOutput,
        CompletedProviderOutputs,
    },
    provider_output_comparison::{
        bind_output_comparison_collection, compare_provider_output_collection,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_request::{provider_request_collection_from_evidence, ProviderRequest},
    provider_runner_registry::{select_provider_runner_adapter, ProviderRunnerAdapter},
    provider_runtime_result_stream::ProviderRuntimeResult,
    provider_sample_payload::{fnv1a64_hex, push_toml_string, ProviderNativeOutputSummary},
    provider_session_registry::{
        select_provider_session_adapter, ProviderSessionLease, ProviderSessionRequest,
    },
    provider_session_summary::bind_session_output,
};
#[cfg(unix)]
use crate::{
    provider_execution_adapter::{select_provider_execution_adapter, ProviderRequestExecution},
    provider_graph_output::completed_additional_worker_outputs,
    provider_process_adapter::{
        provider_output_manifest, validate_provider_code_asset, ProviderProcessAdapterCache,
        PROVIDER_PROCESS_ADAPTER_CACHE_CONTRACT,
    },
    provider_worker_lease::{
        ProviderWorkerAdapterLaunch, ProviderWorkerDispatchIdentity, ProviderWorkerLeaseManager,
    },
    provider_worker_summary::bind_worker_output,
};
use std::{collections::BTreeMap, path::Path};

const RUNTIME_DISPATCH_SESSION_CONTRACT: &str = "nuis-provider-runtime-dispatch-session-v1";

pub(crate) struct NativeProviderOutputs {
    pub(crate) native_outputs: Vec<ProviderNativeOutputSummary>,
    pub(crate) transport_receipts: Vec<ProviderEdgeTransportReceipt>,
    pub(crate) code_asset_identity: Option<ProviderCodeAssetIdentity>,
    pub(crate) compiled_code_asset_selection: Option<CompiledCodeAssetSelectionEvidence>,
    pub(crate) runtime_results: Vec<ProviderRuntimeResult>,
    pub(crate) runtime_session_evidence: Option<ProviderRuntimeDispatchSessionEvidence>,
}

impl NativeProviderOutputs {
    pub(crate) fn empty() -> Self {
        Self {
            native_outputs: Vec::new(),
            transport_receipts: Vec::new(),
            code_asset_identity: None,
            compiled_code_asset_selection: None,
            runtime_results: Vec::new(),
            runtime_session_evidence: None,
        }
    }
}

pub(crate) struct ProviderRuntimeDispatchSessionEvidence {
    invocation_count: usize,
    observation_count: usize,
    worker_count: usize,
    lease_count: usize,
    request_sequences: String,
    adapter_cache_statuses: String,
    evidence_hash: String,
}

impl ProviderRuntimeDispatchSessionEvidence {
    pub(crate) fn from_observations(
        invocation_count: usize,
        observations: &[RuntimeDispatchObservation],
    ) -> Result<Option<Self>, String> {
        if observations.is_empty() {
            return Ok(None);
        }
        let mut leases = BTreeMap::<&str, (&str, usize)>::new();
        let mut workers = std::collections::BTreeSet::new();
        let mut material = format!(
            "{RUNTIME_DISPATCH_SESSION_CONTRACT}\n{invocation_count}\n{}",
            observations.len()
        );
        for observation in observations {
            let expected = leases
                .get(observation.lease_id.as_str())
                .map_or(0, |(_, sequence)| sequence + 1);
            if observation.sequence != expected {
                return Err(
                    "provider runtime dispatch session sequence is not contiguous".to_owned(),
                );
            }
            if leases
                .get(observation.lease_id.as_str())
                .is_some_and(|(worker_pid, _)| *worker_pid != observation.worker_pid)
            {
                return Err("provider runtime dispatch session changed worker identity".to_owned());
            }
            leases.insert(
                observation.lease_id.as_str(),
                (observation.worker_pid.as_str(), observation.sequence),
            );
            workers.insert(observation.worker_pid.as_str());
            material.push_str(&format!(
                "\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                observation.invocation,
                observation.request_id,
                observation.adapter_id,
                observation.lease_id,
                observation.worker_pid,
                observation.sequence,
                observation.adapter_cache_status,
            ));
        }
        Ok(Some(Self {
            invocation_count,
            observation_count: observations.len(),
            worker_count: workers.len(),
            lease_count: leases.len(),
            request_sequences: observations
                .iter()
                .map(|observation| observation.sequence.to_string())
                .collect::<Vec<_>>()
                .join(","),
            adapter_cache_statuses: observations
                .iter()
                .map(|observation| observation.adapter_cache_status.as_str())
                .collect::<Vec<_>>()
                .join(","),
            evidence_hash: fnv1a64_hex(material.as_bytes()),
        }))
    }

    pub(crate) fn append_payload_fields(&self, out: &mut String) {
        for (name, value) in [
            ("contract", RUNTIME_DISPATCH_SESSION_CONTRACT.to_owned()),
            ("status", "verified".to_owned()),
            ("invocation_count", self.invocation_count.to_string()),
            ("observation_count", self.observation_count.to_string()),
            ("worker_count", self.worker_count.to_string()),
            ("lease_count", self.lease_count.to_string()),
            ("request_sequences", self.request_sequences.clone()),
            (
                "adapter_cache_statuses",
                self.adapter_cache_statuses.clone(),
            ),
            ("evidence_hash", self.evidence_hash.clone()),
        ] {
            push_toml_string(out, &format!("runtime_dispatch_session_{name}"), &value);
        }
    }
}

pub(crate) struct RuntimeDispatchObservation {
    invocation: usize,
    request_id: String,
    adapter_id: String,
    lease_id: String,
    worker_pid: String,
    sequence: usize,
    adapter_cache_status: String,
}

pub(crate) fn runtime_dispatch_observations(
    invocation: usize,
    outputs: &[ProviderNativeOutputSummary],
) -> Result<Vec<RuntimeDispatchObservation>, String> {
    outputs
        .iter()
        .map(|output| {
            let sequence = output
                .worker_request_sequence
                .parse::<usize>()
                .map_err(|_| {
                    "provider runtime dispatch session has an invalid worker sequence".to_owned()
                })?;
            if !output.worker_pid.parse::<u32>().is_ok_and(|pid| pid > 0)
                || output.session_lease_id.is_empty()
                || output.session_lease_id == "none"
            {
                return Err("provider runtime dispatch session identity is invalid".to_owned());
            }
            Ok(RuntimeDispatchObservation {
                invocation,
                request_id: output.request_id.clone(),
                adapter_id: output.session_adapter_id.clone(),
                lease_id: output.session_lease_id.clone(),
                worker_pid: output.worker_pid.clone(),
                sequence,
                adapter_cache_status: output.worker_adapter_cache_status.clone(),
            })
        })
        .collect()
}

pub(crate) fn execute_native_provider_outputs(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
    adapter: &ProviderRunnerAdapter,
    runtime_invocation_count: usize,
) -> Result<NativeProviderOutputs, String> {
    let mut session = ProviderRuntimeDispatchSession::open(output_dir);
    let execution = session.execute_repeated(output_dir, record, adapter, runtime_invocation_count);
    let close = session.close();
    match (execution, close) {
        (Ok(outputs), Ok(())) => Ok(outputs),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

pub(crate) struct ProviderRuntimeDispatchSession {
    #[cfg(unix)]
    sessions: BTreeMap<String, ProviderSessionLease>,
    #[cfg(unix)]
    worker_leases: ProviderWorkerLeaseManager,
    #[cfg(unix)]
    process_adapter_cache: ProviderProcessAdapterCache,
}

impl ProviderRuntimeDispatchSession {
    pub(crate) fn open(output_dir: &Path) -> Self {
        Self {
            #[cfg(unix)]
            sessions: BTreeMap::new(),
            #[cfg(unix)]
            worker_leases: ProviderWorkerLeaseManager::new(output_dir),
            #[cfg(unix)]
            process_adapter_cache: ProviderProcessAdapterCache::default(),
        }
    }

    fn execute_repeated(
        &mut self,
        output_dir: &Path,
        record: &NsdbDeviceProviderSampleRecordInfo,
        adapter: &ProviderRunnerAdapter,
        runtime_invocation_count: usize,
    ) -> Result<NativeProviderOutputs, String> {
        let mut outputs = self.execute_graph(output_dir, record, adapter, None)?;
        let mut invocation_count = usize::from(!outputs.native_outputs.is_empty());
        let mut observations = runtime_dispatch_observations(0, &outputs.native_outputs)?;
        if !outputs.runtime_results.is_empty() {
            for invocation in 1..runtime_invocation_count {
                let mut repeated = self.execute_graph(output_dir, record, adapter, None)?;
                observations.extend(runtime_dispatch_observations(
                    invocation,
                    &repeated.native_outputs,
                )?);
                outputs
                    .runtime_results
                    .append(&mut repeated.runtime_results);
                invocation_count += 1;
            }
        }
        outputs.runtime_session_evidence =
            ProviderRuntimeDispatchSessionEvidence::from_observations(
                invocation_count,
                &observations,
            )?;
        Ok(outputs)
    }

    pub(crate) fn execute_graph(
        &mut self,
        output_dir: &Path,
        record: &NsdbDeviceProviderSampleRecordInfo,
        adapter: &ProviderRunnerAdapter,
        runtime_arguments: Option<(
            &yir_core::provider_runtime_ipc::DispatchTarget,
            &yir_core::provider_runtime_ipc::DispatchArguments,
        )>,
    ) -> Result<NativeProviderOutputs, String> {
        #[cfg(not(unix))]
        {
            let _ = (output_dir, record, adapter, runtime_arguments);
            Ok(NativeProviderOutputs::empty())
        }
        #[cfg(unix)]
        {
            self.execute_graph_unix(output_dir, record, adapter, runtime_arguments)
        }
    }

    #[cfg(unix)]
    fn execute_graph_unix(
        &mut self,
        output_dir: &Path,
        record: &NsdbDeviceProviderSampleRecordInfo,
        adapter: &ProviderRunnerAdapter,
        runtime_arguments: Option<(
            &yir_core::provider_runtime_ipc::DispatchTarget,
            &yir_core::provider_runtime_ipc::DispatchArguments,
        )>,
    ) -> Result<NativeProviderOutputs, String> {
        if select_provider_execution_adapter(adapter.kind).is_none() {
            return Ok(NativeProviderOutputs::empty());
        }
        let Some(mut collection) =
            provider_request_collection_from_evidence(&record.input_evidence)
        else {
            if declares_provider_request_contract(&record.input_evidence) {
                return Err(format!(
                    "provider request evidence for trace `{}` declares a request contract but failed validation",
                    record.trace_id
                ));
            }
            return Ok(NativeProviderOutputs::empty());
        };
        collection.compiled_code_asset_selection =
            crate::provider_code_asset::contribution::validate_compiled_contribution_selection(
                output_dir,
                &record.input_evidence,
                &collection.requests,
            )?;
        let arguments = prepare_runtime_request_arguments(
            &mut collection.requests,
            adapter,
            runtime_arguments,
        )?;
        let code_asset_identity = collection.code_asset_identity.clone();
        let compiled_code_asset_selection = collection.compiled_code_asset_selection.clone();
        let mut completed = CompletedProviderOutputs::new();
        let mut summaries = Vec::with_capacity(collection.requests.len());
        let mut transport_receipts = Vec::new();
        let mut runtime_results = Vec::new();
        for request in &collection.requests {
            if request.code_asset.is_some() {
                validate_provider_code_asset(output_dir, request)?;
            }
            let request_adapter = request
                .adapter_binding
                .as_ref()
                .map(|binding| select_provider_runner_adapter(&binding.provider_family));
            let effective_adapter = request_adapter.as_ref().unwrap_or(adapter);
            if request.adapter_binding.as_ref().is_some_and(|binding| {
                binding.execution_requirement == "real-device"
                    && !effective_adapter.real_device_capable
            }) {
                return Err(format!(
                    "provider request `{}` requires an unavailable real-device adapter",
                    request.kernel.id
                ));
            }
            let session_adapter = select_provider_session_adapter(effective_adapter.execution_mode)
                .ok_or_else(|| {
                    format!(
                        "provider adapter `{}` has no registered session adapter",
                        effective_adapter.adapter_id
                    )
                })?;
            let provider_family = request
                .adapter_binding
                .as_ref()
                .map(|binding| binding.provider_family.as_str())
                .unwrap_or(&record.provider_family);
            let session = self
                .sessions
                .entry(effective_adapter.adapter_id.to_owned())
                .or_insert_with(|| {
                    ProviderSessionLease::open(&record.trace_id, provider_family, session_adapter)
                });
            let output_roles = request
                .output_bindings
                .iter()
                .map(|binding| binding.role.clone())
                .collect::<Vec<_>>();
            let session_request =
                session.begin_request_with_output_roles(&request.kernel.id, &output_roles)?;
            let mut execution = execute_native_provider_request(
                NativeProviderRequestContext {
                    output_dir,
                    record,
                    adapter: effective_adapter,
                    request,
                    completed: &completed,
                    provider_family,
                    session_request: &session_request,
                },
                &mut self.worker_leases,
                &mut self.process_adapter_cache,
            )?;
            session.complete_request(&request.kernel.id)?;
            bind_session_output(&mut execution.summary, &session_request);
            bind_output_binding_summary(&mut execution.summary, request);
            let runtime_result = ProviderRuntimeResult::from_execution(
                provider_family,
                request,
                &execution,
                arguments.get(&request.kernel.id),
            )?;
            bind_output_comparisons(output_dir, request, &mut execution)?;
            let primary_binding = request
                .output_bindings
                .first()
                .expect("validated provider request has a primary output binding");
            completed.insert(
                &request.kernel.id,
                CompletedProviderOutput {
                    role: primary_binding.role.clone(),
                    buffer: primary_binding.buffer.clone(),
                    payload: execution.output_payload,
                    transferable: execution.transferable_output,
                },
            )?;
            for output in execution.additional_outputs {
                completed.insert(&request.kernel.id, output)?;
            }
            summaries.push(execution.summary);
            if let Some(runtime_result) = runtime_result {
                runtime_results.push(runtime_result);
            }
            transport_receipts.extend(execution.transport_receipts);
        }
        let graph_output_close = completed.close();
        for summary in &mut summaries {
            bind_provider_completion_evidence(summary, &graph_output_close)?;
        }
        Ok(NativeProviderOutputs {
            native_outputs: summaries,
            transport_receipts,
            code_asset_identity,
            compiled_code_asset_selection,
            runtime_results,
            runtime_session_evidence: None,
        })
    }

    pub(crate) fn close(self) -> Result<(), String> {
        #[cfg(unix)]
        {
            let mut sessions = self.sessions;
            for session in sessions.values_mut() {
                session.close()?;
            }
            self.worker_leases.close()
        }
        #[cfg(not(unix))]
        {
            Ok(())
        }
    }
}

#[cfg(unix)]
fn prepare_runtime_request_arguments(
    requests: &mut [ProviderRequest],
    adapter: &ProviderRunnerAdapter,
    supplied: Option<(
        &yir_core::provider_runtime_ipc::DispatchTarget,
        &yir_core::provider_runtime_ipc::DispatchArguments,
    )>,
) -> Result<BTreeMap<String, yir_core::provider_runtime_ipc::DispatchArguments>, String> {
    let mut prepared = BTreeMap::new();
    // Preflight the whole collection before starting any dependency or opening a worker.
    for request in requests {
        let Some(binding) = request.runtime_result_binding.as_ref() else {
            continue;
        };
        let arguments = match supplied {
            Some((target, arguments))
                if binding.source_yir_fnv1a64 == target.source_yir_fnv1a64
                    && binding.module == target.module
                    && binding.instruction == target.instruction
                    && binding.node == target.node
                    && binding.resource == target.resource =>
            {
                Some(arguments)
            }
            Some(_) => {
                return Err("runtime argument target does not match admitted request".to_owned())
            }
            None => None,
        };
        let request_adapter = request
            .adapter_binding
            .as_ref()
            .map(|binding| select_provider_runner_adapter(&binding.provider_family));
        let prepare =
            select_provider_execution_adapter(request_adapter.as_ref().unwrap_or(adapter).kind)
                .and_then(|adapter| adapter.prepare_runtime_arguments)
                .ok_or("registered provider does not support runtime argument binding")?;
        let arguments = prepare(request, arguments)?;
        if prepared
            .insert(request.kernel.id.clone(), arguments)
            .is_some()
        {
            return Err("runtime request identity is duplicated".to_owned());
        }
    }
    if supplied.is_some() && prepared.len() != 1 {
        return Err("runtime arguments require exactly one admitted request".to_owned());
    }
    Ok(prepared)
}

fn declares_provider_request_contract(input_evidence: &str) -> bool {
    input_evidence.split(';').any(|field| {
        field.trim().split_once('=').is_some_and(|(name, _)| {
            matches!(
                name,
                "provider_request_collection_contract"
                    | "provider_buffer_descriptor_contract"
                    | "provider_kernel_descriptor_contract"
            )
        })
    })
}

#[cfg(unix)]
fn bind_output_comparisons(
    output_dir: &Path,
    request: &ProviderRequest,
    execution: &mut ProviderRequestExecution,
) -> Result<(), String> {
    let mut payloads = vec![(
        request.output_bindings[0].buffer.as_str(),
        execution.output_payload.as_bytes(),
    )];
    payloads.extend(
        execution
            .additional_outputs
            .iter()
            .map(|output| (output.buffer.as_str(), output.payload.as_bytes())),
    );
    let results =
        compare_provider_output_collection(output_dir, &request.output_comparisons, &payloads)?;
    bind_output_comparison_collection(
        &mut execution.summary,
        &results,
        &request.kernel.output_buffer,
    );
    Ok(())
}

#[cfg(unix)]
struct NativeProviderRequestContext<'a> {
    output_dir: &'a Path,
    record: &'a NsdbDeviceProviderSampleRecordInfo,
    adapter: &'a ProviderRunnerAdapter,
    request: &'a ProviderRequest,
    completed: &'a CompletedProviderOutputs,
    provider_family: &'a str,
    session_request: &'a ProviderSessionRequest,
}

#[cfg(unix)]
fn execute_native_provider_request(
    context: NativeProviderRequestContext<'_>,
    worker_leases: &mut ProviderWorkerLeaseManager,
    process_adapter_cache: &mut ProviderProcessAdapterCache,
) -> Result<ProviderRequestExecution, String> {
    let NativeProviderRequestContext {
        output_dir,
        record,
        adapter,
        request,
        completed,
        provider_family,
        session_request,
    } = context;
    let execution_adapter = select_provider_execution_adapter(adapter.kind).ok_or_else(|| {
        format!(
            "provider adapter `{}` has no registered execution implementation",
            adapter.adapter_id
        )
    })?;
    let mut inputs = request
        .input_bindings
        .iter()
        .map(|binding| {
            let transport = request
                .dependencies
                .iter()
                .find(|dependency| dependency.consumer_input_buffer == binding.name)
                .and_then(|dependency| dependency.transport.as_ref());
            PreparedProviderInput::new(
                output_dir,
                binding,
                transport,
                completed,
                execution_adapter.requires_worker_descriptors,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    for upload in request.runtime_uploads.values() {
        inputs.push(PreparedProviderInput::from_runtime_upload(upload)?);
    }
    let (adapter_output_roles, adapter_output_byte_lengths) = provider_output_manifest(request);
    let prepared_worker_adapter = execution_adapter
        .prepare_worker_adapter
        .map(|prepare| prepare(process_adapter_cache, output_dir, request, &inputs))
        .transpose()?
        .flatten();
    let worker_adapter_launch =
        prepared_worker_adapter
            .as_ref()
            .map(|prepared| ProviderWorkerAdapterLaunch {
                executable_path: &prepared.executable_path,
                executable_hash: &prepared.executable_hash,
                runner_contract: prepared.runner_contract,
                cache_contract: PROVIDER_PROCESS_ADAPTER_CACHE_CONTRACT,
                cache_identity: &prepared.cache_identity,
                cache_status: prepared.cache_status,
                arguments: &prepared.arguments,
                output_roles: &adapter_output_roles,
                output_byte_lengths: &adapter_output_byte_lengths,
            });
    let mut worker_receipt = worker_leases.dispatch(
        adapter.adapter_id,
        provider_family,
        ProviderWorkerDispatchIdentity {
            lease_id: &session_request.lease_id,
            sequence: session_request.sequence,
        },
        request,
        &inputs,
        worker_adapter_launch.as_ref(),
    )?;
    let mut request_execution = (execution_adapter.execute)(
        &record.input_evidence,
        provider_family,
        output_dir,
        request,
        &inputs,
        &mut worker_receipt,
    )?;
    if request_execution.summary.request_id != request.kernel.id {
        return Err(format!(
            "provider adapter `{}` returned output for request `{}` while executing `{}`",
            adapter.adapter_id, request_execution.summary.request_id, request.kernel.id
        ));
    }
    request_execution.transport_receipts = inputs
        .into_iter()
        .map(PreparedProviderInput::finish)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    bind_worker_output(
        &mut request_execution.summary,
        &worker_receipt,
        worker_adapter_launch.as_ref(),
    );
    request_execution.additional_outputs = completed_additional_worker_outputs(
        request,
        std::mem::take(&mut worker_receipt.additional_worker_outputs),
    )?;
    Ok(request_execution)
}
