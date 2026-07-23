#[cfg(unix)]
use crate::provider_execution_adapter::{
    select_provider_execution_adapter, ProviderRequestExecution,
};
#[cfg(unix)]
use crate::provider_graph_output::completed_additional_worker_outputs;
#[cfg(unix)]
use crate::provider_worker_lease::{ProviderWorkerAdapterLaunch, ProviderWorkerLeaseManager};
#[cfg(unix)]
use crate::provider_worker_summary::bind_worker_output;
use crate::{
    provider_edge_transport::ProviderEdgeTransportReceipt,
    provider_graph_output::{
        bind_output_binding_summary, CompletedProviderOutput, CompletedProviderOutputs,
    },
    provider_output_comparison::{
        bind_output_comparison_collection, compare_provider_output_collection,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_process_adapter::{
        provider_output_manifest, ProviderProcessAdapterCache,
        PROVIDER_PROCESS_ADAPTER_CACHE_CONTRACT,
    },
    provider_request::{provider_request_collection_from_evidence, ProviderRequest},
    provider_runner_registry::{
        provider_runner_real_device_probe_status, select_provider_runner_adapter,
    },
    provider_sample::{
        read_device_provider_sample_manifest_info, DEVICE_PROVIDER_SAMPLE_PROTOCOL,
        DEVICE_PROVIDER_SAMPLE_SCHEMA,
    },
    provider_sample_execution::provider_execution_outcome,
    provider_sample_payload::{
        fnv1a64_hex, pixelmagic_native_output_summary, provider_output_payload_file_name,
        render_real_device_provider_output_payload, ProviderNativeOutputSummary,
    },
    provider_session_registry::{
        select_provider_session_adapter, ProviderSessionLease, ProviderSessionRequest,
    },
    provider_session_summary::bind_session_output,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub struct ProviderSampleExecuteReport {
    pub status: String,
    pub provider_family_filter: Option<String>,
    pub provider_families: Vec<String>,
    pub record_count: usize,
    pub matched_record_count: usize,
    pub executable_record_count: usize,
    pub output_payload_count: usize,
    pub first_provider_family: String,
    pub first_provider_runner_adapter_id: String,
    pub first_provider_runner_adapter_capability_status: String,
    pub first_provider_runner_real_device_capable: bool,
    pub first_provider_runner_real_device_probe_status: String,
    pub first_provider_execution_mode: String,
    pub first_output_payload_evidence: String,
    pub first_output_payload_comparison_contract: String,
    pub first_output_payload_comparison_status: String,
    pub first_output_payload_input_evidence: String,
    pub first_output_payload_input_evidence_hash: String,
    pub first_output_payload_native_output_kind: String,
    pub first_output_payload_native_output_status: String,
    pub first_output_payload_native_output_bytes: String,
    pub first_output_payload_native_output_hash: String,
    pub first_output_payload_native_execution_contract: String,
    pub first_output_payload_native_execution_status: String,
    pub first_output_payload_native_device: String,
    pub first_output_payload_native_compute_plan_contract: String,
    pub first_output_payload_native_compute_plan_status: String,
    pub first_output_payload_native_compute_plan_layer_count: String,
    pub first_output_payload_native_compute_plan_preferred_devices: String,
    pub first_output_payload_native_compute_plan_supported_devices: String,
    pub next_action: String,
    pub next_command: String,
}

pub fn execute_provider_samples(
    output_dir: &Path,
    provider_family_filter: Option<&str>,
) -> Result<ProviderSampleExecuteReport, String> {
    let manifest = read_device_provider_sample_manifest_info(output_dir);
    if !manifest.available {
        return Err(format!(
            "device provider sample manifest not found at `{}`",
            manifest.path
        ));
    }
    if manifest.protocol != DEVICE_PROVIDER_SAMPLE_PROTOCOL
        || manifest.schema != DEVICE_PROVIDER_SAMPLE_SCHEMA
    {
        return Err(format!(
            "unsupported device provider sample manifest protocol `{}` schema `{}`",
            manifest.protocol, manifest.schema
        ));
    }
    let provider_families = manifest
        .records
        .iter()
        .map(|record| record.provider_family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let matched_records = manifest
        .records
        .iter()
        .filter(|record| {
            provider_family_filter.is_none_or(|family| record.provider_family == family)
        })
        .collect::<Vec<_>>();
    let first_provider_boundary = matched_records
        .first()
        .map(|record| {
            let adapter = select_provider_runner_adapter(&record.provider_family);
            let outcome = provider_execution_outcome(&adapter);
            (
                record.provider_family.clone(),
                adapter.adapter_id.to_owned(),
                adapter.capability_status.to_owned(),
                adapter.real_device_capable,
                provider_runner_real_device_probe_status(&record.provider_family).to_owned(),
                adapter.execution_mode.to_owned(),
                outcome.contract.to_owned(),
                record.input_evidence.clone(),
                fnv1a64_hex(record.input_evidence.as_bytes()),
            )
        })
        .unwrap_or_else(|| {
            (
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                false,
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
            )
        });
    let mut output_payloads = Vec::new();
    for record in &matched_records {
        let adapter = select_provider_runner_adapter(&record.provider_family);
        if !adapter.real_device_capable {
            continue;
        }
        output_payloads.push(write_provider_output_payload(output_dir, record, &adapter)?);
    }
    let first_native_output = output_payloads
        .first()
        .and_then(|payload| payload.native_outputs.first())
        .map(|summary| {
            (
                summary.kind.clone(),
                summary.status.clone(),
                summary.bytes.clone(),
                summary.hash.clone(),
                summary.execution_contract.clone(),
                summary.execution_status.clone(),
                summary.device.clone(),
                summary.compute_plan_contract.clone(),
                summary.compute_plan_status.clone(),
                summary.compute_plan_layer_count.clone(),
                summary.compute_plan_preferred_devices.clone(),
                summary.compute_plan_supported_devices.clone(),
            )
        })
        .or_else(|| {
            matched_records.first().and_then(|record| {
                pixelmagic_native_output_summary(&record.input_evidence, &record.provider_family)
                    .map(|summary| {
                        (
                            summary.kind,
                            summary.status,
                            summary.bytes,
                            summary.hash,
                            summary.execution_contract,
                            summary.execution_status,
                            summary.device,
                            summary.compute_plan_contract,
                            summary.compute_plan_status,
                            summary.compute_plan_layer_count,
                            summary.compute_plan_preferred_devices,
                            summary.compute_plan_supported_devices,
                        )
                    })
            })
        })
        .unwrap_or_else(|| {
            (
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "0".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
            )
        });
    let first_output_payload_comparison_status = output_payloads
        .first()
        .and_then(|payload| payload.native_outputs.first())
        .filter(|summary| summary.comparison_contract != "none")
        .map(|summary| summary.comparison_status.clone())
        .unwrap_or_else(|| {
            output_payload_comparison_status(
                !output_payloads.is_empty(),
                &first_provider_boundary.2,
            )
            .to_owned()
        });
    Ok(ProviderSampleExecuteReport {
        status: if output_payloads.is_empty() {
            "no-real-device-provider-output".to_owned()
        } else {
            "provider-output-payloads-ready".to_owned()
        },
        provider_family_filter: provider_family_filter.map(str::to_owned),
        provider_families,
        record_count: manifest.records.len(),
        matched_record_count: matched_records.len(),
        executable_record_count: output_payloads.len(),
        output_payload_count: output_payloads.len(),
        first_provider_family: first_provider_boundary.0,
        first_provider_runner_adapter_id: first_provider_boundary.1,
        first_provider_runner_adapter_capability_status: first_provider_boundary.2,
        first_provider_runner_real_device_capable: first_provider_boundary.3,
        first_provider_runner_real_device_probe_status: first_provider_boundary.4,
        first_provider_execution_mode: first_provider_boundary.5,
        first_output_payload_evidence: output_payloads
            .first()
            .map(|payload| payload.evidence.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_output_payload_comparison_contract: first_provider_boundary.6,
        first_output_payload_comparison_status,
        first_output_payload_input_evidence: first_provider_boundary.7,
        first_output_payload_input_evidence_hash: first_provider_boundary.8,
        first_output_payload_native_output_kind: first_native_output.0,
        first_output_payload_native_output_status: first_native_output.1,
        first_output_payload_native_output_bytes: first_native_output.2,
        first_output_payload_native_output_hash: first_native_output.3,
        first_output_payload_native_execution_contract: first_native_output.4,
        first_output_payload_native_execution_status: first_native_output.5,
        first_output_payload_native_device: first_native_output.6,
        first_output_payload_native_compute_plan_contract: first_native_output.7,
        first_output_payload_native_compute_plan_status: first_native_output.8,
        first_output_payload_native_compute_plan_layer_count: first_native_output.9,
        first_output_payload_native_compute_plan_preferred_devices: first_native_output.10,
        first_output_payload_native_compute_plan_supported_devices: first_native_output.11,
        next_action: "materialize-provider-samples".to_owned(),
        next_command: format!(
            "nsdb materialize-provider-samples {} --json",
            output_dir.display()
        ),
    })
}

fn output_payload_comparison_status(payload_ready: bool, capability_status: &str) -> &'static str {
    if payload_ready {
        "ready-for-comparison"
    } else if capability_status == "registered-real-device" {
        "awaiting-provider-output-payload"
    } else {
        "host-fallback-output-comparison-deferred"
    }
}

struct WrittenProviderOutput {
    evidence: String,
    native_outputs: Vec<ProviderNativeOutputSummary>,
}

fn write_provider_output_payload(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
) -> Result<WrittenProviderOutput, String> {
    let file_name = provider_output_payload_file_name(&record.provider_family);
    let execution = execute_native_provider_outputs(output_dir, record, adapter)?;
    let content = render_real_device_provider_output_payload(
        record,
        adapter,
        &execution.native_outputs,
        &execution.transport_receipts,
    );
    let hash = fnv1a64_hex(content.as_bytes());
    fs::write(output_dir.join(&file_name), content).map_err(|error| {
        format!("failed to write provider output payload `{file_name}`: {error}")
    })?;
    Ok(WrittenProviderOutput {
        evidence: format!("{file_name}:hash={hash}:status=written"),
        native_outputs: execution.native_outputs,
    })
}

struct NativeProviderOutputs {
    native_outputs: Vec<ProviderNativeOutputSummary>,
    transport_receipts: Vec<ProviderEdgeTransportReceipt>,
}

fn execute_native_provider_outputs(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
) -> Result<NativeProviderOutputs, String> {
    #[cfg(unix)]
    if select_provider_execution_adapter(adapter.kind).is_none() {
        return Ok(NativeProviderOutputs {
            native_outputs: Vec::new(),
            transport_receipts: Vec::new(),
        });
    }
    #[cfg(not(unix))]
    return Ok(NativeProviderOutputs {
        native_outputs: Vec::new(),
        transport_receipts: Vec::new(),
    });
    let Some(collection) = provider_request_collection_from_evidence(&record.input_evidence) else {
        return Ok(NativeProviderOutputs {
            native_outputs: Vec::new(),
            transport_receipts: Vec::new(),
        });
    };
    let mut completed = CompletedProviderOutputs::new();
    let mut sessions = BTreeMap::<String, ProviderSessionLease>::new();
    #[cfg(unix)]
    let mut worker_leases = ProviderWorkerLeaseManager::new(output_dir);
    #[cfg(unix)]
    let mut process_adapter_cache = ProviderProcessAdapterCache::default();
    let mut summaries = Vec::with_capacity(collection.requests.len());
    let mut transport_receipts = Vec::new();
    for request in &collection.requests {
        let request_adapter = request
            .adapter_binding
            .as_ref()
            .map(|binding| select_provider_runner_adapter(&binding.provider_family));
        let effective_adapter = request_adapter.as_ref().unwrap_or(adapter);
        if request.adapter_binding.as_ref().is_some_and(|binding| {
            binding.execution_requirement == "real-device" && !effective_adapter.real_device_capable
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
        let session = sessions
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
            output_dir,
            record,
            effective_adapter,
            request,
            &completed,
            provider_family,
            &session_request,
            #[cfg(unix)]
            &mut worker_leases,
            #[cfg(unix)]
            &mut process_adapter_cache,
        )?;
        session.complete_request(&request.kernel.id)?;
        bind_session_output(&mut execution.summary, &session_request);
        bind_output_binding_summary(&mut execution.summary, request);
        let mut comparison_payloads = vec![(
            request.output_bindings[0].buffer.as_str(),
            execution.output_payload.as_bytes(),
        )];
        comparison_payloads.extend(
            execution
                .additional_outputs
                .iter()
                .map(|output| (output.buffer.as_str(), output.payload.as_bytes())),
        );
        let comparison_results = compare_provider_output_collection(
            output_dir,
            &request.output_comparisons,
            &comparison_payloads,
        )?;
        bind_output_comparison_collection(
            &mut execution.summary,
            &comparison_results,
            &request.kernel.output_buffer,
        );
        let primary_binding = request
            .output_bindings
            .first()
            .expect("validated provider request has a primary output binding");
        let primary_output = CompletedProviderOutput {
            role: primary_binding.role.clone(),
            buffer: primary_binding.buffer.clone(),
            payload: execution.output_payload,
            transferable: execution.transferable_output,
        };
        completed.insert(&request.kernel.id, primary_output)?;
        for output in execution.additional_outputs {
            completed.insert(&request.kernel.id, output)?;
        }
        summaries.push(execution.summary);
        transport_receipts.extend(execution.transport_receipts);
    }
    let graph_output_close = completed.close();
    for session in sessions.values_mut() {
        session.close()?;
    }
    #[cfg(unix)]
    worker_leases.close()?;
    for summary in &mut summaries {
        summary.output_handle_release_status = "released-at-graph-close".to_owned();
        summary.graph_output_ownership_contract = graph_output_close.contract.to_owned();
        summary.graph_output_release_count = graph_output_close.released_output_count.to_string();
        summary.graph_output_release_roles = graph_output_close.released_output_roles.clone();
    }
    Ok(NativeProviderOutputs {
        native_outputs: summaries,
        transport_receipts,
    })
}

fn execute_native_provider_request(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
    request: &ProviderRequest,
    completed: &CompletedProviderOutputs,
    provider_family: &str,
    session_request: &ProviderSessionRequest,
    #[cfg(unix)] worker_leases: &mut ProviderWorkerLeaseManager,
    #[cfg(unix)] process_adapter_cache: &mut ProviderProcessAdapterCache,
) -> Result<ProviderRequestExecution, String> {
    #[cfg(unix)]
    let execution_adapter = select_provider_execution_adapter(adapter.kind).ok_or_else(|| {
        format!(
            "provider adapter `{}` has no registered execution implementation",
            adapter.adapter_id
        )
    })?;
    let inputs = request
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
                #[cfg(unix)]
                execution_adapter.requires_worker_descriptors,
                #[cfg(not(unix))]
                false,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (adapter_output_roles, adapter_output_byte_lengths) = provider_output_manifest(request);
    #[cfg(unix)]
    let prepared_worker_adapter = execution_adapter
        .prepare_worker_adapter
        .map(|prepare| prepare(process_adapter_cache, output_dir, request, &inputs))
        .transpose()?
        .flatten();
    #[cfg(unix)]
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
    #[cfg(unix)]
    let mut worker_receipt = worker_leases.dispatch(
        adapter.adapter_id,
        provider_family,
        &session_request.lease_id,
        session_request.sequence,
        request,
        &inputs,
        worker_adapter_launch.as_ref(),
    )?;
    #[cfg(not(unix))]
    return Err("native provider worker leases require a registered host transport".to_owned());
    let mut request_execution = (execution_adapter.execute)(
        &record.input_evidence,
        provider_family,
        output_dir,
        request,
        &inputs,
        &mut worker_receipt,
    )?;
    request_execution.transport_receipts = inputs
        .into_iter()
        .map(PreparedProviderInput::finish)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    #[cfg(unix)]
    bind_worker_output(
        &mut request_execution.summary,
        &worker_receipt,
        worker_adapter_launch.as_ref(),
    );
    #[cfg(unix)]
    {
        request_execution.additional_outputs = completed_additional_worker_outputs(
            request,
            std::mem::take(&mut worker_receipt.additional_worker_outputs),
        )?;
    }
    Ok(request_execution)
}

pub(crate) fn resolve_provider_payload_path(
    output_dir: &Path,
    relative: &str,
) -> Result<std::path::PathBuf, String> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().count() != 1
        || !matches!(
            relative.components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err("provider input buffer path must be one output-relative file name".to_owned());
    }
    Ok(output_dir.join(relative))
}
