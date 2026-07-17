use crate::{
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_sample::{
        read_device_provider_sample_manifest_info, DEVICE_PROVIDER_SAMPLE_FILE_NAME,
        DEVICE_PROVIDER_SAMPLE_PROTOCOL, DEVICE_PROVIDER_SAMPLE_SCHEMA,
    },
    provider_sample_payload::{
        fnv1a64_hex, provider_materialization_status_for_payload, provider_next_action_for_payload,
        provider_output_payload, provider_output_payload_from_record,
        provider_output_payload_summary, provider_sample_status_for_payload,
        provider_validation_status_for_payload, ProviderOutputPayload,
    },
    provider_sample_runner::{
        provider_execution_for, provider_execution_outcome_for_runner, provider_runner_for,
        ProviderSampleRunner,
    },
};
use std::{collections::BTreeSet, fs, path::Path};

pub struct ProviderSampleMaterializeReport {
    pub path: String,
    pub provider_family_filter: Option<String>,
    pub provider_families: Vec<String>,
    pub status: String,
    pub record_count: usize,
    pub matched_record_count: usize,
    pub materialized_record_count: usize,
    pub skipped_record_count: usize,
    pub first_provider_family: String,
    pub first_provider_runner_contract: String,
    pub first_provider_runner_adapter_contract: String,
    pub first_provider_runner_adapter_id: String,
    pub first_provider_runner_adapter_capability_status: String,
    pub first_provider_runner_registry_protocol: String,
    pub first_provider_runner_registry_source: String,
    pub first_provider_runner_real_device_capable: bool,
    pub first_provider_runner_kind: String,
    pub first_provider_execution_mode: String,
    pub first_provider_execution_comparison_contract: String,
    pub first_provider_execution_comparison_status: String,
    pub first_provider_execution_evidence_status: String,
    pub first_provider_output_payload_contract: String,
    pub first_provider_output_payload_status: String,
    pub first_provider_output_payload_evidence_status: String,
    pub first_provider_output_payload_evidence: String,
    pub first_provider_output_payload_detail: String,
    pub first_provider_output_payload_path: String,
    pub first_provider_output_payload_hash: String,
    pub first_provider_output_payload_attach_status: String,
    pub first_output_evidence: String,
    pub next_action: String,
    pub next_command: String,
    pub return_contract: String,
    pub return_action: String,
    pub return_command: String,
    pub final_output_replay_contract: String,
}

