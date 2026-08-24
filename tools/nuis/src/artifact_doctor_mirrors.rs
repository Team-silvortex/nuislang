use std::{
    fs,
    path::{Path, PathBuf},
};

const PAYLOAD_DECODER_MANIFEST_FILE_NAME: &str = "nuis.nsdb.payload-decoders.toml";
const PAYLOAD_DECODER_MANIFEST_PROTOCOL: &str = "nuis-nsdb-payload-decoders-v1";
const PAYLOAD_DECODER_MANIFEST_SCHEMA: &str = "nsdb-payload-decoder-manifest-v1";
const DEVICE_PROVIDER_SAMPLE_FILE_NAME: &str = "nuis.nsdb.device-provider-samples.toml";
const DEVICE_PROVIDER_SAMPLE_PROTOCOL: &str = "nuis-device-provider-samples-v1";
const DEVICE_PROVIDER_SAMPLE_SCHEMA: &str = "nsdb-yir-device-provider-sample-v1";
const PROVIDER_BUNDLE_REGISTRY_CONTRACT: &str = "nuis-provider-bundle-registry-v1";
const PROVIDER_BUNDLE_MANIFEST_CONTRACT: &str = "nuis-provider-bundle-manifest-v1";
const SELECTED_PROVIDER_BUNDLE_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";

