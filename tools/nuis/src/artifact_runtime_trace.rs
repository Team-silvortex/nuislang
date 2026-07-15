use crate::{
    artifact_doctor::BackendArtifactPayloadEvidence, json_bool_field, json_field,
    json_optional_string_field, json_string_array_field, json_usize_field,
};
use std::collections::BTreeSet;

pub(crate) struct HeteroRuntimeTraceSummary {
    available: bool,
    status: String,
    debugger_contract: &'static str,
    domain_count: usize,
    backend_artifact_count: usize,
    backend_artifact_ready_count: usize,
    payload_evidence_available: bool,
    payload_evidence_count: usize,
    payload_evidence_present_count: usize,
    payload_role_status: String,
    domain_families: Vec<String>,
    backend_families: Vec<String>,
    target_devices: Vec<String>,
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
            payload_evidence_available: payload_evidence.available,
            payload_evidence_count: payload_evidence.count,
            payload_evidence_present_count: payload_evidence.present_count,
            payload_role_status: payload_evidence.role_status.clone(),
            domain_families,
            backend_families,
            target_devices,
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
            payload_evidence_available: false,
            payload_evidence_count: 0,
            payload_evidence_present_count: 0,
            payload_role_status: "unavailable".to_owned(),
            domain_families: Vec::new(),
            backend_families: Vec::new(),
            target_devices: Vec::new(),
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
        println!(
            "  hetero_runtime_trace_first_blocker: {}",
            self.first_blocker.as_deref().unwrap_or("<none>")
        );
        println!("  hetero_runtime_trace_next_action: {}", self.next_action);
    }
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
