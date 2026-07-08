use std::collections::{BTreeMap, BTreeSet};

use yir_core::Node;

use crate::project_contracts::parse_semicolon_kv_contract;

pub(crate) fn verify_scheduler_lane_contract_text(
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler lane contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lanes = fields
        .get("lanes")
        .ok_or_else(|| format!("scheduler contract node `{node_name}` is missing `lanes` field"))?;
    let defaults = fields.get("defaults").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `defaults` field")
    })?;
    let parsed_lanes = lanes
        .split(',')
        .map(str::trim)
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    if parsed_lanes.is_empty() {
        return Err(format!(
            "scheduler contract node `{node_name}` requires at least one declared lane"
        ));
    }
    let mut lanes_from_defaults = BTreeSet::<&str>::new();
    for entry in defaults.split('|') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((pattern, lane)) = entry.split_once('=') else {
            return Err(format!(
                "scheduler contract node `{node_name}` has invalid default lane entry `{entry}`"
            ));
        };
        let pattern = pattern.trim();
        let lane = lane.trim();
        if pattern.is_empty() || lane.is_empty() {
            return Err(format!(
                "scheduler contract node `{node_name}` has invalid default lane entry `{entry}`"
            ));
        }
        if !parsed_lanes.contains(lane) {
            return Err(format!(
                "scheduler contract node `{node_name}` declares default lane `{lane}` outside `{lanes}`"
            ));
        }
        lanes_from_defaults.insert(lane);
    }
    if lanes_from_defaults != parsed_lanes {
        return Err(format!(
            "scheduler contract node `{node_name}` declares lanes `{lanes}` but defaults cover `{}`",
            lanes_from_defaults.into_iter().collect::<Vec<_>>().join(",")
        ));
    }
    Ok(())
}

pub(crate) fn verify_scheduler_clock_contract_text(
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler clock contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    for key in ["domain", "kind", "epoch", "resolution", "bridge"] {
        let value = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if value.trim().is_empty() {
            return Err(format!(
                "scheduler contract node `{node_name}` requires non-empty `{key}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_lane_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler lane capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lane_policy_name = format!("scheduler_contract_{family}_lane_policy_type");
    let lane_policy_node = nodes.get(lane_policy_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling lane policy node `{lane_policy_name}`"
        )
    })?;
    let lane_policy_value = lane_policy_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{lane_policy_name}` must carry a canonical text payload"
            )
        })?;
    let lane_policy_fields = parse_semicolon_kv_contract(
        lane_policy_name.as_str(),
        lane_policy_value,
        "scheduler lane contract",
    )?;
    let declared_lanes = lane_policy_fields
        .get("lanes")
        .ok_or_else(|| {
            format!("scheduler contract node `{lane_policy_name}` is missing `lanes` field")
        })?
        .split(',')
        .map(str::trim)
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    let declared_lane_list = declared_lanes.iter().copied().collect::<Vec<_>>().join(",");
    let capability_lanes = fields
        .iter()
        .filter_map(|(key, value)| (*key != "family").then_some((*key, *value)))
        .collect::<BTreeMap<_, _>>();
    if capability_lanes.is_empty() {
        return Err(format!(
            "scheduler contract node `{node_name}` requires at least one lane capability entry"
        ));
    }
    for lane in &declared_lanes {
        let capability = capability_lanes.get(lane).ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` is missing capability for declared lane `{lane}`"
            )
        })?;
        if capability.trim().is_empty() {
            return Err(format!(
                "scheduler contract node `{node_name}` requires non-empty capability for lane `{lane}`"
            ));
        }
    }
    for lane in capability_lanes.keys() {
        if !declared_lanes.contains(*lane) {
            return Err(format!(
                "scheduler contract node `{node_name}` declares capability for lane `{lane}` outside `{declared_lane_list}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_bridge_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler bridge capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lane_bridge = fields.get("lane_bridge").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `lane_bridge` field")
    })?;
    let clock_bridge = fields.get("clock_bridge").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `clock_bridge` field")
    })?;
    if lane_bridge.trim().is_empty() || clock_bridge.trim().is_empty() {
        return Err(format!(
            "scheduler contract node `{node_name}` requires non-empty `lane_bridge` and `clock_bridge`"
        ));
    }
    if family == "cpu" && *lane_bridge != "cpu_bind_core_lane:host_main_lane|worker_lane" {
        return Err(format!(
            "scheduler contract node `{node_name}` currently expects CPU lane bridge `cpu_bind_core_lane:host_main_lane|worker_lane`, got `{lane_bridge}`"
        ));
    }
    if family != "cpu" && *lane_bridge != "none" {
        return Err(format!(
            "scheduler contract node `{node_name}` currently expects non-CPU lane bridge `none`, got `{lane_bridge}`"
        ));
    }
    let clock_contract_name = format!("scheduler_contract_{family}_clock_type");
    let clock_contract_node = nodes.get(clock_contract_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling clock node `{clock_contract_name}`"
        )
    })?;
    let clock_contract_value = clock_contract_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{clock_contract_name}` must carry a canonical text payload"
            )
        })?;
    let clock_contract_fields = parse_semicolon_kv_contract(
        clock_contract_name.as_str(),
        clock_contract_value,
        "scheduler clock contract",
    )?;
    let declared_clock_bridge = clock_contract_fields.get("bridge").ok_or_else(|| {
        format!("scheduler contract node `{clock_contract_name}` is missing `bridge` field")
    })?;
    if *declared_clock_bridge != *clock_bridge {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `clock_bridge={clock_bridge}`, but `{clock_contract_name}` uses `{declared_clock_bridge}`"
        ));
    }
    Ok(())
}

pub(crate) fn verify_scheduler_result_lane_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler result lane contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lane_policy_name = format!("scheduler_contract_{family}_lane_policy_type");
    let lane_policy_node = nodes.get(lane_policy_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling lane policy node `{lane_policy_name}`"
        )
    })?;
    let lane_policy_value = lane_policy_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{lane_policy_name}` must carry a canonical text payload"
            )
        })?;
    let lane_policy_fields = parse_semicolon_kv_contract(
        lane_policy_name.as_str(),
        lane_policy_value,
        "scheduler lane contract",
    )?;
    let declared_lanes = lane_policy_fields
        .get("lanes")
        .ok_or_else(|| {
            format!("scheduler contract node `{lane_policy_name}` is missing `lanes` field")
        })?
        .split(',')
        .map(str::trim)
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    let declared_lane_list = declared_lanes.iter().copied().collect::<Vec<_>>().join(",");
    for key in ["entry", "probe", "value"] {
        let lane = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if !declared_lanes.contains(*lane) {
            return Err(format!(
                "scheduler contract node `{node_name}` declares result lane `{lane}` for `{key}` outside `{declared_lane_list}`"
            ));
        }
    }
    Ok(())
}
