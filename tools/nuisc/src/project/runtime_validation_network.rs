use std::collections::BTreeSet;

use nuis_semantics::model::NirModule;

use super::super::network_validation::validate_network_profile_slot_contract;
use super::super::profile_usage::{nir_uses_cpu_extern_call, nir_uses_network_profile_slot};
use super::super::support_contracts::require_declared_support_surface;
use super::super::LoadedProject;

#[path = "runtime_validation_network_handles.rs"]
mod handles;
use handles::{validate_network_owned_handle_provenance, validate_network_owned_handle_shape};

#[derive(Clone, Copy)]
struct RuntimeNetworkValidationContext<'a> {
    project: &'a LoadedProject,
    module: &'a NirModule,
    network_support: &'a BTreeSet<String>,
    from: &'a str,
    to: &'a str,
    unit: &'a str,
}

struct NetworkHostCallRule<'a> {
    host_symbol: &'static str,
    required_slots: &'a [&'static str],
    required_surfaces: &'a [&'static str],
}

struct NetworkSlotUsageRule {
    slot: &'static str,
    required_surface: &'static str,
    builtin_name: &'static str,
}

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
    let context = RuntimeNetworkValidationContext {
        project,
        module,
        network_support,
        from,
        to,
        unit,
    };
    let validate_host_call = |host_symbol, required_slots, required_surfaces| {
        validate_network_host_call(
            context,
            NetworkHostCallRule {
                host_symbol,
                required_slots,
                required_surfaces,
            },
        )
    };

    validate_host_call(
        "host_network_connect_probe",
        &["local_port", "remote_port", "connect_timeout_ms"],
        &["network.profile.connect.v1"],
    )?;
    validate_host_call(
        "host_network_open_tcp_stream",
        &["remote_port", "connect_timeout_ms"],
        &["network.profile.connect.v1"],
    )?;
    validate_host_call(
        "host_network_open_udp_datagram",
        &["local_port", "remote_port"],
        &["network.profile.connect.v1"],
    )?;
    validate_host_call(
        "host_network_open_tcp_listener",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_host_call(
        "host_network_bind_udp_datagram",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_host_call(
        "host_network_accept_probe",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_host_call(
        "host_network_accept_owned",
        &["read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_host_call(
        "host_network_send_probe",
        &["stream_window", "send_window", "remote_port"],
        &[
            "network.profile.send.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_host_call(
        "host_network_send_owned",
        &["stream_window", "send_window"],
        &[
            "network.profile.send.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_host_call(
        "host_network_recv_probe",
        &["stream_window", "recv_window", "local_port"],
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_host_call(
        "host_network_recv_owned",
        &["stream_window", "recv_window"],
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
        ],
    )?;
    validate_host_call(
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
    validate_host_call("host_network_close", &[], &["network.profile.close.v1"])?;
    validate_host_call(
        "host_network_close_owned",
        &[],
        &["network.profile.close.v1"],
    )?;
    Ok(())
}

fn validate_network_host_call(
    context: RuntimeNetworkValidationContext<'_>,
    rule: NetworkHostCallRule<'_>,
) -> Result<(), String> {
    if !nir_uses_cpu_extern_call(context.module, rule.host_symbol) {
        return Ok(());
    }
    for surface in rule.required_surfaces {
        require_declared_support_surface(
            context.network_support,
            "network",
            context.unit,
            surface,
        )?;
    }
    validate_network_profile_slot_contract(context.project, context.unit, rule.required_slots)?;
    for slot in rule.required_slots {
        if !nir_uses_network_profile_slot(context.module, context.unit, slot) {
            let builtin_name = network_profile_builtin_name(slot);
            return Err(format!(
                "project link `{}` -> `{}` requires CPU entry to route `{}` through {}(\"{}\")",
                context.from, context.to, rule.host_symbol, builtin_name, context.unit
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
    let context = RuntimeNetworkValidationContext {
        project,
        module,
        network_support,
        from,
        to,
        unit,
    };
    let validate_slot = |slot, required_surface, builtin_name| {
        validate_network_slot_usage(
            context,
            NetworkSlotUsageRule {
                slot,
                required_surface,
                builtin_name,
            },
        )
    };

    validate_slot(
        "transport_family",
        "network.profile.transport.v1",
        "network_profile_transport_family",
    )?;
    validate_slot(
        "local_port",
        "network.profile.connect.v1",
        "network_profile_local_port",
    )?;
    validate_slot(
        "remote_port",
        "network.profile.connect.v1",
        "network_profile_remote_port",
    )?;
    validate_slot(
        "connect_timeout_ms",
        "network.profile.connect.v1",
        "network_profile_connect_timeout",
    )?;
    validate_slot(
        "read_timeout_ms",
        "network.profile.timeout.v1",
        "network_profile_read_timeout",
    )?;
    validate_slot(
        "write_timeout_ms",
        "network.profile.timeout.v1",
        "network_profile_write_timeout",
    )?;
    validate_slot(
        "retry_budget",
        "network.profile.retry.v1",
        "network_profile_retry_budget",
    )?;
    validate_slot(
        "stream_window",
        "network.profile.stream-window.v1",
        "network_profile_stream_window",
    )?;
    validate_slot(
        "recv_window",
        "network.profile.recv.v1",
        "network_profile_recv_window",
    )?;
    validate_slot(
        "send_window",
        "network.profile.send.v1",
        "network_profile_send_window",
    )?;
    validate_slot(
        "protocol_kind",
        "network.profile.protocol.v1",
        "network_profile_protocol_kind",
    )?;
    validate_slot(
        "protocol_version",
        "network.profile.protocol.v1",
        "network_profile_protocol_version",
    )?;
    validate_slot(
        "protocol_header_bytes",
        "network.profile.protocol.v1",
        "network_profile_protocol_header_bytes",
    )?;
    Ok(())
}

fn validate_network_slot_usage(
    context: RuntimeNetworkValidationContext<'_>,
    rule: NetworkSlotUsageRule,
) -> Result<(), String> {
    if !nir_uses_network_profile_slot(context.module, context.unit, rule.slot) {
        return Ok(());
    }
    require_declared_support_surface(
        context.network_support,
        "network",
        context.unit,
        rule.required_surface,
    )?;
    validate_network_profile_slot_contract(context.project, context.unit, &[rule.slot])?;
    let rendered = format!("{}(\"{}\")", rule.builtin_name, context.unit);
    if !nir_uses_network_profile_slot(context.module, context.unit, rule.slot) {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to use {} at NIR level",
            context.from, context.to, rendered
        ));
    }
    Ok(())
}
