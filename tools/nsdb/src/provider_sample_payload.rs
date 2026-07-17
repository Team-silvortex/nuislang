use crate::{
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_sample_runner::{provider_execution_outcome_for_runner, ProviderSampleRunner},
};
use std::{fs, path::Path};

pub(crate) struct ProviderOutputPayload {
    pub(crate) evidence: String,
    pub(crate) detail: String,
    pub(crate) status: String,
    pub(crate) evidence_status: String,
}

pub(crate) struct ProviderOutputPayloadSummary {
    pub(crate) path: String,
    pub(crate) hash: String,
    pub(crate) attach_status: String,
}

const PROVIDER_OUTPUT_PAYLOAD_PROTOCOL: &str = "nuis-provider-output-payload-v1";
const PROVIDER_OUTPUT_PAYLOAD_SCHEMA: &str = "nsdb-provider-output-payload-v1";
const PROVIDER_SAMPLE_EXECUTION_CONTRACT: &str = "nuis-provider-sample-execution-v1";

pub(crate) fn provider_output_payload_summary(
    payload: Option<&ProviderOutputPayload>,
) -> ProviderOutputPayloadSummary {
    payload
        .map(|payload| provider_output_payload_summary_from_evidence(&payload.evidence))
        .unwrap_or_else(|| ProviderOutputPayloadSummary {
            path: "none".to_owned(),
            hash: "none".to_owned(),
            attach_status: "none".to_owned(),
        })
}

fn provider_output_payload_summary_from_evidence(evidence: &str) -> ProviderOutputPayloadSummary {
    let mut parts = evidence.split(':');
    let path = parts.next().unwrap_or("none").to_owned();
    let mut hash = "none".to_owned();
    let mut attach_status = "none".to_owned();
    for part in parts {
        if let Some(value) = part.strip_prefix("hash=") {
            hash = value.to_owned();
        } else if let Some(value) = part.strip_prefix("status=") {
            attach_status = value.to_owned();
        }
    }
    ProviderOutputPayloadSummary {
        path,
        hash,
        attach_status,
    }
}

pub(crate) fn provider_output_payload(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
) -> ProviderOutputPayload {
    let outcome = provider_execution_outcome_for_runner(runner);
    if runner.real_device_capable {
        if let Some(payload) = existing_provider_output_payload(output_dir, record, runner) {
            return payload;
        }
        return ProviderOutputPayload {
            evidence: "not-materialized".to_owned(),
            detail: outcome.detail.to_owned(),
            status: outcome.output_payload_status.to_owned(),
            evidence_status: outcome.output_payload_evidence_status.to_owned(),
        };
    }
    let file_name = provider_output_payload_file_name(&record.provider_family);
    let path = output_dir.join(&file_name);
    let content = render_provider_output_payload(record, runner);
    let hash = fnv1a64_hex(content.as_bytes());
    let write_status = match fs::write(&path, content) {
        Ok(()) => "written",
        Err(_) => "write-failed",
    };
    ProviderOutputPayload {
        evidence: format!("{file_name}:hash={hash}:status={write_status}"),
        detail: format!("deterministic-provider-output-payload:{file_name}:{hash}:{write_status}"),
        status: "host-fallback-output-payload-ready".to_owned(),
        evidence_status: "deterministic-provider-output-anchor".to_owned(),
    }
}

pub(crate) fn provider_output_payload_from_record(
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> Option<ProviderOutputPayload> {
    (record.provider_output_payload_evidence != "none").then(|| ProviderOutputPayload {
        evidence: record.provider_output_payload_evidence.clone(),
        detail: record.provider_output_payload_detail.clone(),
        status: record.provider_output_payload_status.clone(),
        evidence_status: record.provider_output_payload_evidence_status.clone(),
    })
}

pub(crate) fn provider_sample_status_for_payload(payload: &ProviderOutputPayload) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "provider-execution-blocked"
    } else {
        "provider-execution-ready"
    }
}

pub(crate) fn provider_validation_status_for_payload(
    payload: &ProviderOutputPayload,
) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "provider-output-payload-invalid"
    } else {
        "provider-execution-validated"
    }
}

pub(crate) fn provider_materialization_status_for_payload(
    payload: &ProviderOutputPayload,
) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "provider-sample-blocked"
    } else {
        "provider-sample-materialized"
    }
}

pub(crate) fn provider_next_action_for_payload(payload: &ProviderOutputPayload) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "repair-provider-output-payload"
    } else {
        "replay-device-sample"
    }
}

