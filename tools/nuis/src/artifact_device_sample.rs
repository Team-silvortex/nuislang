use crate::{json_field, json_string_array_field, json_usize_field};
use std::{collections::BTreeSet, fs, path::Path};

pub(crate) const DEVICE_SAMPLE_SCHEMA: &str = "nsdb-yir-device-execution-sample-v1";
pub(crate) const DEFERRED_DEVICE_SAMPLE_PROVIDER: &str = "nustar-deferred-device-sample-v1";
pub(crate) const DEVICE_SAMPLE_HANDOFF_PROTOCOL: &str = "nuis-device-sample-provider-handoff-v1";
pub(crate) const DEVICE_PROVIDER_SAMPLE_FILE_NAME: &str = "nuis.nsdb.device-provider-samples.toml";
pub(crate) const DEVICE_PROVIDER_SAMPLE_PROTOCOL: &str = "nuis-device-provider-samples-v1";
pub(crate) const DEVICE_PROVIDER_SAMPLE_SCHEMA: &str = "nsdb-yir-device-provider-sample-v1";
const PIXELMAGIC_STD_PIXEL_PAYLOAD_FILE_NAME: &str = "nuis.pixelmagic.std-preprocessed.gray8.bin";
const PIXELMAGIC_STD_PIXEL_PAYLOAD: &[u8] = &[0, 4, 9, 8];
const WITSAGE_VECTOR_PAYLOAD_FILE_NAME: &str = "nuis.witsage.vector.f32.bin";
const WITSAGE_VECTOR_MODEL_FILE_NAME: &str = "nuis.witsage.vector-affine.mlmodel";
const WITSAGE_VECTOR_EXPECTED_FILE_NAME: &str = "nuis.witsage.vector-affine.expected.f32.bin";
const WITSAGE_CHAINED_EXPECTED_FILE_NAME: &str =
    "nuis.witsage.vector-affine-chained.expected.f32.bin";
const WITSAGE_DENSE_PAYLOAD_FILE_NAME: &str = "nuis.witsage.feature-grid.f32.bin";
const WITSAGE_DENSE_MODEL_FILE_NAME: &str = "nuis.witsage.feature-grid-projection.mlmodel";
const WITSAGE_ADD_MODEL_FILE_NAME: &str = "nuis.witsage.vector-add.mlmodel";
const WITSAGE_ADD_EXPECTED_FILE_NAME: &str = "nuis.witsage.vector-add.expected.f32.bin";
const WITSAGE_METAL_EXPECTED_FILE_NAME: &str = "nuis.witsage.vector-metal-bias.expected.f32.bin";
const WITSAGE_VECTOR_PAYLOAD: &[u8] = &[
    0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x80, 0x40,
];
const WITSAGE_VECTOR_EXPECTED: &[u8] = &[
    0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0xa0, 0x40, 0x00, 0x00, 0xe0, 0x40, 0x00, 0x00, 0x10, 0x41,
];
const WITSAGE_CHAINED_EXPECTED: &[u8] = &[
    0x00, 0x00, 0xe0, 0x40, 0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0x70, 0x41, 0x00, 0x00, 0x98, 0x41,
];
const WITSAGE_ADD_EXPECTED: &[u8] = &[
    0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0x80, 0x41, 0x00, 0x00, 0xb0, 0x41, 0x00, 0x00, 0xe0, 0x41,
];
const WITSAGE_METAL_EXPECTED: &[u8] = &[
    0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0x88, 0x41, 0x00, 0x00, 0xb8, 0x41, 0x00, 0x00, 0xe8, 0x41,
];

pub(crate) struct DeviceSampleContract {
    pub(crate) provider: String,
    provider_family: String,
    sample_kind: String,
    pub(crate) status: String,
    pub(crate) schema: String,
    input_evidence: String,
    output_evidence: String,
    validation_status: String,
    handoff_target: String,
    handoff_status: String,
    pub(crate) next_action: String,
}