pub fn materialize_provider_samples(
    output_dir: &Path,
    provider_family_filter: Option<&str>,
) -> Result<ProviderSampleMaterializeReport, String> {
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
    let mut materialized = 0;
    let provider_families = provider_families(&manifest.records);
    let matched_record_count =
        provider_family_match_count(&manifest.records, provider_family_filter);
    let records = manifest
        .records
        .iter()
        .map(|record| {
            if should_materialize_record(record, provider_family_filter) {
                let record = materialized_record(output_dir, record);
                if record.materialization_status == "provider-sample-materialized" {
                    materialized += 1;
                }
                record
            } else {
                record.clone()
            }
        })
        .collect::<Vec<_>>();
    let path = output_dir.join(DEVICE_PROVIDER_SAMPLE_FILE_NAME);
    fs::write(&path, render_materialized_manifest(&records)).map_err(|error| {
        format!(
            "failed to write device provider sample manifest `{}`: {error}",
            path.display()
        )
    })?;
    let return_action = nsld_return_action(output_dir);
    let return_command = nsld_return_command(output_dir);
    let first_payload_summary = provider_output_payload_summary(
        records
            .first()
            .and_then(provider_output_payload_from_record)
            .as_ref(),
    );
    Ok(ProviderSampleMaterializeReport {
        path: path.display().to_string(),
        provider_family_filter: provider_family_filter.map(str::to_owned),
        provider_families,
        status: materialized_manifest_status(&records),
        record_count: records.len(),
        matched_record_count,
        materialized_record_count: materialized,
        skipped_record_count: records.len().saturating_sub(materialized),
        first_provider_family: records
            .first()
            .map(|record| record.provider_family.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_runner_contract: records
            .first()
            .map(|record| provider_runner_for(record).contract.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_runner_adapter_contract: records
            .first()
            .map(|record| provider_runner_for(record).adapter_contract.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_runner_adapter_id: records
            .first()
            .map(|record| provider_runner_for(record).adapter_id.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_runner_adapter_capability_status: records
            .first()
            .map(|record| {
                provider_runner_for(record)
                    .adapter_capability_status
                    .to_owned()
            })
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_runner_registry_protocol: records
            .first()
            .map(|record| provider_runner_for(record).registry_protocol.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_runner_registry_source: records
            .first()
            .map(|record| provider_runner_for(record).registry_source.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_runner_real_device_capable: records
            .first()
            .is_some_and(|record| provider_runner_for(record).real_device_capable),
        first_provider_runner_kind: records
            .first()
            .map(|record| provider_runner_for(record).kind.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_execution_mode: records
            .first()
            .map(|record| provider_runner_for(record).execution_mode.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_execution_comparison_contract: records
            .first()
            .map(|record| provider_execution_for(record).contract.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_execution_comparison_status: records
            .first()
            .map(|record| provider_execution_for(record).comparison_status.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_execution_evidence_status: records
            .first()
            .map(|record| provider_execution_for(record).evidence_status.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_output_payload_contract: records
            .first()
            .map(|record| record.provider_output_payload_contract.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_output_payload_status: records
            .first()
            .map(|record| record.provider_output_payload_status.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_output_payload_evidence_status: records
            .first()
            .map(|record| record.provider_output_payload_evidence_status.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_output_payload_evidence: records
            .first()
            .map(|record| record.provider_output_payload_evidence.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_output_payload_detail: records
            .first()
            .map(|record| record.provider_output_payload_detail.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_output_payload_path: first_payload_summary.path,
        first_provider_output_payload_hash: first_payload_summary.hash,
        first_provider_output_payload_attach_status: first_payload_summary.attach_status,
        first_output_evidence: records
            .first()
            .map(|record| record.output_evidence.clone())
            .unwrap_or_else(|| "none".to_owned()),
        next_action: "replay-provider-sample".to_owned(),
        next_command: format!("nsdb replay-plan {} --json", output_dir.display()),
        return_contract: "nsld-final-output-boundary-return-v1".to_owned(),
        return_action,
        return_command,
        final_output_replay_contract: "nsdb-payload-execution-replay-plan-v1".to_owned(),
    })
}

fn nsld_return_action(output_dir: &Path) -> String {
    if output_dir.join("nuis.build.manifest.toml").is_file() {
        "resume-nsld-final-output-check".to_owned()
    } else {
        "resume-nsld-final-output-check-manifest-required".to_owned()
    }
}

fn nsld_return_command(output_dir: &Path) -> String {
    if output_dir.join("nuis.build.manifest.toml").is_file() {
        format!("nsld check {} --json", output_dir.display())
    } else {
        "nsld check <nuis.build.manifest.toml> --json".to_owned()
    }
}

fn provider_families(records: &[NsdbDeviceProviderSampleRecordInfo]) -> Vec<String> {
    records
        .iter()
        .map(|record| record.provider_family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn provider_family_match_count(
    records: &[NsdbDeviceProviderSampleRecordInfo],
    provider_family_filter: Option<&str>,
) -> usize {
    records
        .iter()
        .filter(|record| {
            provider_family_filter.is_none_or(|family| record.provider_family == family)
        })
        .count()
}

fn should_materialize_record(
    record: &NsdbDeviceProviderSampleRecordInfo,
    provider_family_filter: Option<&str>,
) -> bool {
    record.materialization_status == "provider-sample-pending"
        && provider_family_filter.is_none_or(|family| record.provider_family == family)
}

fn materialized_record(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> NsdbDeviceProviderSampleRecordInfo {
    let mut record = record.clone();
    let runner = provider_runner_for(&record);
    let artifact = provider_sample_artifact(output_dir, &record, &runner);
    let outcome = provider_execution_outcome_for_runner(&runner);
    record.sample_status = provider_sample_status_for_payload(&artifact.output_payload).to_owned();
    record.validation_status =
        provider_validation_status_for_payload(&artifact.output_payload).to_owned();
    record.output_evidence = artifact.evidence;
    record.provider_output_payload_contract = outcome.output_payload_contract.to_owned();
    record.provider_output_payload_status = artifact.output_payload.status.clone();
    record.provider_output_payload_evidence_status =
        artifact.output_payload.evidence_status.clone();
    record.provider_output_payload_evidence = artifact.output_payload.evidence.clone();
    record.provider_output_payload_detail = artifact.output_payload.detail.clone();
    record.provider_output_payload_next_action = outcome.output_payload_next_action.to_owned();
    record.materialization_status =
        provider_materialization_status_for_payload(&artifact.output_payload).to_owned();
    record.materialization_detail = artifact.detail;
    record.next_action = provider_next_action_for_payload(&artifact.output_payload).to_owned();
    record
}

struct ProviderSampleArtifact {
    evidence: String,
    detail: String,
    output_payload: ProviderOutputPayload,
}

fn provider_sample_artifact(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
) -> ProviderSampleArtifact {
    let file_name = format!(
        "nuis.nsdb.provider-sample.{}.toml",
        sanitize_artifact_component(&record.provider_family)
    );
    let output_payload = provider_output_payload(output_dir, record, runner);
    let path = output_dir.join(&file_name);
    let content = render_provider_sample_artifact(record, runner, &output_payload);
    let hash = fnv1a64_hex(content.as_bytes());
    let write_status = match fs::write(&path, content) {
        Ok(()) => "written",
        Err(_) => "write-failed",
    };
    ProviderSampleArtifact {
        evidence: format!("{file_name}:hash={hash}:status={write_status}"),
        detail: format!("deterministic-provider-sample-artifact:{file_name}:{hash}:{write_status}"),
        output_payload,
    }
}

fn render_provider_sample_artifact(
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
    output_payload: &ProviderOutputPayload,
) -> String {
    let mut out = String::new();
    push_toml_string(
        &mut out,
        "protocol",
        "nuis-nsdb-provider-sample-artifact-v1",
    );
    push_toml_string(&mut out, "schema", "nsdb-yir-provider-sample-artifact-v1");
    push_toml_string(&mut out, "source", "nsdb-materialize-provider-samples");
    push_toml_string(&mut out, "trace_id", &record.trace_id);
    push_toml_string(&mut out, "provider", &record.provider);
    push_toml_string(&mut out, "provider_family", &record.provider_family);
    push_toml_string(&mut out, "provider_runner_contract", runner.contract);
    push_toml_string(
        &mut out,
        "provider_runner_adapter_contract",
        runner.adapter_contract,
    );
    push_toml_string(&mut out, "provider_runner_adapter_id", runner.adapter_id);
    push_toml_string(
        &mut out,
        "provider_runner_adapter_capability_status",
        runner.adapter_capability_status,
    );
    push_toml_string(
        &mut out,
        "provider_runner_registry_protocol",
        runner.registry_protocol,
    );
    push_toml_string(
        &mut out,
        "provider_runner_registry_source",
        runner.registry_source,
    );
    out.push_str(&format!(
        "provider_runner_real_device_capable = {}\n",
        runner.real_device_capable
    ));
    push_toml_string(
        &mut out,
        "provider_runner_real_device_probe_status",
        crate::provider_runner_registry::provider_runner_real_device_probe_status(
            &record.provider_family,
        ),
    );
    push_toml_string(&mut out, "provider_runner_kind", runner.kind);
    push_toml_string(&mut out, "provider_execution_mode", runner.execution_mode);
    push_toml_string(&mut out, "provider_backend", runner.backend);
    push_toml_string(&mut out, "provider_device", runner.device);
    let outcome = provider_execution_outcome_for_runner(runner);
    push_provider_execution_outcome(&mut out, &outcome, Some(output_payload));
    push_toml_string(&mut out, "handoff_target", &record.handoff_target);
    push_toml_string(&mut out, "input_evidence", &record.input_evidence);
    push_toml_string(
        &mut out,
        "sample_status",
        provider_sample_status_for_payload(output_payload),
    );
    push_toml_string(
        &mut out,
        "validation_status",
        provider_validation_status_for_payload(output_payload),
    );
    push_toml_string(&mut out, "materialization_mode", runner.execution_mode);
    out
}

fn push_provider_execution_outcome(
    out: &mut String,
    outcome: &crate::provider_sample_execution::ProviderExecutionOutcome,
    payload: Option<&ProviderOutputPayload>,
) {
    push_toml_string(
        out,
        "provider_execution_comparison_contract",
        outcome.contract,
    );
    push_toml_string(out, "provider_execution_status", outcome.status);
    push_toml_string(
        out,
        "provider_execution_comparison_status",
        outcome.comparison_status,
    );
    push_toml_string(
        out,
        "provider_execution_evidence_status",
        outcome.evidence_status,
    );
    push_toml_string(
        out,
        "provider_output_payload_contract",
        outcome.output_payload_contract,
    );
    push_toml_string(
        out,
        "provider_output_payload_status",
        payload
            .map(|payload| payload.status.as_str())
            .unwrap_or(outcome.output_payload_status),
    );
    push_toml_string(
        out,
        "provider_output_payload_evidence_status",
        payload
            .map(|payload| payload.evidence_status.as_str())
            .unwrap_or(outcome.output_payload_evidence_status),
    );
    push_toml_string(
        out,
        "provider_output_payload_evidence",
        payload
            .map(|payload| payload.evidence.as_str())
            .unwrap_or(outcome.output_payload_file_name),
    );
    push_toml_string(
        out,
        "provider_output_payload_detail",
        payload
            .map(|payload| payload.detail.as_str())
            .unwrap_or(outcome.detail),
    );
    push_toml_string(
        out,
        "provider_output_payload_next_action",
        outcome.output_payload_next_action,
    );
    push_toml_string(out, "provider_execution_next_action", outcome.next_action);
    push_toml_string(out, "provider_execution_detail", outcome.detail);
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

fn render_materialized_manifest(records: &[NsdbDeviceProviderSampleRecordInfo]) -> String {
    let mut out = String::new();
    let ready_count = provider_sample_ready_count(records);
    let pending_count = provider_sample_pending_count(records);
    push_toml_string(&mut out, "protocol", DEVICE_PROVIDER_SAMPLE_PROTOCOL);
    push_toml_string(&mut out, "schema", DEVICE_PROVIDER_SAMPLE_SCHEMA);
    push_toml_string(&mut out, "source", "nsdb-materialize-provider-samples");
    push_toml_string(&mut out, "status", &materialized_manifest_status(records));
    out.push_str(&format!("record_count = {}\n", records.len()));
    out.push_str(&format!("ready_record_count = {ready_count}\n"));
    out.push_str(&format!("pending_record_count = {pending_count}\n"));
    for record in records {
        out.push_str("\n[[device_provider_samples]]\n");
        push_toml_string(&mut out, "trace_id", &record.trace_id);
        push_toml_string(&mut out, "provider", &record.provider);
        push_toml_string(&mut out, "provider_family", &record.provider_family);
        push_toml_string(
            &mut out,
            "requested_runner_contract",
            &record.requested_runner_contract,
        );
        push_toml_string(
            &mut out,
            "requested_runner_adapter_contract",
            &record.requested_runner_adapter_contract,
        );
        push_toml_string(
            &mut out,
            "requested_runner_adapter_id",
            &record.requested_runner_adapter_id,
        );
        push_toml_string(
            &mut out,
            "requested_runner_adapter_capability_status",
            &record.requested_runner_adapter_capability_status,
        );
        let runner = provider_runner_for(record);
        push_toml_string(&mut out, "provider_runner_contract", runner.contract);
        push_toml_string(
            &mut out,
            "provider_runner_adapter_contract",
            runner.adapter_contract,
        );
        push_toml_string(&mut out, "provider_runner_adapter_id", runner.adapter_id);
        push_toml_string(
            &mut out,
            "provider_runner_adapter_capability_status",
            runner.adapter_capability_status,
        );
        push_toml_string(
            &mut out,
            "provider_runner_registry_protocol",
            runner.registry_protocol,
        );
        push_toml_string(
            &mut out,
            "provider_runner_registry_source",
            runner.registry_source,
        );
        out.push_str(&format!(
            "provider_runner_real_device_capable = {}\n",
            runner.real_device_capable
        ));
        push_toml_string(
            &mut out,
            "provider_runner_real_device_probe_status",
            crate::provider_runner_registry::provider_runner_real_device_probe_status(
                &record.provider_family,
            ),
        );
        push_toml_string(&mut out, "provider_runner_kind", runner.kind);
        push_toml_string(&mut out, "provider_execution_mode", runner.execution_mode);
        let outcome = provider_execution_outcome_for_runner(&runner);
        let output_payload = provider_output_payload_from_record(record);
        push_provider_execution_outcome(&mut out, &outcome, output_payload.as_ref());
        push_toml_string(&mut out, "handoff_target", &record.handoff_target);
        push_toml_string(&mut out, "sample_status", &record.sample_status);
        push_toml_string(&mut out, "validation_status", &record.validation_status);
        push_toml_string(&mut out, "input_evidence", &record.input_evidence);
        push_toml_string(&mut out, "output_evidence", &record.output_evidence);
        push_toml_string(
            &mut out,
            "materialization_status",
            &record.materialization_status,
        );
        push_toml_string(
            &mut out,
            "materialization_detail",
            &record.materialization_detail,
        );
        push_toml_string(&mut out, "next_action", &record.next_action);
    }
    out
}

fn materialized_manifest_status(records: &[NsdbDeviceProviderSampleRecordInfo]) -> String {
    if records.is_empty() {
        "empty"
    } else if records
        .iter()
        .any(|record| record.materialization_status == "provider-sample-blocked")
    {
        "blocked"
    } else if provider_sample_pending_count(records) > 0 {
        "awaiting-provider-materialization"
    } else {
        "ready"
    }
    .to_owned()
}

fn provider_sample_ready_count(records: &[NsdbDeviceProviderSampleRecordInfo]) -> usize {
    records
        .iter()
        .filter(|record| {
            matches!(
                record.materialization_status.as_str(),
                "provider-sample-materialized" | "provider-sample-ready"
            )
        })
        .count()
}

fn provider_sample_pending_count(records: &[NsdbDeviceProviderSampleRecordInfo]) -> usize {
    records
        .iter()
        .filter(|record| record.materialization_status == "provider-sample-pending")
        .count()
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    out.push_str("\"\n");
}
