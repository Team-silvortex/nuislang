use crate::{
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_sample::{
        read_device_provider_sample_manifest_info, DEVICE_PROVIDER_SAMPLE_FILE_NAME,
        DEVICE_PROVIDER_SAMPLE_PROTOCOL, DEVICE_PROVIDER_SAMPLE_SCHEMA,
    },
};
use std::{collections::BTreeSet, fs, path::Path};

pub(crate) struct ProviderSampleMaterializeReport {
    pub(crate) path: String,
    pub(crate) provider_family_filter: Option<String>,
    pub(crate) provider_families: Vec<String>,
    pub(crate) status: String,
    pub(crate) record_count: usize,
    pub(crate) matched_record_count: usize,
    pub(crate) materialized_record_count: usize,
    pub(crate) skipped_record_count: usize,
    pub(crate) first_provider_family: String,
    pub(crate) first_output_evidence: String,
    pub(crate) next_action: String,
    pub(crate) next_command: String,
}

pub(crate) fn materialize_provider_samples(
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
                materialized_record(record)
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
        first_output_evidence: records
            .first()
            .map(|record| record.output_evidence.clone())
            .unwrap_or_else(|| "none".to_owned()),
        next_action: "replay-provider-sample".to_owned(),
        next_command: format!("nsdb replay-plan {} --json", output_dir.display()),
    })
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
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> NsdbDeviceProviderSampleRecordInfo {
    let mut record = record.clone();
    record.sample_status = "provider-execution-ready".to_owned();
    record.validation_status = "provider-execution-validated".to_owned();
    record.output_evidence = record.input_evidence.clone();
    record.materialization_status = "provider-sample-materialized".to_owned();
    record.materialization_detail = "mock-provider-runtime-result-materialized".to_owned();
    record.next_action = "replay-device-sample".to_owned();
    record
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
        assert_eq!(report.next_action, "replay-provider-sample");
        assert!(report.next_command.contains("nsdb replay-plan "));
        assert!(report.next_command.contains("--json"));
        assert!(source.contains("source = \"nsdb-materialize-provider-samples\""));
        assert!(source.contains("ready_record_count = 1"));
        assert!(source.contains("pending_record_count = 0"));
        assert!(source.contains("output_evidence = \"metallib:pixelmagic.metallib\""));
        assert!(source.contains("materialization_status = \"provider-sample-materialized\""));
        assert!(source.contains("next_action = \"replay-device-sample\""));

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
        assert!(source.contains("output_evidence = \"metallib:pixelmagic.metallib\""));
        assert!(source.contains("output_evidence = \"not-materialized\""));
        assert!(source.contains("provider_family = \"spirv:vulkan-gpu\""));

        fs::remove_dir_all(output_dir).unwrap();
    }
}