pub(crate) struct DeviceSampleSummary {
    pub(crate) descriptor_count: usize,
    pub(crate) ready_count: usize,
    pub(crate) pending_count: usize,
    pub(crate) pending_validation_count: usize,
    providers: Vec<String>,
    provider_families: Vec<String>,
    validation_statuses: Vec<String>,
    handoff_status: String,
    first_pending_provider_family: String,
}

pub(crate) fn device_sample_contract_for_trace(
    trace_role: &str,
    status: &str,
    backend_family: Option<&str>,
    target_device: Option<&str>,
    payload_format: Option<&str>,
    payload_path: Option<&str>,
) -> DeviceSampleContract {
    if trace_role != "backend-artifact" {
        return DeviceSampleContract {
            provider: "none".to_owned(),
            provider_family: "none".to_owned(),
            sample_kind: "none".to_owned(),
            status: "metadata-only".to_owned(),
            schema: "none".to_owned(),
            input_evidence: "none".to_owned(),
            output_evidence: "none".to_owned(),
            validation_status: "not-applicable".to_owned(),
            handoff_target: "none".to_owned(),
            handoff_status: "not-applicable".to_owned(),
            next_action: "wait-for-backend-execution-record".to_owned(),
        };
    }
    let sample_status = if status == "trace-ready" {
        "sample-descriptor-ready"
    } else if status == "blocked" {
        "blocked"
    } else {
        "device-execution-pending"
    };
    let next_action = if sample_status == "sample-descriptor-ready" {
        "handoff-device-sample-to-nsdb"
    } else if sample_status == "blocked" {
        "resolve-domain-trace-blocker"
    } else {
        "materialize-device-execution-sample"
    };
    DeviceSampleContract {
        provider: DEFERRED_DEVICE_SAMPLE_PROVIDER.to_owned(),
        provider_family: provider_family(backend_family, target_device),
        sample_kind: "deferred-provider-sample-descriptor".to_owned(),
        status: sample_status.to_owned(),
        schema: DEVICE_SAMPLE_SCHEMA.to_owned(),
        input_evidence: input_evidence(backend_family, target_device, payload_format, payload_path),
        output_evidence: "not-materialized".to_owned(),
        validation_status: validation_status(sample_status),
        handoff_target: provider_family(backend_family, target_device),
        handoff_status: handoff_status(sample_status),
        next_action: next_action.to_owned(),
    }
}

pub(crate) fn summarize_device_samples<'a>(
    samples: impl Iterator<Item = &'a DeviceSampleContract>,
) -> DeviceSampleSummary {
    let samples = samples.collect::<Vec<_>>();
    let descriptor_count = samples
        .iter()
        .filter(|sample| sample.schema != "none")
        .count();
    let ready_count = samples
        .iter()
        .filter(|sample| sample.status == "sample-descriptor-ready")
        .count();
    let pending_count = samples
        .iter()
        .filter(|sample| sample.status == "device-execution-pending")
        .count();
    let pending_validation_count = samples
        .iter()
        .filter(|sample| sample.validation_status == "pending-provider-execution")
        .count();
    DeviceSampleSummary {
        descriptor_count,
        ready_count,
        pending_count,
        pending_validation_count,
        providers: collect_unique(&samples, |sample| sample.provider.as_str()),
        provider_families: collect_unique(&samples, |sample| sample.provider_family.as_str()),
        validation_statuses: collect_unique(&samples, |sample| sample.validation_status.as_str()),
        handoff_status: summary_handoff_status(pending_validation_count, ready_count),
        first_pending_provider_family: first_pending_provider_family(&samples),
    }
}

