use std::collections::BTreeMap;

use yir_core::{EdgeKind, YirModule};

use super::sanitize_ident;

pub(super) fn has_xfer_segment(
    module: &YirModule,
    node_families: &BTreeMap<&str, String>,
    from_family: &str,
    to_family: &str,
) -> bool {
    module.edges.iter().any(|edge| {
        edge.kind == EdgeKind::CrossDomainExchange
            && node_family(node_families, &edge.from) == Some(from_family)
            && node_family(node_families, &edge.to) == Some(to_family)
    })
}

pub(super) fn resolve_project_profile_target_name(domain: &str, unit: &str, slot: &str) -> String {
    match (domain, slot) {
        ("shader", "target") => format!(
            "project_profile_shader_{}_profile_target",
            sanitize_ident(unit)
        ),
        ("shader", "viewport") => format!(
            "project_profile_shader_{}_profile_view",
            sanitize_ident(unit)
        ),
        ("shader", "pipeline") => format!(
            "project_profile_shader_{}_profile_pipe",
            sanitize_ident(unit)
        ),
        ("shader", "vertex_count") => format!(
            "project_profile_shader_{}_vertex_count",
            sanitize_ident(unit)
        ),
        ("shader", "instance_count") => format!(
            "project_profile_shader_{}_instance_count",
            sanitize_ident(unit)
        ),
        ("shader", "packet_color_slot") => format!(
            "project_profile_shader_{}_packet_color_slot",
            sanitize_ident(unit)
        ),
        ("shader", "packet_speed_slot") => format!(
            "project_profile_shader_{}_packet_speed_slot",
            sanitize_ident(unit)
        ),
        ("shader", "packet_radius_slot") => format!(
            "project_profile_shader_{}_packet_radius_slot",
            sanitize_ident(unit)
        ),
        ("shader", "slider_color_slot") => format!(
            "project_profile_shader_{}_slider_color_slot",
            sanitize_ident(unit)
        ),
        ("shader", "slider_speed_slot") => format!(
            "project_profile_shader_{}_slider_speed_slot",
            sanitize_ident(unit)
        ),
        ("shader", "slider_radius_slot") => format!(
            "project_profile_shader_{}_slider_radius_slot",
            sanitize_ident(unit)
        ),
        ("shader", "header_accent_slot") => format!(
            "project_profile_shader_{}_header_accent_slot",
            sanitize_ident(unit)
        ),
        ("shader", "toggle_live_slot") => format!(
            "project_profile_shader_{}_toggle_live_slot",
            sanitize_ident(unit)
        ),
        ("shader", "focus_slot") => {
            format!("project_profile_shader_{}_focus_slot", sanitize_ident(unit))
        }
        ("shader", "packet_tag") => {
            format!("project_profile_shader_{}_packet_tag", sanitize_ident(unit))
        }
        ("shader", "material_mode") => format!(
            "project_profile_shader_{}_material_mode",
            sanitize_ident(unit)
        ),
        ("shader", "pass_kind") => {
            format!("project_profile_shader_{}_pass_kind", sanitize_ident(unit))
        }
        ("shader", "packet_field_count") => format!(
            "project_profile_shader_{}_packet_field_count",
            sanitize_ident(unit)
        ),
        ("kernel", "bind_core") => {
            format!("project_profile_kernel_{}_bind_core", sanitize_ident(unit))
        }
        ("kernel", "queue_depth") => format!(
            "project_profile_kernel_{}_queue_depth",
            sanitize_ident(unit)
        ),
        ("kernel", "batch_lanes") => format!(
            "project_profile_kernel_{}_batch_lanes",
            sanitize_ident(unit)
        ),
        ("network", "bind_core") => {
            format!("project_profile_network_{}_bind_core", sanitize_ident(unit))
        }
        ("network", "endpoint_kind") => format!(
            "project_profile_network_{}_endpoint_kind",
            sanitize_ident(unit)
        ),
        ("network", "transport_family") => format!(
            "project_profile_network_{}_transport_family",
            sanitize_ident(unit)
        ),
        ("network", "local_port") => format!(
            "project_profile_network_{}_local_port",
            sanitize_ident(unit)
        ),
        ("network", "remote_port") => format!(
            "project_profile_network_{}_remote_port",
            sanitize_ident(unit)
        ),
        ("network", "connect_timeout_ms") => format!(
            "project_profile_network_{}_connect_timeout_ms",
            sanitize_ident(unit)
        ),
        ("network", "read_timeout_ms") => format!(
            "project_profile_network_{}_read_timeout_ms",
            sanitize_ident(unit)
        ),
        ("network", "write_timeout_ms") => format!(
            "project_profile_network_{}_write_timeout_ms",
            sanitize_ident(unit)
        ),
        ("network", "retry_budget") => format!(
            "project_profile_network_{}_retry_budget",
            sanitize_ident(unit)
        ),
        ("network", "stream_window") => format!(
            "project_profile_network_{}_stream_window",
            sanitize_ident(unit)
        ),
        ("network", "recv_window") => format!(
            "project_profile_network_{}_recv_window",
            sanitize_ident(unit)
        ),
        ("network", "send_window") => format!(
            "project_profile_network_{}_send_window",
            sanitize_ident(unit)
        ),
        ("network", "protocol_kind") => format!(
            "project_profile_network_{}_protocol_kind",
            sanitize_ident(unit)
        ),
        ("network", "protocol_version") => format!(
            "project_profile_network_{}_protocol_version",
            sanitize_ident(unit)
        ),
        ("network", "protocol_header_bytes") => format!(
            "project_profile_network_{}_protocol_header_bytes",
            sanitize_ident(unit)
        ),
        ("data", "bind_core") => format!(
            "project_profile_data_{}_data_bind_core",
            sanitize_ident(unit)
        ),
        ("data", "window_offset") => format!(
            "project_profile_data_{}_window_offset",
            sanitize_ident(unit)
        ),
        ("data", "uplink_len") => {
            format!("project_profile_data_{}_uplink_len", sanitize_ident(unit))
        }
        ("data", "downlink_len") => {
            format!("project_profile_data_{}_downlink_len", sanitize_ident(unit))
        }
        ("data", "handle_table") => format!(
            "project_profile_data_{}_profile_handles",
            sanitize_ident(unit)
        ),
        ("data", marker) if marker.starts_with("marker:") => format!(
            "project_profile_data_{}_{}",
            sanitize_ident(unit),
            sanitize_ident(marker.trim_start_matches("marker:"))
        ),
        _ => format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(domain),
            sanitize_ident(unit),
            sanitize_ident(slot)
        ),
    }
}

fn node_family<'a>(node_families: &'a BTreeMap<&str, String>, node_name: &str) -> Option<&'a str> {
    node_families.get(node_name).map(String::as_str)
}
