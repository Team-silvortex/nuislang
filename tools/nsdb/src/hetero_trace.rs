use crate::model::{
    NsdbDeviceSampleHandoffRecord, NsdbHeteroRuntimeTraceInfo, NsdbHeteroRuntimeTraceRecord,
};
use std::{fs, path::Path};

const HETERO_RUNTIME_TRACE_FILE_NAME: &str = "nuis.nsdb.hetero-runtime-trace.toml";

pub(crate) fn read_hetero_runtime_trace(output_dir: &Path) -> NsdbHeteroRuntimeTraceInfo {
    let path = output_dir.join(HETERO_RUNTIME_TRACE_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return NsdbHeteroRuntimeTraceInfo {
            available: false,
            path: path.display().to_string(),
            protocol: "none".to_owned(),
            debugger_contract: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            ready_record_count: 0,
            backend_execution_record_count: 0,
            device_sample_handoff_record_count: 0,
            device_sample_handoff_protocol: "none".to_owned(),
            first_trace_id: "none".to_owned(),
            first_blocker: "none".to_owned(),
            next_action: "none".to_owned(),
            records: Vec::new(),
            device_sample_handoffs: Vec::new(),
        };
    };
    let protocol =
        parse_string_toml_field(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let debugger_contract =
        parse_string_toml_field(&source, "debugger_contract").unwrap_or_else(|| "none".to_owned());
    let records = parse_hetero_runtime_trace_records(&source);
    let device_sample_handoffs = parse_device_sample_handoffs(&source);
    NsdbHeteroRuntimeTraceInfo {
        available: true,
        path: path.display().to_string(),
        protocol,
        debugger_contract,
        status: parse_string_toml_field(&source, "status").unwrap_or_else(|| "none".to_owned()),
        record_count: parse_usize_toml_field(&source, "record_count").unwrap_or(records.len()),
        ready_record_count: parse_usize_toml_field(&source, "ready_record_count").unwrap_or(0),
        backend_execution_record_count: parse_usize_toml_field(
            &source,
            "backend_execution_record_count",
        )
        .unwrap_or(0),
        device_sample_handoff_record_count: parse_usize_toml_field(
            &source,
            "device_sample_handoff_record_count",
        )
        .unwrap_or(device_sample_handoffs.len()),
        device_sample_handoff_protocol: parse_string_toml_field(
            &source,
            "device_sample_handoff_protocol",
        )
        .unwrap_or_else(|| "none".to_owned()),
        first_trace_id: parse_string_toml_field(&source, "first_trace_id")
            .or_else(|| records.first().map(|record| record.trace_id.clone()))
            .unwrap_or_else(|| "none".to_owned()),
        first_blocker: parse_string_toml_field(&source, "first_blocker")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "none".to_owned()),
        next_action: parse_string_toml_field(&source, "next_action")
            .unwrap_or_else(|| "none".to_owned()),
        records,
        device_sample_handoffs,
    }
}

fn parse_hetero_runtime_trace_records(source: &str) -> Vec<NsdbHeteroRuntimeTraceRecord> {
    source
        .split("[[records]]")
        .skip(1)
        .enumerate()
        .map(|(index, record)| NsdbHeteroRuntimeTraceRecord {
            index,
            trace_id: parse_string_toml_field(record, "trace_id")
                .unwrap_or_else(|| "none".to_owned()),
            trace_role: parse_string_toml_field(record, "trace_role")
                .unwrap_or_else(|| "none".to_owned()),
            status: parse_string_toml_field(record, "status").unwrap_or_else(|| "none".to_owned()),
            domain_family: parse_string_toml_field(record, "domain_family")
                .unwrap_or_else(|| "none".to_owned()),
            backend_family: parse_string_toml_field(record, "backend_family")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "none".to_owned()),
            target_device: parse_string_toml_field(record, "target_device")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "none".to_owned()),
            backend_artifact_key: parse_string_toml_field(record, "backend_artifact_key")
                .unwrap_or_else(|| "none".to_owned()),
            selected_lowering_target: parse_string_toml_field(record, "selected_lowering_target")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "none".to_owned()),
            payload_format: parse_string_toml_field(record, "payload_format")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "none".to_owned()),
            payload_path: parse_string_toml_field(record, "payload_path")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "none".to_owned()),
            bridge_stub_path: parse_string_toml_field(record, "bridge_stub_path")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "none".to_owned()),
            missing_signals: parse_string_array_toml_field(record, "missing_signals"),
            next_action: parse_string_toml_field(record, "next_action")
                .unwrap_or_else(|| "none".to_owned()),
        })
        .collect()
}