impl DeviceSampleContract {
    fn is_provider_handoff_pending(&self) -> bool {
        self.schema != "none" && self.validation_status == "pending-provider-execution"
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_field("device_sample_provider", &self.provider),
            json_field("device_sample_provider_family", &self.provider_family),
            json_field("device_sample_kind", &self.sample_kind),
            json_field("device_sample_status", &self.status),
            json_field("device_sample_schema", &self.schema),
            json_field("device_sample_input_evidence", &self.input_evidence),
            json_field("device_sample_output_evidence", &self.output_evidence),
            json_field("device_sample_validation_status", &self.validation_status),
            json_field("device_sample_handoff_target", &self.handoff_target),
            json_field("device_sample_handoff_status", &self.handoff_status),
            json_field("device_sample_next_action", &self.next_action),
        ]
    }

    pub(crate) fn push_toml_fields(&self, out: &mut String) {
        push_toml_string(out, "device_sample_provider", &self.provider);
        push_toml_string(out, "device_sample_provider_family", &self.provider_family);
        push_toml_string(out, "device_sample_kind", &self.sample_kind);
        push_toml_string(out, "device_sample_status", &self.status);
        push_toml_string(out, "device_sample_schema", &self.schema);
        push_toml_string(out, "device_sample_input_evidence", &self.input_evidence);
        push_toml_string(out, "device_sample_output_evidence", &self.output_evidence);
        push_toml_string(
            out,
            "device_sample_validation_status",
            &self.validation_status,
        );
        push_toml_string(out, "device_sample_handoff_target", &self.handoff_target);
        push_toml_string(out, "device_sample_handoff_status", &self.handoff_status);
        push_toml_string(out, "device_sample_next_action", &self.next_action);
    }
}

impl DeviceSampleSummary {
    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_usize_field(
                "hetero_runtime_trace_device_sample_descriptor_count",
                self.descriptor_count,
            ),
            json_usize_field(
                "hetero_runtime_trace_device_sample_ready_count",
                self.ready_count,
            ),
            json_usize_field(
                "hetero_runtime_trace_device_sample_pending_count",
                self.pending_count,
            ),
            json_usize_field(
                "hetero_runtime_trace_device_sample_pending_validation_count",
                self.pending_validation_count,
            ),
            json_usize_field(
                "hetero_runtime_trace_device_sample_handoff_record_count",
                self.pending_validation_count,
            ),
            json_field(
                "hetero_runtime_trace_device_sample_handoff_protocol",
                DEVICE_SAMPLE_HANDOFF_PROTOCOL,
            ),
            json_string_array_field(
                "hetero_runtime_trace_device_sample_providers",
                &self.providers,
            ),
            json_string_array_field(
                "hetero_runtime_trace_device_sample_provider_families",
                &self.provider_families,
            ),
            json_string_array_field(
                "hetero_runtime_trace_device_sample_validation_statuses",
                &self.validation_statuses,
            ),
            json_field(
                "hetero_runtime_trace_device_sample_handoff_status",
                &self.handoff_status,
            ),
            json_field(
                "hetero_runtime_trace_device_sample_first_pending_provider_family",
                &self.first_pending_provider_family,
            ),
        ]
    }

    pub(crate) fn push_toml_fields(&self, out: &mut String) {
        out.push_str(&format!(
            "device_sample_descriptor_count = {}\n",
            self.descriptor_count
        ));
        out.push_str(&format!(
            "device_sample_ready_count = {}\n",
            self.ready_count
        ));
        out.push_str(&format!(
            "device_sample_pending_count = {}\n",
            self.pending_count
        ));
        out.push_str(&format!(
            "device_sample_pending_validation_count = {}\n",
            self.pending_validation_count
        ));
        out.push_str(&format!(
            "device_sample_handoff_record_count = {}\n",
            self.pending_validation_count
        ));
        push_toml_string(
            out,
            "device_sample_handoff_protocol",
            DEVICE_SAMPLE_HANDOFF_PROTOCOL,
        );
        push_toml_string_array(out, "device_sample_providers", &self.providers);
        push_toml_string_array(
            out,
            "device_sample_provider_families",
            &self.provider_families,
        );
        push_toml_string_array(
            out,
            "device_sample_validation_statuses",
            &self.validation_statuses,
        );
        push_toml_string(out, "device_sample_handoff_status", &self.handoff_status);
        push_toml_string(
            out,
            "device_sample_first_pending_provider_family",
            &self.first_pending_provider_family,
        );
    }
}