fn existing_provider_output_payload(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
) -> Option<ProviderOutputPayload> {
    let file_name = provider_output_payload_file_name(&record.provider_family);
    let path = output_dir.join(&file_name);
    let content = fs::read(&path).ok()?;
    let hash = fnv1a64_hex(&content);
    if let Err(error) = validate_provider_output_payload(record, &content) {
        return Some(ProviderOutputPayload {
            evidence: format!("{file_name}:hash={hash}:status=rejected"),
            detail: format!(
                "real-device-provider-output-payload-invalid:{file_name}:{hash}:{error}"
            ),
            status: "real-device-output-payload-invalid".to_owned(),
            evidence_status: "provider-output-payload-rejected".to_owned(),
        });
    }
    Some(ProviderOutputPayload {
        evidence: format!("{file_name}:hash={hash}:status=attached"),
        detail: format!(
            "real-device-provider-output-payload:{file_name}:{hash}:attached:{}",
            runner.adapter_id
        ),
        status: "real-device-output-payload-attached".to_owned(),
        evidence_status: "provider-output-payload-attached".to_owned(),
    })
}

fn validate_provider_output_payload(
    record: &NsdbDeviceProviderSampleRecordInfo,
    content: &[u8],
) -> Result<(), String> {
    let text = String::from_utf8_lossy(content);
    let expected_hash = fnv1a64_hex(record.input_evidence.as_bytes());
    for (field, needle) in [
        (
            "protocol",
            format!("protocol = \"{PROVIDER_OUTPUT_PAYLOAD_PROTOCOL}\""),
        ),
        (
            "schema",
            format!("schema = \"{PROVIDER_OUTPUT_PAYLOAD_SCHEMA}\""),
        ),
        (
            "sample_execution_contract",
            format!("sample_execution_contract = \"{PROVIDER_SAMPLE_EXECUTION_CONTRACT}\""),
        ),
        (
            "provider_family",
            format!("provider_family = \"{}\"", record.provider_family),
        ),
        (
            "input_evidence_hash",
            format!("input_evidence_hash = \"{expected_hash}\""),
        ),
    ] {
        if !text.contains(&needle) {
            return Err(format!("missing-{field}"));
        }
    }
    Ok(())
}

pub(crate) fn render_real_device_provider_output_payload(
    record: &NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
) -> String {
    let mut out = render_provider_output_payload_header(
        record,
        "nsdb-execute-provider-samples",
        adapter.adapter_id,
        adapter.capability_status,
        adapter.execution_mode,
    );
    push_toml_string(
        &mut out,
        "output_payload_kind",
        "real-device-adapter-output",
    );
    push_toml_string(&mut out, "output_payload_status", "adapter-output-ready");
    push_toml_string(&mut out, "comparison_status", "ready-for-comparison");
    out
}

fn render_provider_output_payload(
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
) -> String {
    let mut out = render_provider_output_payload_header(
        record,
        "nsdb-materialize-provider-samples",
        runner.adapter_id,
        runner.adapter_capability_status,
        runner.execution_mode,
    );
    push_toml_string(&mut out, "output_payload_kind", "host-fallback-anchor");
    push_toml_string(
        &mut out,
        "output_payload_status",
        "host-fallback-anchor-ready",
    );
    push_toml_string(&mut out, "comparison_status", "ready-for-comparison");
    out
}

fn render_provider_output_payload_header(
    record: &NsdbDeviceProviderSampleRecordInfo,
    source: &str,
    adapter_id: &str,
    adapter_capability_status: &str,
    execution_mode: &str,
) -> String {
    let mut out = String::new();
    push_toml_string(&mut out, "protocol", PROVIDER_OUTPUT_PAYLOAD_PROTOCOL);
    push_toml_string(&mut out, "schema", PROVIDER_OUTPUT_PAYLOAD_SCHEMA);
    push_toml_string(&mut out, "source", source);
    push_toml_string(
        &mut out,
        "sample_execution_contract",
        PROVIDER_SAMPLE_EXECUTION_CONTRACT,
    );
    push_toml_string(&mut out, "trace_id", &record.trace_id);
    push_toml_string(&mut out, "provider_family", &record.provider_family);
    push_toml_string(&mut out, "provider_runner_adapter_id", adapter_id);
    push_toml_string(
        &mut out,
        "provider_runner_adapter_capability_status",
        adapter_capability_status,
    );
    push_toml_string(
        &mut out,
        "provider_runner_real_device_probe_status",
        crate::provider_runner_registry::provider_runner_real_device_probe_status(
            &record.provider_family,
        ),
    );
    push_toml_string(&mut out, "provider_execution_mode", execution_mode);
    push_toml_string(&mut out, "input_evidence", &record.input_evidence);
    push_toml_string(
        &mut out,
        "input_evidence_hash",
        &fnv1a64_hex(record.input_evidence.as_bytes()),
    );
    out
}

pub(crate) fn provider_output_payload_file_name(provider_family: &str) -> String {
    format!(
        "nuis.nsdb.provider-output.{}.toml",
        sanitize_artifact_component(provider_family)
    )
}

fn sanitize_artifact_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

pub(crate) fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    out.push_str("\"\n");
}
