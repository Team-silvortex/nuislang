use crate::{
    json_bool_field, json_field, json_optional_bool_field, json_optional_string_field,
    json_usize_field,
};
use std::path::PathBuf;

pub(crate) struct HostRunnerOutput {
    pub(crate) program: PathBuf,
    pub(crate) status: std::process::ExitStatus,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl HostRunnerOutput {
    pub(crate) fn status_code_text(&self) -> String {
        self.status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_owned())
    }
}

pub(crate) struct HostRunnerJsonSurface {
    pub(crate) invoked: bool,
    pub(crate) status: String,
    pub(crate) program: Option<String>,
    pub(crate) exit_status: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) ready: Option<bool>,
    pub(crate) would_enter_lifecycle_hook: Option<bool>,
    pub(crate) nsb_readable: Option<bool>,
    pub(crate) nsb_hash_matches: Option<bool>,
    pub(crate) nsb_payload_region_mapped: Option<bool>,
    pub(crate) nsb_payload_scan_kind: Option<String>,
    pub(crate) container_loader_status: Option<String>,
    pub(crate) container_ready: Option<bool>,
    pub(crate) container_loader_entry_kind: Option<String>,
    pub(crate) container_loader_entry_symbol: Option<String>,
    pub(crate) container_loader_entry_section_id: Option<String>,
    pub(crate) container_loader_handoff_ready: Option<bool>,
    pub(crate) container_loader_handoff_status: Option<String>,
    pub(crate) backend_artifact_payload_count: Option<usize>,
    pub(crate) backend_artifact_payload_parsed_count: Option<usize>,
    pub(crate) backend_artifact_payload_ready_count: Option<usize>,
    pub(crate) backend_artifact_payload_first_id: Option<String>,
    pub(crate) backend_artifact_payload_first_kind: Option<String>,
    pub(crate) backend_artifact_payload_first_role_status: Option<String>,
    pub(crate) backend_artifact_payload_table_hash: Option<String>,
}

impl HostRunnerJsonSurface {
    pub(crate) fn not_invoked(status: &str) -> Self {
        Self {
            invoked: false,
            status: status.to_owned(),
            program: None,
            exit_status: None,
            error: None,
            ready: None,
            would_enter_lifecycle_hook: None,
            nsb_readable: None,
            nsb_hash_matches: None,
            nsb_payload_region_mapped: None,
            nsb_payload_scan_kind: None,
            container_loader_status: None,
            container_ready: None,
            container_loader_entry_kind: None,
            container_loader_entry_symbol: None,
            container_loader_entry_section_id: None,
            container_loader_handoff_ready: None,
            container_loader_handoff_status: None,
            backend_artifact_payload_count: None,
            backend_artifact_payload_parsed_count: None,
            backend_artifact_payload_ready_count: None,
            backend_artifact_payload_first_id: None,
            backend_artifact_payload_first_kind: None,
            backend_artifact_payload_first_role_status: None,
            backend_artifact_payload_table_hash: None,
        }
    }

    pub(crate) fn from_error(program: PathBuf, status: &str, error: String) -> Self {
        Self {
            invoked: true,
            status: status.to_owned(),
            program: Some(program.display().to_string()),
            exit_status: None,
            error: Some(error),
            ready: None,
            would_enter_lifecycle_hook: None,
            nsb_readable: None,
            nsb_hash_matches: None,
            nsb_payload_region_mapped: None,
            nsb_payload_scan_kind: None,
            container_loader_status: None,
            container_ready: None,
            container_loader_entry_kind: None,
            container_loader_entry_symbol: None,
            container_loader_entry_section_id: None,
            container_loader_handoff_ready: None,
            container_loader_handoff_status: None,
            backend_artifact_payload_count: None,
            backend_artifact_payload_parsed_count: None,
            backend_artifact_payload_ready_count: None,
            backend_artifact_payload_first_id: None,
            backend_artifact_payload_first_kind: None,
            backend_artifact_payload_first_role_status: None,
            backend_artifact_payload_table_hash: None,
        }
    }