pub(crate) fn push_device_sample_handoff_queue_toml<'a>(
    out: &mut String,
    samples: impl Iterator<Item = (&'a str, &'a DeviceSampleContract)>,
) {
    for (trace_id, sample) in samples.filter(|(_, sample)| sample.is_provider_handoff_pending()) {
        out.push_str("\n[[device_sample_handoffs]]\n");
        push_toml_string(out, "trace_id", trace_id);
        push_toml_string(out, "protocol", DEVICE_SAMPLE_HANDOFF_PROTOCOL);
        push_toml_string(out, "provider", &sample.provider);
        push_toml_string(out, "provider_family", &sample.provider_family);
        push_toml_string(out, "handoff_target", &sample.handoff_target);
        push_toml_string(out, "handoff_status", &sample.handoff_status);
        push_toml_string(out, "validation_status", &sample.validation_status);
        push_toml_string(out, "input_evidence", &sample.input_evidence);
        push_toml_string(out, "output_evidence", &sample.output_evidence);
        push_toml_string(out, "next_action", &sample.next_action);
    }
}

pub(crate) fn render_device_provider_sample_manifest_toml<'a>(
    samples: impl Iterator<Item = (&'a str, &'a DeviceSampleContract)>,
) -> (String, usize) {
    let records = samples
        .filter(|(_, sample)| sample.is_provider_handoff_pending())
        .collect::<Vec<_>>();
    let mut out = String::new();
    push_toml_string(&mut out, "protocol", DEVICE_PROVIDER_SAMPLE_PROTOCOL);
    push_toml_string(&mut out, "schema", DEVICE_PROVIDER_SAMPLE_SCHEMA);
    push_toml_string(&mut out, "source", "run-artifact-provider-sample-manifest");
    push_toml_string(
        &mut out,
        "status",
        provider_sample_manifest_status(records.len()),
    );
    out.push_str(&format!("record_count = {}\n", records.len()));
    out.push_str("ready_record_count = 0\n");
    out.push_str(&format!("pending_record_count = {}\n", records.len()));
    for (trace_id, sample) in &records {
        out.push_str("\n[[device_provider_samples]]\n");
        push_toml_string(&mut out, "trace_id", trace_id);
        push_toml_string(&mut out, "provider", &sample.provider);
        push_toml_string(&mut out, "provider_family", &sample.provider_family);
        let requested_runner = requested_provider_runner_for(&sample.provider_family);
        push_toml_string(
            &mut out,
            "requested_runner_contract",
            requested_runner.contract,
        );
        push_toml_string(
            &mut out,
            "requested_runner_adapter_contract",
            requested_runner.adapter_contract,
        );
        push_toml_string(
            &mut out,
            "requested_runner_adapter_id",
            requested_runner.adapter_id,
        );
        push_toml_string(
            &mut out,
            "requested_runner_adapter_capability_status",
            requested_runner.adapter_capability_status,
        );
        push_toml_string(&mut out, "handoff_target", &sample.handoff_target);
        push_toml_string(&mut out, "sample_status", "pending-provider-execution");
        push_toml_string(&mut out, "validation_status", &sample.validation_status);
        push_toml_string(&mut out, "input_evidence", &sample.input_evidence);
        push_toml_string(&mut out, "output_evidence", &sample.output_evidence);
        push_toml_string(
            &mut out,
            "materialization_status",
            "provider-sample-pending",
        );
        push_toml_string(
            &mut out,
            "materialization_detail",
            "awaiting-provider-runtime",
        );
        push_toml_string(&mut out, "next_action", "execute-provider-sample");
    }
    (out, records.len())
}

