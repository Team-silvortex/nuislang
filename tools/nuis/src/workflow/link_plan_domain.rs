use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkflowDomainReadiness {
    package_id: String,
    domain_family: String,
    ready: bool,
    selected_lowering_target_present: bool,
    payload_blob_present: bool,
    payload_format_present: bool,
    bridge_stub_present: bool,
    ir_sidecar_present: bool,
    registry_dispatch_readiness_status: String,
    registry_dispatch_readiness_ready: bool,
    registry_dispatch_missing_signals: Vec<String>,
    registry_dispatch_bridge_materialized: bool,
    registry_execution_readiness_materialized: bool,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkflowDomainReadinessSummary {
    pub(super) hetero_units: usize,
    pub(super) ready_units: usize,
    pub(super) registry_dispatch_ready_units: usize,
    pub(super) ready: bool,
    pub(super) domain_families: Vec<String>,
    pub(super) first_unready: Option<String>,
    pub(super) registry_dispatch_first_blocked: Option<String>,
    units: Vec<WorkflowDomainReadiness>,
}

pub(super) fn workflow_link_plan_domain_unit_record(
    unit: &nuisc::linker::LinkPlanDomainUnit,
) -> String {
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", &unit.kind),
            json_field("package_id", &unit.package_id),
            json_field("domain_family", &unit.domain_family),
            json_field("contract_family", &unit.contract_family),
            json_field("packaging_role", &unit.packaging_role),
        ],
    );
    if let Some(value) = unit.abi.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("abi", value)]);
    }
    if let Some(value) = unit.backend_family.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("backend_family", value)]);
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        append_json_field_strings(
            &mut out,
            vec![json_field("selected_lowering_target", value)],
        );
    }
    if let Some(value) = unit.machine_arch.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_arch", value)]);
    }
    if let Some(value) = unit.machine_os.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_os", value)]);
    }
    out.push('}');
    out
}

pub(super) fn workflow_domain_readiness_summary(
    plan: &nuisc::linker::LinkPlan,
) -> WorkflowDomainReadinessSummary {
    let units = plan
        .domain_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .map(workflow_domain_readiness)
        .collect::<Vec<_>>();
    let hetero_units = units.len();
    let ready_units = units.iter().filter(|unit| unit.ready).count();
    let registry_dispatch_ready_units = units
        .iter()
        .filter(|unit| unit.registry_dispatch_readiness_ready)
        .count();
    let domain_families = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let first_unready = units
        .iter()
        .find(|unit| !unit.ready)
        .map(|unit| format!("{}[{}]", unit.package_id, unit.domain_family));
    let registry_dispatch_first_blocked = units
        .iter()
        .find(|unit| !unit.registry_dispatch_readiness_ready)
        .map(|unit| format!("{}[{}]", unit.package_id, unit.domain_family));
    WorkflowDomainReadinessSummary {
        hetero_units,
        ready_units,
        registry_dispatch_ready_units,
        ready: hetero_units == ready_units,
        domain_families,
        first_unready,
        registry_dispatch_first_blocked,
        units,
    }
}

fn workflow_domain_readiness(unit: &nuisc::linker::LinkPlanDomainUnit) -> WorkflowDomainReadiness {
    let selected_lowering_target_present = unit.selected_lowering_target.is_some();
    let payload_blob_present = unit.artifact_payload_blob_path.is_some();
    let payload_format_present = unit.artifact_payload_format.is_some();
    let bridge_stub_present = unit.artifact_bridge_stub_path.is_some();
    let ir_sidecar_present = unit.artifact_ir_sidecar_path.is_some();
    let registry_dispatch = workflow_registry_dispatch_readiness(unit);
    let mut issues = Vec::new();
    if !payload_blob_present {
        issues.push("payload_blob_missing".to_owned());
    }
    if !payload_format_present {
        issues.push("payload_format_missing".to_owned());
    }
    if !bridge_stub_present {
        issues.push("bridge_stub_missing".to_owned());
    }
    if !registry_dispatch.ready {
        issues.push("registry_dispatch_readiness_blocked".to_owned());
    }
    WorkflowDomainReadiness {
        package_id: unit.package_id.clone(),
        domain_family: unit.domain_family.clone(),
        ready: issues.is_empty(),
        selected_lowering_target_present,
        payload_blob_present,
        payload_format_present,
        bridge_stub_present,
        ir_sidecar_present,
        registry_dispatch_readiness_status: registry_dispatch.status,
        registry_dispatch_readiness_ready: registry_dispatch.ready,
        registry_dispatch_missing_signals: registry_dispatch.missing_signals,
        registry_dispatch_bridge_materialized: registry_dispatch.dispatch_bridge_materialized,
        registry_execution_readiness_materialized: registry_dispatch
            .execution_readiness_materialized,
        issues,
    }
}

struct WorkflowRegistryDispatchReadiness {
    status: String,
    ready: bool,
    missing_signals: Vec<String>,
    dispatch_bridge_materialized: bool,
    execution_readiness_materialized: bool,
}

fn workflow_registry_dispatch_readiness(
    unit: &nuisc::linker::LinkPlanDomainUnit,
) -> WorkflowRegistryDispatchReadiness {
    match nuisc::registry::load_manifest_for_domain(
        std::path::Path::new("nustar-packages"),
        &unit.domain_family,
    ) {
        Ok(manifest) => {
            let dispatch = nuisc::registry::dispatch_readiness_summary(&manifest);
            WorkflowRegistryDispatchReadiness {
                status: dispatch.status.clone(),
                ready: dispatch.status == "ready",
                missing_signals: dispatch.missing_signals,
                dispatch_bridge_materialized: dispatch.dispatch_bridge_materialized,
                execution_readiness_materialized: dispatch.execution_readiness_materialized,
            }
        }
        Err(error) => WorkflowRegistryDispatchReadiness {
            status: "unavailable".to_owned(),
            ready: false,
            missing_signals: vec![format!("registry_manifest_unavailable:{error}")],
            dispatch_bridge_materialized: false,
            execution_readiness_materialized: false,
        },
    }
}

pub(super) fn workflow_domain_readiness_units_json(
    summary: &WorkflowDomainReadinessSummary,
) -> Vec<String> {
    summary
        .units
        .iter()
        .map(workflow_domain_readiness_json)
        .collect()
}

fn workflow_domain_readiness_json(unit: &WorkflowDomainReadiness) -> String {
    let fields = [
        json_field("package_id", &unit.package_id),
        json_field("domain_family", &unit.domain_family),
        json_bool_field("ready", unit.ready),
        json_bool_field(
            "selected_lowering_target_present",
            unit.selected_lowering_target_present,
        ),
        json_bool_field("payload_blob_present", unit.payload_blob_present),
        json_bool_field("payload_format_present", unit.payload_format_present),
        json_bool_field("bridge_stub_present", unit.bridge_stub_present),
        json_bool_field("ir_sidecar_present", unit.ir_sidecar_present),
        json_field(
            "registry_dispatch_readiness_status",
            &unit.registry_dispatch_readiness_status,
        ),
        json_bool_field(
            "registry_dispatch_readiness_ready",
            unit.registry_dispatch_readiness_ready,
        ),
        json_string_array_field(
            "registry_dispatch_missing_signals",
            &unit.registry_dispatch_missing_signals,
        ),
        json_bool_field(
            "registry_dispatch_bridge_materialized",
            unit.registry_dispatch_bridge_materialized,
        ),
        json_bool_field(
            "registry_execution_readiness_materialized",
            unit.registry_execution_readiness_materialized,
        ),
        json_string_array_field("issues", &unit.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
