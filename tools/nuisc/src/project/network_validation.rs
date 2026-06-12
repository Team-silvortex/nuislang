use std::collections::BTreeMap;

use super::support_contracts::{
    network_profile_slot_targets, network_support_surface_contract,
    require_declared_support_surface, support_profile_slots_for_domain, support_surface_for_domain,
};
use super::{
    collect_profile_int_bindings, require_declared_profile_slot, split_domain_unit, LoadedProject,
};

pub(super) fn validate_network_profile_for_link(
    project: &LoadedProject,
    endpoint: &str,
) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    if domain != "network" {
        return Ok(());
    }
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "network")?;
    let declared_slots = support_profile_slots_for_domain("network")?;
    for required_surface in network_support_surface_contract() {
        require_declared_support_surface(&declared_support, "network", &unit, required_surface)?;
    }
    for (slot, _node_name) in network_profile_slot_targets(&unit) {
        require_declared_profile_slot(&declared_slots, "network", &unit, slot)?;
    }
    validate_network_profile_slot_contract(
        project,
        &unit,
        &[
            "bind_core",
            "endpoint_kind",
            "local_port",
            "remote_port",
            "connect_timeout_ms",
            "retry_budget",
            "stream_window",
            "recv_window",
            "send_window",
        ],
    )?;
    Ok(())
}

pub(super) fn validate_network_profile_slot_contract(
    project: &LoadedProject,
    unit: &str,
    required_slots: &[&str],
) -> Result<(), String> {
    let profile_module = project
        .modules
        .iter()
        .find(|module| module.ast.domain == "network" && module.ast.unit == unit)
        .ok_or_else(|| format!("project is missing support module `network.{unit}`"))?;
    let profile_fn = profile_module
        .ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
        .ok_or_else(|| {
            format!(
                "project network unit `network.{}` requires a `profile()` function",
                unit
            )
        })?;
    let int_bindings = collect_profile_int_bindings(&profile_fn.body);

    for &slot in required_slots {
        let value = int_bindings.get(slot).copied().ok_or_else(|| {
            format!(
                "project network unit `network.{}` requires `{}` profile const",
                unit, slot
            )
        })?;
        validate_network_profile_slot_value(unit, slot, value)?;
    }

    for (slot, &value) in &int_bindings {
        validate_network_profile_slot_value(unit, slot, value)?;
    }

    Ok(())
}

fn validate_network_profile_slot_value(unit: &str, slot: &str, value: i64) -> Result<(), String> {
    let (predicate, relation) = match slot {
        "bind_core" => (value >= 0, ">= 0"),
        "endpoint_kind" => (value >= 0, ">= 0"),
        "transport_family" => (value >= 0, ">= 0"),
        "local_port" => (value > 0, "> 0"),
        "remote_port" => (value >= 0, ">= 0"),
        "connect_timeout_ms" => (value >= 0, ">= 0"),
        "read_timeout_ms" => (value >= 0, ">= 0"),
        "write_timeout_ms" => (value >= 0, ">= 0"),
        "retry_budget" => (value >= 0, ">= 0"),
        "stream_window" => (value > 0, "> 0"),
        "recv_window" => (value > 0, "> 0"),
        "send_window" => (value > 0, "> 0"),
        "protocol_kind" => (value >= 0, ">= 0"),
        "protocol_version" => (value >= 0, ">= 0"),
        "protocol_header_bytes" => (value >= 0, ">= 0"),
        _ => return Ok(()),
    };
    if predicate {
        return Ok(());
    }
    Err(format!(
        "project network unit `network.{}` requires `{}` {}",
        unit, slot, relation
    ))
}
