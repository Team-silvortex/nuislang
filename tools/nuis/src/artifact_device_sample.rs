use crate::{json_field, json_string_array_field, json_usize_field};
use std::collections::BTreeSet;

pub(crate) const DEVICE_SAMPLE_SCHEMA: &str = "nsdb-yir-device-execution-sample-v1";
pub(crate) const DEFERRED_DEVICE_SAMPLE_PROVIDER: &str = "nustar-deferred-device-sample-v1";
pub(crate) const DEVICE_SAMPLE_HANDOFF_PROTOCOL: &str = "nuis-device-sample-provider-handoff-v1";
pub(crate) const DEVICE_PROVIDER_SAMPLE_FILE_NAME: &str = "nuis.nsdb.device-provider-samples.toml";
pub(crate) const DEVICE_PROVIDER_SAMPLE_PROTOCOL: &str = "nuis-device-provider-samples-v1";
pub(crate) const DEVICE_PROVIDER_SAMPLE_SCHEMA: &str = "nsdb-yir-device-provider-sample-v1";

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
        format!("{base};std-preprocessed-pgm:input_bytes=20")
    } else {
        base
    }
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