    pub(crate) fn from_output(output: &HostRunnerOutput) -> Self {
        let status = if output.status.success() {
            json_bool_value(&output.stdout, "ready")
                .map(|ready| if ready { "ready" } else { "blocked" })
                .unwrap_or("reported")
        } else {
            "failed"
        };
        Self {
            invoked: true,
            status: status.to_owned(),
            program: Some(output.program.display().to_string()),
            exit_status: Some(output.status_code_text()),
            error: if output.status.success() {
                None
            } else {
                Some(format!(
                    "stdout:\n{}\nstderr:\n{}",
                    output.stdout, output.stderr
                ))
            },
            ready: json_bool_value(&output.stdout, "ready"),
            would_enter_lifecycle_hook: json_bool_value(
                &output.stdout,
                "would_enter_lifecycle_hook",
            ),
            nsb_readable: json_bool_value(&output.stdout, "nsb_readable"),
            nsb_hash_matches: json_bool_value(&output.stdout, "nsb_hash_matches"),
            nsb_payload_region_mapped: json_bool_value(&output.stdout, "nsb_payload_region_mapped"),
            nsb_payload_scan_kind: json_string_value(&output.stdout, "nsb_payload_scan_kind"),
            container_loader_status: json_string_value(&output.stdout, "container_loader_status"),
            container_ready: json_bool_value(&output.stdout, "container_ready"),
            container_loader_entry_kind: json_string_value(
                &output.stdout,
                "container_loader_entry_kind",
            ),
            container_loader_entry_symbol: json_string_value(
                &output.stdout,
                "container_loader_entry_symbol",
            ),
            container_loader_entry_section_id: json_string_value(
                &output.stdout,
                "container_loader_entry_section_id",
            ),
            container_loader_handoff_ready: json_bool_value(
                &output.stdout,
                "container_loader_handoff_ready",
            ),
            container_loader_handoff_status: json_string_value(
                &output.stdout,
                "container_loader_handoff_status",
            ),
            backend_artifact_payload_count: json_usize_value(
                &output.stdout,
                "backend_artifact_payload_count",
            ),
            backend_artifact_payload_parsed_count: json_usize_value(
                &output.stdout,
                "backend_artifact_payload_parsed_count",
            ),
            backend_artifact_payload_ready_count: json_usize_value(
                &output.stdout,
                "backend_artifact_payload_ready_count",
            ),
            backend_artifact_payload_first_id: json_string_value(
                &output.stdout,
                "backend_artifact_payload_first_id",
            ),
            backend_artifact_payload_first_kind: json_string_value(
                &output.stdout,
                "backend_artifact_payload_first_kind",
            ),
            backend_artifact_payload_first_role_status: json_string_value(
                &output.stdout,
                "backend_artifact_payload_first_role_status",
            ),
            backend_artifact_payload_table_hash: json_string_value(
                &output.stdout,
                "backend_artifact_payload_table_hash",
            ),
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_bool_field("host_runner_invoked", self.invoked),
            json_field("host_runner_status", &self.status),
            json_optional_string_field("host_runner_program", self.program.as_deref()),
            json_optional_string_field("host_runner_exit_status", self.exit_status.as_deref()),
            json_optional_string_field("host_runner_error", self.error.as_deref()),
            json_optional_bool_field("host_runner_ready", self.ready),
            json_optional_bool_field(
                "host_runner_would_enter_lifecycle_hook",
                self.would_enter_lifecycle_hook,
            ),
            json_optional_bool_field("host_runner_nsb_readable", self.nsb_readable),
            json_optional_bool_field("host_runner_nsb_hash_matches", self.nsb_hash_matches),
            json_optional_bool_field(
                "host_runner_nsb_payload_region_mapped",
                self.nsb_payload_region_mapped,
            ),
            json_optional_string_field(
                "host_runner_nsb_payload_scan_kind",
                self.nsb_payload_scan_kind.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_container_loader_status",
                self.container_loader_status.as_deref(),
            ),
            json_optional_bool_field("host_runner_container_ready", self.container_ready),
            json_optional_string_field(
                "host_runner_container_loader_entry_kind",
                self.container_loader_entry_kind.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_container_loader_entry_symbol",
                self.container_loader_entry_symbol.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_container_loader_entry_section_id",
                self.container_loader_entry_section_id.as_deref(),
            ),
            json_optional_bool_field(
                "host_runner_container_loader_handoff_ready",
                self.container_loader_handoff_ready,
            ),
            json_optional_string_field(
                "host_runner_container_loader_handoff_status",
                self.container_loader_handoff_status.as_deref(),
            ),
            json_optional_usize_field(
                "host_runner_backend_artifact_payload_count",
                self.backend_artifact_payload_count,
            ),
            json_optional_usize_field(
                "host_runner_backend_artifact_payload_parsed_count",
                self.backend_artifact_payload_parsed_count,
            ),
            json_optional_usize_field(
                "host_runner_backend_artifact_payload_ready_count",
                self.backend_artifact_payload_ready_count,
            ),
            json_optional_string_field(
                "host_runner_backend_artifact_payload_first_id",
                self.backend_artifact_payload_first_id.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_backend_artifact_payload_first_kind",
                self.backend_artifact_payload_first_kind.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_backend_artifact_payload_first_role_status",
                self.backend_artifact_payload_first_role_status.as_deref(),
            ),
            json_optional_string_field(
                "host_runner_backend_artifact_payload_table_hash",
                self.backend_artifact_payload_table_hash.as_deref(),
            ),
        ]
    }
}

