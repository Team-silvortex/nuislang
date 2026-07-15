use crate::model::{NsdbPayloadExecutionEvent, NsdbPayloadExecutionHandoffInfo};
use std::{fs, path::Path};

const HANDOFF_FILE_NAME: &str = "nuis.nsdb.payload-execution-handoff.toml";

pub(crate) fn read_payload_execution_handoff(output_dir: &Path) -> NsdbPayloadExecutionHandoffInfo {
    let path = output_dir.join(HANDOFF_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return NsdbPayloadExecutionHandoffInfo {
            available: false,
            path: path.display().to_string(),
            protocol: "none".to_owned(),
            debugger_contract: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: "none".to_owned(),
            first_status: "none".to_owned(),
            first_next_action: "none".to_owned(),
            first_entry_symbol: "none".to_owned(),
            first_execution_phase: "none".to_owned(),
            events: Vec::new(),
        };
    };
    let protocol =
        parse_string_toml_field(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let debugger_contract =
        parse_string_toml_field(&source, "debugger_contract").unwrap_or_else(|| "none".to_owned());
    let record_count = parse_usize_toml_field(&source, "record_count").unwrap_or(0);
    let ready_record_count = parse_usize_toml_field(&source, "ready_record_count").unwrap_or(0);
    let events = parse_payload_execution_events(&source);
    let first_event = events.first();
    let first_status = parse_string_toml_field(&source, "first_status")
        .or_else(|| first_event.map(|event| event.status.clone()))
        .unwrap_or_else(|| "none".to_owned());
    let status = payload_handoff_status(&protocol, &debugger_contract, record_count, &first_status);
    NsdbPayloadExecutionHandoffInfo {
        available: true,
        path: path.display().to_string(),
        protocol,
        debugger_contract,
        status,
        record_count,
        ready_record_count,
        first_trace_id: parse_string_toml_field(&source, "first_trace_id")
            .or_else(|| first_event.map(|event| event.trace_id.clone()))
            .unwrap_or_else(|| "none".to_owned()),
        first_status,
        first_next_action: parse_string_toml_field(&source, "first_next_action")
            .or_else(|| first_event.map(|event| event.next_action.clone()))
            .unwrap_or_else(|| "none".to_owned()),
        first_entry_symbol: first_event
            .map(|event| event.entry_symbol.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_execution_phase: first_event
            .map(|event| event.execution_phase.clone())
            .unwrap_or_else(|| "none".to_owned()),
        events,
    }
}

fn payload_handoff_status(
    protocol: &str,
    debugger_contract: &str,
    record_count: usize,
    first_status: &str,
) -> String {
    if protocol != "nuis-nsdb-payload-execution-handoff-v1" {
        return "unsupported-protocol".to_owned();
    }
    if debugger_contract != "nsdb-yir-payload-execution-trace-v1" {
        return "unsupported-debugger-contract".to_owned();
    }
    if record_count == 0 {
        return "empty".to_owned();
    }
    if first_status == "ready" {
        "ready".to_owned()
    } else {
        "blocked".to_owned()
    }
}

fn parse_payload_execution_events(source: &str) -> Vec<NsdbPayloadExecutionEvent> {
    source
        .split("[[records]]")
        .skip(1)
        .enumerate()
        .map(|(index, record)| NsdbPayloadExecutionEvent {
            index,
            trace_id: parse_string_toml_field(record, "trace_id")
                .unwrap_or_else(|| "none".to_owned()),
            status: parse_string_toml_field(record, "status").unwrap_or_else(|| "none".to_owned()),
            execution_phase: parse_string_toml_field(record, "execution_phase")
                .unwrap_or_else(|| "none".to_owned()),
            target: parse_string_toml_field(record, "target").unwrap_or_else(|| "none".to_owned()),
            entry_symbol: parse_string_toml_field(record, "entry_symbol")
                .unwrap_or_else(|| "none".to_owned()),
            entry_kind: parse_string_toml_field(record, "entry_kind")
                .unwrap_or_else(|| "none".to_owned()),
            entry_section_id: parse_string_toml_field(record, "entry_section_id")
                .unwrap_or_else(|| "none".to_owned()),
            first_blocker: parse_string_toml_field(record, "first_blocker")
                .filter(|value| !value.is_empty())
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
    use super::read_payload_execution_handoff;
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn reads_ready_payload_execution_handoff() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir: PathBuf = env::temp_dir().join(format!("nsdb-handoff-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("nuis.nsdb.payload-execution-handoff.toml"),
            r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
record_count = 2
ready_record_count = 1
first_trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
first_status = "ready"
first_next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
status = "ready"
execution_phase = "container-loader-handoff"
entry_symbol = "nuis.bootstrap.lifecycle.v1"
next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:shader:pixelmagic.blur"
status = "blocked"
execution_phase = "device-dispatch"
target = "shader"
entry_symbol = "pixelmagic.blur"
entry_kind = "shader-kernel"
entry_section_id = "sec0002.shader"
first_blocker = "device-execution-sample-missing"
next_action = "materialize-device-execution-trace"
"#,
        )
        .unwrap();

        let handoff = read_payload_execution_handoff(&dir);

        assert!(handoff.available);
        assert_eq!(handoff.status, "ready");
        assert_eq!(handoff.record_count, 2);
        assert_eq!(handoff.events.len(), 2);
        assert_eq!(handoff.events[0].index, 0);
        assert_eq!(handoff.events[0].trace_id, handoff.first_trace_id);
        assert_eq!(
            handoff.events[0].next_action,
            "handoff-payload-trace-to-nsdb"
        );
        assert_eq!(handoff.events[1].index, 1);
        assert_eq!(handoff.events[1].status, "blocked");
        assert_eq!(
            handoff.events[1].first_blocker,
            "device-execution-sample-missing"
        );
        assert_eq!(handoff.first_execution_phase, "container-loader-handoff");
        assert_eq!(handoff.first_entry_symbol, "nuis.bootstrap.lifecycle.v1");
    }
}
