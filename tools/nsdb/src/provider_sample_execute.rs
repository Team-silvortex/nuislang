use crate::{
    provider_input_binding::ProviderInputBinding,
    provider_output_comparison::compare_provider_output,
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
        coreml_native_output_summary, fnv1a64_hex, pixelmagic_metal_output_summary,
        pixelmagic_native_output_summary, provider_output_payload_file_name,
        render_real_device_provider_output_payload, PixelMagicNativeOutputSummary,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
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
    let native_outputs = execute_native_provider_outputs(output_dir, record, adapter)?;
    let content = render_real_device_provider_output_payload(record, adapter, &native_outputs);
    let hash = fnv1a64_hex(content.as_bytes());
    fs::write(output_dir.join(&file_name), content).map_err(|error| {
        format!("failed to write provider output payload `{file_name}`: {error}")
    })?;
    Ok(WrittenProviderOutput {
        evidence: format!("{file_name}:hash={hash}:status=written"),
        native_outputs,
    })
}

fn execute_native_provider_outputs(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
) -> Result<Vec<PixelMagicNativeOutputSummary>, String> {
    if adapter.kind != "metal-real-device-runner" && adapter.kind != "coreml-real-device-runner" {
        return Ok(Vec::new());
    }
    let Some(collection) = provider_request_collection_from_evidence(&record.input_evidence) else {
        return Ok(Vec::new());
    };
    let mut completed = BTreeMap::<String, Vec<u8>>::new();
    let mut summaries = Vec::with_capacity(collection.requests.len());
    for request in &collection.requests {
        let execution =
            execute_native_provider_request(output_dir, record, adapter, request, &completed)?;
        completed.insert(request.kernel.id.clone(), execution.output_bytes);
        summaries.push(execution.summary);
    }
    Ok(summaries)
}

struct NativeProviderRequestExecution {
    summary: PixelMagicNativeOutputSummary,
    output_bytes: Vec<u8>,
}

fn execute_native_provider_request(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
    request: &ProviderRequest,
    completed: &BTreeMap<String, Vec<u8>>,
) -> Result<NativeProviderRequestExecution, String> {
    let inputs = request
        .input_bindings
        .iter()
        .map(|binding| PreparedProviderInput::new(output_dir, binding, completed))
        .collect::<Result<Vec<_>, _>>()?;
    match adapter.kind {
        "metal-real-device-runner" => {
            if inputs.len() != 1
                || request.buffer.element_type != "u8"
                || !request.buffer.layout.contains("pixel-format=gray8")
                || request.kernel.id != "pixelmagic.gray8.invert"
                || request.kernel.operation != "invert"
            {
                return Err(format!(
                    "Metal provider adapter does not support buffer `{}` with kernel `{}`",
                    request.buffer.layout, request.kernel.id
                ));
            }
            let max_value = request.scalar_u8("max_value").ok_or_else(|| {
                "Metal provider request is missing u8 scalar `max_value`".to_owned()
            })?;
            let execution =
                crate::provider_runner_metal::execute_gray8_invert(&inputs[0].path, max_value)?;
            Ok(NativeProviderRequestExecution {
                summary: pixelmagic_metal_output_summary(&record.input_evidence, &execution),
                output_bytes: execution.output_bytes,
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
            let model = request.model_asset.as_ref().ok_or_else(|| {
                "CoreML provider request is missing a model asset descriptor".to_owned()
            })?;
            let model_path = resolve_provider_payload_path(output_dir, &model.path)?;
            let model_bytes = fs::read(&model_path).map_err(|error| {
                format!(
                    "failed to read provider model asset `{}`: {error}",
                    model_path.display()
                )
            })?;
            if model_bytes.len() != model.byte_length
                || fnv1a64_hex(&model_bytes) != model.content_hash
            {
                return Err("provider model asset size/hash evidence mismatch".to_owned());
            }
            let coreml_inputs = inputs
                .iter()
                .zip(&model.input_features)
                .zip(&request.input_bindings)
                .map(|((input, feature), binding)| {
                    crate::provider_runner_coreml::CoreMlProviderInput {
                        path: &input.path,
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
            let execution = crate::provider_runner_coreml::execute_model_prediction_inputs(
                &model_path,
                &coreml_inputs,
                &model.output_feature,
                output_shape,
            )?;
            let comparison = request
                .output_comparison
                .as_ref()
                .map(|descriptor| {
                    compare_provider_output(output_dir, descriptor, &execution.output_bytes)
                })
                .transpose()?;
            Ok(NativeProviderRequestExecution {
                summary: coreml_native_output_summary(
                    &request.kernel.id,
                    &execution,
                    comparison.as_ref(),
                ),
                output_bytes: execution.output_bytes,
            })
        }
        _ => Err(format!(
            "provider adapter `{}` cannot execute request `{}`",
            adapter.adapter_id, request.kernel.id
        )),
    }
}

struct PreparedProviderInput {
    path: PathBuf,
    remove_on_drop: bool,
}

impl PreparedProviderInput {
    fn new(
        output_dir: &Path,
        binding: &ProviderInputBinding,
        completed: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self, String> {
        if binding.source == "dependency" {
            let bytes = completed.get(&binding.producer_request_id).ok_or_else(|| {
                format!(
                    "provider dependency `{}` has no completed output",
                    binding.producer_request_id
                )
            })?;
            validate_input_bytes(binding, bytes)?;
            let nonce = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "nuis-provider-edge-input-{}-{nonce}.bin",
                std::process::id()
            ));
            fs::write(&path, bytes).map_err(|error| {
                format!("failed to materialize provider dependency input: {error}")
            })?;
            return Ok(Self {
                path,
                remove_on_drop: true,
            });
        }
        let path = resolve_provider_payload_path(output_dir, &binding.payload_path)?;
        let bytes = fs::read(&path).map_err(|error| {
            format!(
                "failed to read provider input buffer `{}`: {error}",
                path.display()
            )
        })?;
        validate_input_bytes(binding, &bytes)?;
        Ok(Self {
            path,
            remove_on_drop: false,
        })
    }
}

fn validate_input_bytes(binding: &ProviderInputBinding, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() != binding.byte_length || fnv1a64_hex(bytes) != binding.content_hash {
        return Err(format!(
            "provider input binding `{}` size/hash evidence mismatch",
            binding.name
        ));
    }
    Ok(())
}

impl Drop for PreparedProviderInput {
    fn drop(&mut self) {
        if self.remove_on_drop {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn resolve_provider_payload_path(
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