fn json_bool_value(source: &str, key: &str) -> Option<bool> {
    let true_needle = format!("\"{key}\":true");
    if source.contains(&true_needle) {
        return Some(true);
    }
    let false_needle = format!("\"{key}\":false");
    if source.contains(&false_needle) {
        return Some(false);
    }
    None
}

fn json_string_value(source: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = source.find(&needle)? + needle.len();
    let tail = &source[start..];
    let end = tail.find('"')?;
    Some(tail[..end].to_owned())
}

fn json_usize_value(source: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\":");
    let start = source.find(&needle)? + needle.len();
    let tail = &source[start..];
    let end = tail
        .find(|value: char| !value.is_ascii_digit())
        .unwrap_or(tail.len());
    if end == 0 {
        return None;
    }
    tail[..end].parse().ok()
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn host_runner_surface_mirrors_backend_payload_scan_fields() {
        let status = Command::new("true")
            .status()
            .expect("true command should be available");
        let output = HostRunnerOutput {
            program: PathBuf::from("nuis-host-runner"),
            status,
            stdout: r#"{"ready":true,"backend_artifact_payload_count":2,"backend_artifact_payload_parsed_count":2,"backend_artifact_payload_ready_count":1,"backend_artifact_payload_first_id":"payload0005.backend-artifact","backend_artifact_payload_first_kind":"nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu","backend_artifact_payload_first_role_status":"ready","backend_artifact_payload_table_hash":"0x7777777777777777"}"#.to_owned(),
            stderr: String::new(),
        };

        let surface = HostRunnerJsonSurface::from_output(&output);
        let json = surface.json_fields().join(",");

        assert!(json.contains("\"host_runner_backend_artifact_payload_count\":2"));
        assert!(json.contains("\"host_runner_backend_artifact_payload_parsed_count\":2"));
        assert!(json.contains("\"host_runner_backend_artifact_payload_ready_count\":1"));
        assert!(json.contains(
            "\"host_runner_backend_artifact_payload_first_id\":\"payload0005.backend-artifact\""
        ));
        assert!(json.contains(
            "\"host_runner_backend_artifact_payload_first_kind\":\"nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu\""
        ));
        assert!(
            json.contains("\"host_runner_backend_artifact_payload_first_role_status\":\"ready\"")
        );
        assert!(json.contains(
            "\"host_runner_backend_artifact_payload_table_hash\":\"0x7777777777777777\""
        ));
    }
}
