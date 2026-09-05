use crate::{
    final_image_provider_dispatch::{
        final_image_provider_dispatch_authority, validate_provider_families_against_final_image,
    },
    provider_runner_registry::{
        provider_runner_real_device_probe_status, select_provider_runner_adapter,
    },
    provider_runtime_dispatch_session::execute_native_provider_outputs,
    provider_runtime_result_stream::{persist_provider_runtime_results, ProviderRuntimeResult},
    provider_sample::{
        read_device_provider_sample_manifest_info, DEVICE_PROVIDER_SAMPLE_PROTOCOL,
        DEVICE_PROVIDER_SAMPLE_SCHEMA,
    },
    provider_sample_execution::provider_execution_outcome,
    provider_sample_payload::{
        fnv1a64_hex, pixelmagic_native_output_summary, provider_output_payload_file_name,
        render_real_device_provider_output_payload, ProviderNativeOutputSummary,
    },
};
use std::{fs, path::Path};

pub struct ProviderSampleExecuteReport {
    pub status: String,
    pub provider_family_filter: Option<String>,
    pub provider_families: Vec<String>,
    pub record_count: usize,
    pub matched_record_count: usize,
    pub executable_record_count: usize,
    pub output_payload_count: usize,
    pub first_provider_family: String,
    pub provider_bundle_registry_contract: String,
    pub provider_bundle_manifest_contract: String,
    pub provider_bundle_manifest_hash: String,
    pub provider_bundle_manifest_entry_count: usize,
    pub first_provider_bundle_package_id: String,
    pub first_provider_bundle_id: String,
    pub selected_provider_bundle_set_contract: String,
    pub selected_provider_bundle_count: usize,
    pub selected_provider_bundle_set_hash: String,
    pub final_image_dispatch_authority_status: String,
    pub final_image_dispatch_image_path: String,
    pub final_image_dispatch_count: usize,
    pub final_image_dispatch_matched_count: usize,
    pub final_image_dispatch_table_hash: String,
    pub final_image_dispatch_selected_set_hash: String,
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
    execute_provider_samples_inner(output_dir, provider_family_filter, 1, false)
}

#[allow(dead_code)] // Public through the library; the standalone CLI has not exposed this mode yet.
pub fn execute_provider_samples_for_runtime(
    output_dir: &Path,
    provider_family_filter: Option<&str>,
    invocation_count: usize,
) -> Result<ProviderSampleExecuteReport, String> {
    if !(1..=64).contains(&invocation_count) {
        return Err("provider runtime invocation count must be between 1 and 64".to_owned());
    }
    execute_provider_samples_inner(output_dir, provider_family_filter, invocation_count, true)
}

