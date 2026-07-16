use crate::{
    artifact_launch_evidence::RunArtifactLaunchEvidence, json_bool_field, json_field,
    json_optional_string_field, json_usize_field,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const NSDB_HANDOFF_PROTOCOL: &str = "nuis-nsdb-payload-execution-handoff-v1";
const NSDB_HANDOFF_FILE_NAME: &str = "nuis.nsdb.payload-execution-handoff.toml";

pub(crate) struct LaunchEvidenceNsdbHandoffPersistence {
    persisted: bool,
    path: Option<PathBuf>,
    record_count: usize,
    ready_record_count: usize,
    first_trace_id: Option<String>,
    error: Option<String>,
}

pub(crate) struct PersistedNsdbHandoffSummary {
    available: bool,
    path: PathBuf,
    protocol: Option<String>,
    debugger_contract: Option<String>,
    record_count: usize,
    ready_record_count: usize,
    first_trace_id: Option<String>,
    first_status: Option<String>,
    first_next_action: Option<String>,
    error: Option<String>,
}

impl PersistedNsdbHandoffSummary {
    pub(crate) fn available(&self) -> bool {
        self.available
    }

    pub(crate) fn record_count(&self) -> usize {
        self.record_count
    }

    pub(crate) fn ready_record_count(&self) -> usize {
        self.ready_record_count
    }

    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        vec![
            json_bool_field(&format!("{prefix}_available"), self.available),
            json_optional_string_field(&format!("{prefix}_protocol"), self.protocol.as_deref()),
            json_optional_string_field(
                &format!("{prefix}_debugger_contract"),
                self.debugger_contract.as_deref(),
            ),
            json_field(&format!("{prefix}_path"), &self.path.display().to_string()),
            json_usize_field(&format!("{prefix}_record_count"), self.record_count),
            json_usize_field(
                &format!("{prefix}_ready_record_count"),
                self.ready_record_count,
            ),
            json_optional_string_field(
                &format!("{prefix}_first_trace_id"),
                self.first_trace_id.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_status"),
                self.first_status.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_next_action"),
                self.first_next_action.as_deref(),
            ),
            json_optional_string_field(&format!("{prefix}_error"), self.error.as_deref()),
        ]
    }
}

pub(crate) fn read_persisted_nsdb_handoff(
    output_dir: Option<&Path>,
) -> PersistedNsdbHandoffSummary {
    let Some(output_dir) = output_dir else {
        return PersistedNsdbHandoffSummary {
            available: false,
            path: PathBuf::from(NSDB_HANDOFF_FILE_NAME),
            protocol: None,
            debugger_contract: None,
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: None,
            first_status: None,
            first_next_action: None,
            error: Some("output_dir-unavailable".to_owned()),
        };
    };
    let path = output_dir.join(NSDB_HANDOFF_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return PersistedNsdbHandoffSummary {
            available: false,
            path,
            protocol: None,
            debugger_contract: None,
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: None,
            first_status: None,
            first_next_action: None,
            error: Some("handoff-metadata-missing".to_owned()),
        };
    };
    PersistedNsdbHandoffSummary {
        available: true,
        path,
        protocol: parse_string_toml_field(&source, "protocol"),
        debugger_contract: parse_string_toml_field(&source, "debugger_contract"),
        record_count: parse_usize_toml_field(&source, "record_count").unwrap_or(0),
        ready_record_count: parse_usize_toml_field(&source, "ready_record_count").unwrap_or(0),
        first_trace_id: parse_string_toml_field(&source, "first_trace_id"),
        first_status: parse_string_toml_field(&source, "first_status"),
        first_next_action: parse_string_toml_field(&source, "first_next_action"),
        error: None,
    }
}

impl LaunchEvidenceNsdbHandoffPersistence {
    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_field(
                "launch_evidence_nsdb_handoff_protocol",
                NSDB_HANDOFF_PROTOCOL,
            ),
            json_bool_field("launch_evidence_nsdb_handoff_persisted", self.persisted),
            json_optional_string_field(
                "launch_evidence_nsdb_handoff_path",
                self.path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_usize_field(
                "launch_evidence_nsdb_handoff_record_count",
                self.record_count,
            ),
            json_usize_field(
                "launch_evidence_nsdb_handoff_ready_record_count",
                self.ready_record_count,
            ),
            json_optional_string_field(
                "launch_evidence_nsdb_handoff_first_trace_id",
                self.first_trace_id.as_deref(),
            ),
            json_optional_string_field("launch_evidence_nsdb_handoff_error", self.error.as_deref()),
        ]
    }

    pub(crate) fn print_text(&self) {
        println!("  launch_evidence_nsdb_handoff_protocol: {NSDB_HANDOFF_PROTOCOL}");
        println!(
            "  launch_evidence_nsdb_handoff_persisted: {}",
            self.persisted
        );
        println!(
            "  launch_evidence_nsdb_handoff_path: {}",
            self.path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<none>".to_owned())
        );
        println!(
            "  launch_evidence_nsdb_handoff_record_count: {}",
            self.record_count
        );
        println!(
            "  launch_evidence_nsdb_handoff_ready_record_count: {}",
            self.ready_record_count
        );
        println!(
            "  launch_evidence_nsdb_handoff_first_trace_id: {}",
            self.first_trace_id.as_deref().unwrap_or("<none>")
        );
        println!(
            "  launch_evidence_nsdb_handoff_error: {}",
            self.error.as_deref().unwrap_or("<none>")
        );
    }
}

