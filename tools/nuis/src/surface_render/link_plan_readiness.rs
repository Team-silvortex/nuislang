use std::{collections::BTreeSet, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LinkPlanDomainReadiness {
    pub(super) package_id: String,
    pub(super) domain_family: String,
    pub(super) backend_family: String,
    pub(super) target_device: String,
    pub(super) backend_artifact_candidate: bool,
    pub(super) backend_artifact_key: String,
    pub(super) backend_artifact_ready: bool,
    pub(super) backend_artifact_missing_signals: Vec<String>,
    pub(super) ready: bool,
    pub(super) selected_lowering_target_present: bool,
    pub(super) payload_blob_present: bool,
    pub(super) payload_format_present: bool,
    pub(super) bridge_stub_present: bool,
    pub(super) ir_sidecar_present: bool,
    pub(super) registry_dispatch_readiness_status: String,
    pub(super) registry_dispatch_readiness_ready: bool,
    pub(super) registry_dispatch_missing_signals: Vec<String>,
    pub(super) registry_dispatch_bridge_materialized: bool,
    pub(super) registry_execution_readiness_materialized: bool,
    pub(super) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LinkPlanDomainReadinessSummary {
    pub(super) hetero_units: usize,
    pub(super) ready_units: usize,
    pub(super) registry_dispatch_ready_units: usize,
    pub(super) backend_artifact_units: usize,
    pub(super) backend_artifact_ready_units: usize,
    pub(super) ready: bool,
    pub(super) domain_families: Vec<String>,
    pub(super) backend_families: Vec<String>,
    pub(super) target_devices: Vec<String>,
    pub(super) first_unready: Option<String>,
    pub(super) backend_artifact_first_unready: Option<String>,
    pub(super) registry_dispatch_first_blocked: Option<String>,
    pub(super) units: Vec<LinkPlanDomainReadiness>,
}

pub(super) fn link_plan_domain_readiness_summary(
    plan: &nuisc::linker::LinkPlan,
) -> LinkPlanDomainReadinessSummary {
    let units = plan
        .domain_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .map(link_plan_domain_readiness)
        .collect::<Vec<_>>();
    let hetero_units = units.len();
    let ready_units = units.iter().filter(|unit| unit.ready).count();
    let registry_dispatch_ready_units = units
        .iter()
        .filter(|unit| unit.registry_dispatch_readiness_ready)
        .count();
    let backend_artifact_units = units
        .iter()
        .filter(|unit| unit.backend_artifact_candidate)
        .count();
    let backend_artifact_ready_units = units
        .iter()
        .filter(|unit| unit.backend_artifact_candidate && unit.backend_artifact_ready)
        .count();
    let mut domain_families = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    domain_families.sort();
    let mut backend_families = units
        .iter()
        .map(|unit| unit.backend_family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    backend_families.sort();
    let mut target_devices = units
        .iter()
        .map(|unit| unit.target_device.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    target_devices.sort();
    let first_unready = units
        .iter()
        .find(|unit| !unit.ready)
        .map(|unit| format!("{}[{}]", unit.package_id, unit.domain_family));
    let backend_artifact_first_unready = units
        .iter()
        .find(|unit| unit.backend_artifact_candidate && !unit.backend_artifact_ready)
        .map(|unit| unit.backend_artifact_key.clone());
    let registry_dispatch_first_blocked = units
        .iter()
        .find(|unit| !unit.registry_dispatch_readiness_ready)
        .map(|unit| format!("{}[{}]", unit.package_id, unit.domain_family));
    LinkPlanDomainReadinessSummary {
        hetero_units,
        ready_units,
        registry_dispatch_ready_units,
        backend_artifact_units,
        backend_artifact_ready_units,
        ready: hetero_units == ready_units,
        domain_families,
        backend_families,
        target_devices,
        first_unready,
        backend_artifact_first_unready,
        registry_dispatch_first_blocked,
        units,
    }
}

fn link_plan_domain_readiness(unit: &nuisc::linker::LinkPlanDomainUnit) -> LinkPlanDomainReadiness {
    let backend_family = unit.backend_family.as_deref().unwrap_or("none").to_owned();
    let target_device = unit.target_device.as_deref().unwrap_or("none").to_owned();
    let selected_lowering_target_present = unit.selected_lowering_target.is_some();
    let payload_blob_present = unit.artifact_payload_blob_path.is_some();
    let payload_format_present = unit.artifact_payload_format.is_some();
    let bridge_stub_present = unit.artifact_bridge_stub_path.is_some();
    let ir_sidecar_present = unit.artifact_ir_sidecar_path.is_some();
    let backend_artifact_missing_signals = link_plan_backend_artifact_missing_signals(
        unit,
        payload_blob_present,
        payload_format_present,
        bridge_stub_present,
    );
    let backend_artifact_candidate = unit.backend_family.is_some()
        || unit.target_device.is_some()
        || unit.selected_lowering_target.is_some();
    let backend_artifact_ready = backend_artifact_missing_signals.is_empty();
    let backend_artifact_key = format!(
        "{}:{}:{}",
        unit.domain_family, backend_family, target_device
    );
    let registry_dispatch = link_plan_registry_dispatch_readiness(unit);
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
    LinkPlanDomainReadiness {
        package_id: unit.package_id.clone(),
        domain_family: unit.domain_family.clone(),
        backend_family,
        target_device,
        backend_artifact_candidate,
        backend_artifact_key,
        backend_artifact_ready,
        backend_artifact_missing_signals,
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

fn link_plan_backend_artifact_missing_signals(
    unit: &nuisc::linker::LinkPlanDomainUnit,
    payload_blob_present: bool,
    payload_format_present: bool,
    bridge_stub_present: bool,
) -> Vec<String> {
    let mut missing = Vec::new();
    if unit.backend_family.is_none() {
        missing.push("backend_family".to_owned());
    }
    if unit.target_device.is_none() {
        missing.push("target_device".to_owned());
    }
    if !payload_blob_present {
        missing.push("artifact_payload_blob".to_owned());
    }
    if !payload_format_present {
        missing.push("artifact_payload_format".to_owned());
    }
    if !bridge_stub_present {
        missing.push("artifact_bridge_stub".to_owned());
    }
    missing
}

struct LinkPlanRegistryDispatchReadiness {
    pub(super) status: String,
    pub(super) ready: bool,
    pub(super) missing_signals: Vec<String>,
    pub(super) dispatch_bridge_materialized: bool,
    pub(super) execution_readiness_materialized: bool,
}

fn link_plan_registry_dispatch_readiness(
    unit: &nuisc::linker::LinkPlanDomainUnit,
) -> LinkPlanRegistryDispatchReadiness {
    match nuisc::registry::load_manifest_for_domain(
        Path::new("nustar-packages"),
        &unit.domain_family,
    ) {
        Ok(manifest) => {
            let dispatch = nuisc::registry::dispatch_readiness_summary(&manifest);
            LinkPlanRegistryDispatchReadiness {
                status: dispatch.status.clone(),
                ready: dispatch.status == "ready",
                missing_signals: dispatch.missing_signals,
                dispatch_bridge_materialized: dispatch.dispatch_bridge_materialized,
                execution_readiness_materialized: dispatch.execution_readiness_materialized,
            }
        }
        Err(error) => LinkPlanRegistryDispatchReadiness {
            status: "unavailable".to_owned(),
            ready: false,
            missing_signals: vec![format!("registry_manifest_unavailable:{error}")],
            dispatch_bridge_materialized: false,
            execution_readiness_materialized: false,
        },
    }
}

pub(super) fn link_plan_domain_readiness_units_json(
    summary: &LinkPlanDomainReadinessSummary,
) -> String {
    summary
        .units
        .iter()
        .map(link_plan_domain_readiness_json)
        .collect::<Vec<_>>()
        .join(",")
}

fn link_plan_domain_readiness_json(unit: &LinkPlanDomainReadiness) -> String {
    let fields = [
        crate::json_field("package_id", &unit.package_id),
        crate::json_field("domain_family", &unit.domain_family),
        crate::json_field("backend_family", &unit.backend_family),
        crate::json_field("target_device", &unit.target_device),
        crate::json_bool_field(
            "backend_artifact_candidate",
            unit.backend_artifact_candidate,
        ),
        crate::json_field("backend_artifact_key", &unit.backend_artifact_key),
        crate::json_bool_field("backend_artifact_ready", unit.backend_artifact_ready),
        crate::json_string_array_field(
            "backend_artifact_missing_signals",
            &unit.backend_artifact_missing_signals,
        ),
        crate::json_bool_field("ready", unit.ready),
        crate::json_bool_field(
            "selected_lowering_target_present",
            unit.selected_lowering_target_present,
        ),
        crate::json_bool_field("payload_blob_present", unit.payload_blob_present),
        crate::json_bool_field("payload_format_present", unit.payload_format_present),
        crate::json_bool_field("bridge_stub_present", unit.bridge_stub_present),
        crate::json_bool_field("ir_sidecar_present", unit.ir_sidecar_present),
        crate::json_field(
            "registry_dispatch_readiness_status",
            &unit.registry_dispatch_readiness_status,
        ),
        crate::json_bool_field(
            "registry_dispatch_readiness_ready",
            unit.registry_dispatch_readiness_ready,
        ),
        crate::json_string_array_field(
            "registry_dispatch_missing_signals",
            &unit.registry_dispatch_missing_signals,
        ),
        crate::json_bool_field(
            "registry_dispatch_bridge_materialized",
            unit.registry_dispatch_bridge_materialized,
        ),
        crate::json_bool_field(
            "registry_execution_readiness_materialized",
            unit.registry_execution_readiness_materialized,
        ),
        crate::json_string_array_field("issues", &unit.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
