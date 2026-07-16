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
            parse_usize_toml_field(&source, "pending_record_count").unwrap_or(0),
        ),
        record_count: records.len(),
        pending_record_count: parse_usize_toml_field(&source, "pending_record_count").unwrap_or(0),
        invalid_record_count,
        first_provider_family: records
            .first()
            .and_then(|record| parse_string_toml_field(record, "provider_family"))
            .unwrap_or_else(|| "none".to_owned()),
        first_materialization_status: records
            .first()
            .and_then(|record| parse_string_toml_field(record, "materialization_status"))
            .unwrap_or_else(|| "none".to_owned()),
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
    pub(crate) invalid_record_count: usize,
    pub(crate) first_provider_family: String,
    pub(crate) first_materialization_status: String,
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
            invalid_record_count: 0,
            first_provider_family: "none".to_owned(),
            first_materialization_status: "none".to_owned(),
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
