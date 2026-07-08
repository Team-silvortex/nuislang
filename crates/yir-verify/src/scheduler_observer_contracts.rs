use std::collections::BTreeMap;

use yir_core::Node;

use crate::project_contracts::parse_semicolon_kv_contract;

pub(crate) fn verify_scheduler_result_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler result capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let result_lane_name = format!("scheduler_contract_{family}_result_lane_type");
    let result_lane_node = nodes.get(result_lane_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling result lane node `{result_lane_name}`"
        )
    })?;
    let result_lane_value = result_lane_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{result_lane_name}` must carry a canonical text payload"
            )
        })?;
    let result_lane_fields = parse_semicolon_kv_contract(
        result_lane_name.as_str(),
        result_lane_value,
        "scheduler result lane contract",
    )?;
    for key in ["entry", "probe", "value"] {
        if !result_lane_fields.contains_key(key) {
            return Err(format!(
                "scheduler contract node `{result_lane_name}` is missing `{key}` field"
            ));
        }
        let capability = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        let expected = match key {
            "entry" => "result-entry",
            "probe" => "result-ready-probe",
            "value" => "result-payload-value",
            _ => unreachable!(),
        };
        if *capability != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={capability}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_observer_role_variant_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer role variant contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let result_capability_name = format!("scheduler_contract_{family}_result_capability_type");
    let _result_capability_node = nodes
        .get(result_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling result capability node `{result_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("config_ready", "config-ready-observer"),
        ("send_ready", "send-ready-observer"),
        ("recv_ready", "recv-ready-observer"),
        ("connect_ready", "connect-ready-observer"),
        ("accept_ready", "accept-ready-observer"),
        ("closed", "closed-observer"),
    ] {
        let variant = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *variant != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={variant}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_summary_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler summary capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let result_capability_name = format!("scheduler_contract_{family}_result_capability_type");
    let _result_capability_node = nodes
        .get(result_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling result capability node `{result_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("policy", "async-policy-summary"),
        ("batch", "async-batch-summary"),
        ("windowed", "async-windowed-summary"),
    ] {
        let capability = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *capability != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={capability}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_summary_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler summary class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let summary_capability_name = format!("scheduler_contract_{family}_summary_capability_type");
    let _summary_capability_node = nodes
        .get(summary_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling summary capability node `{summary_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("transport_split", "transport-split-summary"),
        (
            "transport_windowed_split",
            "transport-windowed-split-summary",
        ),
        (
            "transport_session_bridge_split",
            "transport-session-bridge-split-summary",
        ),
        ("control_split", "control-split-summary"),
        ("control_windowed", "control-windowed-summary"),
        ("control_session_bridge", "control-session-bridge-summary"),
    ] {
        let summary_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *summary_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={summary_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_observer_source_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer source class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let summary_capability_name = format!("scheduler_contract_{family}_summary_capability_type");
    let _summary_capability_node = nodes
        .get(summary_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling summary capability node `{summary_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("profile", "profile-backed"),
        ("result", "result-backed"),
        ("summary", "summary-backed"),
    ] {
        let source_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *source_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={source_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_observer_stage_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer stage class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let source_class_name = format!("scheduler_contract_{family}_observer_source_class_type");
    let _source_class_node = nodes.get(source_class_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling observer source class node `{source_class_name}`"
        )
    })?;
    for (key, expected) in [
        ("entry", "observer-entry-stage"),
        ("ready", "observer-ready-stage"),
        ("payload", "observer-payload-stage"),
        ("policy", "observer-policy-stage"),
        ("batch", "observer-batch-stage"),
        ("windowed", "observer-windowed-stage"),
    ] {
        let stage_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *stage_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={stage_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_observer_scope_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer scope class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let stage_class_name = format!("scheduler_contract_{family}_observer_stage_class_type");
    let _stage_class_node = nodes.get(stage_class_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling observer stage class node `{stage_class_name}`"
        )
    })?;
    for (key, expected) in [
        ("local", "local-scope"),
        ("cross_lane", "cross-lane-scope"),
        ("cross_domain", "cross-domain-scope"),
        ("bridge_visible", "bridge-visible-scope"),
    ] {
        let scope_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *scope_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={scope_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_scheduler_observer_branch_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer branch class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let scope_class_name = format!("scheduler_contract_{family}_observer_scope_class_type");
    let _scope_class_node = nodes.get(scope_class_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling observer scope class node `{scope_class_name}`"
        )
    })?;
    for (key, expected) in [
        ("primary", "primary-branch"),
        ("secondary", "secondary-branch"),
        ("fallback", "fallback-branch"),
        ("send", "send-branch"),
        ("recv", "recv-branch"),
    ] {
        let branch_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *branch_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={branch_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}
