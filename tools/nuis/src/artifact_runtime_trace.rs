use crate::{
    artifact_doctor::BackendArtifactPayloadEvidence, json_bool_field, json_field,
    json_object_array_field, json_optional_string_field, json_string_array_field, json_usize_field,
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const HETERO_RUNTIME_TRACE_FILE_NAME: &str = "nuis.nsdb.hetero-runtime-trace.toml";
const HETERO_RUNTIME_TRACE_PROTOCOL: &str = "nuis-nsdb-hetero-runtime-trace-v1";
const PAYLOAD_DECODER_MANIFEST_FILE_NAME: &str = "nuis.nsdb.payload-decoders.toml";
const PAYLOAD_DECODER_MANIFEST_PROTOCOL: &str = "nuis-nsdb-payload-decoders-v1";
const PAYLOAD_DECODER_MANIFEST_SCHEMA: &str = "nsdb-payload-decoder-manifest-v1";

struct HeteroRuntimeTraceRecord {
    trace_id: String,
    trace_role: String,
    status: String,
    domain_family: String,
    backend_family: Option<String>,
    target_device: Option<String>,
    backend_artifact_key: String,
    selected_lowering_target: Option<String>,
    payload_format: Option<String>,
    payload_path: Option<String>,
    bridge_stub_path: Option<String>,
    missing_signals: Vec<String>,
    next_action: String,
}

pub(crate) struct HeteroRuntimeTraceSummary {
    available: bool,
    status: String,
    debugger_contract: &'static str,
    domain_count: usize,
    backend_artifact_count: usize,
    backend_artifact_ready_count: usize,
    trace_record_count: usize,
    trace_ready_record_count: usize,
    backend_execution_record_count: usize,
    payload_evidence_available: bool,
    payload_evidence_count: usize,
    payload_evidence_present_count: usize,
    payload_role_status: String,
    domain_families: Vec<String>,
    backend_families: Vec<String>,
    target_devices: Vec<String>,
    records: Vec<HeteroRuntimeTraceRecord>,
    first_blocker: Option<String>,
    next_action: String,
}

pub(crate) struct HeteroRuntimeTracePersistence {
    persisted: bool,
    path: Option<PathBuf>,
    record_count: usize,
    first_trace_id: Option<String>,
    error: Option<String>,
    decoder_manifest_persisted: bool,
    decoder_manifest_path: Option<PathBuf>,
    decoder_manifest_record_count: usize,
    decoder_manifest_error: Option<String>,
}

impl HeteroRuntimeTraceSummary {
    pub(crate) fn from_link_plan(
        plan: Option<&nuisc::linker::LinkPlan>,
        payload_evidence: &BackendArtifactPayloadEvidence,
    ) -> Self {
        let Some(plan) = plan else {
            return Self::unavailable("link-plan-unavailable");
        };
        let hetero_units = plan
            .domain_units
            .iter()
            .filter(|unit| unit.domain_family != "cpu")
            .collect::<Vec<_>>();
        let domain_count = hetero_units.len();
        if domain_count == 0 {
            return Self::unavailable("no-heterogeneous-domain");
        }

        let backend_units = hetero_units
            .iter()
            .copied()
            .filter(|unit| {
                unit.backend_family.is_some()
                    || unit.target_device.is_some()
                    || unit.selected_lowering_target.is_some()
            })
            .collect::<Vec<_>>();
        let backend_artifact_count = backend_units.len();
        let backend_artifact_ready_count = backend_units
            .iter()
            .filter(|unit| backend_artifact_missing_signals(unit).is_empty())
            .count();
        let records = hetero_units
            .iter()
            .map(|unit| trace_record_for_unit(unit, payload_evidence))
            .collect::<Vec<_>>();
        let trace_record_count = records.len();
        let trace_ready_record_count = records
            .iter()
            .filter(|record| record.status == "trace-ready")
            .count();
        let backend_execution_record_count = records
            .iter()
            .filter(|record| record.trace_role == "backend-artifact")
            .count();
        let first_backend_blocker = backend_units.iter().find_map(|unit| {
            backend_artifact_missing_signals(unit)
                .first()
                .map(|signal| format!("{}:{signal}", backend_artifact_key(unit)))
        });
        let first_blocker = first_backend_blocker;
        let domain_families = hetero_units
            .iter()
            .map(|unit| unit.domain_family.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let backend_families = backend_units
            .iter()
            .filter_map(|unit| unit.backend_family.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let target_devices = backend_units
            .iter()
            .filter_map(|unit| unit.target_device.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let status = if first_blocker.is_some() {
            "blocked"
        } else if payload_evidence.present_count > 0 {
            "trace-ready"
        } else {
            "execution-pending"
        };
        let next_action = match status {
            "trace-ready" => "handoff-to-nsdb-trace",
            "execution-pending" => "materialize-device-execution-trace",
            _ => "resolve-hetero-runtime-trace-blocker",
        };

        Self {
            available: true,
            status: status.to_owned(),
            debugger_contract: "nsdb-yir-hetero-runtime-trace-v1",
            domain_count,
            backend_artifact_count,
            backend_artifact_ready_count,
            trace_record_count,
            trace_ready_record_count,
            backend_execution_record_count,
            payload_evidence_available: payload_evidence.available,
            payload_evidence_count: payload_evidence.count,
            payload_evidence_present_count: payload_evidence.present_count,
            payload_role_status: payload_evidence.role_status.clone(),
            domain_families,
            backend_families,
            target_devices,
            records,
            first_blocker,
            next_action: next_action.to_owned(),
        }
    }

    fn unavailable(first_blocker: &str) -> Self {
        Self {
            available: false,
            status: "unavailable".to_owned(),
            debugger_contract: "nsdb-yir-hetero-runtime-trace-v1",
            domain_count: 0,
            backend_artifact_count: 0,
            backend_artifact_ready_count: 0,
            trace_record_count: 0,
            trace_ready_record_count: 0,
            backend_execution_record_count: 0,
            payload_evidence_available: false,
            payload_evidence_count: 0,
            payload_evidence_present_count: 0,
            payload_role_status: "unavailable".to_owned(),
            domain_families: Vec::new(),
            backend_families: Vec::new(),
            target_devices: Vec::new(),
            records: Vec::new(),
            first_blocker: Some(first_blocker.to_owned()),
            next_action: "build-heterogeneous-artifact".to_owned(),
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_bool_field("hetero_runtime_trace_available", self.available),
            json_field("hetero_runtime_trace_status", &self.status),
            json_field(
                "hetero_runtime_trace_debugger_contract",
                self.debugger_contract,
            ),
            json_usize_field("hetero_runtime_trace_domain_count", self.domain_count),
            json_usize_field(
                "hetero_runtime_trace_backend_artifact_count",
                self.backend_artifact_count,
            ),
            json_usize_field(
                "hetero_runtime_trace_backend_artifact_ready_count",
                self.backend_artifact_ready_count,
            ),
            json_usize_field("hetero_runtime_trace_record_count", self.trace_record_count),
            json_usize_field(
                "hetero_runtime_trace_ready_record_count",
                self.trace_ready_record_count,
            ),
            json_usize_field(
                "hetero_runtime_trace_backend_execution_record_count",
                self.backend_execution_record_count,
            ),
            json_bool_field(
                "hetero_runtime_trace_payload_evidence_available",
                self.payload_evidence_available,
            ),
            json_usize_field(
                "hetero_runtime_trace_payload_evidence_count",
                self.payload_evidence_count,
            ),
            json_usize_field(
                "hetero_runtime_trace_payload_evidence_present_count",
                self.payload_evidence_present_count,
            ),
            json_field(
                "hetero_runtime_trace_payload_role_status",
                &self.payload_role_status,
            ),
            json_string_array_field(
                "hetero_runtime_trace_domain_families",
                &self.domain_families,
            ),
            json_string_array_field(
                "hetero_runtime_trace_backend_families",
                &self.backend_families,
            ),
            json_string_array_field("hetero_runtime_trace_target_devices", &self.target_devices),
            json_object_array_field(
                "hetero_runtime_trace_records",
                &self
                    .records
                    .iter()
                    .map(HeteroRuntimeTraceRecord::json_object)
                    .collect::<Vec<_>>(),
            ),
            json_optional_string_field(
                "hetero_runtime_trace_first_blocker",
                self.first_blocker.as_deref(),
            ),
            json_field("hetero_runtime_trace_next_action", &self.next_action),
        ]
    }

    pub(crate) fn persist_nsdb_trace(
        &self,
        output_dir: Option<&Path>,
    ) -> HeteroRuntimeTracePersistence {
        let first_trace_id = self.records.first().map(|record| record.trace_id.clone());
        let Some(output_dir) = output_dir else {
            return HeteroRuntimeTracePersistence {
                persisted: false,
                path: None,
                record_count: self.records.len(),
                first_trace_id,
                error: Some("output_dir-unavailable".to_owned()),
                decoder_manifest_persisted: false,
                decoder_manifest_path: None,
                decoder_manifest_record_count: 0,
                decoder_manifest_error: Some("output_dir-unavailable".to_owned()),
            };
        };
        let path = output_dir.join(HETERO_RUNTIME_TRACE_FILE_NAME);
        let decoder_manifest_path = output_dir.join(PAYLOAD_DECODER_MANIFEST_FILE_NAME);
        if !self.available {
            return HeteroRuntimeTracePersistence {
                persisted: false,
                path: Some(path),
                record_count: self.records.len(),
                first_trace_id,
                error: Some("hetero-runtime-trace-unavailable".to_owned()),
                decoder_manifest_persisted: false,
                decoder_manifest_path: Some(decoder_manifest_path),
                decoder_manifest_record_count: 0,
                decoder_manifest_error: Some("hetero-runtime-trace-unavailable".to_owned()),
            };
        }
        match fs::write(&path, self.render_nsdb_trace_toml()) {
            Ok(()) => {
                let decoder_manifest_source = self.render_payload_decoder_manifest_toml();
                let decoder_manifest_record_count = payload_decoder_manifest_record_count(self);
                match fs::write(&decoder_manifest_path, decoder_manifest_source) {
                    Ok(()) => HeteroRuntimeTracePersistence {
                        persisted: true,
                        path: Some(path),
                        record_count: self.records.len(),
                        first_trace_id,
                        error: None,
                        decoder_manifest_persisted: true,
                        decoder_manifest_path: Some(decoder_manifest_path),
                        decoder_manifest_record_count,
                        decoder_manifest_error: None,
                    },
                    Err(error) => HeteroRuntimeTracePersistence {
                        persisted: true,
                        path: Some(path),
                        record_count: self.records.len(),
                        first_trace_id,
                        error: None,
                        decoder_manifest_persisted: false,
                        decoder_manifest_path: Some(decoder_manifest_path),
                        decoder_manifest_record_count,
                        decoder_manifest_error: Some(error.to_string()),
                    },
                }
            }
            Err(error) => HeteroRuntimeTracePersistence {
                persisted: false,
                path: Some(path),
                record_count: self.records.len(),
                first_trace_id,
                error: Some(error.to_string()),
                decoder_manifest_persisted: false,
                decoder_manifest_path: Some(decoder_manifest_path),
                decoder_manifest_record_count: 0,
                decoder_manifest_error: Some("hetero-runtime-trace-persist-failed".to_owned()),
            },
        }
    }

    fn render_nsdb_trace_toml(&self) -> String {
        let mut out = String::new();
        push_toml_string(&mut out, "protocol", HETERO_RUNTIME_TRACE_PROTOCOL);
        push_toml_string(&mut out, "debugger_contract", self.debugger_contract);
        push_toml_string(&mut out, "source", "run-artifact-hetero-runtime-trace");
        push_toml_string(&mut out, "status", &self.status);
        out.push_str(&format!("record_count = {}\n", self.records.len()));
        out.push_str(&format!(
            "ready_record_count = {}\n",
            self.trace_ready_record_count
        ));
        out.push_str(&format!(
            "backend_execution_record_count = {}\n",
            self.backend_execution_record_count
        ));
        push_toml_optional_string(
            &mut out,
            "first_trace_id",
            self.records.first().map(|record| record.trace_id.as_str()),
        );
        push_toml_optional_string(&mut out, "first_blocker", self.first_blocker.as_deref());
        push_toml_string(&mut out, "next_action", &self.next_action);
        for record in &self.records {
            out.push_str("\n[[records]]\n");
            record.push_toml_fields(&mut out);
        }
        out
    }

    fn render_payload_decoder_manifest_toml(&self) -> String {
        let mut out = String::new();
        push_toml_string(&mut out, "protocol", PAYLOAD_DECODER_MANIFEST_PROTOCOL);
        push_toml_string(&mut out, "schema", PAYLOAD_DECODER_MANIFEST_SCHEMA);
        for payload_format in payload_decoder_manifest_formats(self) {
            out.push_str("\n[[decoders]]\n");
            push_toml_string(&mut out, "payload_format", &payload_format);
            push_toml_string(
                &mut out,
                "decoder_id",
                &format!(
                    "nsdb-{}-generated-opaque-decoder-v1",
                    payload_format.replace(['.', ':', '/'], "-")
                ),
            );
            push_toml_string(&mut out, "decoder_capability", "opaque-file-summary");
            push_toml_string(&mut out, "decoder_detail_level", "file-header");
        }
        out
    }

    pub(crate) fn print_text(&self) {
        println!("  hetero_runtime_trace_available: {}", self.available);
        println!("  hetero_runtime_trace_status: {}", self.status);
        println!(
            "  hetero_runtime_trace_debugger_contract: {}",
            self.debugger_contract
        );
        println!("  hetero_runtime_trace_domain_count: {}", self.domain_count);
        println!(
            "  hetero_runtime_trace_backend_artifact_count: {}",
            self.backend_artifact_count
        );
        println!(
            "  hetero_runtime_trace_backend_artifact_ready_count: {}",
            self.backend_artifact_ready_count
        );
        println!(
            "  hetero_runtime_trace_record_count: {}",
            self.trace_record_count
        );
        println!(
            "  hetero_runtime_trace_ready_record_count: {}",
            self.trace_ready_record_count
        );
        println!(
            "  hetero_runtime_trace_backend_execution_record_count: {}",
            self.backend_execution_record_count
        );
        println!(
            "  hetero_runtime_trace_payload_evidence_available: {}",
            self.payload_evidence_available
        );
        println!(
            "  hetero_runtime_trace_payload_evidence_count: {}",
            self.payload_evidence_count
        );
        println!(
            "  hetero_runtime_trace_payload_evidence_present_count: {}",
            self.payload_evidence_present_count
        );
        println!(
            "  hetero_runtime_trace_payload_role_status: {}",
            self.payload_role_status
        );
        println!(
            "  hetero_runtime_trace_domain_families: {}",
            joined_or_none(&self.domain_families)
        );
        println!(
            "  hetero_runtime_trace_backend_families: {}",
            joined_or_none(&self.backend_families)
        );
        println!(
            "  hetero_runtime_trace_target_devices: {}",
            joined_or_none(&self.target_devices)
        );
        for record in &self.records {
            println!(
                "  hetero_runtime_trace_record: {} {} {}",
                record.trace_id, record.trace_role, record.status
            );
        }
        println!(
            "  hetero_runtime_trace_first_blocker: {}",
            self.first_blocker.as_deref().unwrap_or("<none>")
        );
        println!("  hetero_runtime_trace_next_action: {}", self.next_action);
    }
}

impl HeteroRuntimeTracePersistence {
    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_field(
                "hetero_runtime_trace_persistence_protocol",
                HETERO_RUNTIME_TRACE_PROTOCOL,
            ),
            json_bool_field("hetero_runtime_trace_persisted", self.persisted),
            json_optional_string_field(
                "hetero_runtime_trace_path",
                self.path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_usize_field(
                "hetero_runtime_trace_persisted_record_count",
                self.record_count,
            ),
            json_optional_string_field(
                "hetero_runtime_trace_persisted_first_trace_id",
                self.first_trace_id.as_deref(),
            ),
            json_optional_string_field("hetero_runtime_trace_persist_error", self.error.as_deref()),
            json_bool_field(
                "payload_decoder_manifest_persisted",
                self.decoder_manifest_persisted,
            ),
            json_optional_string_field(
                "payload_decoder_manifest_path",
                self.decoder_manifest_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_usize_field(
                "payload_decoder_manifest_persisted_record_count",
                self.decoder_manifest_record_count,
            ),
            json_optional_string_field(
                "payload_decoder_manifest_persist_error",
                self.decoder_manifest_error.as_deref(),
            ),
        ]
    }

    pub(crate) fn print_text(&self) {
        println!("  hetero_runtime_trace_persistence_protocol: {HETERO_RUNTIME_TRACE_PROTOCOL}");
        println!("  hetero_runtime_trace_persisted: {}", self.persisted);
        println!(
            "  hetero_runtime_trace_path: {}",
            self.path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<none>".to_owned())
        );
        println!(
            "  hetero_runtime_trace_persisted_record_count: {}",
            self.record_count
        );
        println!(
            "  hetero_runtime_trace_persisted_first_trace_id: {}",
            self.first_trace_id.as_deref().unwrap_or("<none>")
        );
        println!(
            "  hetero_runtime_trace_persist_error: {}",
            self.error.as_deref().unwrap_or("<none>")
        );
        println!(
            "  payload_decoder_manifest_persisted: {}",
            self.decoder_manifest_persisted
        );
        println!(
            "  payload_decoder_manifest_path: {}",
            self.decoder_manifest_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<none>".to_owned())
        );
        println!(
            "  payload_decoder_manifest_persisted_record_count: {}",
            self.decoder_manifest_record_count
        );
        println!(
            "  payload_decoder_manifest_persist_error: {}",
            self.decoder_manifest_error.as_deref().unwrap_or("<none>")
        );
    }
}

impl HeteroRuntimeTraceRecord {
    fn json_object(&self) -> String {
        let fields = vec![
            json_field("trace_id", &self.trace_id),
            json_field("trace_role", &self.trace_role),
            json_field("status", &self.status),
            json_field("domain_family", &self.domain_family),
            json_optional_string_field("backend_family", self.backend_family.as_deref()),
            json_optional_string_field("target_device", self.target_device.as_deref()),
            json_field("backend_artifact_key", &self.backend_artifact_key),
            json_optional_string_field(
                "selected_lowering_target",
                self.selected_lowering_target.as_deref(),
            ),
            json_optional_string_field("payload_format", self.payload_format.as_deref()),
            json_optional_string_field("payload_path", self.payload_path.as_deref()),
            json_optional_string_field("bridge_stub_path", self.bridge_stub_path.as_deref()),
            json_string_array_field("missing_signals", &self.missing_signals),
            json_field("next_action", &self.next_action),
        ];
        format!("{{{}}}", fields.join(","))
    }

    fn push_toml_fields(&self, out: &mut String) {
        push_toml_string(out, "trace_id", &self.trace_id);
        push_toml_string(out, "trace_role", &self.trace_role);
        push_toml_string(out, "status", &self.status);
        push_toml_string(out, "domain_family", &self.domain_family);
        push_toml_optional_string(out, "backend_family", self.backend_family.as_deref());
        push_toml_optional_string(out, "target_device", self.target_device.as_deref());
        push_toml_string(out, "backend_artifact_key", &self.backend_artifact_key);
        push_toml_optional_string(
            out,
            "selected_lowering_target",
            self.selected_lowering_target.as_deref(),
        );
        push_toml_optional_string(out, "payload_format", self.payload_format.as_deref());
        push_toml_optional_string(out, "payload_path", self.payload_path.as_deref());
        push_toml_optional_string(out, "bridge_stub_path", self.bridge_stub_path.as_deref());
        push_toml_string_array(out, "missing_signals", &self.missing_signals);
        push_toml_string(out, "next_action", &self.next_action);
    }
}

fn trace_record_for_unit(
    unit: &nuisc::linker::LinkPlanDomainUnit,
    payload_evidence: &BackendArtifactPayloadEvidence,
) -> HeteroRuntimeTraceRecord {
    let missing_signals = backend_artifact_missing_signals(unit);
    let trace_role = if is_backend_artifact_unit(unit) {
        "backend-artifact"
    } else {
        "domain-metadata"
    };
    let status = if trace_role == "domain-metadata" {
        "metadata-only"
    } else if !missing_signals.is_empty() {
        "blocked"
    } else if payload_evidence.present_count > 0 {
        "trace-ready"
    } else {
        "execution-pending"
    };
    let next_action = match status {
        "trace-ready" => "handoff-domain-record-to-nsdb",
        "execution-pending" => "materialize-device-execution-trace",
        "metadata-only" => "wait-for-backend-execution-record",
        _ => "resolve-domain-trace-blocker",
    };
    let backend_artifact_key = backend_artifact_key(unit);

    HeteroRuntimeTraceRecord {
        trace_id: format!("hetero-trace:{backend_artifact_key}"),
        trace_role: trace_role.to_owned(),
        status: status.to_owned(),
        domain_family: unit.domain_family.clone(),
        backend_family: unit.backend_family.clone(),
        target_device: unit.target_device.clone(),
        backend_artifact_key,
        selected_lowering_target: unit.selected_lowering_target.clone(),
        payload_format: unit.artifact_payload_format.clone(),
        payload_path: unit.artifact_payload_blob_path.clone(),
        bridge_stub_path: unit.artifact_bridge_stub_path.clone(),
        missing_signals,
        next_action: next_action.to_owned(),
    }
}

fn is_backend_artifact_unit(unit: &nuisc::linker::LinkPlanDomainUnit) -> bool {
    unit.backend_family.is_some()
        || unit.target_device.is_some()
        || unit.selected_lowering_target.is_some()
}

fn backend_artifact_key(unit: &nuisc::linker::LinkPlanDomainUnit) -> String {
    format!(
        "{}:{}:{}",
        unit.domain_family,
        unit.backend_family.as_deref().unwrap_or("none"),
        unit.target_device.as_deref().unwrap_or("none")
    )
}

fn backend_artifact_missing_signals(unit: &nuisc::linker::LinkPlanDomainUnit) -> Vec<String> {
    let mut missing = Vec::new();
    if unit.backend_family.is_none() {
        missing.push("backend_family".to_owned());
    }
    if unit.target_device.is_none() {
        missing.push("target_device".to_owned());
    }
    if unit.artifact_payload_blob_path.is_none() {
        missing.push("artifact_payload_blob".to_owned());
    }
    if unit.artifact_payload_format.is_none() {
        missing.push("artifact_payload_format".to_owned());
    }
    if unit.artifact_bridge_stub_path.is_none() {
        missing.push("artifact_bridge_stub".to_owned());
    }
    missing
}

fn joined_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "<none>".to_owned()
    } else {
        values.join(", ")
    }
}

fn payload_decoder_manifest_formats(summary: &HeteroRuntimeTraceSummary) -> Vec<String> {
    summary
        .records
        .iter()
        .filter_map(|record| record.payload_format.clone())
        .filter(|format| !format.is_empty() && format != "none")
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn payload_decoder_manifest_record_count(summary: &HeteroRuntimeTraceSummary) -> usize {
    payload_decoder_manifest_formats(summary).len()
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

fn push_toml_string_array(out: &mut String, key: &str, values: &[String]) {
    out.push_str(key);
    out.push_str(" = [");
    out.push_str(
        &values
            .iter()
            .map(|value| format!("\"{}\"", toml_escape(value)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("]\n");
}

fn toml_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