fn execute_provider_samples_inner(
    output_dir: &Path,
    provider_family_filter: Option<&str>,
    runtime_invocation_count: usize,
    persist_runtime_results: bool,
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
    let provider_families =
        crate::provider_bundle_registry::provider_families_for_records(&manifest.records)?;
    let matched_records = manifest
        .records
        .iter()
        .filter(|record| {
            provider_family_filter.is_none_or(|family| record.provider_family == family)
        })
        .collect::<Vec<_>>();
    let final_image_dispatch = final_image_provider_dispatch_authority(output_dir);
    if !final_image_dispatch.blockers.is_empty() {
        return Err(final_image_dispatch.blockers.join(", "));
    }
    let matched_record_values = matched_records
        .iter()
        .map(|record| (*record).clone())
        .collect::<Vec<_>>();
    let matched_provider_families =
        crate::provider_bundle_registry::provider_families_for_records(&matched_record_values)?;
    let final_image_dispatch_matched_count = validate_provider_families_against_final_image(
        &final_image_dispatch,
        &matched_provider_families,
    )?;
    if final_image_dispatch.available {
        for record in &matched_records {
            let registered = select_provider_runner_adapter(&record.provider_family);
            if registered.adapter_id != record.provider_runner_adapter_id {
                return Err(format!(
                    "final-image-dispatch:registered-adapter-drift:{}",
                    record.provider_family
                ));
            }
        }
    }
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
    let first_provider_bundle = matched_records.first().and_then(|record| {
        crate::provider_bundle_registry::provider_bundle_evidence(&record.provider_family)
    });
    let selected_provider_bundle_set =
        crate::provider_bundle_registry::selected_provider_bundle_set_for_records(
            &matched_record_values,
        )?;
    let mut completion_records = Vec::new();
    let mut output_payloads = Vec::new();
    let mut runtime_results = Vec::new();
    for record in &matched_records {
        let adapter = select_provider_runner_adapter(&record.provider_family);
        if !adapter.real_device_capable {
            continue;
        }
        let mut output =
            write_provider_output_payload(output_dir, record, &adapter, runtime_invocation_count)?;
        let mut completion = (**record).clone();
        completion.provider_output_payload_evidence = output.evidence.clone();
        completion_records.push(completion);
        runtime_results.append(&mut output.runtime_results);
        output_payloads.push(output);
    }
    if persist_runtime_results {
        persist_provider_runtime_results(output_dir, &runtime_results)?;
    }
    if final_image_dispatch.available {
        crate::handoff::persist_provider_completion_handoff(output_dir, &completion_records)?;
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
        provider_bundle_registry_contract: first_provider_bundle
            .map(|bundle| bundle.registry_contract)
            .unwrap_or("none")
            .to_owned(),
        provider_bundle_manifest_contract: first_provider_bundle
            .map(|bundle| bundle.manifest_contract)
            .unwrap_or("none")
            .to_owned(),
        provider_bundle_manifest_hash: first_provider_bundle
            .map(|bundle| bundle.manifest_hash)
            .unwrap_or("none")
            .to_owned(),
        provider_bundle_manifest_entry_count: first_provider_bundle
            .map(|bundle| bundle.manifest_entry_count)
            .unwrap_or(0),
        first_provider_bundle_package_id: first_provider_bundle
            .map(|bundle| bundle.package_id)
            .unwrap_or("none")
            .to_owned(),
        first_provider_bundle_id: first_provider_bundle
            .map(|bundle| bundle.bundle_id)
            .unwrap_or("none")
            .to_owned(),
        selected_provider_bundle_set_contract: selected_provider_bundle_set
            .as_ref()
            .map(|set| set.contract)
            .unwrap_or("none")
            .to_owned(),
        selected_provider_bundle_count: selected_provider_bundle_set
            .as_ref()
            .map(|set| set.count)
            .unwrap_or(0),
        selected_provider_bundle_set_hash: selected_provider_bundle_set
            .map(|set| set.hash)
            .unwrap_or_else(|| "none".to_owned()),
        final_image_dispatch_authority_status: final_image_dispatch.status,
        final_image_dispatch_image_path: final_image_dispatch
            .image_path
            .unwrap_or_else(|| "none".to_owned()),
        final_image_dispatch_count: final_image_dispatch.entries.len(),
        final_image_dispatch_matched_count,
        final_image_dispatch_table_hash: final_image_dispatch
            .table_hash
            .unwrap_or_else(|| "none".to_owned()),
        final_image_dispatch_selected_set_hash: final_image_dispatch
            .selected_set_hash
            .unwrap_or_else(|| "none".to_owned()),
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
    runtime_results: Vec<ProviderRuntimeResult>,
}

fn write_provider_output_payload(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
    runtime_invocation_count: usize,
) -> Result<WrittenProviderOutput, String> {
    let file_name = provider_output_payload_file_name(&record.provider_family);
    let execution =
        execute_native_provider_outputs(output_dir, record, adapter, runtime_invocation_count)?;
    let result_projection_evidence =
        crate::provider_result_projection::validate_and_render_result_projections(
            &record.input_evidence,
            &execution.native_outputs,
        )?;
    let content = render_real_device_provider_output_payload(
        record,
        adapter,
        &execution,
        &result_projection_evidence,
    );
    let hash = fnv1a64_hex(content.as_bytes());
    fs::write(output_dir.join(&file_name), content).map_err(|error| {
        format!("failed to write provider output payload `{file_name}`: {error}")
    })?;
    Ok(WrittenProviderOutput {
        evidence: format!("{file_name}:hash={hash}:status=written"),
        native_outputs: execution.native_outputs,
        runtime_results: execution.runtime_results,
    })
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
