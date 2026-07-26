use super::toml;
use std::{fs, path::Path};

pub(crate) const DEVICE_PROVIDER_SAMPLE_FILE_NAME: &str = "nuis.nsdb.device-provider-samples.toml";
const DEVICE_PROVIDER_SAMPLE_PROTOCOL: &str = "nuis-device-provider-samples-v1";
const DEVICE_PROVIDER_SAMPLE_SCHEMA: &str = "nsdb-yir-device-provider-sample-v1";
const PROVIDER_BUNDLE_REGISTRY_CONTRACT: &str = "nuis-provider-bundle-registry-v1";
const PROVIDER_BUNDLE_MANIFEST_CONTRACT: &str = "nuis-provider-bundle-manifest-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldDeviceProviderSampleEvidence {
    pub(crate) available: bool,
    pub(crate) path: String,
    pub(crate) status: String,
    pub(crate) record_count: usize,
    pub(crate) ready_record_count: usize,
    pub(crate) pending_record_count: usize,
    pub(crate) blocked_record_count: usize,
    pub(crate) provider_bundle_registry_contract: Option<String>,
    pub(crate) provider_bundle_manifest_contract: Option<String>,
    pub(crate) provider_bundle_manifest_hash: Option<String>,
    pub(crate) provider_bundle_manifest_entry_count: Option<usize>,
    pub(crate) first_provider_bundle_package_id: Option<String>,
    pub(crate) first_provider_bundle_id: Option<String>,
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
            blocked_record_count: 0,
            provider_bundle_registry_contract: None,
            provider_bundle_manifest_contract: None,
            provider_bundle_manifest_hash: None,
            provider_bundle_manifest_entry_count: None,
            first_provider_bundle_package_id: None,
            first_provider_bundle_id: None,
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
    let blocked_record_count = provider_sample_blocked_count(&records);
    let provider_bundle_registry_contract =
        toml::string_value(&source, "provider_bundle_registry_contract");
    let provider_bundle_manifest_contract =
        toml::string_value(&source, "provider_bundle_manifest_contract");
    let provider_bundle_manifest_hash =
        toml::string_value(&source, "provider_bundle_manifest_hash");
    let provider_bundle_manifest_entry_count =
        toml::usize_value(&source, "provider_bundle_manifest_entry_count");
    let first_provider_bundle_package_id = toml::first_table_string_value(
        &source,
        "device_provider_samples",
        "provider_bundle_package_id",
    );
    let first_provider_bundle_id =
        toml::first_table_string_value(&source, "device_provider_samples", "provider_bundle_id");
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
        blocked_record_count,
        provider_bundle_registry_contract.as_deref(),
        provider_bundle_manifest_contract.as_deref(),
        provider_bundle_manifest_hash.as_deref(),
        provider_bundle_manifest_entry_count,
        first_provider_bundle_package_id.as_deref(),
        first_provider_bundle_id.as_deref(),
    );
    let first_blocker = provider_sample_first_blocker(
        &status,
        pending_record_count,
        blocked_record_count,
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
        blocked_record_count,
        provider_bundle_registry_contract,
        provider_bundle_manifest_contract,
        provider_bundle_manifest_hash,
        provider_bundle_manifest_entry_count,
        first_provider_bundle_package_id,
        first_provider_bundle_id,
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
    blocked_record_count: usize,
    provider_bundle_registry_contract: Option<&str>,
    provider_bundle_manifest_contract: Option<&str>,
    provider_bundle_manifest_hash: Option<&str>,
    provider_bundle_manifest_entry_count: Option<usize>,
    first_provider_bundle_package_id: Option<&str>,
    first_provider_bundle_id: Option<&str>,
) -> String {
    if protocol != DEVICE_PROVIDER_SAMPLE_PROTOCOL || schema != DEVICE_PROVIDER_SAMPLE_SCHEMA {
        "unsupported-protocol"
    } else if record_count == 0 {
        "empty"
    } else if !provider_bundle_evidence_is_valid(
        provider_bundle_registry_contract,
        provider_bundle_manifest_contract,
        provider_bundle_manifest_hash,
        provider_bundle_manifest_entry_count,
        first_provider_bundle_package_id,
        first_provider_bundle_id,
    ) {
        "provider-bundle-evidence-invalid"
    } else if blocked_record_count > 0 {
        "blocked-provider-sample"
    } else if pending_record_count > 0 {
        "awaiting-provider-materialization"
    } else if ready_record_count == record_count {
        "ready"
    } else {
        "partial"
    }
    .to_owned()
}

fn provider_bundle_evidence_is_valid(
    registry_contract: Option<&str>,
    manifest_contract: Option<&str>,
    manifest_hash: Option<&str>,
    manifest_entry_count: Option<usize>,
    first_package_id: Option<&str>,
    first_bundle_id: Option<&str>,
) -> bool {
    registry_contract == Some(PROVIDER_BUNDLE_REGISTRY_CONTRACT)
        && manifest_contract == Some(PROVIDER_BUNDLE_MANIFEST_CONTRACT)
        && manifest_hash.is_some_and(|hash| {
            hash.len() == "fnv1a64:0000000000000000".len()
                && hash
                    .strip_prefix("fnv1a64:")
                    .is_some_and(|value| value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        })
        && manifest_entry_count.is_some_and(|count| count > 0)
        && first_package_id.is_some_and(|value| !value.trim().is_empty())
        && first_bundle_id.is_some_and(|value| !value.trim().is_empty())
}

fn provider_sample_first_blocker(
    status: &str,
    pending_record_count: usize,
    blocked_record_count: usize,
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
        "blocked-provider-sample" => Some(format!(
            "device-provider-sample:{}:blocked:{}:{}",
            first_provider_family.unwrap_or("unknown-provider-family"),
            blocked_record_count,
            first_materialization_status.unwrap_or("provider-sample-blocked")
        )),
        "provider-bundle-evidence-invalid" => Some(format!(
            "device-provider-sample:{}:provider-bundle-evidence-invalid",
            first_provider_family.unwrap_or("unknown-provider-family")
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

fn provider_sample_blocked_count(records: &[&str]) -> usize {
    records
        .iter()
        .filter(|record| {
            matches!(
                toml::string_value(record, "materialization_status").as_deref(),
                Some("provider-sample-blocked")
            )
        })
        .count()
}
