#[cfg(unix)]
use crate::provider_graph_output::completed_additional_worker_outputs;
#[cfg(target_os = "macos")]
use crate::provider_process_adapter::worker_descriptor_argument;
#[cfg(unix)]
use crate::provider_worker_lease::{ProviderWorkerAdapterLaunch, ProviderWorkerLeaseManager};
#[cfg(unix)]
use crate::provider_worker_summary::bind_worker_output;
use crate::{
    provider_edge_transport::ProviderEdgeTransportReceipt,
    provider_graph_output::{
        bind_output_binding_summary, CompletedProviderOutput, CompletedProviderOutputs,
    },
    provider_output_carrier_registry::ProviderOutputPayload,
    provider_output_comparison::{
        bind_output_comparison_collection, compare_provider_output_collection,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_process_adapter::{
        coreml_worker_arguments, provider_output_manifest, validate_provider_model_asset,
        ProviderProcessAdapterCache, PROVIDER_PROCESS_ADAPTER_CACHE_CONTRACT,
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
        coreml_native_output_summary, fnv1a64_hex, metal_native_output_summary,
        pixelmagic_metal_output_summary, pixelmagic_native_output_summary,
        provider_output_payload_file_name, render_real_device_provider_output_payload,
        PixelMagicNativeOutputSummary,
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
    native_outputs: Vec<PixelMagicNativeOutputSummary>,
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
    native_outputs: Vec<PixelMagicNativeOutputSummary>,
    transport_receipts: Vec<ProviderEdgeTransportReceipt>,
}

fn execute_native_provider_outputs(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
) -> Result<NativeProviderOutputs, String> {
    if !matches!(
        adapter.kind,
        "metal-real-device-runner" | "coreml-real-device-runner" | "provider-worker-native-runner"
    ) {
        return Ok(NativeProviderOutputs {
            native_outputs: Vec::new(),
            transport_receipts: Vec::new(),
        });
    }
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
    #[cfg(target_os = "macos")]
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
            #[cfg(target_os = "macos")]
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

struct NativeProviderRequestExecution {
    summary: PixelMagicNativeOutputSummary,
    output_payload: ProviderOutputPayload,
    transferable_output:
        Option<crate::provider_carrier_channel_registry::PreparedProviderCarrierChannel>,
    additional_outputs: Vec<CompletedProviderOutput>,
    transport_receipts: Vec<ProviderEdgeTransportReceipt>,
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
    #[cfg(target_os = "macos")] process_adapter_cache: &mut ProviderProcessAdapterCache,
) -> Result<NativeProviderRequestExecution, String> {
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
                matches!(
                    adapter.kind,
                    "metal-real-device-runner" | "coreml-real-device-runner"
                ),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let verified_coreml_model_path = if adapter.kind == "coreml-real-device-runner" {
        Some(validate_provider_model_asset(output_dir, request)?)
    } else {
        None
    };
    let (adapter_output_roles, adapter_output_byte_lengths) = provider_output_manifest(request);
    #[cfg(target_os = "macos")]
    let prepared_worker_adapter = if adapter.kind == "metal-real-device-runner" {
        if request.buffer.element_type == "u8"
            && request.buffer.layout.contains("pixel-format=gray8")
            && request.kernel.operation == "invert"
        {
            request.scalar_u8("max_value").ok_or_else(|| {
                "Metal provider request is missing u8 scalar `max_value`".to_owned()
            })?;
            Some(
                crate::provider_runner_metal::prepare_gray8_worker_invocation(
                    process_adapter_cache,
                )?,
            )
        } else if request.buffer.element_type == "f32"
            && request.buffer.layout == "tensor-contiguous"
            && request.kernel.operation == "bias"
        {
            request
                .scalar_f32("bias")
                .ok_or_else(|| "Metal provider request is missing f32 scalar `bias`".to_owned())?;
            Some(
                crate::provider_runner_metal::prepare_f32_bias_worker_invocation(
                    process_adapter_cache,
                )?,
            )
        } else {
            None
        }
    } else if adapter.kind == "coreml-real-device-runner"
        && !inputs.is_empty()
        && inputs
            .iter()
            .all(|input| input.worker_adapter_argument().is_some())
        && request
            .model_asset
            .as_ref()
            .is_some_and(|model| model.input_features.len() == inputs.len())
    {
        Some(
            crate::provider_runner_coreml::prepare_coreml_worker_invocation(process_adapter_cache)?,
        )
    } else {
        None
    };
    #[cfg(target_os = "macos")]
    let worker_adapter_arguments = if prepared_worker_adapter.is_some() {
        if adapter.kind == "metal-real-device-runner" {
            let input_argument = worker_descriptor_argument(&inputs[0], 0)?;
            let scalar = if request.kernel.operation == "invert" {
                request
                    .scalar_u8("max_value")
                    .expect("validated Metal max value")
                    .to_string()
            } else {
                request
                    .scalar_f32("bias")
                    .expect("validated Metal bias")
                    .to_string()
            };
            vec![input_argument, format!("literal:{scalar}")]
        } else {
            let model_path = verified_coreml_model_path
                .as_ref()
                .expect("validated CoreML model path")
                .as_path();
            coreml_worker_arguments(request, &inputs, model_path)?
        }
    } else {
        Vec::new()
    };
    #[cfg(target_os = "macos")]
    let worker_adapter_launch =
        prepared_worker_adapter
            .as_ref()
            .map(|prepared| ProviderWorkerAdapterLaunch {
                executable_path: prepared.executable_path(),
                executable_hash: prepared.executable_hash(),
                runner_contract: prepared.contract(),
                cache_contract: PROVIDER_PROCESS_ADAPTER_CACHE_CONTRACT,
                cache_identity: prepared.cache_identity,
                cache_status: prepared.cache_status,
                arguments: &worker_adapter_arguments,
                output_roles: &adapter_output_roles,
                output_byte_lengths: &adapter_output_byte_lengths,
            });
    #[cfg(all(unix, not(target_os = "macos")))]
    let worker_adapter_launch: Option<ProviderWorkerAdapterLaunch<'_>> = None;
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
    let mut request_execution = match adapter.kind {
        "provider-worker-native-runner" => {
            let (summary, output_payload, transferable_output) =
                crate::provider_worker_native_execution::take_provider_worker_native_output(
                    &record.input_evidence,
                    provider_family,
                    request,
                    &mut worker_receipt,
                )?;
            Ok(NativeProviderRequestExecution {
                summary,
                output_payload,
                transferable_output,
                additional_outputs: Vec::new(),
                transport_receipts: Vec::new(),
            })
        }
        "metal-real-device-runner" => {
            if inputs.len() != 1 {
                return Err(format!(
                    "Metal provider adapter requires one input for kernel `{}`",
                    request.kernel.id
                ));
            }
            let execution = if request.buffer.element_type == "u8"
                && request.buffer.layout.contains("pixel-format=gray8")
                && request.kernel.operation == "invert"
            {
                let max_value = request.scalar_u8("max_value").ok_or_else(|| {
                    "Metal provider request is missing u8 scalar `max_value`".to_owned()
                })?;
                if worker_receipt.execution_capsule_invocation_mode
                    == "nuis-provider-worker-process-adapter-v5"
                {
                    crate::provider_runner_metal::parse_metal_worker_output(
                        &worker_receipt.worker_output_payload,
                        "nuis-metal-gray8-provider-runner-v1",
                        worker_receipt.worker_output_result.take(),
                    )?
                } else {
                    let path = inputs[0]
                        .input()
                        .path()
                        .ok_or_else(|| "Metal gray8 provider requires a path input".to_owned())?;
                    crate::provider_runner_metal::execute_gray8_invert(path, max_value)?
                }
            } else if request.buffer.element_type == "f32"
                && request.buffer.layout == "tensor-contiguous"
                && request.kernel.operation == "bias"
            {
                let bias = request.scalar_f32("bias").ok_or_else(|| {
                    "Metal provider request is missing f32 scalar `bias`".to_owned()
                })?;
                if worker_receipt.execution_capsule_invocation_mode
                    == "nuis-provider-worker-process-adapter-v5"
                {
                    crate::provider_runner_metal::parse_metal_worker_output(
                        &worker_receipt.worker_output_payload,
                        "nuis-metal-f32-bias-provider-runner-v1",
                        worker_receipt.worker_output_result.take(),
                    )?
                } else if let Some(channel) = inputs[0].direct_channel() {
                    crate::provider_runner_metal::execute_f32_bias_prepared_channel(
                        channel,
                        request.input_bindings[0].byte_length,
                        bias,
                    )?
                } else {
                    crate::provider_runner_metal::execute_f32_bias_input(inputs[0].input(), bias)?
                }
            } else {
                return Err(format!(
                    "Metal provider adapter does not support buffer `{}` operation `{}`",
                    request.buffer.layout, request.kernel.operation
                ));
            };
            Ok(NativeProviderRequestExecution {
                summary: if request.kernel.operation == "invert" {
                    pixelmagic_metal_output_summary(&record.input_evidence, &execution)
                } else {
                    metal_native_output_summary(
                        request.kernel.id.clone(),
                        "provider-tensor-f32",
                        &execution,
                        None,
                    )
                },
                output_payload: execution.output_payload,
                transferable_output: execution.transferable_output,
                additional_outputs: Vec::new(),
                transport_receipts: Vec::new(),
            })
        }
        "coreml-real-device-runner" => {
            if request.buffer.element_type != "f32" || request.buffer.layout != "tensor-contiguous"
            {
                return Err(format!(
                    "CoreML provider adapter requires a contiguous f32 tensor, got `{}` with `{}` elements",
                    request.buffer.layout, request.buffer.element_type
                ));
            }
            let model = request
                .model_asset
                .as_ref()
                .expect("CoreML model descriptor was validated");
            let model_path = verified_coreml_model_path
                .as_ref()
                .expect("CoreML model path was validated");
            let coreml_inputs = inputs
                .iter()
                .zip(&model.input_features)
                .zip(&request.input_bindings)
                .map(|((input, feature), binding)| {
                    let source = input.direct_channel().map_or_else(
                        || {
                            crate::provider_runner_coreml::CoreMlProviderInputSource::Carrier(
                                input.input(),
                            )
                        },
                        crate::provider_runner_coreml::CoreMlProviderInputSource::PreparedChannel,
                    );
                    crate::provider_runner_coreml::CoreMlProviderInput {
                        source,
                        feature,
                        shape: &binding.shape,
                    }
                })
                .collect::<Vec<_>>();
            let output_shape = request
                .output_comparison
                .as_ref()
                .map(|comparison| comparison.shape.as_slice())
                .unwrap_or(request.buffer.shape.as_slice());
            let execution = if worker_receipt.execution_capsule_invocation_mode
                == "nuis-provider-worker-process-adapter-v5"
            {
                crate::provider_runner_coreml::parse_coreml_worker_output(
                    &worker_receipt.worker_output_payload,
                    worker_receipt.worker_output_result.take(),
                )?
            } else {
                crate::provider_runner_coreml::execute_model_prediction_inputs(
                    model_path,
                    &coreml_inputs,
                    &model.output_feature,
                    output_shape,
                )?
            };
            Ok(NativeProviderRequestExecution {
                summary: coreml_native_output_summary(&request.kernel.id, &execution, None),
                output_payload: execution.output_payload,
                transferable_output: execution.transferable_output,
                additional_outputs: Vec::new(),
                transport_receipts: Vec::new(),
            })
        }
        _ => Err(format!(
            "provider adapter `{}` cannot execute request `{}`",
            adapter.adapter_id, request.kernel.id
        )),
    }?;
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