pub(crate) fn persist_device_sample_input_payloads<'a>(
    output_dir: &Path,
    samples: impl Iterator<Item = &'a DeviceSampleContract>,
) -> Result<(), String> {
    let evidence = samples
        .map(|sample| sample.input_evidence.as_str())
        .collect::<Vec<_>>();
    if evidence
        .iter()
        .any(|item| item.contains("std-preprocessed-pgm:input_bytes=20"))
    {
        fs::write(
            output_dir.join(PIXELMAGIC_STD_PIXEL_PAYLOAD_FILE_NAME),
            PIXELMAGIC_STD_PIXEL_PAYLOAD,
        )
        .map_err(|error| format!("failed to persist PixelMagic std pixel payload: {error}"))?;
    }
    if evidence
        .iter()
        .any(|item| item.contains("provider_model_asset_id=witsage."))
    {
        fs::write(
            output_dir.join(WITSAGE_VECTOR_PAYLOAD_FILE_NAME),
            WITSAGE_VECTOR_PAYLOAD,
        )
        .map_err(|error| format!("failed to persist WitSage vector payload: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_VECTOR_MODEL_FILE_NAME),
            crate::artifact_coreml_model::witsage_vector_affine_model(),
        )
        .map_err(|error| format!("failed to persist WitSage CoreML model: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_VECTOR_EXPECTED_FILE_NAME),
            WITSAGE_VECTOR_EXPECTED,
        )
        .map_err(|error| format!("failed to persist WitSage expected vector output: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_CHAINED_EXPECTED_FILE_NAME),
            WITSAGE_CHAINED_EXPECTED,
        )
        .map_err(|error| format!("failed to persist WitSage chained expected output: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_DENSE_PAYLOAD_FILE_NAME),
            witsage_dense_payload(),
        )
        .map_err(|error| format!("failed to persist WitSage dense payload: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_DENSE_MODEL_FILE_NAME),
            crate::artifact_coreml_model::witsage_dense_transform_model(),
        )
        .map_err(|error| format!("failed to persist WitSage dense CoreML model: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_ADD_MODEL_FILE_NAME),
            crate::artifact_coreml_model::witsage_vector_add_model(),
        )
        .map_err(|error| format!("failed to persist WitSage add CoreML model: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_ADD_EXPECTED_FILE_NAME),
            WITSAGE_ADD_EXPECTED,
        )
        .map_err(|error| format!("failed to persist WitSage add expected output: {error}"))?;
        fs::write(
            output_dir.join(WITSAGE_METAL_EXPECTED_FILE_NAME),
            WITSAGE_METAL_EXPECTED,
        )
        .map_err(|error| format!("failed to persist WitSage Metal expected output: {error}"))?;
    }
    Ok(())
}

struct RequestedProviderRunner {
    contract: &'static str,
    adapter_contract: &'static str,
    adapter_id: &'static str,
    adapter_capability_status: &'static str,
}

fn requested_provider_runner_for(provider_family: &str) -> RequestedProviderRunner {
    let adapter_id = match provider_family {
        "metal:apple-silicon-gpu" => "metal.apple-silicon-gpu.host-simulated",
        "coreml:apple-ane" => "coreml.apple-ane.host-simulated",
        _ => "generic.device.host-simulated",
    };
    RequestedProviderRunner {
        contract: "nuis-provider-runner-v1",
        adapter_contract: "nuis-provider-runner-adapter-v1",
        adapter_id,
        adapter_capability_status: "registered-host-simulated",
    }
}

fn provider_sample_manifest_status(record_count: usize) -> &'static str {
    if record_count == 0 {
        "empty"
    } else {
        "awaiting-provider-materialization"
    }
}

fn provider_family(backend_family: Option<&str>, target_device: Option<&str>) -> String {
    format!(
        "{}:{}",
        backend_family.unwrap_or("unknown-backend"),
        target_device.unwrap_or("unknown-device")
    )
}

fn input_evidence(
    backend_family: Option<&str>,
    target_device: Option<&str>,
    payload_format: Option<&str>,
    payload_path: Option<&str>,
) -> String {
    let base = match (payload_format, payload_path) {
        (Some(format), Some(path)) => format!("{format}:{path}"),
        (Some(format), None) => format!("{format}:payload-path-missing"),
        (None, Some(path)) => format!("payload-format-missing:{path}"),
        (None, None) => "payload-evidence-missing".to_owned(),
    };
    if backend_family == Some("metal") && target_device == Some("apple-silicon-gpu") {
        format!(
            "{base};provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.pixels;provider_buffer_element_type=u8;provider_buffer_layout=image-2d-row-major:pixel-format=gray8;provider_buffer_shape=2x2;provider_buffer_row_stride_bytes=2;provider_buffer_byte_length={};provider_buffer_payload_path={PIXELMAGIC_STD_PIXEL_PAYLOAD_FILE_NAME};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=pixelmagic.gray8.invert;provider_kernel_operation=invert;provider_kernel_input_buffer=input.pixels;provider_kernel_output_buffer=output.pixels;provider_kernel_dispatch=2x2x1;provider_kernel_scalar_bindings=max_value:u8:15;std-preprocessed-pgm:input_bytes=20;pixel_format=gray8;pixel_width=2;pixel_height=2;pixel_stride=2;pixel_max_value=15;pixel_operation=invert;pixel_payload_path={PIXELMAGIC_STD_PIXEL_PAYLOAD_FILE_NAME};pixel_payload_bytes={};pixel_payload_hash={}",
            PIXELMAGIC_STD_PIXEL_PAYLOAD.len(),
            fnv1a64_hex(PIXELMAGIC_STD_PIXEL_PAYLOAD),
            PIXELMAGIC_STD_PIXEL_PAYLOAD.len(),
            fnv1a64_hex(PIXELMAGIC_STD_PIXEL_PAYLOAD)
        )
    } else if backend_family == Some("coreml") && target_device == Some("apple-ane") {
        let payload = witsage_dense_payload();
        let model = crate::artifact_coreml_model::witsage_dense_transform_model();
        let singular = format!(
            "{base};provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.features;provider_buffer_element_type=f32;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=16x64x64;provider_buffer_row_stride_bytes=256;provider_buffer_byte_length={};provider_buffer_payload_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=witsage.feature-grid.projection;provider_kernel_operation=model-predict;provider_kernel_input_buffer=input.features;provider_kernel_output_buffer=output.features;provider_kernel_dispatch=16x64x64;provider_model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;provider_model_asset_id=witsage.feature-grid-projection.coreml;provider_model_asset_format=coreml-specification;provider_model_asset_path={WITSAGE_DENSE_MODEL_FILE_NAME};provider_model_asset_byte_length={};provider_model_asset_content_hash={};provider_model_asset_input_feature=input.features;provider_model_asset_output_feature=output.features;provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_output_buffer=output.features;provider_output_comparison_element_type=f32;provider_output_comparison_shape=16x64x64;provider_output_comparison_expected_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};provider_output_comparison_expected_byte_length={};provider_output_comparison_expected_content_hash={};provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject",
            payload.len(),
            fnv1a64_hex(&payload),
            model.len(),
            fnv1a64_hex(&model),
            payload.len(),
            fnv1a64_hex(&payload)
        );
        let dense = witsage_dense_collection_request(0, &payload, &model);
        let affine_model = crate::artifact_coreml_model::witsage_vector_affine_model();
        let affine = witsage_affine_collection_request(1, &affine_model);
        let chained = witsage_chained_affine_collection_request(2, &affine_model);
        let add_model = crate::artifact_coreml_model::witsage_vector_add_model();
        let add = witsage_add_collection_request(3, &add_model);
        let metal = witsage_metal_bias_collection_request(4);
        format!(
            "{singular};provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=5;{dense};{affine};{chained};{add};{metal}"
        )
    } else {
        base
    }
}

fn witsage_dense_payload() -> Vec<u8> {
    vec![1.0f32; 16 * 64 * 64]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect()
}

fn witsage_dense_collection_request(index: usize, payload: &[u8], model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=16x64x64;{prefix}buffer_row_stride_bytes=256;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.feature-grid.projection;{prefix}kernel_operation=model-predict;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=16x64x64;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.feature-grid-projection.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_DENSE_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.features;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=16x64x64;{prefix}output_comparison_expected_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0",
        payload.len(),
        fnv1a64_hex(payload),
        model.len(),
        fnv1a64_hex(model),
        payload.len(),
        fnv1a64_hex(payload)
    );
    format!(
        "{request};{}",
        witsage_input_binding(
            &prefix,
            "artifact",
            "16x64x64",
            payload.len(),
            &fnv1a64_hex(payload),
            WITSAGE_DENSE_PAYLOAD_FILE_NAME,
            "none",
            "none",
        )
    )
}

fn witsage_affine_collection_request(index: usize, model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_VECTOR_PAYLOAD_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.affine;{prefix}kernel_operation=affine;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}kernel_scalar_bindings=scale:f32:2,bias:f32:1;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.vector-affine.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_VECTOR_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.features;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_VECTOR_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0",
        WITSAGE_VECTOR_PAYLOAD.len(),
        fnv1a64_hex(WITSAGE_VECTOR_PAYLOAD),
        model.len(),
        fnv1a64_hex(model),
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED)
    );
    format!(
        "{request};{}",
        witsage_input_binding(
            &prefix,
            "artifact",
            "1x1x4",
            WITSAGE_VECTOR_PAYLOAD.len(),
            &fnv1a64_hex(WITSAGE_VECTOR_PAYLOAD),
            WITSAGE_VECTOR_PAYLOAD_FILE_NAME,
            "none",
            "none",
        )
    )
}

fn witsage_chained_affine_collection_request(index: usize, model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_VECTOR_EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.affine.chained;{prefix}kernel_operation=affine;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}kernel_scalar_bindings=scale:f32:2,bias:f32:1;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.vector-affine-chained.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_VECTOR_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.features;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_CHAINED_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id=witsage.vector.affine;{prefix}dependency_0_producer_output_buffer=output.features;{prefix}dependency_0_consumer_input_buffer=input.features",
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
        model.len(),
        fnv1a64_hex(model),
        WITSAGE_CHAINED_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_CHAINED_EXPECTED)
    );
    format!(
        "{request};{}",
        witsage_input_binding(
            &prefix,
            "dependency",
            "1x1x4",
            WITSAGE_VECTOR_EXPECTED.len(),
            &fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
            "none",
            "witsage.vector.affine",
            "output.features",
        )
    )
}

