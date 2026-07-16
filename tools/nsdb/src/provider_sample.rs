use crate::model::{NsdbDeviceProviderSampleManifestInfo, NsdbDeviceProviderSampleRecordInfo};
use std::{fs, path::Path};

const DEVICE_PROVIDER_SAMPLE_FILE_NAME: &str = "nuis.nsdb.device-provider-samples.toml";
const DEVICE_PROVIDER_SAMPLE_PROTOCOL: &str = "nuis-device-provider-samples-v1";
const DEVICE_PROVIDER_SAMPLE_SCHEMA: &str = "nsdb-yir-device-provider-sample-v1";

pub(crate) fn read_device_provider_sample_manifest_info(
    output_dir: &Path,
) -> NsdbDeviceProviderSampleManifestInfo {
    let path = output_dir.join(DEVICE_PROVIDER_SAMPLE_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return missing_manifest(path.display().to_string(), "manifest-not-found");
    };
    let protocol = toml_string_value(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let schema = toml_string_value(&source, "schema").unwrap_or_else(|| "none".to_owned());
    let records = source
        .split("[[device_provider_samples]]")
        .skip(1)
        .collect::<Vec<_>>();
    let summaries = records
        .iter()
        .enumerate()
        .map(|(index, record)| provider_sample_record_summary(index, record))
        .collect::<Vec<_>>();
    let invalid_record_count = summaries.iter().filter(|record| !record.valid).count();
    let first = summaries.first();
    NsdbDeviceProviderSampleManifestInfo {
        available: true,
        path: path.display().to_string(),
        protocol: protocol.clone(),
        schema: schema.clone(),
        status: provider_sample_manifest_status(
            &protocol,
            &schema,
            summaries.is_empty(),
            invalid_record_count,
            parse_usize_toml_field(&source, "pending_record_count").unwrap_or(0),
        ),
        record_count: summaries.len(),
        ready_record_count: parse_usize_toml_field(&source, "ready_record_count").unwrap_or(0),
        pending_record_count: parse_usize_toml_field(&source, "pending_record_count")
            .unwrap_or_else(|| {
                summaries
                    .iter()
                    .filter(|record| record.materialization_status == "provider-sample-pending")
                    .count()
            }),
        invalid_record_count,
        first_trace_id: first_value(first, |record| &record.trace_id),
        first_provider_family: first_value(first, |record| &record.provider_family),
        first_materialization_status: first_value(first, |record| &record.materialization_status),
        first_diagnostic: first_value(first, |record| &record.diagnostic),
        records: summaries,
    }
}

fn missing_manifest(path: String, diagnostic: &str) -> NsdbDeviceProviderSampleManifestInfo {
    NsdbDeviceProviderSampleManifestInfo {
        available: false,
        path,
        protocol: "none".to_owned(),
        schema: "none".to_owned(),
        status: "missing".to_owned(),
        record_count: 0,
        ready_record_count: 0,
        pending_record_count: 0,
        invalid_record_count: 0,
        first_trace_id: "none".to_owned(),
        first_provider_family: "none".to_owned(),
        first_materialization_status: "none".to_owned(),
        first_diagnostic: diagnostic.to_owned(),
        records: Vec::new(),
    }
}

fn provider_sample_manifest_status(
    protocol: &str,
    schema: &str,
    empty: bool,
    invalid_record_count: usize,
    pending_record_count: usize,
) -> String {
    if protocol == "none" || schema == "none" {
        return "missing-protocol".to_owned();
    }
    if protocol != DEVICE_PROVIDER_SAMPLE_PROTOCOL || schema != DEVICE_PROVIDER_SAMPLE_SCHEMA {
        return "unsupported-protocol".to_owned();
    }
    if invalid_record_count > 0 {
        "invalid-records".to_owned()
    } else if empty {
        "empty".to_owned()
    } else if pending_record_count > 0 {
        "awaiting-provider-materialization".to_owned()
    } else {
        "ready".to_owned()
    }
}

fn provider_sample_record_summary(
    index: usize,
    record: &str,
) -> NsdbDeviceProviderSampleRecordInfo {
    let trace_id = toml_string_value(record, "trace_id").unwrap_or_else(|| "none".to_owned());
    let provider = toml_string_value(record, "provider").unwrap_or_else(|| "none".to_owned());
    let provider_family =
        toml_string_value(record, "provider_family").unwrap_or_else(|| "none".to_owned());
    let materialization_status = toml_string_value(record, "materialization_status")
        .unwrap_or_else(|| "none".to_owned());
    let valid = trace_id != "none"
        && provider != "none"
        && provider_family != "none"
        && materialization_status != "none";
    NsdbDeviceProviderSampleRecordInfo {
        index,
        valid,
        trace_id,
        provider,
        provider_family,
        handoff_target: toml_string_value(record, "handoff_target")
            .unwrap_or_else(|| "none".to_owned()),
        sample_status: toml_string_value(record, "sample_status")
            .unwrap_or_else(|| "none".to_owned()),
        validation_status: toml_string_value(record, "validation_status")
            .unwrap_or_else(|| "none".to_owned()),
        input_evidence: toml_string_value(record, "input_evidence")
            .unwrap_or_else(|| "none".to_owned()),
        output_evidence: toml_string_value(record, "output_evidence")
            .unwrap_or_else(|| "none".to_owned()),
        materialization_status,
        materialization_detail: toml_string_value(record, "materialization_detail")
            .unwrap_or_else(|| "none".to_owned()),
        next_action: toml_string_value(record, "next_action")
            .unwrap_or_else(|| "none".to_owned()),
        diagnostic: if valid {
            "provider-sample-record-loaded".to_owned()
        } else {
            "provider-sample-record-incomplete".to_owned()
        },
    }
}

fn first_value(
    first: Option<&NsdbDeviceProviderSampleRecordInfo>,
    select: fn(&NsdbDeviceProviderSampleRecordInfo) -> &String,
) -> String {
    first
        .map(select)
        .cloned()
        .unwrap_or_else(|| "none".to_owned())
}

fn parse_usize_toml_field(source: &str, key: &str) -> Option<usize> {
    toml_field_value(source, key)?.parse().ok()
}

fn toml_string_value(source: &str, key: &str) -> Option<String> {
    let value = toml_field_value(source, key)?;
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_basic_toml_string)
}

fn toml_field_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
}

fn unescape_basic_toml_string(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(ch);
        }
    }
    out
}
