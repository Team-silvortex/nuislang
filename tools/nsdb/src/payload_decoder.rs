use crate::model::{NsdbPayloadDecoderManifestInfo, NsdbPayloadDecoderManifestRecordInfo};
use std::{fs, path::Path};

const PAYLOAD_DECODER_MANIFEST_FILE_NAME: &str = "nuis.nsdb.payload-decoders.toml";
const PAYLOAD_DECODER_MANIFEST_PROTOCOL: &str = "nuis-nsdb-payload-decoders-v1";
const PAYLOAD_DECODER_MANIFEST_SCHEMA: &str = "nsdb-payload-decoder-manifest-v1";

pub(crate) struct NsdbPayloadDecodeReport {
    pub(crate) decoder_id: String,
    pub(crate) decoder_status: String,
    pub(crate) decoder_detail: String,
    pub(crate) decoder_capability: String,
    pub(crate) decoder_detail_level: String,
    pub(crate) decoder_reads_file_summary: bool,
    pub(crate) decoder_manifest_status: String,
    pub(crate) decoder_manifest_detail: String,
    pub(crate) decoder_format_probe_status: String,
    pub(crate) decoder_format_probe_detail: String,
    pub(crate) content_status: String,
    pub(crate) content_type: String,
    pub(crate) content_summary: String,
}

pub(crate) fn decode_payload_content(
    output_dir: &str,
    payload_format: &str,
    payload_ref: &str,
) -> NsdbPayloadDecodeReport {
    if payload_format == "payload-execution-metadata" {
        return NsdbPayloadDecodeReport {
            decoder_id: "nsdb-metadata-summary-decoder-v1".to_owned(),
            decoder_status: "decoder-ready".to_owned(),
            decoder_detail: "metadata-summary".to_owned(),
            decoder_capability: "metadata-summary".to_owned(),
            decoder_detail_level: "semantic-metadata".to_owned(),
            decoder_reads_file_summary: false,
            decoder_manifest_status: "manifest-not-needed".to_owned(),
            decoder_manifest_detail: "metadata-summary".to_owned(),
            decoder_format_probe_status: "format-probe-not-needed".to_owned(),
            decoder_format_probe_detail: "metadata-summary".to_owned(),
            content_status: "content-metadata-summary".to_owned(),
            content_type: payload_format.to_owned(),
            content_summary: payload_ref.to_owned(),
        };
    }

    let spec = decoder_spec_for_payload_format(output_dir, payload_format);
    let payload_path = Path::new(payload_ref);
    let resolved_path = if payload_path.is_absolute() {
        payload_path.to_path_buf()
    } else {
        Path::new(output_dir).join(payload_path)
    };
    match std::fs::metadata(&resolved_path) {
        Ok(metadata) if metadata.is_file() => {
            let probe = probe_payload_format(&spec, &resolved_path);
            NsdbPayloadDecodeReport {
                decoder_id: spec.decoder_id,
                decoder_status: spec.decoder_status.to_owned(),
                decoder_detail: format!("registered-format:{payload_format}"),
                decoder_capability: spec.decoder_capability,
                decoder_detail_level: spec.decoder_detail_level,
                decoder_reads_file_summary: true,
                decoder_manifest_status: spec.decoder_manifest_status,
                decoder_manifest_detail: spec.decoder_manifest_detail,
                decoder_format_probe_status: probe.status,
                decoder_format_probe_detail: probe.detail,
                content_status: "content-opaque-file-summary".to_owned(),
                content_type: payload_format.to_owned(),
                content_summary: format!(
                    "opaque payload file={} bytes={}",
                    resolved_path.display(),
                    metadata.len()
                ),
            }
        }
        Ok(_) => NsdbPayloadDecodeReport {
            decoder_id: spec.decoder_id,
            decoder_status: "decoder-input-not-file".to_owned(),
            decoder_detail: format!("path-not-file:{}", resolved_path.display()),
            decoder_capability: spec.decoder_capability,
            decoder_detail_level: "invalid-input".to_owned(),
            decoder_reads_file_summary: false,
            decoder_manifest_status: spec.decoder_manifest_status,
            decoder_manifest_detail: spec.decoder_manifest_detail,
            decoder_format_probe_status: "format-probe-skipped".to_owned(),
            decoder_format_probe_detail: "path-not-file".to_owned(),
            content_status: "content-opaque-path-not-file".to_owned(),
            content_type: payload_format.to_owned(),
            content_summary: format!(
                "opaque payload path is not a file: {}",
                resolved_path.display()
            ),
        },
        Err(_) => NsdbPayloadDecodeReport {
            decoder_id: spec.decoder_id,
            decoder_status: "decoder-awaiting-input".to_owned(),
            decoder_detail: format!("payload-missing:{}", resolved_path.display()),
            decoder_capability: spec.decoder_capability,
            decoder_detail_level: "awaiting-input".to_owned(),
            decoder_reads_file_summary: false,
            decoder_manifest_status: spec.decoder_manifest_status,
            decoder_manifest_detail: spec.decoder_manifest_detail,
            decoder_format_probe_status: "format-probe-skipped".to_owned(),
            decoder_format_probe_detail: "payload-missing".to_owned(),
            content_status: "content-awaiting-decoder".to_owned(),
            content_type: payload_format.to_owned(),
            content_summary: format!("opaque payload at {}", resolved_path.display()),
        },
    }
}