#[allow(clippy::too_many_arguments)]
fn witsage_input_binding(
    prefix: &str,
    source: &str,
    shape: &str,
    byte_length: usize,
    content_hash: &str,
    payload_path: &str,
    producer_request_id: &str,
    producer_output_buffer: &str,
) -> String {
    format!(
        "{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.features;{prefix}input_binding_0_source={source};{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape={shape};{prefix}input_binding_0_byte_length={byte_length};{prefix}input_binding_0_content_hash={content_hash};{prefix}input_binding_0_payload_path={payload_path};{prefix}input_binding_0_producer_request_id={producer_request_id};{prefix}input_binding_0_producer_output_buffer={producer_output_buffer};{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=coreml:apple-ane;{prefix}adapter_binding_execution_requirement=real-device"
    )
}

fn witsage_add_collection_request(index: usize, model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.left;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_VECTOR_EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.add;{prefix}kernel_operation=add;{prefix}kernel_input_buffer=input.left;{prefix}kernel_input_buffers=input.left,input.right;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.vector-add.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_ADD_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.left;{prefix}model_asset_input_features=input.left,input.right;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_ADD_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=2;{prefix}dependency_0_producer_request_id=witsage.vector.affine;{prefix}dependency_0_producer_output_buffer=output.features;{prefix}dependency_0_consumer_input_buffer=input.left;{prefix}dependency_1_producer_request_id=witsage.vector.affine.chained;{prefix}dependency_1_producer_output_buffer=output.features;{prefix}dependency_1_consumer_input_buffer=input.right;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=2;{prefix}input_binding_0_name=input.left;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape=1x1x4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id=witsage.vector.affine;{prefix}input_binding_0_producer_output_buffer=output.features;{prefix}input_binding_1_name=input.right;{prefix}input_binding_1_source=dependency;{prefix}input_binding_1_element_type=f32;{prefix}input_binding_1_shape=1x1x4;{prefix}input_binding_1_byte_length={};{prefix}input_binding_1_content_hash={};{prefix}input_binding_1_payload_path=none;{prefix}input_binding_1_producer_request_id=witsage.vector.affine.chained;{prefix}input_binding_1_producer_output_buffer=output.features;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=coreml:apple-ane;{prefix}adapter_binding_execution_requirement=real-device",
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
        model.len(),
        fnv1a64_hex(model),
        WITSAGE_ADD_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_ADD_EXPECTED),
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
        WITSAGE_CHAINED_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_CHAINED_EXPECTED),
    )
}

