use super::toml;
use std::{fs, path::Path};

pub(crate) const DEVICE_PROVIDER_SAMPLE_FILE_NAME: &str = "nuis.nsdb.device-provider-samples.toml";
const DEVICE_PROVIDER_SAMPLE_PROTOCOL: &str = "nuis-device-provider-samples-v1";
const DEVICE_PROVIDER_SAMPLE_SCHEMA: &str = "nsdb-yir-device-provider-sample-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldDeviceProviderSampleEvidence {
    pub(crate) available: bool,
    pub(crate) path: String,
    pub(crate) status: String,
    pub(crate) record_count: usize,
    pub(crate) ready_record_count: usize,
    pub(crate) pending_record_count: usize,
    pub(crate) first_provider_family: Option<String>,
    pub(crate) first_materialization_status: Option<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn nsld_device_provider_sample_evidence(
    output_dir: &str,
) -> NsldDeviceProviderSampleEvidence {
    let path = Path::new(output_dir).join(DEVICE_PROVIDER_SAMPLE_FILE_NAME);
    let path_text = path.display().to_string();
    let Ok(source) = fs::read_to_string(&path) else {
        return NsldDeviceProviderSampleEvidence {
            available: false,
            path: path_text,
            status: "missing".to_owned(),
            record_count: 0,
            ready_record_count: 0,
            pending_record_count: 0,
            first_provider_family: None,
            first_materialization_status: None,
            first_blocker: None,
        };
    };
    let protocol = toml::string_value(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let schema = toml::string_value(&source, "schema").unwrap_or_else(|| "none".to_owned());
    let records = source
        .split("[[device_provider_samples]]")
        .skip(1)
        .collect::<Vec<_>>();
    let record_count = records.len();
    let ready_record_count = toml::usize_value(&source, "ready_record_count")
        .unwrap_or_else(|| provider_sample_ready_count(&records));
    let pending_record_count = toml::usize_value(&source, "pending_record_count")
        .unwrap_or_else(|| provider_sample_pending_count(&records));
    let first_provider_family =
        toml::first_table_string_value(&source, "device_provider_samples", "provider_family");
    let first_materialization_status = toml::first_table_string_value(
        &source,
        "device_provider_samples",
        "materialization_status",
    );
    let status = provider_sample_status(
        &protocol,
        &schema,
        record_count,
        ready_record_count,
        pending_record_count,
    );
    let first_blocker = provider_sample_first_blocker(
        &status,
        pending_record_count,
        first_provider_family.as_deref(),
        first_materialization_status.as_deref(),
    );

    NsldDeviceProviderSampleEvidence {
        available: true,
        path: path_text,
        status,
        record_count,
        ready_record_count,
        pending_record_count,
        first_provider_family,
        first_materialization_status,
        first_blocker,
    }
}

fn provider_sample_status(
    protocol: &str,
    schema: &str,
    record_count: usize,
    ready_record_count: usize,
    pending_record_count: usize,
) -> String {
    if protocol != DEVICE_PROVIDER_SAMPLE_PROTOCOL || schema != DEVICE_PROVIDER_SAMPLE_SCHEMA {
        "unsupported-protocol"
    } else if record_count == 0 {
        "empty"
    } else if pending_record_count > 0 {
        "awaiting-provider-materialization"
    } else if ready_record_count == record_count {
        "ready"
    } else {
        "partial"
    }
    .to_owned()
}

fn provider_sample_first_blocker(
    status: &str,
    pending_record_count: usize,
    first_provider_family: Option<&str>,
    first_materialization_status: Option<&str>,
) -> Option<String> {
    match status {
        "ready" | "empty" => None,
        "awaiting-provider-materialization" => Some(format!(
            "device-provider-sample:{}:pending:{}",
            first_provider_family.unwrap_or("unknown-provider-family"),
            pending_record_count
        )),
        _ => Some(format!(
            "device-provider-sample:{}:{}",
            first_provider_family.unwrap_or("unknown-provider-family"),
            first_materialization_status.unwrap_or(status)
        )),
    }
}

fn provider_sample_ready_count(records: &[&str]) -> usize {
    records
        .iter()
        .filter(|record| {
            matches!(
                toml::string_value(record, "materialization_status").as_deref(),
                Some("provider-sample-materialized" | "provider-sample-ready")
            )
        })
        .count()
}

fn provider_sample_pending_count(records: &[&str]) -> usize {
    records
        .iter()
        .filter(|record| {
            toml::string_value(record, "materialization_status").as_deref()
                == Some("provider-sample-pending")
        })
        .count()
}
