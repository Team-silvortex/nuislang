pub const DATA_BRIDGE_HETERO_DOMAINS: &[&str] = &["shader", "kernel", "network"];

const DATA_COMMON_MARKER_SLOTS: &[&str] = &[
    "marker:uplink_pipe",
    "marker:downlink_pipe",
    "marker:uplink_pipe_class",
    "marker:downlink_pipe_class",
    "marker:uplink_payload_class",
    "marker:downlink_payload_class",
    "marker:uplink_payload_shape",
    "marker:downlink_payload_shape",
    "marker:uplink_window_policy",
    "marker:downlink_window_policy",
];

pub fn data_common_marker_slots() -> &'static [&'static str] {
    DATA_COMMON_MARKER_SLOTS
}

pub fn directional_bridge_marker_tag(from_domain: &str, to_domain: &str) -> Option<String> {
    if from_domain == "cpu" && DATA_BRIDGE_HETERO_DOMAINS.contains(&to_domain) {
        return Some(format!("cpu_to_{to_domain}"));
    }
    if to_domain == "cpu" && DATA_BRIDGE_HETERO_DOMAINS.contains(&from_domain) {
        return Some(format!("{from_domain}_to_cpu"));
    }
    None
}

pub fn directional_bridge_marker_slot(from_domain: &str, to_domain: &str) -> Option<String> {
    directional_bridge_marker_tag(from_domain, to_domain).map(|tag| format!("marker:{tag}"))
}

pub fn all_uplink_directional_marker_slots() -> Vec<String> {
    DATA_BRIDGE_HETERO_DOMAINS
        .iter()
        .map(|domain| format!("marker:cpu_to_{domain}"))
        .collect()
}

pub fn all_downlink_directional_marker_slots() -> Vec<String> {
    DATA_BRIDGE_HETERO_DOMAINS
        .iter()
        .map(|domain| format!("marker:{domain}_to_cpu"))
        .collect()
}

pub fn all_sync_marker_slots() -> Vec<String> {
    let mut slots = all_uplink_directional_marker_slots();
    slots.extend(all_downlink_directional_marker_slots());
    slots
}

pub fn data_marker_surface(tag: &str) -> &'static str {
    if tag.starts_with("cpu_to_") || tag.ends_with("_to_cpu") {
        return "data.profile.sync-markers.v1";
    }
    match tag {
        "uplink_pipe" | "downlink_pipe" => "data.profile.pipe-markers.v1",
        "uplink_pipe_class" | "downlink_pipe_class" => "data.profile.pipe-class.v1",
        "uplink_payload_class" | "downlink_payload_class" => "data.profile.payload-class.v1",
        "uplink_payload_shape" | "downlink_payload_shape" => "data.profile.payload-shape.v1",
        "uplink_window_policy" | "downlink_window_policy" => "data.profile.window-policy.v1",
        _ => "data.profile.sync-markers.v1",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        all_downlink_directional_marker_slots, all_sync_marker_slots,
        all_uplink_directional_marker_slots, data_marker_surface, directional_bridge_marker_slot,
    };

    #[test]
    fn directional_bridge_marker_slot_tracks_supported_domains() {
        assert_eq!(
            directional_bridge_marker_slot("cpu", "network").as_deref(),
            Some("marker:cpu_to_network")
        );
        assert_eq!(
            directional_bridge_marker_slot("kernel", "cpu").as_deref(),
            Some("marker:kernel_to_cpu")
        );
        assert_eq!(directional_bridge_marker_slot("shader", "network"), None);
    }

    #[test]
    fn sync_marker_inventory_covers_all_current_bridge_directions() {
        assert_eq!(
            all_uplink_directional_marker_slots(),
            vec![
                "marker:cpu_to_shader".to_owned(),
                "marker:cpu_to_kernel".to_owned(),
                "marker:cpu_to_network".to_owned(),
            ]
        );
        assert_eq!(
            all_downlink_directional_marker_slots(),
            vec![
                "marker:shader_to_cpu".to_owned(),
                "marker:kernel_to_cpu".to_owned(),
                "marker:network_to_cpu".to_owned(),
            ]
        );
        assert_eq!(all_sync_marker_slots().len(), 6);
    }

    #[test]
    fn marker_surface_classifies_directional_and_pipe_markers() {
        assert_eq!(
            data_marker_surface("cpu_to_kernel"),
            "data.profile.sync-markers.v1"
        );
        assert_eq!(
            data_marker_surface("downlink_pipe"),
            "data.profile.pipe-markers.v1"
        );
        assert_eq!(
            data_marker_surface("uplink_payload_shape"),
            "data.profile.payload-shape.v1"
        );
    }
}