fn witsage_metal_bias_collection_request(index: usize) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_ADD_EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.metal-bias;{prefix}kernel_operation=bias;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}kernel_scalar_bindings=bias:f32:1;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_METAL_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id=witsage.vector.add;{prefix}dependency_0_producer_output_buffer=output.features;{prefix}dependency_0_consumer_input_buffer=input.features;{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token=glm:provider-edge:witsage.vector.add:output.features->witsage.vector.metal-bias:input.features;{prefix}dependency_0_transport_staging_mode=host-visible-owned-file;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-3:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-4:dispatch-ready;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.features;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape=1x1x4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id=witsage.vector.add;{prefix}input_binding_0_producer_output_buffer=output.features;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=metal:apple-silicon-gpu;{prefix}adapter_binding_execution_requirement=real-device",
        WITSAGE_ADD_EXPECTED.len(), fnv1a64_hex(WITSAGE_ADD_EXPECTED),
        WITSAGE_METAL_EXPECTED.len(), fnv1a64_hex(WITSAGE_METAL_EXPECTED),
        WITSAGE_ADD_EXPECTED.len(), fnv1a64_hex(WITSAGE_ADD_EXPECTED),
    );
    request.replace(
        "dependency_0_transport_staging_mode=host-visible-owned-file",
        "dependency_0_transport_staging_mode=auto",
    )
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn validation_status(sample_status: &str) -> String {
    match sample_status {
        "sample-descriptor-ready" => "descriptor-ready-provider-execution-deferred",
        "blocked" => "blocked-before-provider-execution",
        _ => "pending-provider-execution",
    }
    .to_owned()
}

