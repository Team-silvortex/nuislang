use std::collections::BTreeSet;

use nuis_semantics::model::NirModule;

use super::super::network_validation::validate_network_profile_slot_contract;
use super::super::profile_usage::{nir_uses_cpu_extern_call, nir_uses_network_profile_slot};
use super::super::support_contracts::require_declared_support_surface;
use super::super::LoadedProject;

#[path = "runtime_validation_network_handles.rs"]
mod handles;
use handles::{validate_network_owned_handle_provenance, validate_network_owned_handle_shape};

pub(super) fn validate_network_host_call_requirements(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
) -> Result<(), String> {
    validate_network_owned_handle_shape(module, from, to)?;
    validate_network_owned_handle_provenance(module, from, to)?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_connect_probe",
        &["local_port", "remote_port", "connect_timeout_ms"],
        &["network.profile.connect.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_open_tcp_stream",
        &["remote_port", "connect_timeout_ms"],
        &["network.profile.connect.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_open_udp_datagram",
        &["local_port", "remote_port"],
        &["network.profile.connect.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_open_tcp_listener",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_bind_udp_datagram",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_accept_probe",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_accept_owned",
        &["read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_send_probe",
        &["stream_window", "send_window", "remote_port"],
        &[
            "network.profile.send.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_send_owned",
        &["stream_window", "send_window"],
        &[
            "network.profile.send.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_recv_probe",
        &["stream_window", "recv_window", "local_port"],
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_recv_owned",
        &["stream_window", "recv_window"],
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_recv_http_status_owned",
        &[
            "stream_window",
            "recv_window",
            "protocol_kind",
            "protocol_version",
            "protocol_header_bytes",
        ],
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
            "network.profile.protocol.v1",
        ],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_close",
        &[],
        &["network.profile.close.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_close_owned",
        &[],
        &["network.profile.close.v1"],
    )?;
    Ok(())
}

fn validate_network_host_call(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
    host_symbol: &'static str,
    required_slots: &[&str],
    required_surfaces: &[&str],
) -> Result<(), String> {
    if !nir_uses_cpu_extern_call(module, host_symbol) {
        return Ok(());
    }
    for surface in required_surfaces {
        require_declared_support_surface(network_support, "network", unit, surface)?;
    }
    validate_network_profile_slot_contract(project, unit, required_slots)?;
    for slot in required_slots {
        if !nir_uses_network_profile_slot(module, unit, slot) {
            let builtin_name = network_profile_builtin_name(slot);
            return Err(format!(
                "project link `{}` -> `{}` requires CPU entry to route `{}` through {}(\"{}\")",
                from, to, host_symbol, builtin_name, unit
            ));
        }
    }
    Ok(())
}

fn network_profile_builtin_name(slot: &str) -> &str {
    match slot {
        "local_port" => "network_profile_local_port",
        "remote_port" => "network_profile_remote_port",
        "connect_timeout_ms" => "network_profile_connect_timeout",
        "read_timeout_ms" => "network_profile_read_timeout",
        "write_timeout_ms" => "network_profile_write_timeout",
        "stream_window" => "network_profile_stream_window",
        "recv_window" => "network_profile_recv_window",
        "send_window" => "network_profile_send_window",
        "protocol_kind" => "network_profile_protocol_kind",
        "protocol_version" => "network_profile_protocol_version",
        other => other,
    }
}

pub(super) fn validate_network_profile_slot_requirements(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
) -> Result<(), String> {
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "transport_family",
        "network.profile.transport.v1",
        "network_profile_transport_family",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "local_port",
        "network.profile.connect.v1",
        "network_profile_local_port",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "remote_port",
        "network.profile.connect.v1",
        "network_profile_remote_port",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "connect_timeout_ms",
        "network.profile.connect.v1",
        "network_profile_connect_timeout",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "read_timeout_ms",
        "network.profile.timeout.v1",
        "network_profile_read_timeout",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "write_timeout_ms",
        "network.profile.timeout.v1",
        "network_profile_write_timeout",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "retry_budget",
        "network.profile.retry.v1",
        "network_profile_retry_budget",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "stream_window",
        "network.profile.stream-window.v1",
        "network_profile_stream_window",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "recv_window",
        "network.profile.recv.v1",
        "network_profile_recv_window",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "send_window",
        "network.profile.send.v1",
        "network_profile_send_window",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "protocol_kind",
        "network.profile.protocol.v1",
        "network_profile_protocol_kind",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "protocol_version",
        "network.profile.protocol.v1",
        "network_profile_protocol_version",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "protocol_header_bytes",
        "network.profile.protocol.v1",
        "network_profile_protocol_header_bytes",
    )?;
    Ok(())
}

fn validate_network_slot_usage(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
    slot: &'static str,
    required_surface: &'static str,
    builtin_name: &'static str,
) -> Result<(), String> {
    if !nir_uses_network_profile_slot(module, unit, slot) {
        return Ok(());
    }
    require_declared_support_surface(network_support, "network", unit, required_surface)?;
    validate_network_profile_slot_contract(project, unit, &[slot])?;
    let rendered = format!("{builtin_name}(\"{unit}\")");
    if !nir_uses_network_profile_slot(module, unit, slot) {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to use {} at NIR level",
            from, to, rendered
        ));
    }
    Ok(())
}