pub(crate) fn read_payload_decoder_manifest_info(
    output_dir: &Path,
) -> NsdbPayloadDecoderManifestInfo {
    let path = output_dir.join(PAYLOAD_DECODER_MANIFEST_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return NsdbPayloadDecoderManifestInfo {
            available: false,
            path: path.display().to_string(),
            protocol: "none".to_owned(),
            schema: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            valid_record_count: 0,
            invalid_record_count: 0,
            first_payload_format: "none".to_owned(),
            first_decoder_id: "none".to_owned(),
            first_diagnostic: "manifest-not-found".to_owned(),
            records: Vec::new(),
        };
    };
    let protocol = toml_string_value(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let schema = toml_string_value(&source, "schema").unwrap_or_else(|| "none".to_owned());
    let records = source.split("[[decoders]]").skip(1).collect::<Vec<_>>();
    let summaries = records
        .iter()
        .enumerate()
        .map(|(index, record)| payload_decoder_manifest_record_summary(index, record))
        .collect::<Vec<_>>();
    let valid_record_count = summaries.iter().filter(|summary| summary.valid).count();
    let invalid_record_count = summaries.len().saturating_sub(valid_record_count);
    let first = summaries.first();
    NsdbPayloadDecoderManifestInfo {
        available: true,
        path: path.display().to_string(),
        protocol: protocol.clone(),
        schema: schema.clone(),
        status: payload_decoder_manifest_status(
            &protocol,
            &schema,
            summaries.is_empty(),
            invalid_record_count,
        ),
        record_count: summaries.len(),
        valid_record_count,
        invalid_record_count,
        first_payload_format: first
            .map(|summary| summary.payload_format.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_decoder_id: first
            .map(|summary| summary.decoder_id.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_diagnostic: first
            .map(|summary| summary.diagnostic.clone())
            .unwrap_or_else(|| "manifest-empty".to_owned()),
        records: summaries,
    }
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

fn payload_decoder_manifest_record_summary(
    index: usize,
    record: &str,
) -> NsdbPayloadDecoderManifestRecordInfo {
    let payload_format =
        toml_string_value(record, "payload_format").unwrap_or_else(|| "none".to_owned());
    let decoder_id = toml_string_value(record, "decoder_id")
        .unwrap_or_else(|| format!("nsdb-external-{payload_format}-opaque-decoder-v1"));
    let magic = external_magic(record);
    let valid = payload_format != "none"
        && magic.manifest_status != "manifest-external-decoder-invalid-magic";
    NsdbPayloadDecoderManifestRecordInfo {
        index,
        valid,
        payload_format,
        decoder_id,
        diagnostic: magic.manifest_status,
    }
}

struct PayloadFormatProbe {
    status: String,
    detail: String,
}

#[derive(Clone)]
struct PayloadDecoderSpec {
    decoder_id: String,
    decoder_status: &'static str,
    decoder_capability: String,
    decoder_detail_level: String,
    decoder_manifest_status: String,
    decoder_manifest_detail: String,
    magic: Option<PayloadMagic>,
}

#[derive(Clone)]
struct PayloadMagic {
    label: String,
    bytes: Vec<u8>,
}

fn probe_payload_format(spec: &PayloadDecoderSpec, path: &Path) -> PayloadFormatProbe {
    let Ok(bytes) = std::fs::read(path) else {
        return PayloadFormatProbe {
            status: "format-probe-unreadable".to_owned(),
            detail: format!("unreadable:{}", path.display()),
        };
    };
    let Some(magic) = spec.magic.as_ref() else {
        return PayloadFormatProbe {
            status: "format-probe-generic".to_owned(),
            detail: "no-registered-magic".to_owned(),
        };
    };
    if bytes.starts_with(&magic.bytes) {
        PayloadFormatProbe {
            status: "format-probe-matched".to_owned(),
            detail: format!("magic:{}", magic.label),
        }
    } else {
        PayloadFormatProbe {
            status: "format-probe-mismatch".to_owned(),
            detail: format!("expected-magic:{}", magic.label),
        }
    }
}

fn decoder_spec_for_payload_format(output_dir: &str, payload_format: &str) -> PayloadDecoderSpec {
    let manifest_path = Path::new(output_dir).join(PAYLOAD_DECODER_MANIFEST_FILE_NAME);
    match fs::read_to_string(&manifest_path) {
        Ok(source) => {
            if let Some(spec) = external_decoder_spec(&source, payload_format) {
                return spec;
            }
            let mut spec = built_in_decoder_spec(payload_format);
            spec.decoder_manifest_status = "manifest-loaded-no-match".to_owned();
            spec.decoder_manifest_detail = manifest_path.display().to_string();
            spec
        }
        Err(_) => {
            let mut spec = built_in_decoder_spec(payload_format);
            spec.decoder_manifest_status = "manifest-not-found".to_owned();
            spec.decoder_manifest_detail = manifest_path.display().to_string();
            spec
        }
    }
}

fn built_in_decoder_spec(payload_format: &str) -> PayloadDecoderSpec {
    match payload_format {
        "metallib" => registered_spec(
            "nsdb-metallib-opaque-decoder-v1",
            Some(PayloadMagic {
                label: "MTLB".to_owned(),
                bytes: b"MTLB".to_vec(),
            }),
        ),
        "spirv" | "spv" => registered_spec(
            "nsdb-spirv-opaque-decoder-v1",
            Some(PayloadMagic {
                label: "SPIR-V".to_owned(),
                bytes: vec![0x03, 0x02, 0x23, 0x07],
            }),
        ),
        "coreml" | "mlmodelc" => registered_spec(
            "nsdb-coreml-opaque-decoder-v1",
            Some(PayloadMagic {
                label: "coreml".to_owned(),
                bytes: b"coreml".to_vec(),
            }),
        ),
        _ => PayloadDecoderSpec {
            decoder_id: "nsdb-generic-opaque-payload-decoder-v1".to_owned(),
            decoder_status: "decoder-generic-opaque",
            decoder_capability: "opaque-file-summary".to_owned(),
            decoder_detail_level: "file-header".to_owned(),
            decoder_manifest_status: "manifest-builtin-default".to_owned(),
            decoder_manifest_detail: "generic-opaque-payload-decoder".to_owned(),
            magic: None,
        },
    }
}

fn registered_spec(decoder_id: &'static str, magic: Option<PayloadMagic>) -> PayloadDecoderSpec {
    PayloadDecoderSpec {
        decoder_id: decoder_id.to_owned(),
        decoder_status: "decoder-registered-opaque",
        decoder_capability: "opaque-file-summary".to_owned(),
        decoder_detail_level: "file-header".to_owned(),
        decoder_manifest_status: "manifest-builtin-default".to_owned(),
        decoder_manifest_detail: "built-in-decoder-spec".to_owned(),
        magic,
    }
}

fn external_decoder_spec(source: &str, payload_format: &str) -> Option<PayloadDecoderSpec> {
    source
        .split("[[decoders]]")
        .skip(1)
        .find_map(|record| external_decoder_spec_from_record(record, payload_format))
}

fn external_decoder_spec_from_record(
    record: &str,
    payload_format: &str,
) -> Option<PayloadDecoderSpec> {
    let registered_format = toml_string_value(record, "payload_format")?;
    if registered_format != payload_format {
        return None;
    }
    let decoder_id = toml_string_value(record, "decoder_id")
        .unwrap_or_else(|| format!("nsdb-external-{payload_format}-opaque-decoder-v1"));
    let magic = external_magic(record);
    Some(PayloadDecoderSpec {
        decoder_id,
        decoder_status: "decoder-registered-external-opaque",
        decoder_capability: toml_string_value(record, "decoder_capability")
            .unwrap_or_else(|| "opaque-file-summary".to_owned()),
        decoder_detail_level: toml_string_value(record, "decoder_detail_level")
            .unwrap_or_else(|| "file-header".to_owned()),
        decoder_manifest_status: magic.manifest_status,
        decoder_manifest_detail: magic.manifest_detail,
        magic: magic.magic,
    })
}

struct ExternalMagic {
    magic: Option<PayloadMagic>,
    manifest_status: String,
    manifest_detail: String,
}

fn external_magic(record: &str) -> ExternalMagic {
    if let Some(hex) = toml_string_value(record, "magic_hex") {
        if let Some(bytes) = decode_hex_bytes(&hex) {
            return ExternalMagic {
                magic: Some(PayloadMagic {
                    label: toml_string_value(record, "magic_label")
                        .unwrap_or_else(|| "hex".to_owned()),
                    bytes,
                }),
                manifest_status: "manifest-external-decoder-loaded".to_owned(),
                manifest_detail: "external-magic-hex".to_owned(),
            };
        }
        return ExternalMagic {
            magic: None,
            manifest_status: "manifest-external-decoder-invalid-magic".to_owned(),
            manifest_detail: "invalid-magic-hex".to_owned(),
        };
    }
    if let Some(bytes) = toml_string_value(record, "magic_ascii") {
        return ExternalMagic {
            magic: Some(PayloadMagic {
                label: toml_string_value(record, "magic_label").unwrap_or(bytes.clone()),
                bytes: bytes.into_bytes(),
            }),
            manifest_status: "manifest-external-decoder-loaded".to_owned(),
            manifest_detail: "external-magic-ascii".to_owned(),
        };
    }
    ExternalMagic {
        magic: None,
        manifest_status: "manifest-external-decoder-loaded".to_owned(),
        manifest_detail: "external-no-magic".to_owned(),
    }
}

fn decode_hex_bytes(value: &str) -> Option<Vec<u8>> {
    let digits = value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '_')
        .collect::<String>();
    if digits.is_empty() || digits.len() % 2 != 0 {
        return None;
    }
    digits
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let high = hex_nibble(chunk[0])?;
            let low = hex_nibble(chunk[1])?;
            Some((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn toml_string_value(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    let value = source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))?;
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_basic_toml_string)
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