fn handoff_status(sample_status: &str) -> String {
    match sample_status {
        "sample-descriptor-ready" => "ready-for-provider-handoff",
        "blocked" => "blocked-before-provider-handoff",
        _ => "awaiting-provider-handoff",
    }
    .to_owned()
}

fn summary_handoff_status(pending_validation_count: usize, ready_count: usize) -> String {
    if pending_validation_count > 0 {
        "provider-handoff-pending"
    } else if ready_count > 0 {
        "provider-handoff-ready"
    } else {
        "no-provider-handoff"
    }
    .to_owned()
}

fn first_pending_provider_family(samples: &[&DeviceSampleContract]) -> String {
    samples
        .iter()
        .find(|sample| sample.validation_status == "pending-provider-execution")
        .map(|sample| sample.provider_family.clone())
        .unwrap_or_else(|| "none".to_owned())
}

fn collect_unique(
    samples: &[&DeviceSampleContract],
    select: fn(&DeviceSampleContract) -> &str,
) -> Vec<String> {
    samples
        .iter()
        .filter(|sample| sample.schema != "none")
        .map(|sample| select(sample))
        .filter(|value| !value.is_empty() && *value != "none")
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    out.push_str("\"\n");
}

fn push_toml_string_array(out: &mut String, key: &str, values: &[String]) {
    out.push_str(key);
    out.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
        out.push('"');
    }
    out.push_str("]\n");
}