pub(crate) fn persist_launch_evidence_nsdb_handoff(
    output_dir: Option<&Path>,
    evidence: &RunArtifactLaunchEvidence,
) -> LaunchEvidenceNsdbHandoffPersistence {
    let records = evidence.payload_execution_trace_records();
    let ready_record_count = records
        .iter()
        .filter(|record| record.status == "ready")
        .count();
    let first_trace_id = records.first().map(|record| record.trace_id.clone());

    let Some(output_dir) = output_dir else {
        return LaunchEvidenceNsdbHandoffPersistence {
            persisted: false,
            path: None,
            record_count: records.len(),
            ready_record_count,
            first_trace_id,
            error: Some("output_dir-unavailable".to_owned()),
        };
    };
    if records.is_empty() {
        return LaunchEvidenceNsdbHandoffPersistence {
            persisted: false,
            path: Some(output_dir.join(NSDB_HANDOFF_FILE_NAME)),
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: None,
            error: Some("payload-execution-trace-unavailable".to_owned()),
        };
    }

    let path = output_dir.join(NSDB_HANDOFF_FILE_NAME);
    let content = render_launch_evidence_nsdb_handoff(evidence);
    match fs::write(&path, content) {
        Ok(()) => LaunchEvidenceNsdbHandoffPersistence {
            persisted: true,
            path: Some(path),
            record_count: records.len(),
            ready_record_count,
            first_trace_id,
            error: None,
        },
        Err(error) => LaunchEvidenceNsdbHandoffPersistence {
            persisted: false,
            path: Some(path),
            record_count: records.len(),
            ready_record_count,
            first_trace_id,
            error: Some(error.to_string()),
        },
    }
}

fn render_launch_evidence_nsdb_handoff(evidence: &RunArtifactLaunchEvidence) -> String {
    let records = evidence.payload_execution_trace_records();
    let ready_record_count = records
        .iter()
        .filter(|record| record.status == "ready")
        .count();
    let mut out = String::new();
    push_toml_string(&mut out, "protocol", NSDB_HANDOFF_PROTOCOL);
    push_toml_string(
        &mut out,
        "debugger_contract",
        evidence.payload_execution_trace_protocol(),
    );
    push_toml_string(&mut out, "source", "run-artifact-launch-evidence");
    out.push_str(&format!("record_count = {}\n", records.len()));
    out.push_str(&format!("ready_record_count = {ready_record_count}\n"));
    if let Some(first) = records.first() {
        push_toml_string(&mut out, "first_trace_id", &first.trace_id);
        push_toml_string(&mut out, "first_status", &first.status);
        push_toml_string(&mut out, "first_next_action", &first.next_action);
    }
    for record in records {
        out.push_str("\n[[records]]\n");
        push_toml_string(&mut out, "trace_id", &record.trace_id);
        push_toml_string(&mut out, "status", &record.status);
        push_toml_string(&mut out, "execution_phase", &record.execution_phase);
        push_toml_optional_string(&mut out, "target", record.target.as_deref());
        push_toml_optional_string(&mut out, "entry_symbol", record.entry_symbol.as_deref());
        push_toml_optional_string(&mut out, "entry_kind", record.entry_kind.as_deref());
        push_toml_optional_string(
            &mut out,
            "entry_section_id",
            record.entry_section_id.as_deref(),
        );
        push_toml_optional_string(&mut out, "first_blocker", record.first_blocker.as_deref());
        push_toml_string(&mut out, "next_action", &record.next_action);
    }
    out
}

fn push_toml_optional_string(out: &mut String, key: &str, value: Option<&str>) {
    match value {
        Some(value) => push_toml_string(out, key, value),
        None => out.push_str(&format!("{key} = \"\"\n")),
    }
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&toml_escape(value));
    out.push_str("\"\n");
}

fn toml_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
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
