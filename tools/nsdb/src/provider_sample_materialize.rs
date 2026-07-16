use crate::{
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_sample::{
        read_device_provider_sample_manifest_info, DEVICE_PROVIDER_SAMPLE_FILE_NAME,
        DEVICE_PROVIDER_SAMPLE_PROTOCOL, DEVICE_PROVIDER_SAMPLE_SCHEMA,
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
    pub first_provider_runner_kind: String,
    pub first_provider_execution_mode: String,
    pub first_output_evidence: String,
    pub next_action: String,
    pub next_command: String,
    pub return_action: String,
    pub return_command: String,
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
                materialized += 1;
                materialized_record(output_dir, record)
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
        first_provider_runner_kind: records
            .first()
            .map(|record| provider_runner_for(record).kind.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_provider_execution_mode: records
            .first()
            .map(|record| provider_runner_for(record).execution_mode.to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        first_output_evidence: records
            .first()
            .map(|record| record.output_evidence.clone())
            .unwrap_or_else(|| "none".to_owned()),
        next_action: "replay-provider-sample".to_owned(),
        next_command: format!("nsdb replay-plan {} --json", output_dir.display()),
        return_action,
        return_command,
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
    record.sample_status = "provider-execution-ready".to_owned();
    record.validation_status = "provider-execution-validated".to_owned();
    record.output_evidence = artifact.evidence;
    record.materialization_status = "provider-sample-materialized".to_owned();
    record.materialization_detail = artifact.detail;
    record.next_action = "replay-device-sample".to_owned();
    record
}

struct ProviderSampleRunner {
    contract: &'static str,
    adapter_contract: &'static str,
    adapter_id: &'static str,
    adapter_capability_status: &'static str,
    kind: &'static str,
    execution_mode: &'static str,
    backend: &'static str,
    device: &'static str,
}

fn provider_runner_for(record: &NsdbDeviceProviderSampleRecordInfo) -> ProviderSampleRunner {
    match record.provider_family.as_str() {
        "metal:apple-silicon-gpu" => ProviderSampleRunner {
            contract: "nuis-provider-runner-v1",
            adapter_contract: "nuis-provider-runner-adapter-v1",
            adapter_id: "metal.apple-silicon-gpu.host-simulated",
            adapter_capability_status: "registered-host-simulated",
            kind: "metal-host-simulated-runner",
            execution_mode: "host-simulated-provider-runner",
            backend: "metal",
            device: "apple-silicon-gpu",
        },
        "coreml:apple-ane" => ProviderSampleRunner {
            contract: "nuis-provider-runner-v1",
            adapter_contract: "nuis-provider-runner-adapter-v1",
            adapter_id: "coreml.apple-ane.host-simulated",
            adapter_capability_status: "registered-host-simulated",
            kind: "coreml-host-simulated-runner",
            execution_mode: "host-simulated-provider-runner",
            backend: "coreml",
            device: "apple-ane",
        },
        _ => ProviderSampleRunner {
            contract: "nuis-provider-runner-v1",
            adapter_contract: "nuis-provider-runner-adapter-v1",
            adapter_id: "generic.device.host-simulated",
            adapter_capability_status: "registered-host-simulated",
            kind: "generic-host-simulated-runner",
            execution_mode: "host-simulated-provider-runner",
            backend: "generic",
            device: "generic-device",
        },
    }
}

struct ProviderSampleArtifact {
    evidence: String,
    detail: String,
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
    let path = output_dir.join(&file_name);
    let content = render_provider_sample_artifact(record, runner);
    let hash = fnv1a64_hex(content.as_bytes());
    let write_status = match fs::write(&path, content) {
        Ok(()) => "written",
        Err(_) => "write-failed",
    };
    ProviderSampleArtifact {
        evidence: format!("{file_name}:hash={hash}:status={write_status}"),
        detail: format!("deterministic-provider-sample-artifact:{file_name}:{hash}:{write_status}"),
    }
}

fn render_provider_sample_artifact(
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
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
    push_toml_string(&mut out, "provider_runner_kind", runner.kind);
    push_toml_string(&mut out, "provider_execution_mode", runner.execution_mode);
    push_toml_string(&mut out, "provider_backend", runner.backend);
    push_toml_string(&mut out, "provider_device", runner.device);
    push_toml_string(&mut out, "handoff_target", &record.handoff_target);
    push_toml_string(&mut out, "input_evidence", &record.input_evidence);
    push_toml_string(&mut out, "sample_status", "provider-execution-ready");
    push_toml_string(
        &mut out,
        "validation_status",
        "provider-execution-validated",
    );
    push_toml_string(&mut out, "materialization_mode", runner.execution_mode);
    out
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

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
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
        push_toml_string(&mut out, "provider_runner_kind", runner.kind);
        push_toml_string(&mut out, "provider_execution_mode", runner.execution_mode);
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

#[cfg(test)]
mod tests {
    use super::materialize_provider_samples;
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn materializes_pending_provider_sample_manifest() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output_dir = env::temp_dir().join(format!("nsdb-provider-materialize-{nonce}"));
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(
            output_dir.join("nuis.nsdb.device-provider-samples.toml"),
            r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
        )
        .unwrap();

        let report = materialize_provider_samples(&output_dir, None).unwrap();
        let source =
            fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();

        assert_eq!(report.status, "ready");
        assert_eq!(
            report.provider_families,
            vec!["metal:apple-silicon-gpu".to_owned()]
        );
        assert_eq!(report.matched_record_count, 1);
        assert_eq!(report.materialized_record_count, 1);
        assert_eq!(report.skipped_record_count, 0);
        assert_eq!(
            report.first_provider_runner_contract,
            "nuis-provider-runner-v1"
        );
        assert_eq!(
            report.first_provider_runner_adapter_contract,
            "nuis-provider-runner-adapter-v1"
        );
        assert_eq!(
            report.first_provider_runner_adapter_id,
            "metal.apple-silicon-gpu.host-simulated"
        );
        assert_eq!(
            report.first_provider_runner_adapter_capability_status,
            "registered-host-simulated"
        );
        assert_eq!(
            report.first_provider_runner_kind,
            "metal-host-simulated-runner"
        );
        assert_eq!(
            report.first_provider_execution_mode,
            "host-simulated-provider-runner"
        );
        assert_eq!(report.next_action, "replay-provider-sample");
        assert!(report.next_command.contains("nsdb replay-plan "));
        assert!(report.next_command.contains("--json"));
        assert_eq!(
            report.return_action,
            "resume-nsld-final-output-check-manifest-required"
        );
        assert_eq!(
            report.return_command,
            "nsld check <nuis.build.manifest.toml> --json"
        );
        assert!(source.contains("source = \"nsdb-materialize-provider-samples\""));
        assert!(source.contains("ready_record_count = 1"));
        assert!(source.contains("pending_record_count = 0"));
        assert!(source.contains(
            "output_evidence = \"nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml:hash=0x"
        ));
        assert!(source.contains("materialization_status = \"provider-sample-materialized\""));
        assert!(source.contains("provider_runner_contract = \"nuis-provider-runner-v1\""));
        assert!(source
            .contains("provider_runner_adapter_contract = \"nuis-provider-runner-adapter-v1\""));
        assert!(source
            .contains("provider_runner_adapter_id = \"metal.apple-silicon-gpu.host-simulated\""));
        assert!(source
            .contains("provider_runner_adapter_capability_status = \"registered-host-simulated\""));
        assert!(source.contains("provider_runner_kind = \"metal-host-simulated-runner\""));
        assert!(source.contains("provider_execution_mode = \"host-simulated-provider-runner\""));
        assert!(source.contains(
            "materialization_detail = \"deterministic-provider-sample-artifact:nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml:0x"
        ));
        assert!(source.contains("next_action = \"replay-device-sample\""));
        let artifact = fs::read_to_string(
            output_dir.join("nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml"),
        )
        .unwrap();
        assert!(artifact.contains("protocol = \"nuis-nsdb-provider-sample-artifact-v1\""));
        assert!(artifact.contains("schema = \"nsdb-yir-provider-sample-artifact-v1\""));
        assert!(artifact.contains("provider_runner_contract = \"nuis-provider-runner-v1\""));
        assert!(artifact
            .contains("provider_runner_adapter_contract = \"nuis-provider-runner-adapter-v1\""));
        assert!(artifact
            .contains("provider_runner_adapter_id = \"metal.apple-silicon-gpu.host-simulated\""));
        assert!(artifact
            .contains("provider_runner_adapter_capability_status = \"registered-host-simulated\""));
        assert!(artifact.contains("provider_runner_kind = \"metal-host-simulated-runner\""));
        assert!(artifact.contains("provider_backend = \"metal\""));
        assert!(artifact.contains("provider_device = \"apple-silicon-gpu\""));
        assert!(artifact.contains("materialization_mode = \"host-simulated-provider-runner\""));

        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn materializer_returns_concrete_nsld_check_when_manifest_is_in_output_dir() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output_dir = env::temp_dir().join(format!("nsdb-provider-return-{nonce}"));
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(
            output_dir.join("nuis.build.manifest.toml"),
            "manifest = true\n",
        )
        .unwrap();
        fs::write(
            output_dir.join("nuis.nsdb.device-provider-samples.toml"),
            r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
        )
        .unwrap();

        let report = materialize_provider_samples(&output_dir, None).unwrap();

        assert_eq!(report.return_action, "resume-nsld-final-output-check");
        assert_eq!(
            report.return_command,
            format!("nsld check {} --json", output_dir.display())
        );

        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn materializes_only_matching_provider_family() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output_dir = env::temp_dir().join(format!("nsdb-provider-filter-{nonce}"));
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(
            output_dir.join("nuis.nsdb.device-provider-samples.toml"),
            r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 2
ready_record_count = 0
pending_record_count = 2

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"

[[device_provider_samples]]
trace_id = "hetero-trace:shader:spirv:vulkan-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "spirv:vulkan-gpu"
handoff_target = "spirv:vulkan-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "spv:pixelmagic.spv"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
        )
        .unwrap();

        let report =
            materialize_provider_samples(&output_dir, Some("metal:apple-silicon-gpu")).unwrap();
        let source =
            fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();

        assert_eq!(
            report.provider_family_filter.as_deref(),
            Some("metal:apple-silicon-gpu")
        );
        assert_eq!(
            report.provider_families,
            vec![
                "metal:apple-silicon-gpu".to_owned(),
                "spirv:vulkan-gpu".to_owned()
            ]
        );
        assert_eq!(report.status, "awaiting-provider-materialization");
        assert_eq!(report.matched_record_count, 1);
        assert_eq!(report.materialized_record_count, 1);
        assert_eq!(report.skipped_record_count, 1);
        assert!(source.contains("ready_record_count = 1"));
        assert!(source.contains("pending_record_count = 1"));
        assert!(source.contains(
            "output_evidence = \"nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml:hash=0x"
        ));
        assert!(source.contains("output_evidence = \"not-materialized\""));
        assert!(source.contains("provider_family = \"spirv:vulkan-gpu\""));

        fs::remove_dir_all(output_dir).unwrap();
    }
}
