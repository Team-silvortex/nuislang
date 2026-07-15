use crate::{
    artifact_doctor::BackendArtifactPayloadEvidence, json_bool_field, json_field,
    json_object_array_field, json_optional_string_field, json_string_array_field, json_usize_field,
};
use std::collections::BTreeSet;

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