fn parse_device_sample_handoffs(source: &str) -> Vec<NsdbDeviceSampleHandoffRecord> {
    source
        .split("[[device_sample_handoffs]]")
        .skip(1)
        .enumerate()
        .map(|(index, record)| NsdbDeviceSampleHandoffRecord {
            index,
            trace_id: parse_string_toml_field(record, "trace_id")
                .unwrap_or_else(|| "none".to_owned()),
            protocol: parse_string_toml_field(record, "protocol")
                .unwrap_or_else(|| "none".to_owned()),
            provider: parse_string_toml_field(record, "provider")
                .unwrap_or_else(|| "none".to_owned()),
            provider_family: parse_string_toml_field(record, "provider_family")
                .unwrap_or_else(|| "none".to_owned()),
            handoff_target: parse_string_toml_field(record, "handoff_target")
                .unwrap_or_else(|| "none".to_owned()),
            handoff_status: parse_string_toml_field(record, "handoff_status")
                .unwrap_or_else(|| "none".to_owned()),
            validation_status: parse_string_toml_field(record, "validation_status")
                .unwrap_or_else(|| "none".to_owned()),
            input_evidence: parse_string_toml_field(record, "input_evidence")
                .unwrap_or_else(|| "none".to_owned()),
            output_evidence: parse_string_toml_field(record, "output_evidence")
                .unwrap_or_else(|| "none".to_owned()),
            next_action: parse_string_toml_field(record, "next_action")
                .unwrap_or_else(|| "none".to_owned()),
        })
        .collect()
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
    let Some(body) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    body.split(',')
        .filter_map(|part| {
            part.trim()
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .map(unescape_basic_toml_string)
        })
        .collect()
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
    use super::read_hetero_runtime_trace;
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn reads_persisted_hetero_runtime_trace_records() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir: PathBuf = env::temp_dir().join(format!("nsdb-hetero-trace-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("nuis.nsdb.hetero-runtime-trace.toml"),
            r#"
protocol = "nuis-nsdb-hetero-runtime-trace-v1"
debugger_contract = "nsdb-yir-hetero-runtime-trace-v1"
source = "run-artifact-hetero-runtime-trace"
status = "execution-pending"
record_count = 1
ready_record_count = 0
backend_execution_record_count = 1
device_sample_handoff_record_count = 1
device_sample_handoff_protocol = "nuis-device-sample-provider-handoff-v1"
first_trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
first_blocker = ""
next_action = "materialize-device-execution-trace"

[[device_sample_handoffs]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
protocol = "nuis-device-sample-provider-handoff-v1"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
handoff_status = "awaiting-provider-handoff"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
next_action = "materialize-device-execution-sample"

[[records]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
trace_role = "backend-artifact"
status = "execution-pending"
domain_family = "shader"
backend_family = "metal"
target_device = "apple-silicon-gpu"
backend_artifact_key = "shader:metal:apple-silicon-gpu"
selected_lowering_target = "metal"
payload_format = "metallib"
payload_path = "pixelmagic.metallib"
bridge_stub_path = "pixelmagic.bridge"
missing_signals = []
next_action = "materialize-device-execution-trace"
"#,
        )
        .unwrap();

        let trace = read_hetero_runtime_trace(&dir);

        assert!(trace.available);
        assert_eq!(trace.protocol, "nuis-nsdb-hetero-runtime-trace-v1");
        assert_eq!(trace.debugger_contract, "nsdb-yir-hetero-runtime-trace-v1");
        assert_eq!(trace.record_count, 1);
        assert_eq!(trace.backend_execution_record_count, 1);
        assert_eq!(trace.device_sample_handoff_record_count, 1);
        assert_eq!(
            trace.device_sample_handoff_protocol,
            "nuis-device-sample-provider-handoff-v1"
        );
        assert_eq!(trace.device_sample_handoffs.len(), 1);
        assert_eq!(
            trace.device_sample_handoffs[0].provider_family,
            "metal:apple-silicon-gpu"
        );
        assert_eq!(
            trace.device_sample_handoffs[0].handoff_status,
            "awaiting-provider-handoff"
        );
        assert_eq!(trace.records.len(), 1);
        assert_eq!(
            trace.records[0].trace_id,
            "hetero-trace:shader:metal:apple-silicon-gpu"
        );
        assert_eq!(trace.records[0].domain_family, "shader");
        assert_eq!(trace.records[0].backend_family, "metal");
        assert_eq!(trace.records[0].missing_signals, Vec::<String>::new());

        fs::remove_dir_all(dir).unwrap();
    }
}
