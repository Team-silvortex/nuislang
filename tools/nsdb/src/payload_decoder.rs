use std::{fs, path::Path};

const PAYLOAD_DECODER_MANIFEST_FILE_NAME: &str = "nuis.nsdb.payload-decoders.toml";

pub(crate) struct NsdbPayloadDecodeReport {
    pub(crate) decoder_id: String,
    pub(crate) decoder_status: String,
    pub(crate) decoder_detail: String,
    pub(crate) decoder_capability: &'static str,
    pub(crate) decoder_detail_level: &'static str,
    pub(crate) decoder_reads_file_summary: bool,
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
            decoder_capability: "metadata-summary",
            decoder_detail_level: "semantic-metadata",
            decoder_reads_file_summary: false,
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
                decoder_capability: "opaque-file-summary",
                decoder_detail_level: "file-header",
                decoder_reads_file_summary: true,
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
            decoder_capability: "opaque-file-summary",
            decoder_detail_level: "invalid-input",
            decoder_reads_file_summary: false,
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
            decoder_capability: "opaque-file-summary",
            decoder_detail_level: "awaiting-input",
            decoder_reads_file_summary: false,
            decoder_format_probe_status: "format-probe-skipped".to_owned(),
            decoder_format_probe_detail: "payload-missing".to_owned(),
            content_status: "content-awaiting-decoder".to_owned(),
            content_type: payload_format.to_owned(),
            content_summary: format!("opaque payload at {}", resolved_path.display()),
        },
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
    if let Some(spec) = external_decoder_spec(output_dir, payload_format) {
        return spec;
    }
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
            magic: None,
        },
    }
}

fn registered_spec(decoder_id: &'static str, magic: Option<PayloadMagic>) -> PayloadDecoderSpec {
    PayloadDecoderSpec {
        decoder_id: decoder_id.to_owned(),
        decoder_status: "decoder-registered-opaque",
        magic,
    }
}

fn external_decoder_spec(output_dir: &str, payload_format: &str) -> Option<PayloadDecoderSpec> {
    let path = Path::new(output_dir).join(PAYLOAD_DECODER_MANIFEST_FILE_NAME);
    let source = fs::read_to_string(path).ok()?;
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
        magic,
    })
}

fn external_magic(record: &str) -> Option<PayloadMagic> {
    if let Some(hex) = toml_string_value(record, "magic_hex") {
        let bytes = decode_hex_bytes(&hex)?;
        return Some(PayloadMagic {
            label: toml_string_value(record, "magic_label").unwrap_or_else(|| "hex".to_owned()),
            bytes,
        });
    }
    toml_string_value(record, "magic_ascii").map(|bytes| PayloadMagic {
        label: toml_string_value(record, "magic_label").unwrap_or(bytes.clone()),
        bytes: bytes.into_bytes(),
    })
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
