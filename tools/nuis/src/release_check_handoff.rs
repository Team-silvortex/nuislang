use std::path::{Path, PathBuf};

use crate::{json_bool_field, json_field, json_optional_string_field, json_usize_field};

const HETERO_RUNTIME_TRACE_FILE_NAME: &str = "nuis.nsdb.hetero-runtime-trace.toml";

pub(crate) struct DeviceSampleHandoffMirror {
    pub(crate) available: bool,
    pub(crate) path: Option<PathBuf>,
    pub(crate) protocol: String,
    pub(crate) status: String,
    pub(crate) record_count: usize,
    pub(crate) pending_validation_count: usize,
    pub(crate) first_provider_family: String,
    pub(crate) first_handoff_target: String,
    pub(crate) first_validation_status: String,
    pub(crate) first_input_evidence: String,
    pub(crate) first_next_action: String,
}

impl DeviceSampleHandoffMirror {
    fn unavailable(path: PathBuf) -> Self {
        Self {
            available: false,
            path: Some(path),
            protocol: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            pending_validation_count: 0,
            first_provider_family: "none".to_owned(),
            first_handoff_target: "none".to_owned(),
            first_validation_status: "none".to_owned(),
            first_input_evidence: "none".to_owned(),
            first_next_action: "run-artifact-json".to_owned(),
        }
    }

    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        vec![
            json_bool_field(&format!("{prefix}_available"), self.available),
            json_optional_string_field(
                &format!("{prefix}_path"),
                self.path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_field(&format!("{prefix}_protocol"), &self.protocol),
            json_field(&format!("{prefix}_status"), &self.status),
            json_usize_field(&format!("{prefix}_record_count"), self.record_count),
            json_usize_field(
                &format!("{prefix}_pending_validation_count"),
                self.pending_validation_count,
            ),
            json_field(
                &format!("{prefix}_first_provider_family"),
                &self.first_provider_family,
            ),
            json_field(
                &format!("{prefix}_first_handoff_target"),
                &self.first_handoff_target,
            ),
            json_field(
                &format!("{prefix}_first_validation_status"),
                &self.first_validation_status,
            ),
            json_field(
                &format!("{prefix}_first_input_evidence"),
                &self.first_input_evidence,
            ),
            json_field(
                &format!("{prefix}_first_next_action"),
                &self.first_next_action,
            ),
        ]
    }
}

pub(crate) fn collect_device_sample_handoff_mirror(output_dir: &Path) -> DeviceSampleHandoffMirror {
    let path = output_dir.join(HETERO_RUNTIME_TRACE_FILE_NAME);
    let Ok(source) = std::fs::read_to_string(&path) else {
        return DeviceSampleHandoffMirror::unavailable(path);
    };
    let first_handoff = source.split("[[device_sample_handoffs]]").nth(1);
    let protocol = parse_string_toml_field(&source, "device_sample_handoff_protocol")
        .or_else(|| first_handoff.and_then(|record| parse_string_toml_field(record, "protocol")))
        .unwrap_or_else(|| "none".to_owned());
    let record_count = parse_usize_toml_field(&source, "device_sample_handoff_record_count")
        .unwrap_or_else(|| source.split("[[device_sample_handoffs]]").skip(1).count());
    let pending_validation_count =
        parse_usize_toml_field(&source, "device_sample_pending_validation_count").unwrap_or(0);
    let status = parse_string_toml_field(&source, "device_sample_handoff_status")
        .unwrap_or_else(|| device_sample_handoff_status(record_count, pending_validation_count));
    DeviceSampleHandoffMirror {
        available: true,
        path: Some(path),
        protocol,
        status,
        record_count,
        pending_validation_count,
        first_provider_family: parse_string_toml_field(
            &source,
            "device_sample_first_pending_provider_family",
        )
        .or_else(|| {
            first_handoff.and_then(|record| parse_string_toml_field(record, "provider_family"))
        })
        .unwrap_or_else(|| "none".to_owned()),
        first_handoff_target: first_handoff
            .and_then(|record| parse_string_toml_field(record, "handoff_target"))
            .unwrap_or_else(|| "none".to_owned()),
        first_validation_status: first_handoff
            .and_then(|record| parse_string_toml_field(record, "validation_status"))
            .unwrap_or_else(|| "none".to_owned()),
        first_input_evidence: first_handoff
            .and_then(|record| parse_string_toml_field(record, "input_evidence"))
            .unwrap_or_else(|| "none".to_owned()),
        first_next_action: first_handoff
            .and_then(|record| parse_string_toml_field(record, "next_action"))
            .unwrap_or_else(|| "none".to_owned()),
    }
}

fn device_sample_handoff_status(record_count: usize, pending_validation_count: usize) -> String {
    if pending_validation_count > 0 {
        "provider-handoff-pending".to_owned()
    } else if record_count > 0 {
        "provider-handoff-ready".to_owned()
    } else {
        "no-provider-handoff".to_owned()
    }
}

fn parse_usize_toml_field(source: &str, key: &str) -> Option<usize> {
    parse_toml_field_value(source, key)?.parse().ok()
}

fn parse_string_toml_field(source: &str, key: &str) -> Option<String> {
    parse_toml_field_value(source, key)?
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_basic_toml_string)
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
    use super::collect_device_sample_handoff_mirror;
    use std::{env, fs};

    #[test]
    fn mirrors_device_sample_handoff_queue() {
        let dir =
            env::temp_dir().join(format!("nuis-release-check-handoff-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp output dir");
        fs::write(
            dir.join("nuis.nsdb.hetero-runtime-trace.toml"),
            r#"device_sample_handoff_record_count = 1
device_sample_pending_validation_count = 1
device_sample_handoff_protocol = "nuis-device-sample-provider-handoff-v1"
device_sample_handoff_status = "provider-handoff-pending"
device_sample_first_pending_provider_family = "metal:apple-silicon-gpu"

[[device_sample_handoffs]]
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
validation_status = "pending-provider-execution"
input_evidence = "ndpb-v2:pixelmagic.ndpb"
next_action = "materialize-device-execution-sample"
"#,
        )
        .expect("write hetero trace");

        let mirror = collect_device_sample_handoff_mirror(&dir);
        assert!(mirror.available);
        assert_eq!(mirror.record_count, 1);
        assert_eq!(mirror.pending_validation_count, 1);
        assert_eq!(mirror.status, "provider-handoff-pending");
        assert_eq!(mirror.first_provider_family, "metal:apple-silicon-gpu");
        assert_eq!(mirror.first_handoff_target, "metal:apple-silicon-gpu");
        assert_eq!(mirror.first_validation_status, "pending-provider-execution");
        assert_eq!(mirror.first_input_evidence, "ndpb-v2:pixelmagic.ndpb");
        assert_eq!(
            mirror.first_next_action,
            "materialize-device-execution-sample"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
