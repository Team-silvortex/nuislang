use std::collections::BTreeSet;

use crate::registry::NustarPackageManifest;

pub(super) fn render_lane_policy_contract(family: &str, default_lanes: &[String]) -> String {
    let mut lanes = BTreeSet::<String>::new();
    let mut defaults = Vec::<String>::new();
    for entry in default_lanes {
        let Some((pattern, lane)) = entry.split_once('=') else {
            continue;
        };
        let pattern = pattern.trim();
        let lane = lane.trim();
        if pattern.is_empty() || lane.is_empty() {
            continue;
        }
        lanes.insert(lane.to_owned());
        defaults.push(format!("{pattern}={lane}"));
    }
    format!(
        "family={family};lanes={};defaults={}",
        lanes.into_iter().collect::<Vec<_>>().join(","),
        defaults.join("|")
    )
}

pub(super) fn render_clock_contract(family: &str, manifest: &NustarPackageManifest) -> String {
    format!(
        "family={family};domain={};kind={};epoch={};resolution={};bridge={}",
        manifest.clock_domain_id,
        manifest.clock_kind,
        manifest.clock_epoch_kind,
        manifest.clock_resolution,
        manifest.clock_bridge_default
    )
}

pub(super) fn render_lane_capability_contract(family: &str, default_lanes: &[String]) -> String {
    let lanes = default_lanes
        .iter()
        .filter_map(|entry| entry.split_once('='))
        .map(|(_, lane)| lane.trim())
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    let mut fields = vec![format!("family={family}")];
    for lane in lanes {
        let capability = lane_capability_for(family, lane);
        fields.push(format!("{lane}={capability}"));
    }
    fields.join(";")
}

pub(super) fn render_bridge_capability_contract(
    family: &str,
    manifest: &NustarPackageManifest,
) -> String {
    let lane_bridge = match family {
        "cpu" => "cpu_bind_core_lane:host_main_lane|worker_lane",
        _ => "none",
    };
    format!(
        "family={family};lane_bridge={lane_bridge};clock_bridge={}",
        manifest.clock_bridge_default
    )
}

fn lane_capability_for(family: &str, lane: &str) -> &'static str {
    match (family, lane) {
        ("cpu", "main") => "host-entry",
        ("cpu", "mem") => "memory-ownership",
        ("data", "control") => "control-plane",
        ("data", "uplink") => "uplink-window",
        ("data", "downlink") => "downlink-window",
        ("data", "fabric") => "fabric-transfer",
        ("shader", "setup") => "render-setup",
        ("shader", "render") => "render-pass",
        ("kernel", "compute") | ("npu", "compute") => "compute-dispatch",
        (_, "contract") => "contract-metadata",
        _ => "general",
    }
}

pub(super) fn render_result_lane_contract(family: &str) -> String {
    let lane = match family {
        "cpu" => "main",
        "data" => "fabric",
        "shader" => "setup",
        "network" => "control",
        "kernel" | "npu" => "compute",
        _ => "main",
    };
    format!("family={family};entry={lane};probe={lane};value={lane}")
}

pub(super) fn render_result_capability_contract(family: &str) -> String {
    format!(
        "family={family};entry=result-entry;probe=result-ready-probe;value=result-payload-value"
    )
}

pub(super) fn render_observer_role_variant_contract(family: &str) -> String {
    format!(
        "family={family};config_ready=config-ready-observer;send_ready=send-ready-observer;recv_ready=recv-ready-observer;connect_ready=connect-ready-observer;accept_ready=accept-ready-observer;closed=closed-observer"
    )
}

pub(super) fn render_summary_capability_contract(family: &str) -> String {
    format!(
        "family={family};policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"
    )
}

pub(super) fn render_summary_class_contract(family: &str) -> String {
    format!(
        "family={family};transport_split=transport-split-summary;transport_windowed_split=transport-windowed-split-summary;transport_session_bridge_split=transport-session-bridge-split-summary;control_split=control-split-summary;control_windowed=control-windowed-summary;control_session_bridge=control-session-bridge-summary"
    )
}

pub(super) fn render_observer_source_class_contract(family: &str) -> String {
    format!("family={family};profile=profile-backed;result=result-backed;summary=summary-backed")
}

pub(super) fn render_observer_stage_class_contract(family: &str) -> String {
    format!(
        "family={family};entry=observer-entry-stage;ready=observer-ready-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"
    )
}

pub(super) fn render_observer_scope_class_contract(family: &str) -> String {
    format!(
        "family={family};local=local-scope;cross_lane=cross-lane-scope;cross_domain=cross-domain-scope;bridge_visible=bridge-visible-scope"
    )
}

pub(super) fn render_observer_branch_class_contract(family: &str) -> String {
    format!(
        "family={family};primary=primary-branch;secondary=secondary-branch;fallback=fallback-branch;send=send-branch;recv=recv-branch"
    )
}