pub(crate) fn collect_payload_decoder_manifest_mirror(
    output_dir: Option<&Path>,
) -> PayloadDecoderManifestMirror {
    let Some(output_dir) = output_dir else {
        return PayloadDecoderManifestMirror::unavailable(None, "output-dir-unavailable");
    };
    let path = output_dir.join(PAYLOAD_DECODER_MANIFEST_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return PayloadDecoderManifestMirror::unavailable(Some(path), "manifest-not-found");
    };
    let protocol =
        parse_string_toml_field(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let schema = parse_string_toml_field(&source, "schema").unwrap_or_else(|| "none".to_owned());
    let records = source.split("[[decoders]]").skip(1).collect::<Vec<_>>();
    let invalid_record_count = records
        .iter()
        .filter(|record| payload_decoder_manifest_record_invalid(record))
        .count();
    let first_diagnostic = records
        .first()
        .map(|record| payload_decoder_manifest_record_diagnostic(record))
        .unwrap_or_else(|| "manifest-empty".to_owned());
    PayloadDecoderManifestMirror {
        available: true,
        path: Some(path),
        protocol: protocol.clone(),
        schema: schema.clone(),
        status: payload_decoder_manifest_status(
            &protocol,
            &schema,
            records.is_empty(),
            invalid_record_count,
        ),
        record_count: records.len(),
        invalid_record_count,
        first_diagnostic,
    }
}

pub(crate) fn collect_backend_artifact_payload_evidence(
    output_dir: Option<&Path>,
) -> BackendArtifactPayloadEvidence {
    let Some(output_dir) = output_dir else {
        return BackendArtifactPayloadEvidence::unavailable();
    };
    let path = output_dir.join("nuis.nsld.final-executable-image-dry-run.toml");
    let Ok(source) = fs::read_to_string(&path) else {
        return BackendArtifactPayloadEvidence {
            available: false,
            path: Some(path),
            ..BackendArtifactPayloadEvidence::unavailable()
        };
    };
    BackendArtifactPayloadEvidence {
        available: true,
        path: Some(path),
        count: parse_usize_toml_field(&source, "backend_artifact_payload_count").unwrap_or(0),
        present_count: parse_usize_toml_field(&source, "backend_artifact_payload_present_count")
            .unwrap_or(0),
        role_status: parse_string_toml_field(&source, "backend_artifact_payload_role_status")
            .unwrap_or_else(|| "unknown".to_owned()),
        ids: parse_string_array_toml_field(&source, "backend_artifact_payload_ids"),
        kinds: parse_string_array_toml_field(&source, "backend_artifact_payload_kinds"),
        first_missing: parse_string_toml_field(&source, "backend_artifact_payload_first_missing")
            .filter(|value| !value.is_empty()),
    }
}

pub(crate) fn collect_device_provider_sample_manifest_mirror(
    output_dir: Option<&Path>,
) -> DeviceProviderSampleManifestMirror {
    let Some(output_dir) = output_dir else {
        return DeviceProviderSampleManifestMirror::unavailable(None, "output-dir-unavailable");
    };
    let path = output_dir.join(DEVICE_PROVIDER_SAMPLE_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return DeviceProviderSampleManifestMirror::unavailable(Some(path), "manifest-not-found");
    };
    let protocol =
        parse_string_toml_field(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let schema = parse_string_toml_field(&source, "schema").unwrap_or_else(|| "none".to_owned());
    let records = source
        .split("[[device_provider_samples]]")
        .skip(1)
        .collect::<Vec<_>>();
    let invalid_record_count = records
        .iter()
        .filter(|record| device_provider_sample_record_invalid(record))
        .count();
    let pending_record_count = parse_usize_toml_field(&source, "pending_record_count")
        .unwrap_or_else(|| device_provider_sample_pending_count(&records));
    let blocked_record_count = device_provider_sample_blocked_count(&records);
    let provider_bundle_registry_contract =
        parse_string_toml_field(&source, "provider_bundle_registry_contract")
            .unwrap_or_else(|| "none".to_owned());
    let provider_bundle_manifest_contract =
        parse_string_toml_field(&source, "provider_bundle_manifest_contract")
            .unwrap_or_else(|| "none".to_owned());
    let provider_bundle_manifest_hash =
        parse_string_toml_field(&source, "provider_bundle_manifest_hash")
            .unwrap_or_else(|| "none".to_owned());
    let provider_bundle_manifest_entry_count =
        parse_usize_toml_field(&source, "provider_bundle_manifest_entry_count").unwrap_or(0);
    let first_provider_bundle_package_id = records
        .first()
        .and_then(|record| parse_string_toml_field(record, "provider_bundle_package_id"))
        .unwrap_or_else(|| "none".to_owned());
    let first_provider_bundle_id = records
        .first()
        .and_then(|record| parse_string_toml_field(record, "provider_bundle_id"))
        .unwrap_or_else(|| "none".to_owned());
    let selected_provider_bundle_set_contract =
        parse_string_toml_field(&source, "selected_provider_bundle_set_contract")
            .unwrap_or_else(|| "none".to_owned());
    let selected_provider_bundle_count =
        parse_usize_toml_field(&source, "selected_provider_bundle_count").unwrap_or(0);
    let selected_provider_bundle_set_hash =
        parse_string_toml_field(&source, "selected_provider_bundle_set_hash")
            .unwrap_or_else(|| "none".to_owned());
    let selected_provider_bundle_set_validation_status =
        selected_provider_bundle_set_validation_status(
            &source,
            &records,
            &selected_provider_bundle_set_contract,
            selected_provider_bundle_count,
            &selected_provider_bundle_set_hash,
        );
    let provider_bundle_evidence_status = provider_bundle_evidence_status(
        records.is_empty(),
        (
            &provider_bundle_registry_contract,
            &provider_bundle_manifest_contract,
            &provider_bundle_manifest_hash,
            provider_bundle_manifest_entry_count,
        ),
        (&first_provider_bundle_package_id, &first_provider_bundle_id),
        &selected_provider_bundle_set_validation_status,
    );
    DeviceProviderSampleManifestMirror {
        available: true,
        path: Some(path),
        protocol: protocol.clone(),
        schema: schema.clone(),
        status: device_provider_sample_manifest_status(
            &protocol,
            &schema,
            records.is_empty(),
            invalid_record_count,
            pending_record_count,
            blocked_record_count,
        ),
        record_count: records.len(),
        pending_record_count,
        blocked_record_count,
        invalid_record_count,
        first_provider_family: records
            .first()
            .and_then(|record| parse_string_toml_field(record, "provider_family"))
            .unwrap_or_else(|| "none".to_owned()),
        first_materialization_status: records
            .first()
            .and_then(|record| parse_string_toml_field(record, "materialization_status"))
            .unwrap_or_else(|| "none".to_owned()),
        provider_bundle_registry_contract,
        provider_bundle_manifest_contract,
        provider_bundle_manifest_hash,
        provider_bundle_manifest_entry_count,
        first_provider_bundle_package_id,
        first_provider_bundle_id,
        provider_bundle_evidence_status,
        selected_provider_bundle_set_contract,
        selected_provider_bundle_count,
        selected_provider_bundle_set_hash,
        selected_provider_bundle_set_validation_status,
    }
}

pub(crate) struct PayloadDecoderManifestMirror {
    pub(crate) available: bool,
    pub(crate) path: Option<PathBuf>,
    pub(crate) protocol: String,
    pub(crate) schema: String,
    pub(crate) status: String,
    pub(crate) record_count: usize,
    pub(crate) invalid_record_count: usize,
    pub(crate) first_diagnostic: String,
}

impl PayloadDecoderManifestMirror {
    fn unavailable(path: Option<PathBuf>, diagnostic: &str) -> Self {
        Self {
            available: false,
            path,
            protocol: "none".to_owned(),
            schema: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            invalid_record_count: 0,
            first_diagnostic: diagnostic.to_owned(),
        }
    }

    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        vec![
            crate::json_bool_field(&format!("{prefix}_available"), self.available),
            crate::json_optional_string_field(
                &format!("{prefix}_path"),
                self.path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            crate::json_field(&format!("{prefix}_protocol"), &self.protocol),
            crate::json_field(&format!("{prefix}_schema"), &self.schema),
            crate::json_field(&format!("{prefix}_status"), &self.status),
            crate::json_usize_field(&format!("{prefix}_record_count"), self.record_count),
            crate::json_usize_field(
                &format!("{prefix}_invalid_record_count"),
                self.invalid_record_count,
            ),
            crate::json_optional_string_field(
                &format!("{prefix}_first_diagnostic"),
                (!self.first_diagnostic.is_empty()).then_some(self.first_diagnostic.as_str()),
            ),
        ]
    }
}

pub(crate) struct DeviceProviderSampleManifestMirror {
    pub(crate) available: bool,
    pub(crate) path: Option<PathBuf>,
    pub(crate) protocol: String,
    pub(crate) schema: String,
    pub(crate) status: String,
    pub(crate) record_count: usize,
    pub(crate) pending_record_count: usize,
    pub(crate) blocked_record_count: usize,
    pub(crate) invalid_record_count: usize,
    pub(crate) first_provider_family: String,
    pub(crate) first_materialization_status: String,
    pub(crate) provider_bundle_registry_contract: String,
    pub(crate) provider_bundle_manifest_contract: String,
    pub(crate) provider_bundle_manifest_hash: String,
    pub(crate) provider_bundle_manifest_entry_count: usize,
    pub(crate) first_provider_bundle_package_id: String,
    pub(crate) first_provider_bundle_id: String,
    pub(crate) provider_bundle_evidence_status: String,
    pub(crate) selected_provider_bundle_set_contract: String,
    pub(crate) selected_provider_bundle_count: usize,
    pub(crate) selected_provider_bundle_set_hash: String,
    pub(crate) selected_provider_bundle_set_validation_status: String,
}

impl DeviceProviderSampleManifestMirror {
    fn unavailable(path: Option<PathBuf>, _diagnostic: &str) -> Self {
        Self {
            available: false,
            path,
            protocol: "none".to_owned(),
            schema: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            pending_record_count: 0,
            blocked_record_count: 0,
            invalid_record_count: 0,
            first_provider_family: "none".to_owned(),
            first_materialization_status: "none".to_owned(),
            provider_bundle_registry_contract: "none".to_owned(),
            provider_bundle_manifest_contract: "none".to_owned(),
            provider_bundle_manifest_hash: "none".to_owned(),
            provider_bundle_manifest_entry_count: 0,
            first_provider_bundle_package_id: "none".to_owned(),
            first_provider_bundle_id: "none".to_owned(),
            provider_bundle_evidence_status: "not-applicable".to_owned(),
            selected_provider_bundle_set_contract: "none".to_owned(),
            selected_provider_bundle_count: 0,
            selected_provider_bundle_set_hash: "none".to_owned(),
            selected_provider_bundle_set_validation_status: "not-applicable".to_owned(),
        }
    }

    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        vec![
            crate::json_bool_field(&format!("{prefix}_available"), self.available),
            crate::json_optional_string_field(
                &format!("{prefix}_path"),
                self.path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            crate::json_field(&format!("{prefix}_protocol"), &self.protocol),
            crate::json_field(&format!("{prefix}_schema"), &self.schema),
            crate::json_field(&format!("{prefix}_status"), &self.status),
            crate::json_usize_field(&format!("{prefix}_record_count"), self.record_count),
            crate::json_usize_field(
                &format!("{prefix}_pending_record_count"),
                self.pending_record_count,
            ),
            crate::json_usize_field(
                &format!("{prefix}_blocked_record_count"),
                self.blocked_record_count,
            ),
            crate::json_usize_field(
                &format!("{prefix}_invalid_record_count"),
                self.invalid_record_count,
            ),
            crate::json_field(
                &format!("{prefix}_first_provider_family"),
                &self.first_provider_family,
            ),
            crate::json_field(
                &format!("{prefix}_first_materialization_status"),
                &self.first_materialization_status,
            ),
            crate::json_field(
                &format!("{prefix}_provider_bundle_registry_contract"),
                &self.provider_bundle_registry_contract,
            ),
            crate::json_field(
                &format!("{prefix}_provider_bundle_manifest_contract"),
                &self.provider_bundle_manifest_contract,
            ),
            crate::json_field(
                &format!("{prefix}_provider_bundle_manifest_hash"),
                &self.provider_bundle_manifest_hash,
            ),
            crate::json_usize_field(
                &format!("{prefix}_provider_bundle_manifest_entry_count"),
                self.provider_bundle_manifest_entry_count,
            ),
            crate::json_field(
                &format!("{prefix}_first_provider_bundle_package_id"),
                &self.first_provider_bundle_package_id,
            ),
            crate::json_field(
                &format!("{prefix}_first_provider_bundle_id"),
                &self.first_provider_bundle_id,
            ),
            crate::json_field(
                &format!("{prefix}_provider_bundle_evidence_status"),
                &self.provider_bundle_evidence_status,
            ),
            crate::json_field(
                &format!("{prefix}_selected_provider_bundle_set_contract"),
                &self.selected_provider_bundle_set_contract,
            ),
            crate::json_usize_field(
                &format!("{prefix}_selected_provider_bundle_count"),
                self.selected_provider_bundle_count,
            ),
            crate::json_field(
                &format!("{prefix}_selected_provider_bundle_set_hash"),
                &self.selected_provider_bundle_set_hash,
            ),
            crate::json_field(
                &format!("{prefix}_selected_provider_bundle_set_validation_status"),
                &self.selected_provider_bundle_set_validation_status,
            ),
        ]
    }
}

pub(crate) struct BackendArtifactPayloadEvidence {
    pub(crate) available: bool,
    pub(crate) path: Option<PathBuf>,
    pub(crate) count: usize,
    pub(crate) present_count: usize,
    pub(crate) role_status: String,
    pub(crate) ids: Vec<String>,
    pub(crate) kinds: Vec<String>,
    pub(crate) first_missing: Option<String>,
}

impl BackendArtifactPayloadEvidence {
    pub(crate) fn unavailable() -> Self {
        Self {
            available: false,
            path: None,
            count: 0,
            present_count: 0,
            role_status: "unavailable".to_owned(),
            ids: Vec::new(),
            kinds: Vec::new(),
            first_missing: None,
        }
    }
}

fn device_provider_sample_manifest_status(
    protocol: &str,
    schema: &str,
    empty: bool,
    invalid_record_count: usize,
    pending_record_count: usize,
    blocked_record_count: usize,
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
    } else if blocked_record_count > 0 {
        "blocked-provider-sample".to_owned()
    } else if pending_record_count > 0 {
        "awaiting-provider-materialization".to_owned()
    } else {
        "ready".to_owned()
    }
}

fn provider_bundle_evidence_status(
    empty: bool,
    manifest: (&str, &str, &str, usize),
    first_bundle: (&str, &str),
    selected_provider_bundle_set_validation_status: &str,
) -> String {
    let (registry_contract, manifest_contract, manifest_hash, manifest_entry_count) = manifest;
    let (first_package_id, first_bundle_id) = first_bundle;
    if empty {
        return "not-applicable".to_owned();
    }
    let valid_hash = manifest_hash
        .strip_prefix("fnv1a64:")
        .is_some_and(|hash| hash.len() == 16 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
    if registry_contract == PROVIDER_BUNDLE_REGISTRY_CONTRACT
        && manifest_contract == PROVIDER_BUNDLE_MANIFEST_CONTRACT
        && valid_hash
        && manifest_entry_count > 0
        && first_package_id != "none"
        && !first_package_id.trim().is_empty()
        && first_bundle_id != "none"
        && !first_bundle_id.trim().is_empty()
        && selected_provider_bundle_set_validation_status == "verified"
    {
        "verified".to_owned()
    } else {
        "provider-bundle-evidence-invalid".to_owned()
    }
}

fn selected_provider_bundle_set_validation_status(
    source: &str,
    records: &[&str],
    contract: &str,
    count_claim: usize,
    hash_claim: &str,
) -> String {
    if records.is_empty() {
        return "not-applicable".to_owned();
    }
    let mut selected = Vec::new();
    let mut seen_bundle_ids = std::collections::BTreeSet::new();
    let dispatch = source
        .split("[[provider_dispatch]]")
        .skip(1)
        .collect::<Vec<_>>();
    let identities = if dispatch.is_empty() {
        records
    } else {
        &dispatch
    };
    for record in identities {
        let Some(package_id) = parse_string_toml_field(record, "provider_bundle_package_id") else {
            return "mismatch".to_owned();
        };
        let Some(bundle_id) = parse_string_toml_field(record, "provider_bundle_id") else {
            return "mismatch".to_owned();
        };
        let Some(provider_family) = parse_string_toml_field(record, "provider_family") else {
            return "mismatch".to_owned();
        };
        if seen_bundle_ids.insert(bundle_id.clone()) {
            selected.push((package_id, bundle_id, provider_family));
        }
    }
    let mut canonical = format!("{SELECTED_PROVIDER_BUNDLE_SET_CONTRACT}\n");
    for (index, (package_id, bundle_id, provider_family)) in selected.iter().enumerate() {
        canonical.push_str(&format!(
            "{index}|{package_id}|{bundle_id}|{provider_family}\n"
        ));
    }
    let actual_hash = format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes()));
    if contract == SELECTED_PROVIDER_BUNDLE_SET_CONTRACT
        && count_claim == selected.len()
        && hash_claim == actual_hash
    {
        "verified".to_owned()
    } else {
        "mismatch".to_owned()
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn device_provider_sample_pending_count(records: &[&str]) -> usize {
    records
        .iter()
        .filter(|record| {
            parse_string_toml_field(record, "materialization_status").as_deref()
                == Some("provider-sample-pending")
        })
        .count()
}

fn device_provider_sample_blocked_count(records: &[&str]) -> usize {
    records
        .iter()
        .filter(|record| {
            parse_string_toml_field(record, "materialization_status").as_deref()
                == Some("provider-sample-blocked")
        })
        .count()
}

fn device_provider_sample_record_invalid(record: &str) -> bool {
    parse_string_toml_field(record, "trace_id").is_none()
        || parse_string_toml_field(record, "provider_family").is_none()
        || parse_string_toml_field(record, "materialization_status").is_none()
}

fn payload_decoder_manifest_status(
    protocol: &str,
    schema: &str,
    empty: bool,
    invalid_record_count: usize,
) -> String {
    if protocol == "none" || schema == "none" {
        return "missing-protocol".to_owned();
    }
    if protocol != PAYLOAD_DECODER_MANIFEST_PROTOCOL || schema != PAYLOAD_DECODER_MANIFEST_SCHEMA {
        return "unsupported-protocol".to_owned();
    }
    if invalid_record_count > 0 {
        "invalid-records".to_owned()
    } else if empty {
        "empty".to_owned()
    } else {
        "ready".to_owned()
    }
}

fn payload_decoder_manifest_record_invalid(record: &str) -> bool {
    parse_string_toml_field(record, "payload_format").is_none()
        || parse_string_toml_field(record, "magic_hex")
            .is_some_and(|value| !valid_hex_bytes(&value))
}

fn payload_decoder_manifest_record_diagnostic(record: &str) -> String {
    if parse_string_toml_field(record, "payload_format").is_none() {
        "manifest-external-decoder-missing-payload-format".to_owned()
    } else if parse_string_toml_field(record, "magic_hex")
        .is_some_and(|value| !valid_hex_bytes(&value))
    {
        "manifest-external-decoder-invalid-magic".to_owned()
    } else {
        "manifest-external-decoder-loaded".to_owned()
    }
}

fn valid_hex_bytes(value: &str) -> bool {
    let digits = value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '_')
        .collect::<String>();
    !digits.is_empty()
        && digits.len() % 2 == 0
        && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_usize_toml_field(source: &str, key: &str) -> Option<usize> {
    parse_toml_field_value(source, key)?.parse().ok()
}

fn parse_string_toml_field(source: &str, key: &str) -> Option<String> {
    let value = parse_toml_field_value(source, key)?;
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_basic_toml_string)
}

fn parse_string_array_toml_field(source: &str, key: &str) -> Vec<String> {
    let Some(value) = parse_toml_field_value(source, key) else {
        return Vec::new();
    };
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in inner.chars() {
        if !in_string {
            if ch == '"' {
                in_string = true;
                current.clear();
            }
            continue;
        }
        if escaped {
            current.push(match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            in_string = false;
            values.push(current.clone());
        } else {
            current.push(ch);
        }
    }
    values
}

fn parse_toml_field_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
}

fn unescape_basic_toml_string(value: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            out.push(match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_provider_sample_manifest_status_reports_blocked_samples() {
        let records = vec![
            r#"
trace_id = "hetero-trace:shader:metal"
provider_family = "metal:apple-silicon-gpu"
materialization_status = "provider-sample-blocked"
"#,
        ];

        assert_eq!(device_provider_sample_blocked_count(&records), 1);
        assert_eq!(device_provider_sample_pending_count(&records), 0);
        assert_eq!(
            device_provider_sample_manifest_status(
                DEVICE_PROVIDER_SAMPLE_PROTOCOL,
                DEVICE_PROVIDER_SAMPLE_SCHEMA,
                false,
                0,
                0,
                device_provider_sample_blocked_count(&records),
            ),
            "blocked-provider-sample"
        );
    }

    #[test]
    fn provider_bundle_evidence_requires_provider_neutral_provenance() {
        assert_eq!(
            provider_bundle_evidence_status(
                false,
                (
                    PROVIDER_BUNDLE_REGISTRY_CONTRACT,
                    PROVIDER_BUNDLE_MANIFEST_CONTRACT,
                    "fnv1a64:08a971e5a543be2e",
                    3,
                ),
                ("official.shader", "metal.apple-silicon-gpu.bundle.v1"),
                "verified",
            ),
            "verified"
        );
        assert_eq!(
            provider_bundle_evidence_status(
                false,
                (
                    PROVIDER_BUNDLE_REGISTRY_CONTRACT,
                    PROVIDER_BUNDLE_MANIFEST_CONTRACT,
                    "none",
                    0,
                ),
                ("none", "none"),
                "mismatch",
            ),
            "provider-bundle-evidence-invalid"
        );
        assert_eq!(
            provider_bundle_evidence_status(
                true,
                ("none", "none", "none", 0),
                ("none", "none"),
                "not-applicable",
            ),
            "not-applicable"
        );
    }

    #[test]
    fn selected_provider_bundle_set_is_recomputed_from_all_records() {
        let records = vec![
            r#"
provider_family = "coreml:apple-ane"
provider_bundle_package_id = "official.kernel"
provider_bundle_id = "coreml.apple-ane.bundle.v1"
"#,
            r#"
provider_family = "coreml:apple-ane"
provider_bundle_package_id = "official.kernel"
provider_bundle_id = "coreml.apple-ane.bundle.v1"
"#,
            r#"
provider_family = "metal:apple-silicon-gpu"
provider_bundle_package_id = "official.shader"
provider_bundle_id = "metal.apple-silicon-gpu.bundle.v1"
"#,
        ];
        assert_eq!(
            selected_provider_bundle_set_validation_status(
                "",
                &records,
                SELECTED_PROVIDER_BUNDLE_SET_CONTRACT,
                2,
                "fnv1a64:0126ed9d38f1895f",
            ),
            "verified"
        );
    }
}
